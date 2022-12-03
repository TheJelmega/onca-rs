use proc_macro::*;
//use proc_macro2 as pm2;
//use quote::*;
use syn::*;

pub fn flags(args: TokenStream, input: TokenStream) -> TokenStream
{
	let annotated_parsed_res = syn::parse::<syn::Type>(args);
	let base_type = match annotated_parsed_res
	{
		Ok(typ) => typ,
		Err(_) => syn::parse_str::<Type>("u32").unwrap() 	
	};

	let input_parsed = parse_macro_input!(input as DeriveInput);
	let vis = input_parsed.vis;
	let name = input_parsed.ident;
	let enum_attrs = input_parsed.attrs;

	let body_data = match input_parsed.data
	{
		Data::Enum(body) => body,
		_ => panic!("Not an enum")	
	};

	let mut idents = Vec::<syn::Ident>::new();
	let mut vals = Vec::<u128>::new();
	let mut attrs = Vec::<Vec::<Attribute>>::new();
	let mut i : u128 = 1;
	for it in body_data.variants.into_iter()
	{
		idents.push(it.ident);
		attrs.push(it.attrs);
		match it.discriminant
		{
			Some((_, expr)) =>
			{
				if let Expr::Lit(ExprLit{ lit: Lit::Int(ref lit_int), .. }) = &expr
				{
					i = lit_int.base10_parse().expect("Invalid literal");
					vals.push(i);
					i <<= 1;
				}
				else
				{
					panic!("Only integer literals are supported")
				}
			},
			None =>
			{
				assert!(i == 0 || i.is_power_of_two(), "Previous enum value needs to be a power of 2");
				vals.push(i);
				i <<= 1;
			}
		};
	}

	quote::quote!(

		#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
		#(#enum_attrs)*
		#vis struct #name {
			bits : #base_type
		}

		#[allow(non_upper_case_globals)]
		impl #name {
			/// Value representing that no flag is set.
			#vis const None : #name = #name::none();
			
			#(#(#attrs)* #vis const #idents : #name = #name{ bits: #vals as #base_type };)*

			#vis const fn none() -> Self {
				Self { bits: 0 }
			}

			#vis const fn all() -> Self {
				const bits : #base_type = 0 #( | #name::#idents.bits)*;
				Self { bits: 0 }
			}

			#vis const fn bits(&self) -> #base_type {
				self.bits
			}

			#vis const fn is_set(&self, flag: #name) -> bool {
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

			#vis fn set(&mut self, flag: #name, set: bool) {
				if set {
					self.bits |= flag.bits;
				} else {
					self.bits &= !flag.bits;
				}
			}
		}

		impl ::core::ops::Not for #name {
			type Output = Self;
			fn not(self) -> Self {
				Self{ bits: !self.bits }
			}
		}

		impl ::core::ops::BitAnd for #name {
			type Output = Self;
			fn bitand(self, rhs: Self) -> Self {
				Self{ bits: self.bits.bitand(rhs.bits) }
			}
		}

		impl ::core::ops::BitAndAssign for #name {
			fn bitand_assign(&mut self, rhs: Self) {
				self.bits.bitand_assign(rhs.bits);
			}
		}

		impl ::core::ops::BitOr for #name {
			type Output = Self;
			fn bitor(self, rhs: Self) -> Self {
				Self{ bits: self.bits.bitor(rhs.bits) }
			}
		}

		impl ::core::ops::BitOrAssign for #name {
			fn bitor_assign(&mut self, rhs: Self) {
				self.bits.bitor_assign(rhs.bits);
			}
		}

		impl ::core::ops::BitXor for #name {
			type Output = Self;
			fn bitxor(self, rhs: Self) -> Self {
				Self{ bits: self.bits.bitxor(rhs.bits) }
			}
		}

		impl ::core::ops::BitXorAssign for #name {
			fn bitxor_assign(&mut self, rhs: Self) {
				self.bits.bitxor_assign(rhs.bits);
			}
		}

		impl From<#base_type> for #name {
			fn from(bits: #base_type) -> Self {
				#name{ bits }
			}
		}

		impl From<#name> for #base_type {
			fn from(val: #name) -> #base_type {
				val.bits
			}
		}

		impl Default for #name {
			fn default() -> #name {
				#name::None
			}
		}

		impl ::core::fmt::Debug for #name {
			fn fmt(&self, f: &mut ::core::fmt::Formatter<'_>) -> ::core::fmt::Result {
				let mut flags = *self;
				let mut started = false;

				#(
					if flags.is_set(#name::#idents) {
						if started {
							f.write_str(" | ")?;
						}
						f.write_str(stringify!(#idents))?;
						flags &= !#name::#idents;
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
		impl ::core::fmt::Display for #name {
			fn fmt(&self, mut f: &mut ::core::fmt::Formatter<'_>) -> ::core::fmt::Result {
				::core::fmt::Debug::fmt(&self, &mut f)
			}
		}
	).into()
}