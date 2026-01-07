use async_trait::async_trait;
use meshestra::error::MeshestraError;
use meshestra::prelude::Injectable;
use meshestra::transactional::{Transaction, TransactionManager, TransactionOptions};
use sea_orm::{DatabaseConnection, DatabaseTransaction, TransactionTrait};
use std::any::Any;
use std::sync::Arc;

/// A SeaORM transaction implementation that wraps `sea_orm::DatabaseTransaction`.
/// The actual transaction object from SeaORM is stored inside an Option
/// because SeaORM's commit/rollback methods consume the transaction object.
pub struct SeaOrmTransaction {
    pub inner: Option<DatabaseTransaction>,
}

#[async_trait]
impl Transaction for SeaOrmTransaction {
    /// Commits the transaction to the database.
    async fn commit(&mut self) -> Result<(), MeshestraError> {
        tracing::info!("SeaOrmTransaction: Committing transaction.");
        if let Some(inner) = self.inner.take() {
            inner
                .commit()
                .await
                .map_err(|e| MeshestraError::Internal(e.to_string()))
        } else {
            // This can happen if a transaction is used after being committed/rolled back,
            // which indicates a logic error in the application.
            Err(MeshestraError::Internal(
                "Attempted to commit a transaction that has already been finalized.".to_string(),
            ))
        }
    }

    /// Rolls back the transaction, discarding any changes.
    async fn rollback(&mut self) -> Result<(), MeshestraError> {
        tracing::info!("SeaOrmTransaction: Rolling back transaction.");
        if let Some(inner) = self.inner.take() {
            inner
                .rollback()
                .await
                .map_err(|e| MeshestraError::Internal(e.to_string()))
        } else {
            // This is not necessarily an error, as a failed operation might try to roll back
            // a transaction that was never successfully started or already finalized.
            Ok(())
        }
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
}

/// An injectable transaction manager for SeaORM.
/// It depends on the `DatabaseConnection` pool provided in the DI container.
#[derive(Injectable, Clone)]
pub struct SeaOrmTransactionManager {
    conn: Arc<DatabaseConnection>,
}

#[async_trait]
impl TransactionManager for SeaOrmTransactionManager {
    /// Begins a new database transaction.
    async fn begin(
        &self,
        _options: TransactionOptions, // Options like isolation level can be applied here in a real implementation
    ) -> Result<Box<dyn Transaction>, MeshestraError> {
        tracing::info!("SeaOrmTransactionManager: Beginning transaction.");

        // Start a new transaction from the connection pool
        let db_tx = self
            .conn
            .begin()
            .await
            .map_err(|e| MeshestraError::Internal(e.to_string()))?;

        // Wrap it in our custom transaction type
        let transaction = SeaOrmTransaction { inner: Some(db_tx) };

        Ok(Box::new(transaction))
    }
}
