use proc_macro2::*;
use quote::{quote, ToTokens};
use syn::{punctuated::Punctuated, *};

struct CommaSeparatedList {
	list: Punctuated::<TokenStream, Token![,]>
}

impl syn::parse::Parse for CommaSeparatedList {
    fn parse(input: parse::ParseStream) -> Result<Self> {
        let list = Punctuated::parse_terminated(input)?;
		Ok(Self { list })
    }
}

pub fn flags(args: TokenStream, input: TokenStream) -> TokenStream {
	// While we don't exactly are deriving, the `#[flags]` macro is close enough
	let parsed_res = syn::parse2::<DeriveInput>(input);
	let input_parsed = match parsed_res {
	    Ok(derived_input) => derived_input,
	    Err(err) => return err.to_compile_error().into(),
	};

	let vis = input_parsed.vis;
	let flag_name = input_parsed.ident;
	let enum_attrs = input_parsed.attrs;
	

	// Extract the body
	let body_data = match input_parsed.data {
		Data::Enum(body) => body,
		_ => return quote!( compile_error!("Not an enum"); )
	};

	// Define the u128 type to use
	let u128_type = syn::parse_str::<Type>("u128").unwrap();
	
	let mut idents = Vec::<syn::Ident>::new();
	let mut vals = Vec::<syn::Expr>::new();
	let mut attrs = Vec::<Vec::<Attribute>>::new();
	let mut i : u128 = 1;
	let mut max_val : u128 = 0;
	let mut has_zero = false;
	let mut parse_names = Vec::new();
	let mut none_name = "None".to_string();
	
	// Extract each variant and the data needed
	for (idx, it) in body_data.variants.into_iter().enumerate() {
		let ident_name = it.ident.to_string();
		idents.push(it.ident);

		// Extract parse_name attribute for elements
		let mut elem_attrs = Vec::new();
		for attr in it.attrs {
			if let Meta::List(meta_list) = &attr.meta {
				if meta_list.path.get_ident().map_or(false, |iden| iden == "parse_name") {
					if parse_names.len() != idx {
						let error_msg = format!("Duplicate `parse_name` for member '{}'", ident_name);
						return quote!(compile_error!(#error_msg));
					}


					let lit = match meta_list.parse_args::<LitStr>() {
    				    Ok(lit) => lit,
    				    Err(_) => {
							let error_msg = format!("Expected a string literal as a `parse_name` for member '{}'", ident_name);
							return quote!(compile_error!(#error_msg))
						},
    				};
					parse_names.push(lit.value());
					continue;
				}
			}
			elem_attrs.push(attr);
		}

		if parse_names.len() == idx {
			parse_names.push(ident_name.clone());
		}


		attrs.push(elem_attrs);
		match it.discriminant {
			Some((_, expr)) => {
				let res = gen_bits_val_expr(expr, &flag_name, &u128_type);
				let (bits_val, int) = match res {
    			    Ok(bits_val) => bits_val,
    			    Err(toks) => return toks,
    			};
				vals.push(construct_flag(bits_val));

				if let Some(int) = int {
					if int == 0 {
						has_zero = true;
						none_name = ident_name;
					} else {
						max_val = max_val.max(int);
						i = int << 1u128;
					}
				}
			},
			None => {
				if i == 0 {
					i = 1;
				} else if !i.is_power_of_two() {
					return quote!( compile_error!("Previous enum value needs to be a power of 2"); );
				}

				match create_expr_from_lit(i, u128_type.clone()) {
					Ok(val) => vals.push(val),
					Err(err) => return err.to_compile_error().into(),
				}
				max_val = max_val.max(i);
				i <<= 1u128;
			}
		};
	}
	
	let non_variant = if has_zero {
		quote!()
	} else {
		quote!(
			/// Value representing that no flag is set.
			#vis const None : #flag_name = #flag_name::none();
		)
	};

	
	let args = match parse2::<CommaSeparatedList>(args) {
		Ok(data) => data.list,
		Err(err) => {
			return TokenStream::from(err.to_compile_error());
		}
	};
	
	let mut base_type = None;
	let mut parse_from_name = false;
	for elem in args {
		if let Ok(ty) = syn::parse2::<syn::TypePath>(elem.clone()) {
			if ty.path.get_ident().map_or(false, |path| ["u8", "u16", "u32", "u64", "u128"].iter().any(|s| path == s)) {
				base_type = Some(ty);
				continue;
			}
		} 
		if let Ok(iden) = syn::parse2::<Ident>(elem) {
			if iden == "parse_from_name" {
				parse_from_name = true;
				continue;
			}
		}
	}

	let base_type = base_type.unwrap_or_else(|| if max_val <= u8::MAX as u128 {
		syn::parse_str::<TypePath>("u8").unwrap()
	} else if max_val <= u16::MAX as u128 {
		syn::parse_str::<TypePath>("u16").unwrap()
	} else if max_val <= u32::MAX as u128 {
		syn::parse_str::<TypePath>("u32").unwrap()
	} else if max_val <= u64::MAX as u128 {
		syn::parse_str::<TypePath>("u64").unwrap()
	} else {
		syn::parse_str::<TypePath>("u128").unwrap()
	});

	let parse = if parse_from_name {
		quote!{
			#vis fn parse(name: &str) -> Option<Self> {
				let mut flags = Self::none();
				for sub_name in name.split("|").map(|val| val.trim()) {
					flags |= match sub_name {
						#(#parse_names => Self::#idents,)*
						_ => return None
					}
				}
				Some(flags)
			}
		}
	} else {
		quote!{}
	};

	// Write out the new structure
	quote!(

		#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
		#(#enum_attrs)*
		#[repr(transparent)]
		#vis struct #flag_name {
			bits : #base_type
		}
		#[allow(non_upper_case_globals)]
		impl #flag_name {
			#non_variant
			
			#(#(#attrs)* #vis const #idents : #flag_name = #vals;)*

			// Helper function to create const values from a u128
			const fn new_u128(val: u128) -> Self {
				Self::new(val as #base_type)
			}

			const fn new(val: #base_type) -> Self {
				Self { bits: val }
			}

			/// Create flags instance with no flag set.
			#vis const fn none() -> Self {
				Self { bits: 0 }
			}

			/// Create flags instance with all valid flags set.
			#vis const fn all() -> Self {
				const BITS : #base_type = 0 #( | #flag_name::#idents.bits)*;
				Self { bits: BITS }
			}

			/// Get the flags' bits
			#vis const fn bits(&self) -> #base_type {
				self.bits
			}

			/// Check if a given flag(s) is/are set (if multiple flags are checked, all flags need to be set).
			#[deprecated = "Use `contains` instead"]
			#vis const fn is_set(&self, flag: #flag_name) -> bool {
				self.bits & flag.bits == flag.bits
			}
			
			/// Check if a given flag(s) is/are set (if multiple flags are checked, all flags need to be set).
			#vis const fn contains(&self, flag: #flag_name) -> bool {
				self.bits & flag.bits == flag.bits
			}
			
			/// Check if any of the given flags are set.
			#[deprecated = "Use `intersects` instead"]
			#vis const fn is_any_set(&self, flag: #flag_name) -> bool {
				self.bits & flag.bits != 0
			}

			/// Check if any of the given flags are set.
			#vis const fn intersects(&self, flag: #flag_name) -> bool {
				self.bits & flag.bits != 0
			}

			/// Check if no flag is set.
			#vis const fn is_none(&self) -> bool {
				self.bits == 0
			}

			/// Check if any flag is set.
			#vis const fn is_any(&self) -> bool {
				self.bits != 0
			}

			/// Check if all valid flags are set.
			#vis const fn is_all(&self) -> bool {
				self.bits == Self::all().bits
			}

			/// Check if exactly 1 is set
			#vis const fn is_single_bit_set(&self) -> bool {
				self.bits.count_ones() == 1
			}

			/// Set the state of a given flag to `set`.
			#vis fn set(&mut self, flag: #flag_name, set: bool) {
				if set {
					self.bits |= flag.bits;
				} else {
					self.bits &= !flag.bits;
				}
			}

			/// Enable a given flag.
			#vis fn enable(&mut self, flag: #flag_name) {
				self.bits |= flag.bits;
			}

			/// Disable a given flag.
			#vis fn disable(&mut self, flag: #flag_name) {
				self.bits &= !flag.bits;
			}

			/// Const implementation of not
			#vis const fn not(self) -> Self {
				Self { bits: !self.bits }
			}

			/// Const implementation of bitand
			#vis const fn bitand(self, rhs: Self) -> Self {
				Self { bits: self.bits & rhs.bits }
			}

			/// Const implementation of bitor
			#vis const fn bitor(self, rhs: Self) -> Self {
				Self { bits: self.bits | rhs.bits }
			}

			// Const implementation of bitxor
			#vis const fn bitxor(self, rhs: Self) -> Self {
				Self { bits: self.bits ^ rhs.bits }
			}

			#parse
		}

		impl ::core::ops::Not for #flag_name {
			type Output = Self;
			fn not(self) -> Self {
				Self{ bits: !self.bits }
			}
		}

		impl ::core::ops::BitAnd for #flag_name {
			type Output = Self;
			fn bitand(self, rhs: Self) -> Self {
				Self { bits: self.bits.bitand(rhs.bits) }
			}
		}

		impl ::core::ops::BitAndAssign for #flag_name {
			fn bitand_assign(&mut self, rhs: Self) {
				self.bits.bitand_assign(rhs.bits);
			}
		}

		impl ::core::ops::BitOr for #flag_name {
			type Output = Self;
			fn bitor(self, rhs: Self) -> Self {
				Self { bits: self.bits.bitor(rhs.bits) }
			}
		}

		impl ::core::ops::BitOrAssign for #flag_name {
			fn bitor_assign(&mut self, rhs: Self) {
				self.bits.bitor_assign(rhs.bits);
			}
		}

		impl ::core::ops::BitXor for #flag_name {
			type Output = Self;
			fn bitxor(self, rhs: Self) -> Self {
				Self { bits: self.bits.bitxor(rhs.bits) }
			}
		}

		impl ::core::ops::BitXorAssign for #flag_name {
			fn bitxor_assign(&mut self, rhs: Self) {
				self.bits.bitxor_assign(rhs.bits);
			}
		}

		impl From<#base_type> for #flag_name {
			fn from(bits: #base_type) -> Self {
				#flag_name { bits }
			}
		}

		impl From<#flag_name> for #base_type {
			fn from(val: #flag_name) -> #base_type {
				val.bits
			}
		}

		impl Default for #flag_name {
			fn default() -> #flag_name {
				#flag_name::none()
			}
		}

		impl ::core::fmt::Debug for #flag_name {
			fn fmt(&self, f: &mut ::core::fmt::Formatter<'_>) -> ::core::fmt::Result {
				use core::fmt::Write;

				if self.is_none() {
					if f.alternate() {
						write!(f, "{}::", stringify!(#flag_name))?;
					}
					write!(f, #none_name)?;
					return Ok(());
				}

				let mut flags = *self;
				let mut started = false;
				#(
					if flags.contains(#flag_name::#idents) {
						if started {
							write!(f, " | ")?;
						}
						if f.alternate() {
							write!(f, "{}::", stringify!(#flag_name))?;
						}
						write!(f, stringify!(#idents))?;
						flags &= !#flag_name::#idents;
						started = true;
					}
				)*

				if flags.is_any() {
					if started {
						write!(f, " | ")?;
					}
					write!(f, "{:o}", flags.bits)?;
				}

				Ok(())
			}
		}

		/// Implicitly implemtents onca_common::string::ToString
		impl ::core::fmt::Display for #flag_name {
			fn fmt(&self, mut f: &mut ::core::fmt::Formatter<'_>) -> ::core::fmt::Result {
				::core::fmt::Debug::fmt(&self, &mut f)
			}
		}
	).into()
}

fn create_expr_from_path(path: Path) -> core::result::Result<syn::Expr, &'static str> {
	if path.segments.len() > 1 {
		return Err("Only single identifiers are allowed");
	}

	let mut segments = syn::punctuated::Punctuated::new();
	segments.push(PathSegment{ ident: syn::Ident::new("Self", proc_macro2::Span::mixed_site()), arguments: PathArguments::None });

	let segment = match path.segments.last() {
	    Some(segment) => segment.clone(),
	    None => return Err("Invalid path"),
	};
	segments.push(segment);

	let new_path = Path{ leading_colon: None, segments };

	Ok(syn::Expr::Path(syn::ExprPath{ path: new_path, qself: None, attrs: Default::default() }))
}

fn int_to_lit_expr(val: u128, base_type: Type) -> Expr {
	let lit_expr = Expr::Lit(ExprLit {
		lit: Lit::Int(LitInt::new(&val.to_string(), Span::mixed_site())),
		attrs: Default::default(),
	});

	cast_to_base_type(lit_expr, base_type).0
}

fn cast_to_base_type(expr: Expr, base_type: Type) -> (Expr, Option<u128>) {
	let int = if let Expr::Lit(ExprLit{ lit: Lit::Int(lit), .. }) = &expr {
		Some(lit.base10_parse().unwrap())
	} else {
		None
	};

	(Expr::Cast(ExprCast {
		expr: Box::new(expr),
		as_token: Default::default(),
		ty: Box::new(base_type),
		attrs: Default::default(),
	}), int)
}

fn get_bits_from_expr(expr: Expr, base_type: Type) -> Expr {
	let bits = Expr::Field(ExprField {
		attrs: Default::default(), 
		base: Box::new(expr),
		dot_token: Default::default(),
		member: Member::Named(Ident::new("bits", Span::mixed_site()))
	});
	cast_to_base_type(bits, base_type).0
}

fn construct_flag(bits_val: Expr) -> Expr {
	let mut args = syn::punctuated::Punctuated::new();
	args.push(bits_val);

	let mut segments = syn::punctuated::Punctuated::new();
	segments.push(PathSegment {
	    ident: Ident::new("Self", proc_macro2::Span::mixed_site()),
	    arguments: PathArguments::None,
	});
	segments.push(PathSegment {
	    ident: Ident::new("new_u128", proc_macro2::Span::mixed_site()),
	    arguments: PathArguments::None,
	});

	let func = Expr::Path(ExprPath {
    attrs: Default::default(),
    qself: Default::default(),
    path: Path {
	        leading_colon: Default::default(),
	        segments,
	    },
	});

	Expr::Call(ExprCall {
    attrs: Default::default(),
    func: Box::new(func),
    paren_token: Default::default(),
    args,
	})
}

fn gen_bits_val_expr(expr: Expr, flag_name: &Ident, base_type: &Type) -> core::result::Result<(Expr, Option<u128>), TokenStream> {
	match expr {
		lit_expr @ Expr::Lit(ExprLit{ lit: Lit::Int(_), .. }) => {
			Ok(cast_to_base_type(lit_expr, base_type.clone()))
		},
		Expr::Path(path) => {
			let res = create_expr_from_path(path.path);
			match res {
			   Ok(path_expr) => Ok((get_bits_from_expr(path_expr, base_type.clone()), None)),
			   Err(err) => Err(quote!( compile_error!(#err); )),
			}
		},
		Expr::Binary(bin_expr) => {
			create_ored_expr(bin_expr, flag_name, base_type).map(|expr| (expr, None))
		}
		_ => Err(quote!( compile_error!("Only integer literals or single paths are supported"); )),
	}
}

fn create_expr_from_lit(i: u128, base_type: Type) -> Result<Expr> {
	Ok(construct_flag(int_to_lit_expr(i, base_type)))
}

fn create_ored_expr(bin_expr: ExprBinary, flag_name: &Ident, base_type: &Type) -> core::result::Result<Expr, TokenStream> {
	if let ExprBinary{ left, right, op: BinOp::BitOr(_), .. } = bin_expr {

		let left_expr = gen_bits_val_expr(*left, flag_name, base_type)?;
		let right_expr = gen_bits_val_expr(*right, flag_name, base_type)?;

		Ok(Expr::Binary(ExprBinary{ 
			left: Box::new(left_expr.0),
			op: BinOp::BitOr(Default::default()),
			right: Box::new(right_expr.0),
			attrs: Default::default() 
		}))
	} else {
		Err(quote!( compile_error!("Unsupported expression component"); ))
	}
}