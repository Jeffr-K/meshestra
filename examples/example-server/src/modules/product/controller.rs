use crate::modules::product::model::{CreateProductRequest, Product};
use crate::modules::product::service::ProductService;
use meshestra::common::StatusCode;
use meshestra::prelude::*;
use std::sync::Arc;

#[controller(path = "/products")]
pub struct ProductController {
    service: Arc<ProductService>,
}

#[routes(ProductController)]
impl ProductController {
    #[post("/")]
    pub async fn create(&self, #[body] req: CreateProductRequest) -> ApiResponse<Product> {
        let product = self.service.create(req).await;
        ApiResponse::success(product)
    }

    #[get("/:id")]
    pub async fn get_one(&self, #[param] id: String) -> ApiResponse<Product> {
        match self.service.get(id).await {
            Ok(product) => ApiResponse::success(product),
            Err(e) => ApiResponse::error(StatusCode::NotFound, format!("Product not found: {}", e)),
        }
    }

    #[get("/all")]
    pub async fn list(&self) -> ApiResponse<Vec<Product>> {
        let products = self.service.list().await;
        ApiResponse::success(products)
    }
}
