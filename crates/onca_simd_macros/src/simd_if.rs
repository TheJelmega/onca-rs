use proc_macro::TokenStream;

pub fn simd_else_scalar(_args: TokenStream, input: TokenStream) -> TokenStream
{
    let parsed_input = syn::parse::<syn::DeriveInput>(input).unwrap();
    quote::quote!{
        else {
            #parsed_input
        }
    }.into()
}

pub fn simd_if_sse(_args: TokenStream, input: TokenStream) -> TokenStream
{
    let parsed_input = syn::parse::<syn::DeriveInput>(input).unwrap();
    quote::quote!{
        #[cfg(target_feature = "sse4.2")]
        if onca_simd::has_intrin(BackendType::SSE) {
            #parsed_input
        }
    }.into()
}

pub fn simd_else_if_sse(_args: TokenStream, input: TokenStream) -> TokenStream
{
    let parsed_input = syn::parse::<syn::DeriveInput>(input).unwrap();
    quote::quote!{
        #[cfg(target_feature = "sse4.2")]
        else if onca_simd::has_intrin(BackendType::SSE) {
            #parsed_input
        }
    }.into()
}

pub fn simd_if_avx(_args: TokenStream, input: TokenStream) -> TokenStream
{
    let parsed_input = syn::parse::<syn::DeriveInput>(input).unwrap();
    quote::quote!{
        #[cfg(target_feature = "avx")]
        if onca_simd::has_intrin(BackendType::AVX) {
            #parsed_input
        }
    }.into()
}

pub fn simd_else_if_avx(_args: TokenStream, input: TokenStream) -> TokenStream
{
    let parsed_input = syn::parse::<syn::DeriveInput>(input).unwrap();
    quote::quote!{
        #[cfg(target_feature = "avx")]
        else if onca_simd::has_intrin(BackendType::AVX) {
            #parsed_input
        }
    }.into()
}

pub fn simd_if_avx2(_args: TokenStream, input: TokenStream) -> TokenStream
{
    let parsed_input = syn::parse::<syn::DeriveInput>(input).unwrap();
    quote::quote!{
        #[cfg(target_feature = "avx2")]
        if onca_simd::has_intrin(BackendType::AVX2) {
            #parsed_input
        }
    }.into()
}

pub fn simd_else_if_avx2(_args: TokenStream, input: TokenStream) -> TokenStream
{
    let parsed_input = syn::parse::<syn::DeriveInput>(input).unwrap();
    quote::quote!{
        #[cfg(target_feature = "avx2")]
        else if onca_simd::has_intrin(BackendType::AVX2) {
            #parsed_input
        }
    }.into()
}

pub fn simd_if_avx512(_args: TokenStream, input: TokenStream) -> TokenStream
{
    let parsed_input = syn::parse::<syn::DeriveInput>(input).unwrap();
    quote::quote!{
        // TODO(jel): figure out correct avx512 flags
        #[cfg(target_feature = "avx512")]
        if onca_simd::has_intrin(BackendType::AVX512) {
            #parsed_input
        }
    }.into()
}