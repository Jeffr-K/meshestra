use proc_macro::TokenStream;
use proc_macro2::TokenStream as TokenStream2;
use quote::quote;
use syn::{
    parse::{Parse, ParseStream},
    parse_macro_input, Expr, ExprMethodCall, ExprPath, GenericArgument, ItemStruct, Path, Token,
    Type,
};

// Simplified parsing for items like `UserService` or `AppModule`
struct ModuleItem {
    path: Path,
}

impl Parse for ModuleItem {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let path = input.parse()?;
        Ok(ModuleItem { path })
    }
}

// Parses a provider expression, which can be a simple type or a trait binding.
enum Provider {
    Struct(ExprPath),
    Trait {
        impl_path: ExprPath,
        trait_path: Type,
    },
}

impl Parse for Provider {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let expr: Expr = input.parse()?;

        match expr {
            Expr::Path(path) => Ok(Provider::Struct(path)),
            Expr::MethodCall(method_call) => {
                if method_call.method == "for_trait" {
                    parse_for_trait_call(method_call)
                } else {
                    Err(syn::Error::new_spanned(
                        method_call,
                        "Expected method call to be `.for_trait()`",
                    ))
                }
            }
            _ => Err(syn::Error::new_spanned(
                expr,
                "Expected a struct type or `Provider::new(...).for_trait()`",
            )),
        }
    }
}

fn parse_for_trait_call(method_call: ExprMethodCall) -> syn::Result<Provider> {
    // Extract `dyn Trait` from `.for_trait::<dyn Trait>()`
    let trait_path =
        match method_call.turbofish {
            Some(tf) => match tf.args.first() {
                Some(GenericArgument::Type(ty)) => ty.clone(),
                _ => {
                    return Err(syn::Error::new_spanned(
                        tf,
                        "Expected a trait type argument for `.for_trait()`",
                    ))
                }
            },
            None => return Err(syn::Error::new_spanned(
                method_call.method,
                "`.for_trait()` requires a generic argument, e.g., `for_trait::<dyn MyTrait>()`",
            )),
        };

    // The receiver should be `Provider::new(Impl)`
    let receiver_call = match *method_call.receiver {
        Expr::Call(call) => call,
        _ => {
            return Err(syn::Error::new_spanned(
                method_call.receiver,
                "Expected receiver to be a `Provider::new(...)` call",
            ))
        }
    };

    // Extract `Impl` from `Provider::new(Impl)`
    let impl_path = match receiver_call.args.first() {
        Some(Expr::Path(path)) => path.clone(),
        _ => {
            return Err(syn::Error::new_spanned(
                receiver_call.args,
                "Expected a single struct type as argument to `Provider::new()`",
            ))
        }
    };

    Ok(Provider::Trait {
        impl_path,
        trait_path,
    })
}

// Main struct to parse the macro arguments: `imports = [...], providers = [...]`
struct ModuleArgs {
    imports: Vec<ModuleItem>,
    controllers: Vec<ModuleItem>,
    providers: Vec<Provider>,
}

impl Parse for ModuleArgs {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let mut imports = Vec::new();
        let mut controllers = Vec::new();
        let mut providers = Vec::new();

        while !input.is_empty() {
            let name: syn::Ident = input.parse()?;
            input.parse::<Token![=]>()?;

            let content;
            syn::bracketed!(content in input);

            if name == "imports" {
                imports = content
                    .parse_terminated(ModuleItem::parse, Token![,])?
                    .into_iter()
                    .collect();
            } else if name == "controllers" {
                controllers = content
                    .parse_terminated(ModuleItem::parse, Token![,])?
                    .into_iter()
                    .collect();
            } else if name == "providers" {
                providers = content
                    .parse_terminated(Provider::parse, Token![,])?
                    .into_iter()
                    .collect();
            } else {
                return Err(syn::Error::new(
                    name.span(),
                    "Expected `imports`, `controllers`, or `providers`",
                ));
            }

            if input.peek(Token![,]) {
                input.parse::<Token![,]>()?;
            }
        }

        Ok(ModuleArgs {
            imports,
            controllers,
            providers,
        })
    }
}

// The main function for the `#[module]` attribute macro
pub fn module_attribute(attr: TokenStream, item: TokenStream) -> TokenStream {
    let args = parse_macro_input!(attr as ModuleArgs);
    let input = parse_macro_input!(item as ItemStruct);
    let expanded = generate_module_impl(&args, &input);
    TokenStream::from(expanded)
}

// Generates the `impl Module for ...` block
fn generate_module_impl(args: &ModuleArgs, input: &ItemStruct) -> TokenStream2 {
    let module_name = &input.ident;

    let import_registrations = args.imports.iter().map(|item| {
        let path = &item.path;
        quote! { #path::register(container)?; }
    });

    let provider_registrations = args.providers.iter().map(|provider| match provider {
        Provider::Struct(path) => {
            quote! {
                {
                    let instance = <#path as ::meshestra::Injectable>::inject(container)?;
                    container.register(instance);
                }
            }
        }
        Provider::Trait {
            impl_path,
            trait_path,
        } => {
            quote! {
                {
                    // First, register the concrete implementation so it can be injected elsewhere if needed
                    let instance = <#impl_path as ::meshestra::Injectable>::inject(container)?;
                    container.register(instance);

                    // Then, register the trait binding
                    container.register_trait::<#trait_path, #impl_path, _>(|i| i as std::sync::Arc<#trait_path>);
                }
            }
        }
    });

    let controller_registrations = args.controllers.iter().map(|item| {
        let path = &item.path;
        quote! {
            {
                let instance = <#path as ::meshestra::Injectable>::inject(container)?;
                container.register(instance);
            }
        }
    });

    quote! {
        #input

        impl #module_name {
            /// Registers the module's imports, providers, and controllers.
            pub fn register(container: &mut ::meshestra::Container) -> ::meshestra::Result<()> {
                #(#import_registrations)*
                #(#provider_registrations)*
                #(#controller_registrations)*
                Ok(())
            }

            /// Creates a new DI container and registers this module.
            pub fn create_container() -> ::meshestra::Result<::meshestra::Container> {
                let mut container = ::meshestra::Container::new();
                Self::register(&mut container)?;
                Ok(container)
            }
        }
    }
}
