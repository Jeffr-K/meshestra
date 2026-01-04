use crate::interceptor::{Interceptor, InterceptorResult, Next};
use async_trait::async_trait;
use axum::{body::Body, http::Request, response::Response};
use std::future::Future;
use std::pin::Pin;
use std::task::{Context, Poll};
use tower::{Layer, Service};

/// Tower Layer for invoking a chain of Meshestra Interceptors
pub struct InterceptorLayer {
    interceptors: Vec<Box<dyn Interceptor>>,
}

impl InterceptorLayer {
    pub fn new(interceptors: Vec<Box<dyn Interceptor>>) -> Self {
        Self { interceptors }
    }
}

impl<S> Layer<S> for InterceptorLayer {
    type Service = InterceptorMiddleware<S>;
    
    fn layer(&self, inner: S) -> Self::Service {
        InterceptorMiddleware {
            inner,
            interceptors: std::sync::Arc::new(self.interceptors.iter().map(|i| {
               // Interceptors must be cloneable or wrapped in Arc?
               // Interceptor trait requires Send + Sync. 
               // For Middleware we need to share them.
               // Let's store Arc<Vec<Box<dyn Interceptor>>> in Middleware?
               // Yes.
               // But `self.interceptors` is `Vec<Box<...>>`.
               // We need to move or clone it? Interceptors are mostly stateless.
               // But box is unique.
               // Let's require user to pass Arc? Or we wrap in Arc ourselves?
               // Since `layer` takes `&self` and returns `Service`, we need to clone the list.
               // But `Box<dyn Interceptor>` is not cloneable.
               // So we should store `Arc<dyn Interceptor>` or wrap the whole connection in `Arc`.
               // The implementation in spec showed `self.interceptors.clone()` which implies something is clonable.
               // If Interceptor is just a trait, Box isn't cloneable.
               // We will use `Arc` for the vector itself.
               unimplemented!("Wait, cannot clone Box<dyn Interceptor>")
            }).collect::<Vec<_>>())
        }
    }
}

// We change InterceptorLayer to hold Arc<Vec<Box<dyn Interceptor>>> from the start?
// Or we wrap it in Arc inside `new`?
// Let's redefine.

pub struct SharedInterceptorLayer {
    interceptors: std::sync::Arc<Vec<Box<dyn Interceptor>>>,
}

impl SharedInterceptorLayer {
    pub fn new(interceptors: Vec<Box<dyn Interceptor>>) -> Self {
        Self {
            interceptors: std::sync::Arc::new(interceptors),
        }
    }
}

impl<S> Layer<S> for SharedInterceptorLayer {
    type Service = InterceptorMiddleware<S>;

    fn layer(&self, inner: S) -> Self::Service {
        InterceptorMiddleware {
            inner,
            interceptors: self.interceptors.clone(),
        }
    }
}

#[derive(Clone)]
pub struct InterceptorMiddleware<S> {
    inner: S,
    interceptors: std::sync::Arc<Vec<Box<dyn Interceptor>>>,
}

impl<S> Service<Request<Body>> for InterceptorMiddleware<S>
where
    S: Service<Request<Body>, Response = Response> + Clone + Send + 'static,
    S::Future: Send + 'static,
    S::Error: Into<Box<dyn std::error::Error + Send + Sync>> + Send,
{
    type Response = Response;
    type Error = Box<dyn std::error::Error + Send + Sync>;
    type Future = Pin<Box<dyn Future<Output = Result<Response, Self::Error>> + Send>>;
    
    fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.inner.poll_ready(cx).map_err(Into::into)
    }
    
    fn call(&mut self, request: Request<Body>) -> Self::Future {
        // Clone the arc to move into future
        let interceptors = self.interceptors.clone();
        
        // Clone inner service (must be clonable)
        // Note: poll_ready was called on &mut self.inner, but we need to move inner into future.
        // Tower middleware pattern usually involves cloning inner.
        let mut inner = self.inner.clone();
        
        Box::pin(async move {
            // Define the base handler (calling the inner service)
            // We need `inner` to be `Clone` to call it here.
            
            // The chain is built recursively.
            // Base chain: executes inner service.
            // We need to wrap it in `Next`.
            // `Next` expects `Box<dyn FnOnce(Request) -> Future ...>`.
            
            // We need a stable reference to `inner`.
            // BUT `Next` takes ownership of its closure.
            // Closure takes ownership of `inner`.
            // So `inner` moves into the base chain closure.
            
            let base_handler = move |req: Request<Body>| -> Pin<Box<dyn Future<Output = InterceptorResult> + Send>> {
                Box::pin(async move {
                    inner.call(req).await.map_err(Into::into)
                })
            };
            
            // Now wrap in interceptors in reverse
            let mut current_next = Next::new(base_handler);
            
            // We iterate in reverse so the first interceptor wraps everything else.
            // interceptors[0] wraps (interceptors[1] wraps ... (base))
            
            for interceptor in interceptors.iter().rev() {
                // We need to pass `interceptor` (ref) and `current_next` (owned) to a closure that `Next` will run.
                // But `Next` runs `intercept`.
                // Wait. `interceptor.intercept(req, next)` is called.
                
                // We need to construct a NEW `Next` that calls THIS interceptor.
                // NO. `Next` represents the "rest of the chain".
                // When we are at index i, `current_next` IS the chain from i+1 to end.
                // We want to create `Next` representing chain from i to end.
                // This `Next`'s run function should call `interceptors[i].intercept(req, current_next)`.
                
                // Problem: `current_next` must be moved into the closure.
                // `interceptor` is behind Arc. We can clone the ref? No, we need sendable future.
                // `interceptors` Arc is moved into the outer Future (async move).
                // Inside the loop, we are building closures.
                
                // We can't move non-static reference `interceptor` into the static closure required by `Next`.
                // Interceptor trait is 'static. `interceptors` is Arc.
                // We can clone the Arc for each closure, and index it?
                // Or just move the Arc into each wrapper.
                
                let my_interceptors = interceptors.clone();
                // We need the index or reference.
                // Actually `interceptor` variable is a reference to Box in `interceptors` vec.
                // We can get the pointer? No. 
                // We can rely on Arc.
                
                // Let's use recursion or indices?
                // Or just move `current_next` into a closure that also holds `interceptor` (via Arc/index).
                // Let's do indices.
                // But `rev` iterator yields references.
                
                // Let's restart the loop logic. 
                // We have `base_handler`.
                // Let chain = base_handler.
                // for i in (0..len).rev() {
                //    let next = Next::new(chain);
                //    let interceptors_ref = interceptors.clone();
                //    chain = move |req| { interceptors_ref[i].intercept(req, next) }
                // }
                // But `chain` type changes? 
                // `Next` takes `FnOnce`. Our chain is `FnOnce`.
                // Yes.
                
                // Let's implement this loop.
            }
            
            // Re-implementation logic inside the future:
            
            // 1. Initial next: call inner
            let mut chain = Next::new(move |req| {
                Box::pin(async move {
                    inner.call(req).await.map_err(Into::into)
                })
            });
            
            // 2. Wrap
            for i in (0..interceptors.len()).rev() {
                let interceptors_arc = interceptors.clone();
                // interceptors_arc[i] is the interceptor we want to run.
                // It takes `chain` (the `Next` for i+1).
                
                // The invalid part of my thinking:
                // `Next` stores `Box<dyn FnOnce...>`. 
                // We need to create a `FnOnce` that calls `interceptor.intercept`.
                
                let next_chain = chain; // Move previous chain
                
                chain = Next::new(move |req| {
                    Box::pin(async move {
                        // We access interceptor from Arc
                        let interceptor = &interceptors_arc[i];
                        interceptor.intercept(req, next_chain).await
                    })
                });
            }
            
            // 3. Execute the final chain
            chain.run(request).await
        })
    }
}
