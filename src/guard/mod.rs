use async_trait::async_trait;
use axum::{body::Body, http::Request};

/// Standard Result type for Guard
/// Ok(()) means allowed
/// Err(GuardError) means denied
pub type GuardResult = Result<(), GuardError>;

#[derive(Debug, thiserror::Error)]
pub enum GuardError {
    #[error("Forbidden: {0}")]
    Forbidden(String),

    #[error("Unauthorized: {0}")]
    Unauthorized(String),
}

/// The Guard trait
/// Implement this to protect routes
#[async_trait]
pub trait Guard: Send + Sync + 'static {
    async fn can_activate(&self, request: &Request<Body>) -> GuardResult;
}
