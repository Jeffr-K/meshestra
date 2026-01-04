use proc_macro::TokenStream;
use proc_macro2::TokenStream as TokenStream2;
use quote::quote;
use syn::{parse_macro_input, Data, DeriveInput, Fields, Type};

pub fn derive_injectable(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);

    let expanded = generate_injectable_impl(&input);
    TokenStream::from(expanded)
}

fn generate_injectable_impl(input: &DeriveInput) -> TokenStream2 {
    let struct_name = &input.ident;
    let (impl_generics, ty_generics, where_clause) = input.generics.split_for_impl();

    // Extract fields from the struct
    let fields = match &input.data {
        Data::Struct(data) => match &data.fields {
            Fields::Named(fields) => &fields.named,
            _ => panic!("#[derive(Injectable)] only supports structs with named fields"),
        },
        _ => panic!("#[derive(Injectable)] can only be applied to structs"),
    };

    // Generate field injection code
    let field_injections = fields.iter().map(|field| {
        let field_name = &field.ident;
        let field_type = extract_injectable_type(&field.ty);

        let resolve_method = match &field_type {
            Type::TraitObject(_) => quote!(resolve_trait),
            // Handle Type::Path that starts with `dyn` keyword? 
            // syn parses `dyn Trait` as TraitObject.
            // But sometimes old syntax `Box<Trait>` works.
            // We assume standard `dyn`.
            _ => quote!(resolve),
        };

        quote! {
            #field_name: container.#resolve_method::<#field_type>()?
        }
    });

    quote! {
        impl #impl_generics ::meshestra::Injectable for #struct_name #ty_generics #where_clause {
            fn inject(
                container: &::meshestra::Container
            ) -> ::meshestra::Result<Self> {
                Ok(Self {
                    #(#field_injections),*
                })
            }
        }
    }
}

/// Extract the inner type from Arc<T> or Arc<dyn Trait>
fn extract_injectable_type(ty: &Type) -> Type {
    if let Type::Path(type_path) = ty {
        if let Some(segment) = type_path.path.segments.last() {
            if segment.ident == "Arc" {
                if let syn::PathArguments::AngleBracketed(args) = &segment.arguments {
                    if let Some(syn::GenericArgument::Type(inner_type)) = args.args.first() {
                        return inner_type.clone();
                    }
                }
            }
        }
    }

    // If not Arc<T>, return as-is
    ty.clone()
}
