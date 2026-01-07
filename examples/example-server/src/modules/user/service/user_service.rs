use crate::modules::user::domain::{CreateUserRequest, User};
use crate::modules::user::repository::UserRepository;
use crate::user_entity;
use meshestra::prelude::*;
use meshestra::transactional::TransactionManager;
use sea_orm::ActiveValue::Set;
use std::sync::Arc;
use uuid::Uuid;

#[derive(Injectable)]
pub struct UserService {
    repository: Arc<dyn UserRepository>,
    transaction_manager: Arc<dyn TransactionManager>,
}

impl UserService {
    #[transactional]
    pub async fn create_user(&self, req: CreateUserRequest) -> Result<User> {
        // 1. [수정] ActiveModel이 아닌 일반 User 구조체를 생성합니다.
        let user = User {
            id: uuid::Uuid::new_v4().to_string(),
            name: req.name,
            email: req.email,
        };

        // 2. [해결] 리포지토리의 save(&User) 규격에 맞춰 참조(&)를 전달합니다.
        // 이제 "expected &User, found ActiveModel" 에러가 사라집니다.
        let saved_user = self.repository.save(&user).await?;

        Ok(saved_user)
    }

    pub async fn get(&self, id: String) -> Result<User> {
        let user_opt = self.repository.find_by_id(&id).await?;
        user_opt.ok_or_else(|| MeshestraError::DependencyNotFound {
            type_name: format!("User {}", id),
        })
    }

    pub async fn list(&self) -> Result<Vec<User>> {
        let users = self.repository.find_all().await?;
        Ok(users)
    }

    #[transactional(propagation = RequiresNew)]
    pub async fn create_transaction_test(&self, req: CreateUserRequest) -> Result<User> {
        // 1. [수정] ActiveModel 대신 순수 도메인 모델(User)을 생성합니다.
        let user = User {
            id: uuid::Uuid::new_v4().to_string(),
            name: req.name,
            email: req.email,
        };

        // 2. [해결] 리포지토리의 save(&User) 규격에 맞춰 참조(&)를 전달합니다.
        // 이제 "expected &User, found ActiveModel" 에러가 해결됩니다.
        let saved_user = self.repository.save(&user).await?;

        // 3. 정상 반환
        Ok(saved_user)
    }
}
