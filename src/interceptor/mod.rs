use async_trait::async_trait;
use axum::{body::Body, http::Request, response::Response};
use std::future::Future;
use std::pin::Pin;

/// standard return type for Interceptors
pub type InterceptorResult = Result<Response, InterceptorError>;

/// A type-erased error for interceptors
pub type InterceptorError = Box<dyn std::error::Error + Send + Sync>;

/// Represents the next handler in the chain
pub struct Next {
    pub(crate) run: Box<dyn FnOnce(Request<Body>) -> Pin<Box<dyn Future<Output = InterceptorResult> + Send>> + Send>,
}

impl Next {
    /// Create a new Next handler
    pub fn new<F>(f: F) -> Self
    where
        F: FnOnce(Request<Body>) -> Pin<Box<dyn Future<Output = InterceptorResult> + Send>> + Send + 'static,
    {
        Self {
            run: Box::new(f),
        }
    }

    /// Execute the next handler
    pub async fn run(self, request: Request<Body>) -> InterceptorResult {
        (self.run)(request).await
    }
}

/// The Interceptor trait
/// 
/// Interceptors can inspect/modify the request before it reaches the handler,
/// and inspect/modify the response after the handler returns.
///
/// # Example
/// ```
/// struct LoggingInterceptor;
/// 
/// #[async_trait]
/// impl Interceptor for LoggingInterceptor {
///     async fn intercept(&self, req: Request, next: Next) -> InterceptorResult {
///         println!("Before request");
///         let res = next.run(req).await?;
///         println!("After request");
///         Ok(res)
///     }
/// }
/// ```
#[async_trait]
pub trait Interceptor: Send + Sync + 'static {
    async fn intercept(&self, request: Request<Body>, next: Next) -> InterceptorResult;
}
