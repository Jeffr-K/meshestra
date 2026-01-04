use meshestra::prelude::*;

pub mod controller;
mod model;
mod repository;
mod service;

// pub use model::{Product, CreateProductRequest};
pub use controller::ProductController;
pub use repository::{ProductRepository, ProductRepositoryImpl};
pub use service::ProductService;

#[module(
    controllers = [ProductController],
    providers = [ProductRepositoryImpl, ProductService]
)]
pub struct ProductModule;
