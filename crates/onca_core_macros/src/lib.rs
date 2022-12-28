mod flags;

use proc_macro::TokenStream;

#[proc_macro_attribute]
pub fn flags(args: TokenStream, input: TokenStream) -> TokenStream
{
    flags::flags(args.into(), input.into()).into()
}