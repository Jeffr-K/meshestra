use crate::error::MeshestraError;
use crate::interceptor::{Interceptor, InterceptorResult, Next};
use async_trait::async_trait;
use axum::{body::Body, http::Request, response::Response};
use std::sync::Arc;

/// Result type for Aspect hooks.
/// Uses the internal `MeshestraError` to maintain consistent error responses.
pub type AspectResult = Result<(), MeshestraError>;

/// # Aspect
///
/// Defines cross-cutting concerns with simple `before` and `after` hooks.
/// Aspects are easier to implement than Interceptors when you don't need
/// to control the full execution flow.
///
/// ### Example
///
/// ```rust
/// use meshestra::prelude::*;
/// use async_trait::async_trait;
/// use axum::{body::Body, http::Request};
///
/// pub struct AuthAspect;
///
/// #[async_trait]
/// impl Aspect for AuthAspect {
///     async fn before(&self, req: &mut Request<Body>) -> AspectResult {
///         if req.headers().contains_key("Authorization") {
///             Ok(())
///         } else {
///             Err(MeshestraError::Unauthorized("Missing token".into()))
///         }
///     }
/// }
/// ```
#[async_trait]
pub trait Aspect: Send + Sync + 'static {
    /// Executed before the request reaches the handler.
    /// Useful for validation, logging, or injecting headers.
    async fn before(&self, _request: &mut Request<Body>) -> AspectResult {
        Ok(())
    }

    /// Executed after the handler successfully returns a response.
    /// Useful for modifying response headers or logging results.
    async fn after(&self, _response: &mut Response) -> AspectResult {
        Ok(())
    }

    /// Executed when an error occurs during the handler or interceptor execution.
    async fn on_error(&self, _error: &(dyn std::error::Error + Send + Sync)) {
        // Default: No-op for error logging or metrics
    }
}

/// Adapter that wraps an [`Aspect`] to work within the [`Interceptor`] system.
pub struct AspectInterceptor<A: Aspect> {
    aspect: Arc<A>,
}

impl<A: Aspect> AspectInterceptor<A> {
    /// Creates a new adapter for the given aspect.
    pub fn new(aspect: A) -> Self {
        Self {
            aspect: Arc::new(aspect),
        }
    }
}

#[async_trait]
impl<A: Aspect> Interceptor for AspectInterceptor<A> {
    async fn intercept(&self, mut request: Request<Body>, next: Next) -> InterceptorResult {
        // 1. Run Before hook
        if let Err(e) = self.aspect.before(&mut request).await {
            // Box the error to match the InterceptorResult signature
            return Err(Box::new(e));
        }

        // 2. Proceed to the next interceptor or handler
        let result = next.run(request).await;

        match result {
            Ok(mut response) => {
                // 3. Run After hook on success
                if let Err(e) = self.aspect.after(&mut response).await {
                    return Err(Box::new(e));
                }
                Ok(response)
            }
            Err(e) => {
                // 4. Run Error hook on failure
                self.aspect.on_error(e.as_ref()).await;
                Err(e)
            }
        }
    }
}
