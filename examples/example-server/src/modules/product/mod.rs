use meshestra::prelude::*;

pub mod controller;
pub mod model;
pub mod product_entity;
pub mod repository;
pub mod service;

pub use controller::ProductController;
pub use repository::{ProductRepository, ProductRepositoryImpl};
pub use service::ProductService;

#[module(
    controllers = [ProductController],
    providers = [ProductRepositoryImpl, ProductService]
)]
pub struct ProductModule;
