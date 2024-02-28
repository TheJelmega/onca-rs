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
        impl onca_base::EnumCountT for #ident {
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

    let mut variants = Vec::with_capacity(body_data.variants.len());
    let mut indices = Vec::with_capacity(body_data.variants.len());
    let mut i = 0;
    for variant in body_data.variants { 
        let idx = match variant.discriminant {
            Some((_, expr)) => match expr {
                Expr::Lit(lit) => match lit.lit {
                    Lit::Int(int) => match int.base10_parse::<usize>() {
                        Ok(int) => int,
                        Err(_) => match int.base10_parse::<isize>() {
                            Ok(int) => int as usize,
                            Err(err) => {
                                let msg = err.to_string();
                                return quote!(compile_error!(#msg));
                            },
                        },
                    },
                    _ => return quote!(compile_error!("Only integer descriminants are supported by EnumFromIndex")),
                },
                _ => return quote!(compile_error!("Only integer descriminants are supported by EnumFromIndex")),
            },
            None => i,
        };
        
        variants.push(variant.ident);
        indices.push(idx);

        i = idx + 1;
    }

    quote!{
        impl onca_base::EnumFromIndexT for #ident {
            fn from_idx(idx: usize) -> Option<Self> {
                match idx {
                    #(#indices => Some(Self::#variants),)*
                    _ => None,
                }
            }
            
            fn from_idx_or(idx: usize, default: Self) -> Self {
                match idx {
                    #(#indices => Self::#variants,)*
                    _ => default,
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
        .filter(|attr| attr.path().get_ident().map_or(false, |ident| ident.to_string() == "display"))
        .map(|attr| attr.parse_args::<LitStr>().map_or_else(|err| err.to_compile_error(), |parsed| {
            let val = parsed.value();
            quote!(#val)
        }))
        .nth(0)
        .unwrap_or_else(|| {
            let val =variant.ident.to_string();
            quote!(#val)
        });
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

pub fn enum_from_name(item: TokenStream) -> TokenStream {
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
        .filter(|attr| attr.path().get_ident().map_or(false, |ident| ident.to_string() == "parse_name"))
        .map(|attr| attr.parse_args::<LitStr>().map_or_else(|err| err.to_compile_error(), |parsed| {
            let val = parsed.value();
            quote!(#val)
        }))
        .nth(0)
        .unwrap_or_else(|| {
            let val =variant.ident.to_string();
            quote!(#val)
        });
        names.push(val);
    }

    quote!{
        impl onca_base::EnumFromNameT for #ident {
            fn parse(s: &str) -> Option<Self> {
                match s {
                    #(#names => Some(Self::#members),)*
                    _ => None,
                }
            }
        }
    }
}