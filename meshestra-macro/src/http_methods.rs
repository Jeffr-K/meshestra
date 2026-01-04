use proc_macro::TokenStream;
use quote::quote;
use syn::parse_macro_input;

pub fn http_method_attribute(_method: &str, _attr: TokenStream, item: TokenStream) -> TokenStream {
    // For now, just pass through the item
    // The actual routing logic will be implemented when we parse impl blocks
    let input = parse_macro_input!(item as syn::ImplItemFn);

    TokenStream::from(quote! {
        #input
    })
}
