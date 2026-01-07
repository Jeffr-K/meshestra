use crate::infrastructure::transaction::SeaOrmTransaction;
use crate::modules::user::domain::{user_entity, User};
use async_trait::async_trait;
use meshestra::prelude::*;
use meshestra::transactional::get_current_transaction;
use sea_orm::{ActiveModelTrait, ActiveValue, DatabaseConnection, DbErr, EntityTrait, RuntimeErr};
use std::ops::DerefMut;
use std::sync::Arc;

#[async_trait]
pub trait UserRepository: Send + Sync {
    async fn find_by_id(&self, id: &str) -> std::result::Result<Option<User>, DbErr>;
    /// 서비스 계층에서 도메인 모델(&User)을 받아 저장합니다.
    async fn save(&self, user: &User) -> std::result::Result<User, DbErr>;
    async fn find_all(&self) -> std::result::Result<Vec<User>, DbErr>;
}

#[derive(Injectable, Clone)]
pub struct UserRepositoryImpl {
    db: Arc<DatabaseConnection>,
}

#[async_trait]
impl UserRepository for UserRepositoryImpl {
    async fn find_by_id(&self, id: &str) -> std::result::Result<Option<User>, DbErr> {
        if let Some(tx_arc) = get_current_transaction() {
            let mut guard = tx_arc.lock().await;
            let sea_tx = guard
                .deref_mut()
                .as_any_mut()
                .downcast_mut::<SeaOrmTransaction>()
                .expect("Failed to downcast to SeaOrmTransaction");

            if let Some(inner_tx) = &sea_tx.inner {
                user_entity::Entity::find_by_id(id.to_string())
                    .one(inner_tx)
                    .await
                    .map(|opt| opt.map(Into::into))
            } else {
                Err(DbErr::Conn(RuntimeErr::Internal(
                    "Transaction already finalized".to_string(),
                )))
            }
        } else {
            user_entity::Entity::find_by_id(id.to_string())
                .one(&*self.db)
                .await
                .map(|opt| opt.map(Into::into))
        }
    }

    /// [수정] Trait 선언과 일치하도록 &User를 인자로 받습니다.
    async fn save(&self, user: &User) -> std::result::Result<User, DbErr> {
        let active_model = user_entity::ActiveModel {
            id: ActiveValue::Set(user.id.clone()),
            name: ActiveValue::Set(user.name.clone()),
            email: ActiveValue::Set(user.email.clone()),
        };

        if let Some(tx_arc) = get_current_transaction() {
            let mut guard = tx_arc.lock().await;
            let sea_tx = guard
                .deref_mut()
                .as_any_mut()
                .downcast_mut::<SeaOrmTransaction>()
                .expect("Failed to downcast to SeaOrmTransaction");

            if let Some(inner_tx) = &sea_tx.inner {
                // insert 수행 후 결과를 User 도메인 모델로 변환
                let saved = active_model.insert(inner_tx).await?;
                Ok(saved.into())
            } else {
                Err(DbErr::Conn(RuntimeErr::Internal(
                    "Transaction already finalized".into(),
                )))
            }
        } else {
            // 일반 연결로 insert 수행
            let saved = active_model.insert(&*self.db).await?;
            Ok(saved.into())
        }
    }

    async fn find_all(&self) -> std::result::Result<Vec<User>, DbErr> {
        if let Some(tx_arc) = get_current_transaction() {
            let mut guard = tx_arc.lock().await;
            let sea_tx = guard
                .deref_mut()
                .as_any_mut()
                .downcast_mut::<SeaOrmTransaction>()
                .expect("Failed to downcast to SeaOrmTransaction");

            if let Some(inner_tx) = &sea_tx.inner {
                user_entity::Entity::find()
                    .all(inner_tx)
                    .await
                    .map(|models| models.into_iter().map(Into::into).collect())
            } else {
                Err(DbErr::Conn(RuntimeErr::Internal(
                    "Transaction already finalized".to_string(),
                )))
            }
        } else {
            user_entity::Entity::find()
                .all(&*self.db)
                .await
                .map(|models| models.into_iter().map(Into::into).collect())
        }
    }
}

impl From<user_entity::Model> for User {
    fn from(model: user_entity::Model) -> Self {
        User {
            id: model.id,
            name: model.name,
            email: model.email,
        }
    }
}
