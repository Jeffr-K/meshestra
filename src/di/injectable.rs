use crate::di::Container;
use crate::error::Result;

/// Trait for types that can be injected from the DI container
///
/// This trait is typically implemented automatically via the `#[derive(Injectable)]` macro.
///
/// # Example
/// ```
/// use meshestra::Injectable;
///
/// #[derive(Injectable)]
/// pub struct UserService {
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
