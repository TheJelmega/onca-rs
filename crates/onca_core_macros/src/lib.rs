mod flags;

use proc_macro::*;

#[proc_macro_attribute]
pub fn flags(args: TokenStream, input: TokenStream) -> TokenStream
{
    flags::flags(args, input)
}