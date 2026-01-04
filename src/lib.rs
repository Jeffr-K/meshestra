//! # Meshestra
//!
//! A high-performance web framework with built-in dependency injection for Rust.
//!
//! Meshestra combines the best of NestJS (declarative modules and DI) with FastAPI
//! (parameter injection) to provide an ergonomic web development experience in Rust.
//!
//! ## Features
//!
//! - **Dependency Injection**: Compile-time safe DI container with automatic resolution
//! - **Controller-based Routing**: NestJS-style controllers with method-level routing
//! - **Type-safe Extractors**: FastAPI-style parameter injection for Axum handlers
//! - **Trait Object Support**: Inject `Arc<dyn Trait>` with automatic mapping
//! - **Modular Architecture**: Organize code with `#[module]` declarations
//!
//! ## Quick Start
//!
//! ```rust,no_run
//! use meshestra::{Injectable, controller, module, Inject, Container, HasContainer};
//! use axum::{Router, Json, extract::Path};
//! use std::sync::Arc;
//!
//! // 1. Define your service
//! #[derive(Injectable)]
//! pub struct UserService {
//!     // Dependencies are automatically injected
//! }
//!
//! impl UserService {
//!     pub async fn find_one(&self, id: String) -> Result<User, Error> {
//!         // Business logic
//!         todo!()
//!     }
//! }
//!
//! // 2. Define your controller
//! #[controller(path = "/users")]
//! pub struct UserController {
//!     user_service: Arc<UserService>,
//! }
//!
//! impl UserController {
//!     #[get("/{id}")]
//!     async fn get_user(&self, Path(id): Path<String>) -> Json<User> {
//!         let user = self.user_service.find_one(id).await.unwrap();
//!         Json(user)
//!     }
//! }
//!
//! // 3. Define your module
//! #[module(
//!     controllers = [UserController],
//!     providers = [UserService],
//! )]
//! pub struct AppModule;
//!
//! // 4. Bootstrap your application
//! #[derive(Clone)]
//! struct AppState {
//!     container: Arc<Container>,
//! }
//!
//! impl HasContainer for AppState {
//!     fn get_container(&self) -> &Container {
//!         &self.container
//!     }
//! }
//!
//! #[tokio::main]
//! async fn main() {
//!     let mut container = Container::new();
//!     AppModule::register(&mut container).unwrap();
//!
//!     let state = AppState {
//!         container: Arc::new(container),
//!     };
//!
//!     let app = Router::new()
//!         .with_state(state);
//!
//!     // Serve your app...
//! }
//! ```

pub mod aspect;
pub mod common;
pub mod controller;
pub mod di;
pub mod error;
pub mod exception;
pub mod guard;
pub mod interceptor;
pub mod lifecycle;
pub mod messaging;
pub mod module;
pub mod pipe;
pub mod saga;
pub mod transactional;
pub mod worker;

// Re-export core types
pub use common::ApiResponse;
pub use di::{Container, ContainerBuilder, HasContainer, Inject, Injectable, Lazy};
pub use error::{MeshestraError, Result};
pub use module::Module;

// Re-export macros
pub use meshestra_macro::{
    Injectable as DeriveInjectable, body, controller, delete, exception_filter, get, handle,
    module, param, patch, post, put, query, routes, transactional,
};

// Re-export commonly used types from dependencies
pub use async_trait::async_trait;
pub use axum;

/// Prelude module for convenient imports
///
/// ```
/// use meshestra::prelude::*;
/// ```
pub mod prelude {
    pub use crate::aspect::Aspect;
    pub use crate::common::ApiResponse;
    pub use crate::di::{Container, ContainerBuilder, HasContainer, Inject, Injectable, Lazy};
    pub use crate::error::{MeshestraError, Result};
    pub use crate::exception::{ArgumentsHost, ExceptionFilter};
    pub use crate::guard::{Guard, GuardError, GuardResult};
    pub use crate::interceptor::{Interceptor, InterceptorResult, Next};
    pub use crate::lifecycle::{
        Application, ApplicationBuilder, LifecycleError, LifecycleManager, OnApplicationBootstrap,
        OnApplicationShutdown, OnModuleDestroy, OnModuleInit, ShutdownHandler, shutdown_signal,
    };
    pub use crate::messaging::EventBus;
    pub use crate::module::Module;
    pub use crate::pipe::builtins::*;
    pub use crate::pipe::{Pipe, PipeError, PipeResult};
    pub use crate::saga::{SagaOrchestrator, SagaStep};
    pub use crate::transactional::{ActiveTransaction, Transaction, TransactionManager};
    pub use crate::worker::WorkerPool;
    // Re-export specific filters if needed, but maybe not in prelude to avoid clutter
    // pub use crate::exception::http::HttpExceptionFilter;
    pub use crate::{
        DeriveInjectable as Injectable, body, controller, delete, exception_filter, get, handle,
        module, param, patch, post, put, query, routes, transactional,
    };
    pub use async_trait::async_trait;
    pub use axum::{
        Json, Router,
        extract::{Path, Query, State},
        http::StatusCode,
        response::{IntoResponse, Response},
    };
    pub use std::sync::Arc;
}
