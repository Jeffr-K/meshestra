use async_trait::async_trait;
use std::fmt::Debug;

pub mod builtins;

pub type PipeResult<T> = Result<T, PipeError>;

#[derive(Debug, thiserror::Error)]
pub enum PipeError {
    #[error("Validation failed: {0}")]
    Validation(String),

    #[error("Transformation failed: {0}")]
    Transformation(String),

    #[error("Internal pipe error: {0}")]
    Internal(String),
}

/// The Pipe trait for transformation and validation
#[async_trait]
pub trait Pipe: Send + Sync + 'static {
    type Input: Send + 'static;
    type Output: Send + 'static;

    async fn transform(&self, input: Self::Input) -> PipeResult<Self::Output>;
}
