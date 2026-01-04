use proc_macro::TokenStream;
use proc_macro2::TokenStream as TokenStream2;
use quote::quote;
use syn::{
    parse::Parse, parse::ParseStream, parse_macro_input, Attribute, FnArg, ImplItem, ItemImpl,
    ItemStruct, LitStr, Token,
};

struct ControllerArgs {
    path: String,
}

impl Parse for ControllerArgs {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let mut path = None;
        while !input.is_empty() {
            let name: syn::Ident = input.parse()?;
            input.parse::<Token![=]>()?;
            if name == "path" {
                let lit: LitStr = input.parse()?;
                path = Some(lit.value());
            } else {
                let _: syn::Expr = input.parse()?;
            }
            if input.peek(Token![,]) {
                input.parse::<Token![,]>()?;
            }
        }
        Ok(ControllerArgs { path: path.unwrap_or_else(|| "/".to_string()) })
    }
}

pub fn controller_attribute(attr: TokenStream, item: TokenStream) -> TokenStream {
    let args = parse_macro_input!(attr as ControllerArgs);
    let input = parse_macro_input!(item as ItemStruct);
    let expanded = generate_controller_impl(&args, &input);
    TokenStream::from(expanded)
}

fn generate_controller_impl(args: &ControllerArgs, input: &ItemStruct) -> TokenStream2 {
    let struct_name = &input.ident;
    let base_path = &args.path;
    let injectable_impl = generate_injectable_for_controller(input);
    let router_method = quote! {
        impl #struct_name {
            pub fn base_path() -> &'static str { #base_path }
        }
    };
    quote! {
        #input
        #injectable_impl
        #router_method
    }
}

fn generate_injectable_for_controller(input: &ItemStruct) -> TokenStream2 {
    let struct_name = &input.ident;
    let fields = match &input.fields {
        syn::Fields::Named(fields) => &fields.named,
        _ => panic!("#[controller] only supports structs with named fields"),
    };
    let field_injections = fields.iter().map(|field| {
        let field_name = &field.ident;
        let field_type = extract_injectable_type(&field.ty);
        quote! { #field_name: container.resolve::<#field_type>()? }
    });
    quote! {
        impl ::meshestra::Injectable for #struct_name {
            fn inject(container: &::meshestra::Container) -> ::meshestra::Result<Self> {
                Ok(Self { #(#field_injections),* })
            }
        }
    }
}

fn extract_injectable_type(ty: &syn::Type) -> syn::Type {
    if let syn::Type::Path(type_path) = ty {
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
    ty.clone()
}

#[derive(Clone)]
enum ParamKind { Body, Param, Query, Raw }

struct ParamInfo {
    ty: syn::Type,
    kind: ParamKind,
}

struct RouteInfo {
    method: String,
    path: String,
    fn_name: syn::Ident,
    params: Vec<ParamInfo>,
    aspects: Vec<syn::Type>,
}

pub fn routes_attribute(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let input = parse_macro_input!(item as ItemImpl);
    let expanded = generate_routes_impl(input);
    TokenStream::from(expanded)
}

fn generate_routes_impl(input: ItemImpl) -> TokenStream2 {
    let mut routes: Vec<RouteInfo> = Vec::new();
    let mut clean_items: Vec<ImplItem> = Vec::new();

    for item in input.items.iter() {
        if let ImplItem::Fn(method) = item {
            if let Some(route_info) = extract_route_info(method) {
                routes.push(route_info);
                let mut clean_method = method.clone();
                clean_method.attrs.retain(|attr| !is_http_method_attr(attr) && !attr.path().is_ident("aspect"));
                for input in clean_method.sig.inputs.iter_mut() {
                    if let FnArg::Typed(pat_type) = input {
                        pat_type.attrs.retain(|attr| !is_param_attr(attr));
                    }
                }
                clean_items.push(ImplItem::Fn(clean_method));
            } else {
                clean_items.push(item.clone());
            }
        } else {
            clean_items.push(item.clone());
        }
    }

    let route_registrations = routes.iter().map(|route| {
        let method_ident = match route.method.as_str() {
            "GET" => quote! { ::axum::routing::get },
            "POST" => quote! { ::axum::routing::post },
            "PUT" => quote! { ::axum::routing::put },
            "DELETE" => quote! { ::axum::routing::delete },
            "PATCH" => quote! { ::axum::routing::patch },
            _ => quote! { ::axum::routing::get },
        };

        let path = &route.path;
        let fn_name = &route.fn_name;
        let aspects = &route.aspects;

        let extractor_patterns: Vec<_> = route.params.iter().enumerate().map(|(i, p)| {
            let temp_ident = quote::format_ident!("__p_{}", i);
            let ty = &p.ty;
            match p.kind {
                ParamKind::Body => quote! { ::axum::Json(#temp_ident): ::axum::Json<#ty> },
                ParamKind::Param => quote! { ::axum::extract::Path(#temp_ident): ::axum::extract::Path<#ty> },
                ParamKind::Query => quote! { ::axum::extract::Query(#temp_ident): ::axum::extract::Query<#ty> },
                ParamKind::Raw => quote! { #temp_ident: #ty },
            }
        }).collect();

        let internal_args: Vec<_> = route.params.iter().enumerate().map(|(i, _)| {
            quote::format_ident!("__p_{}", i)
        }).collect();

        if aspects.is_empty() {
            quote! {
                .route(#path, #method_ident({
                    let controller = controller.clone();
                    move |#(#extractor_patterns),*| {
                        let controller = controller.clone();
                        async move { 
                            use ::axum::response::IntoResponse;
                            controller.#fn_name(#(#internal_args),*).await.into_response()
                        }
                    }
                }))
            }
        } else {
            quote! {
                .route(#path, #method_ident({
                    let controller = controller.clone();
                    move |__state: ::axum::extract::State<S>, #(#extractor_patterns,)* __parts: ::axum::http::request::Parts| {
                        let controller = controller.clone();
                        async move { 
                            use ::axum::response::IntoResponse;
                            let mut execution = {
                                let controller = controller.clone();
                                #(let #internal_args = #internal_args.clone();)*
                                Box::pin(async move {
                                    controller.#fn_name(#(#internal_args),*).await.into_response()
                                })
                            };
                            #(
                                let container = __state.get_container();
                                let aspect = container.resolve::<#aspects>().expect("Aspect resolve failed");
                                let interceptor = ::meshestra::aspect::AspectInterceptor::new(aspect);
                                let mut req = ::axum::http::Request::builder()
                                    .method(__parts.method.clone())
                                    .uri(__parts.uri.clone())
                                    .version(__parts.version)
                                    .body(::axum::body::Body::empty()).unwrap();
                                *req.headers_mut() = __parts.headers.clone();
                                let next_logic = execution;
                                let next = ::meshestra::interceptor::Next::new(move |_| next_logic);
                                execution = Box::pin(async move {
                                    interceptor.intercept(req, next).await.unwrap_or_else(|e| {
                                        (::axum::http::StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response()
                                    })
                                });
                            )*
                            execution.await
                        }
                    }
                }))
            }
        }
    });

    let self_ty = &input.self_ty;
    let impl_generics = &input.generics;

    quote! {
        impl #impl_generics #self_ty {
            #(#clean_items)*
            pub fn router<S>(controller: ::std::sync::Arc<Self>) -> ::axum::Router<S>
            where
                S: Clone + Send + Sync + ::meshestra::di::HasContainer + 'static,
            {
                ::axum::Router::new() #(#route_registrations)*
            }
        }
    }
}

fn extract_route_info(method: &syn::ImplItemFn) -> Option<RouteInfo> {
    let mut http_method = None;
    let mut path = String::new();
    let mut aspects = Vec::new();

    for attr in &method.attrs {
        if let Some(ident) = attr.path().get_ident() {
            let name = ident.to_string();
            if ["get", "post", "put", "delete", "patch"].contains(&name.as_str()) {
                http_method = Some(name.to_uppercase());
                if let syn::Meta::List(meta_list) = &attr.meta {
                    let tokens = meta_list.tokens.to_string();
                    path = tokens.trim_matches('"').to_string();
                }
            } else if name == "aspect" {
                if let Ok(ty) = attr.parse_args::<syn::Type>() {
                    aspects.push(ty);
                }
            }
        }
    }
    let http_method = http_method?;

    let mut params = Vec::new();
    for input in method.sig.inputs.iter() {
        if let FnArg::Typed(pat_type) = input {
            let ty = (*pat_type.ty).clone();
            let kind = get_param_kind(&pat_type.attrs);
            params.push(ParamInfo { ty, kind });
        }
    }
    Some(RouteInfo { method: http_method, path, fn_name: method.sig.ident.clone(), params, aspects })
}

fn get_param_kind(attrs: &[Attribute]) -> ParamKind {
    for attr in attrs {
        if let Some(ident) = attr.path().get_ident() {
            let name = ident.to_string();
            match name.as_str() {
                "body" => return ParamKind::Body,
                "param" => return ParamKind::Param,
                "query" => return ParamKind::Query,
                _ => {}
            }
        }
    }
    ParamKind::Raw
}

fn is_http_method_attr(attr: &Attribute) -> bool {
    attr.path().get_ident().map_or(false, |ident| {
        ["get", "post", "put", "delete", "patch"].contains(&ident.to_string().as_str())
    })
}

fn is_param_attr(attr: &Attribute) -> bool {
    attr.path().get_ident().map_or(false, |ident| {
        ["body", "param", "query"].contains(&ident.to_string().as_str())
    })
}