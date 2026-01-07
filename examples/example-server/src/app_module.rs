use crate::infrastructure::transaction::SeaOrmTransactionManager;
use crate::modules::product::{ProductModule, ProductRepository, ProductRepositoryImpl};
use crate::modules::user::{UserModule, UserRepository, UserRepositoryImpl};
use meshestra::prelude::*;
use meshestra::transactional::TransactionManager;

/// Root application module
///
/// Configures all imports, bindings, and providers for the application.
/// Similar to NestJS's AppModule or Spring's @SpringBootApplication.
#[module(
    imports = [UserModule, ProductModule],
    bindings = [
        (dyn TransactionManager => SeaOrmTransactionManager),
        (dyn UserRepository => UserRepositoryImpl),
        (dyn ProductRepository => ProductRepositoryImpl),
        (dyn TransactionManager => SeaOrmTransactionManager),
    ],
)]
pub struct AppModule;
