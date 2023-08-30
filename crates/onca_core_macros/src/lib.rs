mod flags;
mod derive;

use proc_macro::TokenStream;

#[proc_macro_attribute]
pub fn flags(args: TokenStream, input: TokenStream) -> TokenStream {
    flags::flags(args.into(), input.into()).into()
}

#[proc_macro_derive(EnumCount)]
pub fn enum_count(item: TokenStream) -> TokenStream {
    derive::enum_count(item.into()).into()
}

#[proc_macro_derive(EnumFromIndex)]
pub fn enum_from_index(item: TokenStream) -> TokenStream {
    derive::enum_from_index(item.into()).into()
}


#[proc_macro_derive(EnumDisplay, attributes(display))]
pub fn enum_display(item: TokenStream) -> TokenStream {
    derive::enum_display(item.into()).into()
}