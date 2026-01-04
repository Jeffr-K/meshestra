use crate::di::Container;
use crate::error::Result;

/// Trait for application modules
///
/// Modules are typically defined using the `#[module]` macro, which automatically
/// implements this trait and generates the registration logic.
///
/// # Example
/// ```
/// use meshestra::module;
///
/// #[module(
///     controllers = [UserController],
///     providers = [UserService, UserRepository],
/// )]
/// pub struct AppModule;
/// ```
pub trait Module {
    /// Register all providers and controllers in this module
    fn register(container: &mut Container) -> Result<()>;
}
