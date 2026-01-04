use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, ItemFn, ItemStruct};

pub fn exception_filter_attribute(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let input = parse_macro_input!(item as ItemStruct);

    // We just mark it for now, or we could generate the 'ExceptionFilter' trait impl
    // if we wanted to enforce structure.
    // For now, it mostly acts as a marker or Injectable.

    quote! {
        #[derive(::meshestra::Injectable)]
        #input
    }
    .into()
}

pub fn handle_attribute(_attr: TokenStream, item: TokenStream) -> TokenStream {
    // This is used on methods inside an exception filter.
    // It is primarily metadata. The real registration logic usually happens in
    // the filters 'new' or 'init' or via reflection if we had it.
    // In Rust, we might rely on the user to register these handlers,
    // or generate a 'register_handlers' method.

    // For this implementation, we'll let it pass through but ensure it's valid.
    let input = parse_macro_input!(item as ItemFn);

    // We could parse 'attr' to get the exception type: #[handle(UserError)]

    quote! {
        #input
    }
    .into()
}
