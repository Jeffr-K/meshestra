use thiserror::Error;

pub type Result<T> = std::result::Result<T, MeshestraError>;

#[derive(Debug, Error)]
pub enum MeshestraError {
    #[error("Dependency not found: {type_name}")]
    DependencyNotFound { type_name: String },

    #[error("Failed to downcast type: {type_name}")]
    DowncastFailed { type_name: String },

    #[error("Circular dependency detected: {cycle}")]
    CircularDependency { cycle: String },

    #[error("Scope mismatch: {message}")]
    ScopeMismatch { message: String },

    #[error("Module registration failed: {message}")]
    ModuleRegistrationFailed { message: String },

    #[error("Internal error: {0}")]
    Internal(String),
}

#[cfg(feature = "sea-orm-db")]
impl From<sea_orm::DbErr> for MeshestraError {
    fn from(err: sea_orm::DbErr) -> Self {
        // A real application would have more sophisticated error mapping
        MeshestraError::Internal(format!("Database error: {}", err))
    }
}

impl axum::response::IntoResponse for MeshestraError {
    fn into_response(self) -> axum::response::Response {
        let (status, message) = match &self {
            MeshestraError::DependencyNotFound { .. } => (
                axum::http::StatusCode::INTERNAL_SERVER_ERROR,
                self.to_string(),
            ),
            MeshestraError::DowncastFailed { .. } => (
                axum::http::StatusCode::INTERNAL_SERVER_ERROR,
                self.to_string(),
            ),
            MeshestraError::CircularDependency { .. } => (
                axum::http::StatusCode::INTERNAL_SERVER_ERROR,
                self.to_string(),
            ),
            MeshestraError::ScopeMismatch { .. } => (
                axum::http::StatusCode::INTERNAL_SERVER_ERROR,
                self.to_string(),
            ),
            MeshestraError::ModuleRegistrationFailed { .. } => (
                axum::http::StatusCode::INTERNAL_SERVER_ERROR,
                self.to_string(),
            ),
            MeshestraError::Internal(msg) => {
                (axum::http::StatusCode::INTERNAL_SERVER_ERROR, msg.clone())
            }
        };
        (status, message).into_response()
    }
}
