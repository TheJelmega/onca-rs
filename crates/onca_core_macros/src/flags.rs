use proc_macro2::*;
use quote::quote;
use syn::*;

pub fn flags(args: TokenStream, input: TokenStream) -> TokenStream
{
	let annotated_parsed_res = syn::parse2::<syn::Type>(args);
	let base_type = match annotated_parsed_res
	{
		Ok(typ) => typ,
		Err(_) => syn::parse_str::<Type>("u32").unwrap() 	
	};

	// While we don't exactly are deriving, the `#[flags]` macro is close enough
	let parsed_res = syn::parse2::<DeriveInput>(input);
	let input_parsed = match parsed_res {
	    Ok(derived_input) => derived_input,
	    Err(err) => return err.to_compile_error().into(),
	};

	let vis = input_parsed.vis;
	let flag_name = input_parsed.ident;
	let enum_attrs = input_parsed.attrs;

	let body_data = match input_parsed.data
	{
		Data::Enum(body) => body,
		_ => return quote!( compile_error!("Not an enum"); )
	};

	
	let mut idents = Vec::<syn::Ident>::new();
	let mut vals = Vec::<syn::Expr>::new();
	let mut attrs = Vec::<Vec::<Attribute>>::new();
	let mut i : u128 = 1;
	let mut has_zero = false;
	for it in body_data.variants.into_iter()
	{
		idents.push(it.ident);
		attrs.push(it.attrs);
		match it.discriminant
		{
			Some((_, expr)) =>
			{
				let res = gen_bits_val_expr(expr, &flag_name, &base_type);
				let (bits_val, int) = match res {
    			    Ok(bits_val) => bits_val,
    			    Err(toks) => return toks,
    			};
				vals.push(construct_flag(flag_name.clone(), bits_val));

				if let Some(int) = int {
					if int == 0 {
						has_zero = true;
					}
					i = int << 1;
				}
			},
			None =>
			{
				if i == 0 {
					i = 1;
				} else if !i.is_power_of_two() {
					return quote!( compile_error!("Previous enum value needs to be a power of 2"); );
				}

				match create_expr_from_lit(flag_name.clone(), i, base_type.clone()) {
					Ok(val) => vals.push(val),
					Err(err) => return err.to_compile_error().into(),
				}
				i <<= 1;
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

	quote!(

		#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
		#(#enum_attrs)*
		#vis struct #flag_name {
			bits : #base_type
		}
		#[allow(non_upper_case_globals)]
		impl #flag_name {
			#non_variant
			
			#(#(#attrs)* #vis const #idents : #flag_name = #vals;)*

			// Helper function for integer literals, as `#flag_name{ bits: #i as #base_type }` didn't seem to work in `create_expr_from_lit`
			const fn new(val: #base_type) -> Self {
				Self { bits: val }
			}

			#vis const fn none() -> Self {
				Self { bits: 0 }
			}

			#vis const fn all() -> Self {
				const bits : #base_type = 0 #( | #flag_name::#idents.bits)*;
				Self { bits }
			}

			#vis const fn bits(&self) -> #base_type {
				self.bits
			}

			#vis const fn is_set(&self, flag: #flag_name) -> bool {
				self.bits & flag.bits == flag.bits
			}

			#vis const fn is_none(&self) -> bool {
				self.bits == 0
			}

			#vis const fn is_any(&self) -> bool {
				self.bits != 0
			}

			#vis const fn is_all(&self) -> bool {
				self.bits == Self::all().bits
			}

			#vis fn set(&mut self, flag: #flag_name, set: bool) {
				if set {
					self.bits |= flag.bits;
				} else {
					self.bits &= !flag.bits;
				}
			}
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
				Self{ bits: self.bits.bitand(rhs.bits) }
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
				Self{ bits: self.bits.bitor(rhs.bits) }
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
				Self{ bits: self.bits.bitxor(rhs.bits) }
			}
		}

		impl ::core::ops::BitXorAssign for #flag_name {
			fn bitxor_assign(&mut self, rhs: Self) {
				self.bits.bitxor_assign(rhs.bits);
			}
		}

		impl From<#base_type> for #flag_name {
			fn from(bits: #base_type) -> Self {
				#flag_name{ bits }
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
				let mut flags = *self;
				let mut started = false;

				#(
					if flags.is_set(#flag_name::#idents) {
						if started {
							f.write_str(" | ")?;
						}
						f.write_str(stringify!(#idents))?;
						flags &= !#flag_name::#idents;
						started = true;
					}
				)*

				if flags.is_any() {
					if started {
						f.write_str(" | ")?;
					}
					f.write_fmt(format_args!("{:o}", flags.bits))?;
				}

				Ok(())
			}
		}

		/// Implicitly implemtents onca_core::string::ToString
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

fn get_bits_from_expr(expr: Expr) -> Expr {
	Expr::Field(ExprField {
		attrs: Default::default(), 
		base: Box::new(expr),
		dot_token: Default::default(),
		member: Member::Named(Ident::new("bits", Span::mixed_site()))
	})
}

fn ident_to_path(ident: Ident) -> Path {
	let mut segments = syn::punctuated::Punctuated::new();
	segments.push(PathSegment{ ident, arguments: PathArguments::None });
	Path{ leading_colon: None, segments, }
}

fn construct_flag(flag_name: Ident, bits_val: Expr) -> Expr {
	let mut fields = syn::punctuated::Punctuated::new();
	fields.push(FieldValue{
		member: Member::Named(Ident::new("bits", Span::mixed_site())),
	    expr: bits_val,
	    colon_token: Some(Default::default()),
	    attrs: Default::default(),
	});

	Expr::Struct(ExprStruct{
		path: ident_to_path(flag_name),
		brace_token: Default::default(),
		fields,
		dot2_token: None,
		rest: None,
		attrs: Default::default(),
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
			   Ok(path_expr) => Ok((get_bits_from_expr(path_expr), None)),
			   Err(err) => Err(quote!( compile_error!(#err); )),
			}
		},
		Expr::Binary(bin_expr) => {
			create_ored_expr(bin_expr, flag_name, base_type).map(|expr| (expr, None))
		}
		_ => Err(quote!( compile_error!("Only integer literals or single paths are supported"); )),
	}
}

fn create_expr_from_lit(flag_name: Ident, i: u128, base_type: Type) -> Result<Expr> {
	Ok(construct_flag(flag_name, int_to_lit_expr(i, base_type)))
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