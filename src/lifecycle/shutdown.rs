//! Graceful Shutdown Handler
//!
//! Handles OS signals and performs graceful shutdown of the application.

use super::LifecycleManager;
use std::sync::Arc;
use tokio::signal;

/// Handles graceful shutdown of the application
///
/// ShutdownHandler listens for OS signals (SIGTERM, SIGINT) and
/// coordinates the shutdown process by invoking lifecycle hooks.
///
/// # Example
///
/// ```rust,ignore
/// use meshestra::lifecycle::{LifecycleManager, ShutdownHandler};
/// use std::sync::Arc;
///
/// let lifecycle_manager = Arc::new(LifecycleManager::new());
/// let shutdown_handler = ShutdownHandler::new(Arc::clone(&lifecycle_manager));
///
/// // Spawn shutdown handler
/// tokio::spawn(async move {
///     shutdown_handler.wait_for_shutdown().await;
///     std::process::exit(0);
/// });
/// ```
pub struct ShutdownHandler {
    lifecycle_manager: Arc<LifecycleManager>,
}

impl ShutdownHandler {
    /// Create a new ShutdownHandler
    pub fn new(lifecycle_manager: Arc<LifecycleManager>) -> Self {
        Self { lifecycle_manager }
    }

    /// Wait for a shutdown signal and perform graceful shutdown
    ///
    /// This method blocks until either SIGTERM or SIGINT is received,
    /// then executes all registered shutdown and destroy hooks.
    pub async fn wait_for_shutdown(&self) {
        self.wait_for_signal().await;
        self.shutdown().await;
    }

    /// Wait for shutdown signal (Ctrl+C or SIGTERM)
    async fn wait_for_signal(&self) {
        let ctrl_c = async {
            signal::ctrl_c()
                .await
                .expect("Failed to install Ctrl+C handler");
        };

        #[cfg(unix)]
        let terminate = async {
            signal::unix::signal(signal::unix::SignalKind::terminate())
                .expect("Failed to install SIGTERM handler")
                .recv()
                .await;
        };

        #[cfg(not(unix))]
        let terminate = std::future::pending::<()>();

        tokio::select! {
            _ = ctrl_c => {
                tracing::info!("Received Ctrl+C signal");
            },
            _ = terminate => {
                tracing::info!("Received SIGTERM signal");
            },
        }
    }

    /// Perform graceful shutdown
    async fn shutdown(&self) {
        tracing::info!("Starting graceful shutdown...");

        // Call shutdown hooks
        if let Err(e) = self.lifecycle_manager.call_application_shutdown().await {
            tracing::error!("Error during application shutdown: {}", e);
        }

        // Call destroy hooks
        if let Err(e) = self.lifecycle_manager.call_module_destroy().await {
            tracing::error!("Error during module destroy: {}", e);
        }

        tracing::info!("Graceful shutdown complete");
    }
}

/// Create a future that completes when a shutdown signal is received
///
/// This is a standalone function that can be used without a ShutdownHandler.
///
/// # Example
///
/// ```rust,ignore
/// use meshestra::lifecycle::shutdown_signal;
///
/// tokio::select! {
///     _ = shutdown_signal() => {
///         println!("Shutdown signal received");
///     }
///     _ = server.serve() => {}
/// }
/// ```
pub async fn shutdown_signal() {
    let ctrl_c = async {
        signal::ctrl_c()
            .await
            .expect("Failed to install Ctrl+C handler");
    };

    #[cfg(unix)]
    let terminate = async {
        signal::unix::signal(signal::unix::SignalKind::terminate())
            .expect("Failed to install SIGTERM handler")
            .recv()
            .await;
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        _ = ctrl_c => {
            tracing::info!("Received Ctrl+C signal");
        },
        _ = terminate => {
            tracing::info!("Received SIGTERM signal");
        },
    }
}
