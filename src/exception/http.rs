use crate::exception::ExceptionFilter;
use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde_json::json;
use std::error::Error;

/// A default exception filter that handles common errors
#[derive(Default)]
pub struct HttpExceptionFilter;

impl ExceptionFilter for HttpExceptionFilter {
    fn catch(&self, error: Box<dyn Error + Send + Sync>) -> Response {
        // Log the error?
        println!("Exception intercepted: {:?}", error);

        // Map error to proper status code
        // For simplicity, everything is 500 or 400.
        // In real app, we check if error is of specific type.

        let (status, message) =
            if let Some(meshestra_error) = error.downcast_ref::<crate::error::MeshestraError>() {
                match meshestra_error {
                    crate::error::MeshestraError::DependencyNotFound { .. } => (
                        StatusCode::INTERNAL_SERVER_ERROR,
                        meshestra_error.to_string(),
                    ),
                    crate::error::MeshestraError::DowncastFailed { .. } => (
                        StatusCode::INTERNAL_SERVER_ERROR,
                        meshestra_error.to_string(),
                    ),
                    _ => (
                        StatusCode::INTERNAL_SERVER_ERROR,
                        meshestra_error.to_string(),
                    ),
                }
            } else {
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "Internal Server Error".to_string(),
                )
            };

        (
            status,
            Json(json!({
                "statusCode": status.as_u16(),
                "message": message,
                "timestamp": chrono::Utc::now().to_rfc3339(),
            })),
        )
            .into_response()
    }
}
