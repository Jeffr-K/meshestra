use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, Item};

pub fn aspect_attribute(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let input = parse_macro_input!(item as Item);

    quote! {
        #input
    }
    .into()
}
