use axum::response::Response;
use std::error::Error;

pub mod http;

/// Context for exception handling
pub struct ArgumentsHost {
    // In Axum, we might just need the error and maybe the request context?
    // For now, simple error handling.
}

/// The ExceptionFilter trait
///
/// Filters handle errors thrown during request processing.
/// They must return a valid Response.
pub trait ExceptionFilter: Send + Sync + 'static {
    /// Catch an exception and return a response
    fn catch(&self, error: Box<dyn Error + Send + Sync>) -> Response;
}
