use meshestra::prelude::*;
use std::sync::Arc;
use uuid::Uuid;
use super::model::{Product, CreateProductRequest};
use super::repository::ProductRepository;

#[derive(Injectable)]
pub struct ProductService {
    repository: Arc<dyn ProductRepository>,
}

impl ProductService {
    pub async fn create(&self, req: CreateProductRequest) -> Product {
        let product = Product {
            id: Uuid::new_v4().to_string(),
            name: req.name,
            price: req.price,
        };
        self.repository.save(product.clone()).await;
        product
    }

    pub async fn get(&self, id: String) -> Result<Product> {
        self.repository.find_by_id(&id).await
            .ok_or_else(|| MeshestraError::DependencyNotFound { 
                type_name: format!("Product {}", id) 
            })
    }
    
    pub async fn list(&self) -> Vec<Product> {
        self.repository.find_all().await
    }
}
