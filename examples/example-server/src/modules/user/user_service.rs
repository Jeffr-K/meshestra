use super::model::{CreateUserRequest, User};
use super::user_repository::UserRepository;
use meshestra::prelude::*;
use std::sync::Arc;
use uuid::Uuid;

use meshestra::transactional::TransactionManager;

#[derive(Injectable)]
pub struct UserService {
    repository: Arc<dyn UserRepository>,
    transaction_manager: Arc<dyn TransactionManager>,
}

impl UserService {
    #[transactional(isolation = Serializable, propagation = RequiresNew)]
    pub async fn create(&self, req: CreateUserRequest) -> Result<User> {
        let user = User {
            id: Uuid::new_v4().to_string(),
            name: req.name,
            email: req.email,
        };

        // In a real app, we would pass 'tx' (implicit or explicit) to repository
        // for now we just test that the macro compiles and commits

        self.repository.save(user.clone()).await;
        Ok(user)
    }

    pub async fn get(&self, id: String) -> Result<User> {
        self.repository
            .find_by_id(&id)
            .await
            .ok_or_else(|| MeshestraError::DependencyNotFound {
                type_name: format!("User {}", id).into(),
            })
    }

    pub async fn list(&self) -> Vec<User> {
        self.repository.find_all().await
    }

    #[transactional(isolation = Serializable, propagation = RequiresNew)]
    pub async fn create_transaction_test(&self, req: CreateUserRequest) -> Result<User> {
        let user = User {
            id: Uuid::new_v4().to_string(),
            name: req.name,
            email: req.email,
        };

        // 1. Save succeeds
        self.repository.save(user.clone()).await;

        // 2. But then an error occurs
        Err(MeshestraError::Internal(
            "Simulated Failure for Rollback Test".to_string(),
        ))
    }
}
