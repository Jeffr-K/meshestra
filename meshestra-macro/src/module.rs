use proc_macro::TokenStream;
use proc_macro2::TokenStream as TokenStream2;
use quote::quote;
use syn::{
    parse::Parse, parse::ParseStream, parse_macro_input, Attribute, ItemStruct, Path, Token, Type,
};

struct ModuleItem {
    attrs: Vec<Attribute>,
    path: Path,
}

impl Parse for ModuleItem {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let attrs = input.call(Attribute::parse_outer)?;
        let path = input.parse()?;
        Ok(ModuleItem { attrs, path })
    }
}

/// Represents a trait binding: (dyn Trait => Impl)
struct BindingItem {
    trait_type: Type,
    impl_type: Path,
}

impl Parse for BindingItem {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        // Parse: (dyn Trait => Impl)
        let content;
        syn::parenthesized!(content in input);

        let trait_type: Type = content.parse()?;
        content.parse::<Token![=>]>()?;
        let impl_type: Path = content.parse()?;

        Ok(BindingItem {
            trait_type,
            impl_type,
        })
    }
}

struct ModuleArgs {
    imports: Vec<ModuleItem>,
    controllers: Vec<ModuleItem>,
    providers: Vec<ModuleItem>,
    bindings: Vec<BindingItem>,
}

impl Parse for ModuleArgs {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let mut imports = Vec::new();
        let mut controllers = Vec::new();
        let mut providers = Vec::new();
        let mut bindings = Vec::new();

        while !input.is_empty() {
            let name: syn::Ident = input.parse()?;
            input.parse::<Token![=]>()?;

            // Parse array: [Item1, Item2, ...]
            let content;
            syn::bracketed!(content in input);

            if name == "imports" {
                let items = content.parse_terminated(ModuleItem::parse, Token![,])?;
                imports = items.into_iter().collect();
            } else if name == "controllers" {
                let items = content.parse_terminated(ModuleItem::parse, Token![,])?;
                controllers = items.into_iter().collect();
            } else if name == "providers" {
                let items = content.parse_terminated(ModuleItem::parse, Token![,])?;
                providers = items.into_iter().collect();
            } else if name == "bindings" {
                let items = content.parse_terminated(BindingItem::parse, Token![,])?;
                bindings = items.into_iter().collect();
            }

            if input.peek(Token![,]) {
                input.parse::<Token![,]>()?;
            }
        }

        Ok(ModuleArgs {
            imports,
            controllers,
            providers,
            bindings,
        })
    }
}

pub fn module_attribute(attr: TokenStream, item: TokenStream) -> TokenStream {
    let args = parse_macro_input!(attr as ModuleArgs);
    let input = parse_macro_input!(item as ItemStruct);
    let expanded = generate_module_impl(&args, &input);

    TokenStream::from(expanded)
}

fn generate_module_impl(args: &ModuleArgs, input: &ItemStruct) -> TokenStream2 {
    let module_name = &input.ident;
    let imports = &args.imports;
    let providers = &args.providers;
    let controllers = &args.controllers;
    let bindings = &args.bindings;

    // Generate import registrations (call other modules' register)
    let import_registrations = imports.iter().map(|item| {
        let path = &item.path;
        let attrs = &item.attrs;
        quote! {
            #(#attrs)*
            #path::register(container)?;
        }
    });

    // Generate binding registrations
    let binding_registrations = bindings.iter().map(|binding| {
        let trait_type = &binding.trait_type;
        let impl_type = &binding.impl_type;
        quote! {
            container.register_trait::<#trait_type, #impl_type, _>(|i| {
                i as ::std::sync::Arc<#trait_type>
            });
        }
    });

    // Generate registration code for providers
    let provider_registrations = providers.iter().map(|item| {
        let path = &item.path;
        let attrs = &item.attrs;
        quote! {
            #(#attrs)*
            {
                let instance = <#path as ::meshestra::Injectable>::inject(container)?;
                container.register(instance);
            }
        }
    });

    // Generate registration code for controllers
    let controller_registrations = controllers.iter().map(|item| {
        let path = &item.path;
        let attrs = &item.attrs;
        quote! {
            #(#attrs)*
            {
                let instance = <#path as ::meshestra::Injectable>::inject(container)?;
                container.register(instance);
            }
        }
    });

    quote! {
        #input

        impl #module_name {
            /// Register this module's providers and controllers into the container
            pub fn register(
                container: &mut ::meshestra::Container
            ) -> ::meshestra::Result<()> {
                // 1. Register trait bindings first
                #(#binding_registrations)*

                // 2. Register imported modules
                #(#import_registrations)*

                // 3. Register providers
                #(#provider_registrations)*

                // 4. Register controllers
                #(#controller_registrations)*

                Ok(())
            }

            /// Create a new container and register this module
            pub fn create_container() -> ::meshestra::Result<::meshestra::Container> {
                let mut container = ::meshestra::Container::new();
                Self::register(&mut container)?;
                Ok(container)
            }
        }
    }
}
