use meshestra::prelude::*;

pub mod controller;
pub mod domain;
mod repository;
mod service;

pub use controller::UserController;
// pub use domain::{CreateUserRequest, User};
pub use repository::{UserRepository, UserRepositoryImpl};
pub use service::UserService;

#[module(
    controllers = [UserController],
    providers = [UserRepositoryImpl, UserService]
)]
pub struct UserModule;
