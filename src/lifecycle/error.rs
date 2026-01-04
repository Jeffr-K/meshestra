//! Lifecycle-specific error types

use thiserror::Error;

/// Errors that can occur during lifecycle operations
#[derive(Debug, Error)]
pub enum LifecycleError {
    /// Service initialization failed
    #[error("Initialization failed: {0}")]
    InitializationFailed(String),

    /// Shutdown operation failed
    #[error("Shutdown failed: {0}")]
    ShutdownFailed(String),

    /// Operation timed out
    #[error("Timeout during {phase}: {message}")]
    Timeout {
        /// The lifecycle phase where timeout occurred
        phase: String,
        /// Additional error message
        message: String,
    },

    /// Hook execution failed
    #[error("Hook execution failed for {service}: {message}")]
    HookFailed {
        /// Name of the service that failed
        service: String,
        /// Error message
        message: String,
    },
}

impl LifecycleError {
    /// Create an initialization failure error
    pub fn init_failed(msg: impl Into<String>) -> Self {
        Self::InitializationFailed(msg.into())
    }

    /// Create a shutdown failure error
    pub fn shutdown_failed(msg: impl Into<String>) -> Self {
        Self::ShutdownFailed(msg.into())
    }

    /// Create a timeout error
    pub fn timeout(phase: impl Into<String>, message: impl Into<String>) -> Self {
        Self::Timeout {
            phase: phase.into(),
            message: message.into(),
        }
    }

    /// Create a hook failure error
    pub fn hook_failed(service: impl Into<String>, message: impl Into<String>) -> Self {
        Self::HookFailed {
            service: service.into(),
            message: message.into(),
        }
    }
}

/// A specialized Result type for lifecycle operations
pub type Result<T> = std::result::Result<T, LifecycleError>;
