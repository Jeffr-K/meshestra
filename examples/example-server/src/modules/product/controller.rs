use crate::modules::product::model::{CreateProductRequest, Product};
use crate::modules::product::service::ProductService;
use meshestra::prelude::*;
use std::sync::Arc;

#[controller(path = "/products")]
pub struct ProductController {
    service: Arc<ProductService>,
}

#[routes(ProductController)]
impl ProductController {
    #[post("/")]
    pub async fn create(&self, #[body] req: CreateProductRequest) -> Result<Json<Product>> {
        let product = self.service.create(req).await?;
        Ok(Json(product))
    }

    #[get("/{id}")]
    pub async fn get_one(&self, #[param] id: String) -> Result<Json<Product>> {
        let product = self.service.get(id).await?;
        Ok(Json(product))
    }

    #[get("/all")]
    pub async fn list(&self) -> Result<Json<Vec<Product>>> {
        let products = self.service.list().await?;
        Ok(Json(products))
    }
}
