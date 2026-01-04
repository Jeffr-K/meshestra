use proc_macro::TokenStream;
use quote::quote;
use syn::{parse::Parse, parse::ParseStream, parse_macro_input, ItemStruct, Token};

struct InterceptorArgs {
    interceptors: Vec<syn::Expr>,
}

impl Parse for InterceptorArgs {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let paths = input.parse_terminated(syn::Expr::parse, Token![,])?;
        Ok(InterceptorArgs {
            interceptors: paths.into_iter().collect(),
        })
    }
}

pub fn interceptor_attribute(attr: TokenStream, item: TokenStream) -> TokenStream {
    let args = parse_macro_input!(attr as InterceptorArgs);
    let input = parse_macro_input!(item as ItemStruct);

    let struct_name = &input.ident;
    let interceptors = &args.interceptors;

    // Generate code to implement a trait or method providing interceptors
    // We assume the Controller trait might be enhanced to support `get_interceptors()`?
    // Or we simply implement a standalone method `interceptors` as requested in spec.
    /*
        impl #struct_name {
            pub fn interceptors() -> Vec<Box<dyn ::meshestra::interceptor::Interceptor>> {
                vec![
                    #(Box::new(#interceptors)),*
                ]
            }
        }
    */
    // Problem: Box::new(#path) assumes #path is a unit struct or constructor.
    // If it's a struct `LoggingInterceptor`, Box::new(LoggingInterceptor) works.
    // If it needs arguments, the user handles it? Spec says `#[interceptor(LoggingInterceptor)]`.
    // If user wants `CacheInterceptor::new(60)`, the spec says: `#[interceptor(CacheInterceptor::new(Duration::from_secs(60)))]`.
    // My parser expects `Path`. `CacheInterceptor::new(...)` is an `Expr`.
    // So InterceptorArgs should parse `Expr`?

    // Let's change parsing logic to accept Expressions.

    let interceptor_impl = quote! {
        impl #struct_name {
            pub fn interceptors() -> Vec<Box<dyn ::meshestra::interceptor::Interceptor>> {
                vec![
                    #(Box::new(#interceptors)),*
                ]
            }
        }
    };

    TokenStream::from(quote! {
        #input
        #interceptor_impl
    })
}
