use async_trait::async_trait;
use meshestra::error::MeshestraError;
use meshestra::transactional::{Transaction, TransactionManager, TransactionOptions};

pub struct MockTransaction;

#[async_trait]
impl Transaction for MockTransaction {
    async fn commit(&mut self) -> Result<(), MeshestraError> {
        println!("MockTransaction: Commit");
        Ok(())
    }

    async fn rollback(&mut self) -> Result<(), MeshestraError> {
        println!("MockTransaction: Rollback");
        Ok(())
    }
}

pub struct MockTransactionManager;

#[async_trait]
impl TransactionManager for MockTransactionManager {
    async fn begin(
        &self,
        options: TransactionOptions,
    ) -> Result<Box<dyn Transaction>, MeshestraError> {
        println!("MockTransactionManager: Begin with options {:?}", options);
        Ok(Box::new(MockTransaction))
    }
}
