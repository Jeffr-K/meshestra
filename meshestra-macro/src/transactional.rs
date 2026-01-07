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
            use ::meshestra::transactional::{get_current_transaction, ACTIVE_TRANSACTION, Propagation, Transaction, TransactionManager};
            use ::meshestra::MeshestraError;
            use ::std::sync::Arc;
            use ::tokio::sync::Mutex;

            let options = #options_expr;

            // This logic handles Propagation::Required
            if options.propagation == Propagation::Required {
                if let Some(_existing_tx) = get_current_transaction() {
                    // A transaction is already active. Just run the function body.
                    // The outer transactional scope will handle commit/rollback.
                    (async move { #block }).await
                } else {
                    // No active transaction. We need to start one.
                    let tx_manager = &self.transaction_manager;
                    let tx_box = tx_manager.begin(options).await.map_err(|e| MeshestraError::Internal(e.to_string()))?;

                    let tx_arc = Arc::new(Mutex::new(tx_box));

                    // Set the transaction in the task local for the scope of the function
                    let result = ACTIVE_TRANSACTION.scope(Some(tx_arc.clone()), async {
                        (async move { #block }).await
                    }).await;

                    // After the function runs, commit or rollback.
                    let mut guard = tx_arc.lock().await;
                    match &result {
                        Ok(_) => {
                            if let Err(e) = guard.commit().await {
                                 return Err(MeshestraError::Internal(format!("Failed to commit transaction: {}", e)).into());
                            }
                        },
                        Err(_) => {
                            if let Err(e) = guard.rollback().await {
                                // Log rollback failure? For now, the original error is more important.
                            }
                        }
                    }

                    result
                }
            } else {
                 panic!("Only Propagation::Required is currently supported by #[transactional]");
            }
        }
    };

    input.block = syn::parse2(new_block).expect("Failed to generate transactional wrapper");

    TokenStream::from(quote! {
        #input
    })
}
