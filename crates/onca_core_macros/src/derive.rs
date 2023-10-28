use proc_macro2::*;
use quote::quote;
use syn::*;

pub fn enum_count(item: TokenStream) -> TokenStream {
	let parsed_res = syn::parse2::<DeriveInput>(item);
	let input_parsed = match parsed_res {
	    Ok(derived_input) => derived_input,
	    Err(err) => return err.to_compile_error().into(),
	};

    let body_data = match input_parsed.data {
		Data::Enum(body) => body,
		_ => return quote!( compile_error!("Not an enum"); )
	};

    let ident = input_parsed.ident;
    let count = body_data.variants.len();

    quote!{
        impl onca_core::utils::EnumCountT for #ident {
            const COUNT: usize = #count;
        }
    }
}


pub fn enum_from_index(item: TokenStream) -> TokenStream {
    let parsed_res = syn::parse2::<DeriveInput>(item);
	let input_parsed = match parsed_res {
	    Ok(derived_input) => derived_input,
	    Err(err) => return err.to_compile_error().into(),
	};

    let body_data = match input_parsed.data {
		Data::Enum(body) => body,
		_ => return quote!( compile_error!("Not an enum"); )
	};

    let ident = input_parsed.ident;

    let variants = body_data.variants.iter().map(|variant| &variant.ident).collect::<Vec<_>>();
    let mut indices = Vec::with_capacity(variants.len());
    for i in 0..variants.len() {
        indices.push(i);
    }

    quote!{
        impl onca_core::utils::EnumFromIndexT for #ident {
            fn from_idx(idx: usize) -> Option<Self> {
                match idx {
                    #(#indices => Some(Self::#variants),)*
                    _ => None,
                }
            }

            unsafe fn from_idx_unchecked(idx: usize) -> Self {
                match idx {
                    #(#indices => Self::#variants,)*
                    _ => unreachable!()
                }
            }
        }
    }
}

pub fn enum_display(item: TokenStream) -> TokenStream {
    let parsed_res = syn::parse2::<DeriveInput>(item);
	let input_parsed = match parsed_res {
	    Ok(derived_input) => derived_input,
	    Err(err) => return err.to_compile_error().into(),
	};

    let body_data = match input_parsed.data {
		Data::Enum(body) => body,
		_ => return quote!( compile_error!("Not an enum"); )
	};

    let ident = input_parsed.ident;

    let mut members = Vec::with_capacity(body_data.variants.len());
    let mut names = Vec::with_capacity(body_data.variants.len());

    for variant in &body_data.variants {
        members.push(variant.ident.clone());
        let val = variant.attrs.iter()
        .filter(|attr| attr.path.get_ident().map_or(false, |ident| ident.to_string() == "display"))
        .map(|attr| attr.parse_args::<LitStr>().map_or(String::new(), |parsed| parsed.value()))
        .nth(0)
        .unwrap_or(variant.ident.to_string());
        names.push(val);
    }

    quote!{
        impl core::fmt::Display for #ident {
            fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
                match self {
                    #(#ident::#members => #names.fmt(f),)*
                }
            }
        }
    }
}