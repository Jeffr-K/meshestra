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
/// use meshestra::common::response::ApiResponse;
/// use meshestra::common::status_code::StatusCode;
/// use serde::Serialize;
///
/// #[derive(Serialize)]
/// struct User {
///     id: String,
///     name: String,
/// }
///
/// async fn get_user(id: String) -> ApiResponse<User> {
///     if id == "1" {
///         let user = User { id: "1".to_string(), name: "Test User".to_string() };
///         ApiResponse::success(user)
///     } else {
///         ApiResponse::error(StatusCode::NotFound, "User not found")
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
    /// use meshestra::common::response::ApiResponse;
    /// use meshestra::common::status_code::StatusCode;
    ///
    /// // Returns a 404 response with code: "NotFound"
    /// let response: ApiResponse<()> = ApiResponse::error(StatusCode::NotFound, "Resource missing");
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
