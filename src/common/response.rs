use axum::{
    Json,
    http::StatusCode as HttpStatusCode,
    response::{IntoResponse, Response},
};
use serde::Serialize;

/// Standard API response wrapper
///
/// Provides a consistent response format for all API endpoints.
///
/// # Example
/// ```
/// use your_crate::controller::{ApiResponse, StatusCode};
///
/// #[derive(Serialize)]
/// struct User {
///     id: String,
///     name: String,
/// }
///
/// async fn get_user(id: String) -> ApiResponse<User> {
///     let user_result: Result<User, String> = service::find_user(id).await;
///
///     match user_result {
///         Ok(user) => ApiResponse::success(user),
///         Err(_) => ApiResponse::error(StatusCode::NotFound, "User not found"),
///     }
/// }
/// ```
#[derive(Debug, Serialize)]
pub struct ApiResponse<T: Serialize> {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<T>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<ApiError>,

    pub success: bool,

    #[serde(skip)]
    pub http_status: HttpStatusCode,
}

#[derive(Debug, Serialize)]
pub struct ApiError {
    pub code: String,
    pub message: String,
}

impl<T: Serialize> ApiResponse<T> {
    /// Create a successful response with data
    ///
    /// Defaults to HTTP 200 OK.
    pub fn success(data: T) -> Self {
        Self {
            data: Some(data),
            error: None,
            success: true,
            http_status: HttpStatusCode::OK,
        }
    }

    /// Create an error response
    ///
    /// Automatically derives the error `code` from the `StatusCode` variant name
    /// using `strum`'s Display implementation.
    ///
    /// # Example
    /// ```
    /// // Returns a 404 response with code: "NotFound"
    /// ApiResponse::error(StatusCode::NotFound, "Resource missing")
    /// ```
    pub fn error(status: crate::common::StatusCode, message: impl Into<String>) -> ApiResponse<T> {
        ApiResponse {
            data: None,
            error: Some(ApiError {
                code: status.to_string(),
                message: message.into(),
            }),
            success: false,
            http_status: status.into(),
        }
    }
}

impl<T: Serialize> IntoResponse for ApiResponse<T> {
    fn into_response(self) -> Response {
        // Use the stored http_status to provide accurate HTTP semantics
        (self.http_status, Json(self)).into_response()
    }
}
