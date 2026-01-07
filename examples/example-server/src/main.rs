use axum::Router;
use meshestra::prelude::*;
use std::sync::Arc;
use std::time::Duration;

// SeaORM 2.0-rc: SchemaBuilder 사용
use sea_orm::{Schema, SchemaBuilder};

mod app_module;
mod infrastructure;
mod modules;

use app_module::AppModule;
use infrastructure::transaction::SeaOrmTransactionManager;
use modules::{
    product::{product_entity, ProductController},
    user::{domain::user_entity, UserController},
};

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
    dotenvy::dotenv().ok();

    tracing::info!("🚀 Starting Example Server (SeaORM 2.0-rc Edition)...");

    // 1. 데이터베이스 연결
    let db_url = std::env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    let db = sea_orm::Database::connect(&db_url)
        .await
        .unwrap_or_else(|e| panic!("Failed to connect to database: {}", e));
    tracing::info!("✅ Database connection established.");

    // 2. [Entity-First] 스키마 동기화
    let schema_helper = Schema::new(db.get_database_backend());
    let builder = SchemaBuilder::new(schema_helper)
        .register(user_entity::Entity)
        .register(product_entity::Entity);

    builder
        .sync(&db)
        .await
        .expect("Failed to synchronize database schema");
    tracing::info!("✅ Database schema synchronized.");

    // 3. Container 구축 (E0382 Move 에러 해결)
    // .register()가 self를 소모하므로 체이닝을 끝까지 이어가거나 변수를 갱신해야 합니다.
    let mut container = ContainerBuilder::new().register(db.clone()).build(); // 여기서 소유권 흐름이 깔끔하게 마무리됩니다.

    // 4. 의존성 주입으로 TransactionManager 생성 및 등록
    // 이미 빌드된 container에서 DatabaseConnection을 찾아 SeaOrmTransactionManager를 만듭니다.
    let transaction_manager = SeaOrmTransactionManager::inject(&container)
        .expect("Failed to inject SeaOrmTransactionManager");

    // 주입된 매니저를 컨테이너에 다시 등록
    container.register(transaction_manager);

    // 5. AppModule 등록 (Interface Binding)
    // 이제 컨테이너 안에 SeaOrmTransactionManager가 들어있으므로 바인딩이 성공합니다.
    AppModule::register(&mut container).expect("Failed to register AppModule");

    let shared_container = Arc::new(container);

    // 6. Application 빌드
    let app = Application::builder()
        .container((*shared_container).clone())
        .init_timeout(Duration::from_secs(30))
        .build()
        .await
        .expect("Failed to initialize application");

    // 7. Axum 설정 및 컨트롤러 주입
    let state = AppState {
        container: shared_container.clone(),
    };

    let user_controller =
        Arc::new(UserController::inject(state.get_container()).expect("User injection failed"));
    let product_controller = Arc::new(
        ProductController::inject(state.get_container()).expect("Product injection failed"),
    );

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

    // 8. 서버 실행
    let addr = "0.0.0.0:3000";
    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    tracing::info!("✅ Server running on http://127.0.0.1:3000");

    axum::serve(listener, router)
        .with_graceful_shutdown(async move {
            tokio::signal::ctrl_c().await.ok();
            tracing::info!("🛑 Initiating graceful shutdown...");
            let _ = app.shutdown().await;
        })
        .await
        .unwrap();

    tracing::info!("👋 Server stopped");
}
