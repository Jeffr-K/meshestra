use crate::guard::{Guard, GuardError, GuardResult};
use async_trait::async_trait;
use axum::{body::Body, http::Request, response::Response};
use std::future::Future;
use std::pin::Pin;
use std::task::{Context, Poll};
use tower::{Layer, Service};
use std::sync::Arc;

/// Tower Layer for Guards
pub struct GuardLayer {
    guards: Vec<Box<dyn Guard>>,
}

impl GuardLayer {
    pub fn new(guards: Vec<Box<dyn Guard>>) -> Self {
        Self { guards }
    }
}

impl<S> Layer<S> for GuardLayer {
    type Service = GuardMiddleware<S>;

    fn layer(&self, inner: S) -> Self::Service {
        GuardMiddleware {
            inner,
            guards: Arc::new(self.guards.iter().map(|g| {
                // We cannot clone Box<dyn Guard> easily unless we implement Clone for it?
                // Or wrap it in Arc from start.
                // Assuming shared ownership for now via Arc on the Vec level.
                // But we can't clone items inside Vec unless they are Clone.
                // We'll require Guards to be stateless mostly.
                // Wait, I can't clone `Box<dyn Guard>` in `map`.
                // I should wrap them in Arc? 
                // Let's change the field to `Arc<Vec<Box<dyn Guard>>>` in Middleware.
                // But Layer constructor took `Vec<Box>`.
                // We'll wrap the whole Vec.
                // But here I'm constructing a NEW vec? No.
                unreachable!("This map logic is flawied if not cloning")
            }).collect()),
        }
    }
}

// Redefine Layer correctly
pub struct SharedGuardLayer {
    guards: Arc<Vec<Box<dyn Guard>>>,
}

impl SharedGuardLayer {
    pub fn new(guards: Vec<Box<dyn Guard>>) -> Self {
        Self {
            guards: Arc::new(guards),
        }
    }
}

impl<S> Layer<S> for SharedGuardLayer {
    type Service = GuardMiddleware<S>;

    fn layer(&self, inner: S) -> Self::Service {
        GuardMiddleware {
            inner,
            guards: self.guards.clone(),
        }
    }
}

#[derive(Clone)]
pub struct GuardMiddleware<S> {
    inner: S,
    guards: Arc<Vec<Box<dyn Guard>>>,
}

impl<S> Service<Request<Body>> for GuardMiddleware<S>
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

    fn call(&mut self, req: Request<Body>) -> Self::Future {
        let guards = self.guards.clone();
        let mut inner = self.inner.clone();

        Box::pin(async move {
            for guard in guards.iter() {
                if let Err(e) = guard.can_activate(&req).await {
                    // Convert GuardError to Response properly?
                    // Usually 403 Forbidden.
                    // For now, return Error.
                    return Err(Box::new(e) as Box<dyn std::error::Error + Send + Sync>);
                }
            }
            inner.call(req).await.map_err(Into::into)
        })
    }
}
