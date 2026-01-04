//! Lifecycle Hooks Module
//!
//! This module provides lifecycle hooks for managing initialization and cleanup
//! of services during application startup and shutdown.
//!
//! # Lifecycle Phases
//!
//! ```text
//! 1. Configuration Loading
//!    ↓
//! 2. DI Container Creation
//!    ↓
//! 3. Module Registration
//!    ↓
//! 4. OnModuleInit (each service)       ← Lifecycle Hook
//!    ↓
//! 5. OnApplicationBootstrap            ← Lifecycle Hook
//!    ↓
//! 6. Server Start
//!    ↓
//! [Running...]
//!    ↓
//! 7. Shutdown Signal (SIGTERM/SIGINT)
//!    ↓
//! 8. OnApplicationShutdown             ← Lifecycle Hook
//!    ↓
//! 9. OnModuleDestroy (each service)    ← Lifecycle Hook
//!    ↓
//! 10. Server Stop
//! ```
//!
//! # Example
//!
//! ```rust,ignore
//! use meshestra::lifecycle::{OnModuleInit, OnModuleDestroy, LifecycleError};
//! use async_trait::async_trait;
//!
//! #[derive(Injectable)]
//! pub struct DatabaseService {
//!     config: Arc<DatabaseConfig>,
//! }
//!
//! #[async_trait]
//! impl OnModuleInit for DatabaseService {
//!     async fn on_module_init(&mut self) -> Result<(), LifecycleError> {
//!         tracing::info!("Initializing database connection");
//!         Ok(())
//!     }
//! }
//!
//! #[async_trait]
//! impl OnModuleDestroy for DatabaseService {
//!     async fn on_module_destroy(&mut self) -> Result<(), LifecycleError> {
//!         tracing::info!("Closing database connections");
//!         Ok(())
//!     }
//! }
//! ```

mod application;
mod error;
mod manager;
mod shutdown;
mod traits;

pub use application::{Application, ApplicationBuilder};
pub use error::{LifecycleError, Result};
pub use manager::LifecycleManager;
pub use shutdown::{shutdown_signal, ShutdownHandler};
pub use traits::{OnApplicationBootstrap, OnApplicationShutdown, OnModuleDestroy, OnModuleInit};
