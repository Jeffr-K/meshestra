//! Application Bootstrap
//!
//! Provides a high-level API for bootstrapping Meshestra applications
//! with integrated lifecycle management.

use super::{
    LifecycleError, LifecycleManager, OnApplicationBootstrap, OnApplicationShutdown,
    OnModuleDestroy, OnModuleInit, Result, ShutdownHandler,
};
use crate::di::Container;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::RwLock;

/// Application builder for bootstrapping Meshestra applications
///
/// Provides a fluent API for configuring and starting applications
/// with integrated lifecycle management.
///
/// # Example
///
/// ```rust,ignore
/// use meshestra::lifecycle::Application;
///
/// #[tokio::main]
/// async fn main() {
///     let app = Application::builder()
///         .container(container)
///         .register_lifecycle(database_service, "DatabaseService")
///         .register_lifecycle(cache_warmer, "CacheWarmer")
///         .build()
///         .await
///         .expect("Failed to initialize application");
///
///     // Start server...
///
///     app.shutdown().await;
/// }
/// ```
pub struct Application {
    container: Arc<Container>,
    lifecycle_manager: Arc<LifecycleManager>,
}

impl Application {
    /// Create a new application builder
    pub fn builder() -> ApplicationBuilder {
        ApplicationBuilder::new()
    }

    /// Get a reference to the container
    pub fn container(&self) -> &Arc<Container> {
        &self.container
    }

    /// Get a reference to the lifecycle manager
    pub fn lifecycle_manager(&self) -> &Arc<LifecycleManager> {
        &self.lifecycle_manager
    }

    /// Create a shutdown handler for graceful shutdown
    pub fn shutdown_handler(&self) -> ShutdownHandler {
        ShutdownHandler::new(Arc::clone(&self.lifecycle_manager))
    }

    /// Perform graceful shutdown
    ///
    /// This will call OnApplicationShutdown and OnModuleDestroy hooks.
    pub async fn shutdown(&self) -> Result<()> {
        tracing::info!("Shutting down application...");

        self.lifecycle_manager.call_application_shutdown().await?;
        self.lifecycle_manager.call_module_destroy().await?;

        tracing::info!("Application shutdown complete");
        Ok(())
    }

    /// Spawn a background task that waits for shutdown signals
    /// and performs graceful shutdown automatically.
    ///
    /// Returns a handle that can be used to wait for the shutdown to complete.
    pub fn spawn_shutdown_handler(&self) -> tokio::task::JoinHandle<()> {
        let shutdown_handler = self.shutdown_handler();
        tokio::spawn(async move {
            shutdown_handler.wait_for_shutdown().await;
        })
    }
}

/// Builder for Application
pub struct ApplicationBuilder {
    container: Option<Container>,
    lifecycle_manager: LifecycleManager,
    init_timeout: Option<Duration>,
    bootstrap_timeout: Option<Duration>,
}

impl Default for ApplicationBuilder {
    fn default() -> Self {
        Self::new()
    }
}

impl ApplicationBuilder {
    /// Create a new application builder
    pub fn new() -> Self {
        Self {
            container: None,
            lifecycle_manager: LifecycleManager::new(),
            init_timeout: None,
            bootstrap_timeout: None,
        }
    }

    /// Set the DI container
    pub fn container(mut self, container: Container) -> Self {
        self.container = Some(container);
        self
    }

    /// Set a timeout for OnModuleInit hooks
    pub fn init_timeout(mut self, timeout: Duration) -> Self {
        self.init_timeout = Some(timeout);
        self
    }

    /// Set a timeout for OnApplicationBootstrap hooks
    pub fn bootstrap_timeout(mut self, timeout: Duration) -> Self {
        self.bootstrap_timeout = Some(timeout);
        self
    }

    /// Register a service that implements OnModuleInit
    pub fn on_init<T>(mut self, service: Arc<RwLock<T>>, name: impl Into<String>) -> Self
    where
        T: OnModuleInit + 'static,
    {
        self.lifecycle_manager.register_init(service, name);
        self
    }

    /// Register a service that implements OnApplicationBootstrap
    pub fn on_bootstrap<T>(mut self, service: Arc<RwLock<T>>, name: impl Into<String>) -> Self
    where
        T: OnApplicationBootstrap + 'static,
    {
        self.lifecycle_manager.register_bootstrap(service, name);
        self
    }

    /// Register a service that implements OnApplicationShutdown
    pub fn on_shutdown<T>(mut self, service: Arc<RwLock<T>>, name: impl Into<String>) -> Self
    where
        T: OnApplicationShutdown + 'static,
    {
        self.lifecycle_manager.register_shutdown(service, name);
        self
    }

    /// Register a service that implements OnModuleDestroy
    pub fn on_destroy<T>(mut self, service: Arc<RwLock<T>>, name: impl Into<String>) -> Self
    where
        T: OnModuleDestroy + 'static,
    {
        self.lifecycle_manager.register_destroy(service, name);
        self
    }

    /// Register a service for all lifecycle hooks it implements
    ///
    /// This is a convenience method that registers the service for
    /// init, bootstrap, shutdown, and destroy hooks.
    pub fn register_lifecycle<T>(self, service: Arc<RwLock<T>>, name: impl Into<String>) -> Self
    where
        T: OnModuleInit + OnModuleDestroy + 'static,
    {
        let name = name.into();
        self.on_init(Arc::clone(&service), name.clone())
            .on_destroy(service, name)
    }

    /// Register a service for all lifecycle hooks (full lifecycle)
    pub fn register_full_lifecycle<T>(
        self,
        service: Arc<RwLock<T>>,
        name: impl Into<String>,
    ) -> Self
    where
        T: OnModuleInit
            + OnApplicationBootstrap
            + OnApplicationShutdown
            + OnModuleDestroy
            + 'static,
    {
        let name = name.into();
        self.on_init(Arc::clone(&service), name.clone())
            .on_bootstrap(Arc::clone(&service), name.clone())
            .on_shutdown(Arc::clone(&service), name.clone())
            .on_destroy(service, name)
    }

    /// Build and initialize the application
    ///
    /// This will:
    /// 1. Call all OnModuleInit hooks
    /// 2. Call all OnApplicationBootstrap hooks
    ///
    /// # Errors
    ///
    /// Returns an error if any lifecycle hook fails.
    pub async fn build(self) -> Result<Application> {
        let container = self
            .container
            .ok_or_else(|| LifecycleError::init_failed("Container not provided"))?;

        tracing::info!("Starting application initialization...");

        // Call OnModuleInit hooks
        if let Some(timeout) = self.init_timeout {
            self.lifecycle_manager
                .call_module_init_with_timeout(timeout)
                .await?;
        } else {
            self.lifecycle_manager.call_module_init().await?;
        }

        // Call OnApplicationBootstrap hooks
        if let Some(timeout) = self.bootstrap_timeout {
            self.lifecycle_manager
                .call_application_bootstrap_with_timeout(timeout)
                .await?;
        } else {
            self.lifecycle_manager.call_application_bootstrap().await?;
        }

        tracing::info!("Application initialization complete");

        Ok(Application {
            container: Arc::new(container),
            lifecycle_manager: Arc::new(self.lifecycle_manager),
        })
    }
}
