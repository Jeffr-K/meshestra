use crate::di::Container;
use std::sync::Arc;

/// Builder for constructing a dependency injection container
///
/// Use this to configure and register services before building the final immutable container.
///
/// # Example
/// ```
/// use meshestra::di::ContainerBuilder;
/// use std::sync::Arc;
///
/// // 1. Define your trait and implementation
/// trait Database: Send + Sync {
///     fn query(&self) -> &'static str;
/// }
///
/// struct PostgresDatabase;
/// impl Database for PostgresDatabase {
///     fn query(&self) -> &'static str { "Postgres" }
/// }
///
/// // 2. Build the container
/// let builder = ContainerBuilder::new()
///     .register(PostgresDatabase) // Register the concrete type
///     .bind::<dyn Database, _, _>(|c| c as Arc<dyn Database>); // Bind trait to impl
///
/// let container = builder.build();
///
/// // 3. Resolve the trait
/// let db = container.resolve_trait::<dyn Database>().unwrap();
/// assert_eq!(db.query(), "Postgres");
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
