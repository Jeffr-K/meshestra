//! Lifecycle hook traits
//!
//! These traits define the contract for services that need to participate
//! in application lifecycle events.

use super::LifecycleError;
use async_trait::async_trait;

/// Called after the module's dependencies are resolved
///
/// Use this hook to:
/// - Initialize database connections
/// - Warm up caches
/// - Subscribe to message queues
/// - Establish external service connections
///
/// # Example
///
/// ```rust,ignore
/// use meshestra::lifecycle::{OnModuleInit, LifecycleError};
/// use async_trait::async_trait;
///
/// #[async_trait]
/// impl OnModuleInit for DatabaseService {
///     async fn on_module_init(&mut self) -> Result<(), LifecycleError> {
///         self.connection_pool = create_pool(&self.config).await
///             .map_err(|e| LifecycleError::init_failed(e.to_string()))?;
///         Ok(())
///     }
/// }
/// ```
#[async_trait]
pub trait OnModuleInit: Send + Sync {
    /// Called when the module is initialized
    ///
    /// This is invoked after all dependencies are resolved but before
    /// the application starts accepting requests.
    async fn on_module_init(&mut self) -> Result<(), LifecycleError>;
}

/// Called after all modules are initialized
///
/// Use this hook to:
/// - Start background tasks
/// - Schedule cron jobs
/// - Register event listeners
/// - Perform warm-up operations that depend on other services
///
/// # Example
///
/// ```rust,ignore
/// use meshestra::lifecycle::{OnApplicationBootstrap, LifecycleError};
/// use async_trait::async_trait;
///
/// #[async_trait]
/// impl OnApplicationBootstrap for CacheWarmer {
///     async fn on_application_bootstrap(&mut self) -> Result<(), LifecycleError> {
///         // Pre-load frequently accessed data
///         self.warm_cache().await
///             .map_err(|e| LifecycleError::init_failed(e.to_string()))?;
///         Ok(())
///     }
/// }
/// ```
#[async_trait]
pub trait OnApplicationBootstrap: Send + Sync {
    /// Called after all modules have been initialized
    ///
    /// This is the last hook before the application starts accepting requests.
    async fn on_application_bootstrap(&mut self) -> Result<(), LifecycleError>;
}

/// Called when the application receives a shutdown signal
///
/// Use this hook to:
/// - Stop accepting new requests
/// - Wait for in-flight requests to complete
/// - Flush buffers
/// - Cancel pending background tasks
///
/// # Example
///
/// ```rust,ignore
/// use meshestra::lifecycle::{OnApplicationShutdown, LifecycleError};
/// use async_trait::async_trait;
///
/// #[async_trait]
/// impl OnApplicationShutdown for JobScheduler {
///     async fn on_application_shutdown(&mut self) -> Result<(), LifecycleError> {
///         // Stop background jobs
///         for job in &self.jobs {
///             job.abort();
///         }
///         Ok(())
///     }
/// }
/// ```
#[async_trait]
pub trait OnApplicationShutdown: Send + Sync {
    /// Called when the application is about to shut down
    ///
    /// This is invoked when a shutdown signal is received, before
    /// individual modules are destroyed.
    async fn on_application_shutdown(&mut self) -> Result<(), LifecycleError>;
}

/// Called when the application is shutting down
///
/// Use this hook to:
/// - Close database connections
/// - Disconnect from message queues
/// - Clean up temporary files
/// - Release acquired resources
///
/// # Note
///
/// Services are destroyed in **reverse order** of their initialization
/// to properly handle dependencies.
///
/// # Example
///
/// ```rust,ignore
/// use meshestra::lifecycle::{OnModuleDestroy, LifecycleError};
/// use async_trait::async_trait;
///
/// #[async_trait]
/// impl OnModuleDestroy for DatabaseService {
///     async fn on_module_destroy(&mut self) -> Result<(), LifecycleError> {
///         if let Some(conn) = &self.connection {
///             conn.close().await
///                 .map_err(|e| LifecycleError::shutdown_failed(e.to_string()))?;
///         }
///         Ok(())
///     }
/// }
/// ```
#[async_trait]
pub trait OnModuleDestroy: Send + Sync {
    /// Called when the module is being destroyed
    ///
    /// This is invoked during application shutdown, after
    /// OnApplicationShutdown has been called.
    async fn on_module_destroy(&mut self) -> Result<(), LifecycleError>;
}
