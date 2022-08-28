use proc_macro::TokenStream;

mod simd_if;

#[proc_macro_attribute]
pub fn simd_else_scalar(args: TokenStream, input: TokenStream) -> TokenStream
{
    simd_if::simd_else_scalar(args, input)
}

#[proc_macro_attribute]
pub fn simd_if_sse(args: TokenStream, input: TokenStream) -> TokenStream
{
    simd_if::simd_if_sse(args, input)
}

#[proc_macro_attribute]
pub fn simd_else_if_sse(args: TokenStream, input: TokenStream) -> TokenStream
{
    simd_if::simd_else_if_sse(args, input)
}

#[proc_macro_attribute]
pub fn simd_if_avx(args: TokenStream, input: TokenStream) -> TokenStream
{
    simd_if::simd_if_avx(args, input)
}

#[proc_macro_attribute]
pub fn simd_else_if_avx(args: TokenStream, input: TokenStream) -> TokenStream
{
    simd_if::simd_else_if_avx(args, input)
}

#[proc_macro_attribute]
pub fn simd_if_avx2(args: TokenStream, input: TokenStream) -> TokenStream
{
    simd_if::simd_if_avx2(args, input)
}

#[proc_macro_attribute]
pub fn simd_else_if_avx2(args: TokenStream, input: TokenStream) -> TokenStream
{
    simd_if::simd_else_if_avx2(args, input)
}

#[proc_macro_attribute]
pub fn simd_if_avx512(args: TokenStream, input: TokenStream) -> TokenStream
{
    simd_if::simd_if_avx512(args, input)
}

// TODO(jel)
//#[proc_macro_attribute]
//pub fn simd_dynamic_dispatch(args: TokenStream, input: TokenStream) -> TokenStream
//{
//    todo!()
//}