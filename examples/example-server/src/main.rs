use axum::Router;
use meshestra::prelude::*;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::RwLock;

mod app_module;
mod infrastructure;
mod modules;

use app_module::AppModule;
use infrastructure::database::Database;
use infrastructure::transaction::MockTransactionManager;
use modules::product::ProductController;
use modules::user::UserController;

#[derive(Clone)]
struct AppState {
    container: Arc<Container>,
}

impl HasContainer for AppState {
    fn get_container(&self) -> &Container {
        &self.container
    }
}

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();

    tracing::info!("🚀 Starting Example Server...");

    // 1. Create database with lifecycle
    let db = Arc::new(RwLock::new(Database::new()));

    // 2. Build container:
    //    - First register infrastructure (no DI dependencies)
    //    - Then AppModule handles imports, bindings, and services
    let mut container = ContainerBuilder::new()
        .register(db.read().await.clone())
        .register(MockTransactionManager)
        .build();

    // Register AppModule (handles bindings, imports, providers)
    AppModule::register(&mut container).expect("Failed to register AppModule");

    // 3. Build Application with Lifecycle management
    let app = Application::builder()
        .container(container)
        .register_lifecycle(Arc::clone(&db), "Database")
        .init_timeout(Duration::from_secs(30))
        .build()
        .await
        .expect("Failed to initialize application");

    // 4. Create Router
    let state = AppState {
        container: app.container().clone(),
    };
    let user_controller = Arc::new(UserController::inject(state.get_container()).unwrap());
    let product_controller = Arc::new(ProductController::inject(state.get_container()).unwrap());

    let router = Router::new()
        .nest(
            UserController::base_path(),
            UserController::router(user_controller),
        )
        .nest(
            ProductController::base_path(),
            ProductController::router(product_controller),
        )
        .with_state(state);

    // 5. Start server with graceful shutdown
    let host = std::env::var("HOST").unwrap_or_else(|_| "0.0.0.0".to_string());
    let port = std::env::var("PORT").unwrap_or_else(|_| "3000".to_string());
    let addr = format!("{}:{}", host, port);

    tracing::info!("✅ Server running on http://127.0.0.1:{}", port);

    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();

    axum::serve(listener, router)
        .with_graceful_shutdown(async move {
            shutdown_signal().await;
            tracing::info!("🛑 Initiating graceful shutdown...");
            if let Err(e) = app.shutdown().await {
                tracing::error!("Error during shutdown: {}", e);
            }
        })
        .await
        .unwrap();

    tracing::info!("👋 Server stopped");
}
