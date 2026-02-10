use crate::di::Container;
use crate::error::Result;
use std::marker::PhantomData;

/// A marker struct used in the `#[module]` macro to configure providers.
///
/// This struct and its methods are placeholders for the macro parser and have
/// no runtime behavior. They provide a fluent API for defining trait-based
/// providers.
///
/// # Example
/// ```ignore
/// #[module(
///     providers = [
///         UserService, // Standard provider
///         Provider::new(UserRepositoryImpl).for_trait::<dyn UserRepository>(), // Trait provider
///     ]
/// )]
/// pub struct UserModule;
/// ```
pub struct Provider<T: ?Sized>(PhantomData<T>);

impl<T> Provider<T> {
    /// Marks a struct as a provider.
    pub fn new<U>(_: U) -> Self {
        Provider(PhantomData)
    }
}

impl<T: ?Sized> Provider<T> {
    /// Binds the provider to a specific trait.
    pub fn for_trait<U: ?Sized>(&self) -> Provider<U> {
        Provider(PhantomData)
    }
}

/// Trait for application modules
///
/// Modules are typically defined using the `#[module]` macro, which automatically
/// implements this trait and generates the registration logic.
pub trait Module {
    /// Register all providers and controllers in this module
    fn register(container: &mut Container) -> Result<()>;
}
