use crate::infrastructure::transaction::SeaOrmTransaction;
use crate::modules::product::{model::Product, product_entity};
use async_trait::async_trait;
use meshestra::prelude::*;
use meshestra::transactional::get_current_transaction;
use sea_orm::{
    entity::prelude::*, ActiveModelTrait, ActiveValue, DatabaseConnection, DbErr, EntityTrait,
    TryIntoModel,
};
use std::ops::DerefMut;
use std::sync::Arc;

#[async_trait]
pub trait ProductRepository: Send + Sync {
    async fn find_by_id(&self, id: &str) -> std::result::Result<Option<Product>, DbErr>;
    async fn save(&self, product: &Product) -> std::result::Result<Product, DbErr>;
    async fn find_all(&self) -> std::result::Result<Vec<Product>, DbErr>;
}

#[derive(Injectable, Clone)]
pub struct ProductRepositoryImpl {
    db: Arc<DatabaseConnection>,
}

#[async_trait]
impl ProductRepository for ProductRepositoryImpl {
    async fn find_by_id(&self, id: &str) -> std::result::Result<Option<Product>, DbErr> {
        if let Some(tx_arc) = get_current_transaction() {
            let mut guard = tx_arc.lock().await;
            let sea_tx = guard
                .deref_mut()
                .as_any_mut()
                .downcast_mut::<SeaOrmTransaction>()
                .expect("Failed to downcast to SeaOrmTransaction");

            if let Some(inner_tx) = &sea_tx.inner {
                product_entity::Entity::find_by_id(id.to_string())
                    .one(inner_tx)
                    .await
                    .map(|opt| opt.map(Into::into))
            } else {
                Err(DbErr::Conn(RuntimeErr::Internal(
                    "Transaction already finalized".to_string(),
                )))
            }
        } else {
            product_entity::Entity::find_by_id(id.to_string())
                .one(&*self.db)
                .await
                .map(|opt| opt.map(Into::into))
        }
    }

    async fn save(&self, product: &Product) -> std::result::Result<Product, DbErr> {
        let active_model = product_entity::ActiveModel {
            id: ActiveValue::Set(product.id.clone()),
            name: ActiveValue::Set(product.name.clone()),
            price: ActiveValue::Set(product.price),
        };

        if let Some(tx_arc) = get_current_transaction() {
            let mut guard = tx_arc.lock().await;
            let sea_tx = guard
                .deref_mut()
                .as_any_mut()
                .downcast_mut::<SeaOrmTransaction>()
                .expect("Failed to downcast to SeaOrmTransaction");

            if let Some(inner_tx) = &sea_tx.inner {
                let saved = active_model.save(inner_tx).await?;
                Ok(saved.try_into_model()?.into())
            } else {
                Err(DbErr::Conn(RuntimeErr::Internal(
                    "Transaction already finalized".to_string(),
                )))
            }
        } else {
            let saved = active_model.save(&*self.db).await?;
            Ok(saved.try_into_model()?.into())
        }
    }

    async fn find_all(&self) -> std::result::Result<Vec<Product>, DbErr> {
        if let Some(tx_arc) = get_current_transaction() {
            let mut guard = tx_arc.lock().await;
            let sea_tx = guard
                .deref_mut()
                .as_any_mut()
                .downcast_mut::<SeaOrmTransaction>()
                .expect("Failed to downcast to SeaOrmTransaction");

            if let Some(inner_tx) = &sea_tx.inner {
                product_entity::Entity::find()
                    .all(inner_tx)
                    .await
                    .map(|models| models.into_iter().map(Into::into).collect())
            } else {
                Err(DbErr::Conn(RuntimeErr::Internal(
                    "Transaction already finalized".to_string(),
                )))
            }
        } else {
            product_entity::Entity::find()
                .all(&*self.db)
                .await
                .map(|models| models.into_iter().map(Into::into).collect())
        }
    }
}

impl From<product_entity::Model> for Product {
    fn from(model: product_entity::Model) -> Self {
        Product {
            id: model.id,
            name: model.name,
            price: model.price,
        }
    }
}
