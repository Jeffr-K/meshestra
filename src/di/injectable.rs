use crate::di::Container;
use crate::error::Result;

/// Trait for types that can be injected from the DI container
///
/// This trait is typically implemented automatically via the `#[derive(Injectable)]` macro.
///
/// # Example
/// ```
/// use meshestra::Injectable; // The trait
/// use meshestra_macro::Injectable; // The derive macro
/// use std::sync::Arc;
///
/// // 1. Define a trait
/// trait UserRepository: Send + Sync {}
///
/// // 2. Derive Injectable on a struct
/// #[derive(Injectable)]
/// pub struct UserService {
///     // This field will be resolved from the container
///     repository: Arc<dyn UserRepository>,
/// }
/// ```
pub trait Injectable: Sized + Send + Sync + 'static {
    /// Create an instance by resolving dependencies from the container
    ///
    /// # Errors
    /// Returns an error if any required dependency is not found in the container.
    fn inject(container: &Container) -> Result<Self>;
}
