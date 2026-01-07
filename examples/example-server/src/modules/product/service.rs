use super::model::{CreateProductRequest, Product};
use super::repository::ProductRepository;
use meshestra::prelude::*;
use meshestra::transactional::TransactionManager;
use std::sync::Arc;
use uuid::Uuid;

#[derive(Injectable)]
pub struct ProductService {
    repository: Arc<dyn ProductRepository>,
    transaction_manager: Arc<dyn TransactionManager>,
}

impl ProductService {
    #[transactional]
    pub async fn create(&self, req: CreateProductRequest) -> Result<Product> {
        let product = Product {
            id: Uuid::new_v4().to_string(),
            name: req.name,
            price: req.price,
        };
        self.repository.save(&product).await?;
        Ok(product)
    }

    pub async fn get(&self, id: String) -> Result<Product> {
        let product_opt = self.repository.find_by_id(&id).await?;
        product_opt.ok_or_else(|| MeshestraError::DependencyNotFound {
            type_name: format!("Product {}", id),
        })
    }

    pub async fn list(&self) -> Result<Vec<Product>> {
        let products = self.repository.find_all().await?;
        Ok(products)
    }
}
