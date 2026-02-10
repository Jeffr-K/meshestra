use proc_macro::TokenStream;
use proc_macro2::TokenStream as TokenStream2;
use quote::quote;
use syn::{parse_macro_input, Data, DeriveInput, Fields, GenericArgument, PathArguments, Type};

pub fn derive_injectable(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let expanded = generate_injectable_impl(&input);
    TokenStream::from(expanded)
}

fn generate_injectable_impl(input: &DeriveInput) -> TokenStream2 {
    let struct_name = &input.ident;
    let (impl_generics, ty_generics, where_clause) = input.generics.split_for_impl();

    let fields = match &input.data {
        Data::Struct(s) => match &s.fields {
            Fields::Named(f) => &f.named,
            _ => panic!("#[derive(Injectable)] only supports structs with named fields."),
        },
        _ => panic!("#[derive(Injectable)] can only be used on structs."),
    };

    let field_injections = fields.iter().map(|field| {
        let field_name = field.ident.as_ref().unwrap();
        let field_ty = &field.ty;

        // Check for `Lazy<T>`
        if let Some(_inner_type) = get_generic_type(field_ty, "Lazy") {
            return quote! {
                #field_name: ::meshestra::Lazy::new(container)
            };
        }

        // Check for `Arc<T>`
        if let Some(inner_type) = get_generic_type(field_ty, "Arc") {
            // Check if inner type is `dyn Trait`
            if let Type::TraitObject(_) = inner_type {
                return quote! {
                    #field_name: container.resolve_trait::<#inner_type>()?
                };
            }
            // Otherwise, it's a concrete type
            return quote! {
                #field_name: container.resolve::<#inner_type>()?
            };
        }

        // Default case: assume it's a concrete type to be resolved directly
        quote! {
            #field_name: container.resolve::<#field_ty>()?
        }
    });

    quote! {
        impl #impl_generics ::meshestra::Injectable for #struct_name #ty_generics #where_clause {
            fn inject(container: &::meshestra::Container) -> ::meshestra::Result<Self> {
                Ok(Self {
                    #(#field_injections),*
                })
            }
        }
    }
}

/// Helper to extract the inner type from a generic wrapper like `Arc<T>` or `Lazy<T>`.
/// Returns `Some(T)` if `ty` matches `wrapper_name<T>`, otherwise `None`.
fn get_generic_type<'a>(ty: &'a Type, wrapper_name: &str) -> Option<&'a Type> {
    if let Type::Path(type_path) = ty {
        if let Some(segment) = type_path.path.segments.last() {
            if segment.ident == wrapper_name {
                if let PathArguments::AngleBracketed(args) = &segment.arguments {
                    if let Some(GenericArgument::Type(inner_type)) = args.args.first() {
                        return Some(inner_type);
                    }
                }
            }
        }
    }
    None
}
