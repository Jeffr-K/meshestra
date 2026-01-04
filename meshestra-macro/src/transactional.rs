use proc_macro::TokenStream;
use quote::quote;
use syn::{parse::Parse, parse::ParseStream, parse_macro_input, ItemFn, LitBool, Path, Token};

struct TransactionArgs {
    isolation: Option<Path>,
    propagation: Option<Path>,
    read_only: Option<bool>,
}

impl Parse for TransactionArgs {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let mut isolation = None;
        let mut propagation = None;
        let mut read_only = None;

        while !input.is_empty() {
            let key: syn::Ident = input.parse()?;
            input.parse::<Token![=]>()?;

            if key == "isolation" {
                // Parse as Path (e.g., IsolationLevel::Serializable or Serializable)
                let p: Path = input.parse()?;
                isolation = Some(p);
            } else if key == "propagation" {
                let p: Path = input.parse()?;
                propagation = Some(p);
            } else if key == "read_only" {
                let b: LitBool = input.parse()?;
                read_only = Some(b.value);
            } else {
                // Ignore or error
            }

            if input.peek(Token![,]) {
                input.parse::<Token![,]>()?;
            }
        }

        Ok(TransactionArgs {
            isolation,
            propagation,
            read_only,
        })
    }
}

pub fn transactional_attribute(attr: TokenStream, item: TokenStream) -> TokenStream {
    let args = parse_macro_input!(attr as TransactionArgs);
    let mut input = parse_macro_input!(item as ItemFn);

    // Ensure function is async
    if input.sig.asyncness.is_none() {
        return syn::Error::new_spanned(
            input.sig.fn_token,
            "#[transactional] can only be used on async functions",
        )
        .to_compile_error()
        .into();
    }

    let block = &input.block;

    // Generate Isolation Option
    // We assume the user provides a valid enum variant path.
    // To be user-friendly, we might want to automatically prepend `::meshestra::transactional::IsolationLevel::` if just the variant is given.
    // But for "Enum" request, allowing full path is standard Rust way.
    // Let's support intelligent mapping: if it's a single identifier, we map it to our enum.
    let isolation_code = match args.isolation {
        Some(path) => {
            if path.segments.len() == 1 {
                let ident = &path.segments.first().unwrap().ident;
                quote! { Some(::meshestra::transactional::IsolationLevel::#ident) }
            } else {
                quote! { Some(#path) }
            }
        }
        None => quote! { None },
    };

    let propagation_code = match args.propagation {
        Some(path) => {
            if path.segments.len() == 1 {
                let ident = &path.segments.first().unwrap().ident;
                quote! { ::meshestra::transactional::Propagation::#ident }
            } else {
                quote! { #path }
            }
        }
        None => quote! { ::meshestra::transactional::Propagation::Required },
    };

    let read_only_code = args.read_only.unwrap_or(false);

    let options_expr = quote! {
        ::meshestra::transactional::TransactionOptions {
            isolation: #isolation_code,
            propagation: #propagation_code,
            read_only: #read_only_code,
        }
    };

    let new_block = quote! {
        {
            use ::meshestra::transactional::TransactionManager;

            let mut tx = self.transaction_manager
                .begin(#options_expr)
                .await
                .map_err(|e| ::meshestra::MeshestraError::Internal(e.to_string()))?;

            let result = (async move { #block }).await;

            match &result {
                Ok(_) => {
                    if let Err(e) = tx.commit().await {
                         return Err(::meshestra::MeshestraError::Internal(e.to_string()).into());
                    }
                },
                Err(_) => {
                    if let Err(e) = tx.rollback().await {
                    }
                }
            }

            result
        }
    };

    input.block = syn::parse2(new_block).expect("Failed to generate transactional wrapper");

    TokenStream::from(quote! {
        #input
    })
}
