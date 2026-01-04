use std::sync::Arc;
use crate::di::Container;

/// Builder for constructing a dependency injection container
///
/// Use this to configure and register services before building the final immutable container.
///
/// # Example
/// ```
/// let container = ContainerBuilder::new()
///     .register(Database::new())
///     .bind::<dyn Database, PostgresDatabase>()
///     .build();
/// ```
pub struct ContainerBuilder {
    container: Container,
}

impl ContainerBuilder {
    /// Create a new container builder
    pub fn new() -> Self {
        Self {
            container: Container::new(),
        }
    }

    /// Register a service instance
    pub fn register<T: 'static + Send + Sync>(mut self, instance: T) -> Self {
        self.container.register(instance);
        self
    }

    /// Bind a trait to a concrete implementation
    ///
    /// This enables resolving `Arc<dyn Trait>` to the registered implementation.
    /// The implementation must have been registered first (or will be).
    pub fn bind<Trait, Impl, F>(mut self, caster: F) -> Self 
    where
        Trait: ?Sized + 'static + Send + Sync,
        Impl: 'static + Send + Sync,
        F: Fn(Arc<Impl>) -> Arc<Trait> + 'static + Send + Sync,
    {
        self.container.register_trait::<Trait, Impl, F>(caster);
        self
    }

    /// Build the container
    pub fn build(self) -> Container {
        self.container
    }
}

impl Default for ContainerBuilder {
    fn default() -> Self {
        Self::new()
    }
}
