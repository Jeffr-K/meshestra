//! Lifecycle Manager
//!
//! Manages the registration and execution of lifecycle hooks.

use super::{
    LifecycleError, OnApplicationBootstrap, OnApplicationShutdown, OnModuleDestroy, OnModuleInit,
    Result,
};
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::RwLock;

/// A wrapper for services that implement lifecycle hooks
struct LifecycleHook<T: ?Sized> {
    service: Arc<RwLock<T>>,
    name: String,
}

impl<T: ?Sized> LifecycleHook<T> {
    fn new(service: Arc<RwLock<T>>, name: impl Into<String>) -> Self {
        Self {
            service,
            name: name.into(),
        }
    }
}

/// Manages lifecycle hooks for all registered services
///
/// The LifecycleManager is responsible for:
/// - Registering services that implement lifecycle hooks
/// - Executing hooks in the correct order
/// - Handling errors during lifecycle transitions
///
/// # Example
///
/// ```rust,ignore
/// use meshestra::lifecycle::LifecycleManager;
///
/// let mut manager = LifecycleManager::new();
///
/// // Register services
/// manager.register_init(db_service.clone(), "DatabaseService");
/// manager.register_destroy(db_service, "DatabaseService");
///
/// // Execute hooks
/// manager.call_module_init().await?;
/// // ... application runs ...
/// manager.call_module_destroy().await?;
/// ```
pub struct LifecycleManager {
    on_init_hooks: Vec<LifecycleHook<dyn OnModuleInit>>,
    on_bootstrap_hooks: Vec<LifecycleHook<dyn OnApplicationBootstrap>>,
    on_shutdown_hooks: Vec<LifecycleHook<dyn OnApplicationShutdown>>,
    on_destroy_hooks: Vec<LifecycleHook<dyn OnModuleDestroy>>,
}

impl Default for LifecycleManager {
    fn default() -> Self {
        Self::new()
    }
}

impl LifecycleManager {
    /// Create a new LifecycleManager
    pub fn new() -> Self {
        Self {
            on_init_hooks: Vec::new(),
            on_bootstrap_hooks: Vec::new(),
            on_shutdown_hooks: Vec::new(),
            on_destroy_hooks: Vec::new(),
        }
    }

    /// Register a service that implements OnModuleInit
    pub fn register_init<T>(&mut self, service: Arc<RwLock<T>>, name: impl Into<String>)
    where
        T: OnModuleInit + 'static,
    {
        self.on_init_hooks.push(LifecycleHook::new(service, name));
    }

    /// Register a service that implements OnApplicationBootstrap
    pub fn register_bootstrap<T>(&mut self, service: Arc<RwLock<T>>, name: impl Into<String>)
    where
        T: OnApplicationBootstrap + 'static,
    {
        self.on_bootstrap_hooks
            .push(LifecycleHook::new(service, name));
    }

    /// Register a service that implements OnApplicationShutdown
    pub fn register_shutdown<T>(&mut self, service: Arc<RwLock<T>>, name: impl Into<String>)
    where
        T: OnApplicationShutdown + 'static,
    {
        self.on_shutdown_hooks
            .push(LifecycleHook::new(service, name));
    }

    /// Register a service that implements OnModuleDestroy
    pub fn register_destroy<T>(&mut self, service: Arc<RwLock<T>>, name: impl Into<String>)
    where
        T: OnModuleDestroy + 'static,
    {
        self.on_destroy_hooks
            .push(LifecycleHook::new(service, name));
    }

    /// Execute all OnModuleInit hooks
    ///
    /// Hooks are executed in the order they were registered.
    pub async fn call_module_init(&self) -> Result<()> {
        tracing::info!("Calling OnModuleInit hooks...");

        for hook in &self.on_init_hooks {
            tracing::debug!("Initializing: {}", hook.name);
            let mut service = hook.service.write().await;
            service.on_module_init().await.map_err(|e| {
                tracing::error!("OnModuleInit failed for {}: {}", hook.name, e);
                LifecycleError::hook_failed(&hook.name, e.to_string())
            })?;
            tracing::debug!("Initialized: {}", hook.name);
        }

        tracing::info!(
            "OnModuleInit complete ({} hooks executed)",
            self.on_init_hooks.len()
        );
        Ok(())
    }

    /// Execute all OnModuleInit hooks with a timeout
    pub async fn call_module_init_with_timeout(&self, timeout: Duration) -> Result<()> {
        tokio::time::timeout(timeout, self.call_module_init())
            .await
            .map_err(|_| {
                LifecycleError::timeout("OnModuleInit", format!("Timeout after {:?}", timeout))
            })?
    }

    /// Execute all OnApplicationBootstrap hooks
    ///
    /// Hooks are executed in the order they were registered.
    pub async fn call_application_bootstrap(&self) -> Result<()> {
        tracing::info!("Calling OnApplicationBootstrap hooks...");

        for hook in &self.on_bootstrap_hooks {
            tracing::debug!("Bootstrapping: {}", hook.name);
            let mut service = hook.service.write().await;
            service.on_application_bootstrap().await.map_err(|e| {
                tracing::error!("OnApplicationBootstrap failed for {}: {}", hook.name, e);
                LifecycleError::hook_failed(&hook.name, e.to_string())
            })?;
            tracing::debug!("Bootstrapped: {}", hook.name);
        }

        tracing::info!(
            "OnApplicationBootstrap complete ({} hooks executed)",
            self.on_bootstrap_hooks.len()
        );
        Ok(())
    }

    /// Execute all OnApplicationBootstrap hooks with a timeout
    pub async fn call_application_bootstrap_with_timeout(&self, timeout: Duration) -> Result<()> {
        tokio::time::timeout(timeout, self.call_application_bootstrap())
            .await
            .map_err(|_| {
                LifecycleError::timeout(
                    "OnApplicationBootstrap",
                    format!("Timeout after {:?}", timeout),
                )
            })?
    }

    /// Execute all OnApplicationShutdown hooks
    ///
    /// Hooks are executed in the order they were registered.
    pub async fn call_application_shutdown(&self) -> Result<()> {
        tracing::info!("Calling OnApplicationShutdown hooks...");

        for hook in &self.on_shutdown_hooks {
            tracing::debug!("Shutting down: {}", hook.name);
            let mut service = hook.service.write().await;
            if let Err(e) = service.on_application_shutdown().await {
                // Log error but continue with other hooks
                tracing::error!("OnApplicationShutdown failed for {}: {}", hook.name, e);
            }
            tracing::debug!("Shutdown complete: {}", hook.name);
        }

        tracing::info!(
            "OnApplicationShutdown complete ({} hooks executed)",
            self.on_shutdown_hooks.len()
        );
        Ok(())
    }

    /// Execute all OnModuleDestroy hooks
    ///
    /// Hooks are executed in **reverse order** to properly handle dependencies.
    pub async fn call_module_destroy(&self) -> Result<()> {
        tracing::info!("Calling OnModuleDestroy hooks...");

        // Execute in reverse order
        for hook in self.on_destroy_hooks.iter().rev() {
            tracing::debug!("Destroying: {}", hook.name);
            let mut service = hook.service.write().await;
            if let Err(e) = service.on_module_destroy().await {
                // Log error but continue with other hooks
                tracing::error!("OnModuleDestroy failed for {}: {}", hook.name, e);
            }
            tracing::debug!("Destroyed: {}", hook.name);
        }

        tracing::info!(
            "OnModuleDestroy complete ({} hooks executed)",
            self.on_destroy_hooks.len()
        );
        Ok(())
    }

    /// Execute all OnModuleDestroy hooks with a timeout
    pub async fn call_module_destroy_with_timeout(&self, timeout: Duration) -> Result<()> {
        tokio::time::timeout(timeout, self.call_module_destroy())
            .await
            .map_err(|_| {
                LifecycleError::timeout("OnModuleDestroy", format!("Timeout after {:?}", timeout))
            })?
    }

    /// Get the number of registered init hooks
    pub fn init_hook_count(&self) -> usize {
        self.on_init_hooks.len()
    }

    /// Get the number of registered bootstrap hooks
    pub fn bootstrap_hook_count(&self) -> usize {
        self.on_bootstrap_hooks.len()
    }

    /// Get the number of registered shutdown hooks
    pub fn shutdown_hook_count(&self) -> usize {
        self.on_shutdown_hooks.len()
    }

    /// Get the number of registered destroy hooks
    pub fn destroy_hook_count(&self) -> usize {
        self.on_destroy_hooks.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct TestService {
        initialized: bool,
        bootstrapped: bool,
        shutdown: bool,
        destroyed: bool,
    }

    impl TestService {
        fn new() -> Self {
            Self {
                initialized: false,
                bootstrapped: false,
                shutdown: false,
                destroyed: false,
            }
        }
    }

    #[async_trait::async_trait]
    impl OnModuleInit for TestService {
        async fn on_module_init(&mut self) -> Result<()> {
            self.initialized = true;
            Ok(())
        }
    }

    #[async_trait::async_trait]
    impl OnApplicationBootstrap for TestService {
        async fn on_application_bootstrap(&mut self) -> Result<()> {
            self.bootstrapped = true;
            Ok(())
        }
    }

    #[async_trait::async_trait]
    impl OnApplicationShutdown for TestService {
        async fn on_application_shutdown(&mut self) -> Result<()> {
            self.shutdown = true;
            Ok(())
        }
    }

    #[async_trait::async_trait]
    impl OnModuleDestroy for TestService {
        async fn on_module_destroy(&mut self) -> Result<()> {
            self.destroyed = true;
            Ok(())
        }
    }

    #[tokio::test]
    async fn test_lifecycle_hooks() {
        let service = Arc::new(RwLock::new(TestService::new()));

        let mut manager = LifecycleManager::new();
        manager.register_init(Arc::clone(&service), "TestService");
        manager.register_bootstrap(Arc::clone(&service), "TestService");
        manager.register_shutdown(Arc::clone(&service), "TestService");
        manager.register_destroy(Arc::clone(&service), "TestService");

        // Test init
        manager.call_module_init().await.unwrap();
        assert!(service.read().await.initialized);

        // Test bootstrap
        manager.call_application_bootstrap().await.unwrap();
        assert!(service.read().await.bootstrapped);

        // Test shutdown
        manager.call_application_shutdown().await.unwrap();
        assert!(service.read().await.shutdown);

        // Test destroy
        manager.call_module_destroy().await.unwrap();
        assert!(service.read().await.destroyed);
    }

    #[tokio::test]
    async fn test_destroy_reverse_order() {
        let order = Arc::new(RwLock::new(Vec::new()));

        struct OrderedService {
            id: usize,
            order: Arc<RwLock<Vec<usize>>>,
        }

        #[async_trait::async_trait]
        impl OnModuleDestroy for OrderedService {
            async fn on_module_destroy(&mut self) -> Result<()> {
                self.order.write().await.push(self.id);
                Ok(())
            }
        }

        let mut manager = LifecycleManager::new();

        for i in 0..3 {
            let service = Arc::new(RwLock::new(OrderedService {
                id: i,
                order: Arc::clone(&order),
            }));
            manager.register_destroy(service, format!("Service{}", i));
        }

        manager.call_module_destroy().await.unwrap();

        // Should be destroyed in reverse order
        let order = order.read().await;
        assert_eq!(*order, vec![2, 1, 0]);
    }
}
