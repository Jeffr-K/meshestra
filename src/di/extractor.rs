use crate::di::Container;
use axum::{
    extract::FromRequestParts,
    http::{StatusCode as HttpStatusCode, request::Parts},
};
use std::sync::Arc;

/// Axum extractor for dependency injection
///
/// This extractor allows you to inject services directly into handler function parameters,
/// similar to FastAPI's `Depends()`.
///
/// # Example
/// ```
/// use crate::Inject;
/// use axum::{Json, extract::Path};
///
/// async fn get_user(
///     Inject(service): Inject<UserService>,
///     Path(id): Path<String>,
/// ) -> Result<Json<User>, ApiError> {
///     let user = service.find_one(id).await?;
///     Ok(Json(user))
/// }
/// ```
pub struct Inject<T>(pub Arc<T>);

/// Trait that AppState must implement to provide the DI container
pub trait HasContainer {
    fn get_container(&self) -> &Container;
}

impl<S, T> FromRequestParts<S> for Inject<T>
where
    S: Send + Sync + HasContainer,
    T: 'static + Send + Sync,
{
    type Rejection = (HttpStatusCode, String);

    async fn from_request_parts(_parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        let container = state.get_container();

        container.resolve::<T>().map(Inject).map_err(|e| {
            (
                HttpStatusCode::INTERNAL_SERVER_ERROR,
                format!("Dependency injection failed: {}", e),
            )
        })
    }
}

/// Deref implementation for convenient access to the inner service
impl<T> std::ops::Deref for Inject<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

/// Clone implementation to allow sharing the Arc
impl<T> Clone for Inject<T> {
    fn clone(&self) -> Self {
        Inject(Arc::clone(&self.0))
    }
}
