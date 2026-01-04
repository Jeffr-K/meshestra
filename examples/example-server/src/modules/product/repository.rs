use meshestra::prelude::*;
use async_trait::async_trait;
use std::sync::Arc;
use crate::infrastructure::database::Database;
use super::model::Product;

#[async_trait]
pub trait ProductRepository: Send + Sync {
    async fn find_by_id(&self, id: &str) -> Option<Product>;
    async fn save(&self, product: Product);
    async fn find_all(&self) -> Vec<Product>;
}

#[derive(Injectable)]
pub struct ProductRepositoryImpl {
    db: Arc<Database>,
}

#[async_trait]
impl ProductRepository for ProductRepositoryImpl {
    async fn find_by_id(&self, id: &str) -> Option<Product> {
        self.db.get("products", id)
            .and_then(|data| serde_json::from_str(&data).ok())
    }

    async fn save(&self, product: Product) {
        if let Ok(data) = serde_json::to_string(&product) {
            self.db.insert("products", &product.id, data);
        }
    }
    
    async fn find_all(&self) -> Vec<Product> {
        self.db.scan("products")
            .into_iter()
            .filter_map(|json| serde_json::from_str(&json).ok())
            .collect()
    }
}
