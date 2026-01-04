use crate::interceptor::{Interceptor, InterceptorResult, Next};
use async_trait::async_trait;
use axum::{body::Body, http::Request};
use std::time::Instant;

/// An interceptor that logs request timing and status
#[derive(Clone, Default)]
pub struct LoggingInterceptor;

#[async_trait]
impl Interceptor for LoggingInterceptor {
    async fn intercept(&self, request: Request<Body>, next: Next) -> InterceptorResult {
        let method = request.method().clone();
        let uri = request.uri().clone();
        let start = Instant::now();
        
        println!("--> {} {}", method, uri);

        match next.run(request).await {
            Ok(response) => {
                let duration = start.elapsed();
                let status = response.status();
                println!("<-- {} {} {} {:?}", method, uri, status, duration);
                Ok(response)
            },
            Err(e) => {
                let duration = start.elapsed();
                println!("<-- {} {} ERROR: {} {:?}", method, uri, e, duration);
                Err(e)
            }
        }
    }
}
