use crate::error::MeshestraError;
use crate::interceptor::{Interceptor, InterceptorResult, Next};
use async_trait::async_trait;
use axum::{body::Body, http::Request};
use std::sync::Arc;
use tokio::sync::Mutex;

tokio::task_local! {
    /// Task-local storage for the active transaction.
    ///
    /// This holds the transaction context for the duration of an async task,
    /// allowing nested functions (like repository methods) to access it
    /// without explicit passing.
    pub static ACTIVE_TRANSACTION: Option<Arc<Mutex<Box<dyn Transaction>>>>;
}

/// Retrieves the currently active transaction from task-local storage.
///
/// This allows repositories or services to get access to the transaction
/// started by a `#[transactional]` method without needing it to be passed
/// as an explicit argument. Returns `None` if no transaction is active.
pub fn get_current_transaction() -> Option<Arc<Mutex<Box<dyn Transaction>>>> {
    ACTIVE_TRANSACTION.try_with(|tx| tx.clone()).unwrap_or(None)
}

/// Represents the isolation levels for database transactions.
///
/// Isolation levels determine how transaction integrity is visible to other
/// transactions and how they interact with each other.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IsolationLevel {
    /// **Level 0: Read Uncommitted**
    ///
    /// Allows a transaction to read data that has been modified by other
    /// transactions but not yet committed.
    ///
    /// - **Pros:** Highest performance (lowest overhead).
    /// - **Cons:** Susceptible to **Dirty Reads**, Non-repeatable Reads, and Phantom Reads.
    /// - **Common use:** Reporting on large datasets where absolute accuracy is not critical.
    ReadUncommitted,

    /// **Level 1: Read Committed**
    ///
    /// Ensures that any data read is committed at the moment it is read.
    /// This is the **default isolation level** for many databases (e.g., PostgreSQL, SQL Server).
    ///
    /// - **Prevents:** Dirty Reads.
    /// - **Cons:** Susceptible to **Non-repeatable Reads** and Phantom Reads.
    /// - **Behavior:** A query within a transaction might see different data if
    ///   another transaction commits changes between reads.
    ReadCommitted,

    /// **Level 2: Repeatable Read**
    ///
    /// Guarantees that if a transaction reads data once, it can read the same
    /// data again and get the same results, even if other transactions commit changes.
    ///
    /// - **Prevents:** Dirty Reads and Non-repeatable Reads.
    /// - **Cons:** Susceptible to **Phantom Reads** (new rows appearing in range queries).
    /// - **Behavior:** Often implemented using locks or MVCC (Multi-Version Concurrency Control).
    RepeatableRead,

    /// **Level 3: Serializable**
    ///
    /// The highest isolation level. It ensures that transactions are executed
    /// in a way that the result is the same as if they were executed serially (one after another).
    ///
    /// - **Prevents:** All concurrency side effects (Dirty Reads, Non-repeatable Reads, Phantom Reads).
    /// - **Pros:** Highest data consistency.
    /// - **Cons:** Lowest performance due to heavy locking or high risk of serialization failures.
    Serializable,
}

/// Represents transaction propagation behaviors.
///
/// Propagation settings determine how a new transaction boundary interacts
/// with an existing transaction context.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Propagation {
    /// **Required (Default)**
    ///
    /// Support a current transaction, create a new one if none exists.
    ///
    /// - **If transaction exists:** Participate in the existing transaction.
    /// - **If no transaction:** Start a new transaction.
    /// - **Behavior:** This is the most common setting and suitable for most cases.
    Required,

    /// **RequiresNew**
    ///
    /// Create a new transaction, and suspend the current transaction if one exists.
    ///
    /// - **If transaction exists:** Suspend it, execute in a new separate transaction,
    ///   then resume the original one.
    /// - **If no transaction:** Start a new transaction.
    /// - **Behavior:** The new transaction commits or rolls back independently of
    ///   the outer transaction.
    RequiresNew,

    /// **Supports**
    ///
    /// Support a current transaction, execute non-transactionally if none exists.
    ///
    /// - **If transaction exists:** Participate in it.
    /// - **If no transaction:** Execute without a transaction context.
    /// - **Behavior:** Useful for methods that can benefit from a transaction
    ///   but don't strictly require one (e.g., read-only operations).
    Supports,

    /// **Mandatory**
    ///
    /// Support a current transaction, throw an error if none exists.
    ///
    /// - **If transaction exists:** Participate in it.
    /// - **If no transaction:** An error/exception will be raised.
    /// - **Behavior:** Use this when a method must always run within a transaction
    ///   provided by a caller.
    Mandatory,

    /// **Nested**
    ///
    /// Execute within a nested transaction if a current transaction exists,
    /// behave like 'Required' otherwise.
    ///
    /// - **If transaction exists:** Create a **Savepoint**. If the nested transaction
    ///   fails, it rolls back only to the savepoint without affecting the outer transaction.
    /// - **If no transaction:** Start a new transaction.
    /// - **Behavior:** Note that this requires database support for savepoints.
    Nested,

    /// **Never**
    ///
    /// Execute non-transactionally, throw an error if a transaction exists.
    ///
    /// - **If transaction exists:** An error/exception will be raised.
    /// - **If no transaction:** Execute normally without a transaction.
    Never,

    /// **NotSupported**
    ///
    /// Execute non-transactionally, suspend the current transaction if one exists.
    ///
    /// - **If transaction exists:** Suspend it, execute without a transaction,
    ///   then resume the original one.
    /// - **If no transaction:** Execute normally without a transaction.
    NotSupported,
}

#[derive(Debug, Clone)]
pub struct TransactionOptions {
    pub isolation: Option<IsolationLevel>,
    pub propagation: Propagation,
    pub read_only: bool,
}

impl Default for TransactionOptions {
    fn default() -> Self {
        Self {
            isolation: None, // Default depends on DB
            propagation: Propagation::Required,
            read_only: false,
        }
    }
}

/// Trait for managing transactions
#[async_trait]
pub trait TransactionManager: Send + Sync + 'static {
    /// Begin a new transaction with options
    async fn begin(
        &self,
        options: TransactionOptions,
    ) -> Result<Box<dyn Transaction>, MeshestraError>;
}

/// A generic transaction abstraction
#[async_trait]
pub trait Transaction: Send + Sync + std::any::Any {
    /// Commit the transaction
    async fn commit(&mut self) -> Result<(), MeshestraError>;

    /// Rollback the transaction
    async fn rollback(&mut self) -> Result<(), MeshestraError>;

    /// Gets this trait object as a mutable `Any` reference for downcasting.
    fn as_any_mut(&mut self) -> &mut dyn std::any::Any;
}

/// Wrapper to store the active transaction in the request extensions.
/// This allows handlers/repositories to retrieve the ongoing transaction.
#[derive(Clone)]
pub struct ActiveTransaction(pub Arc<Mutex<Box<dyn Transaction>>>);

/// Interceptor that wraps the request in a transaction
pub struct TransactionalInterceptor {
    manager: Arc<dyn TransactionManager>,
}

impl TransactionalInterceptor {
    pub fn new(manager: Arc<dyn TransactionManager>) -> Self {
        Self { manager }
    }
}

#[async_trait]
impl Interceptor for TransactionalInterceptor {
    async fn intercept(&self, mut request: Request<Body>, next: Next) -> InterceptorResult {
        // 1. Begin transaction
        let tx = self
            .manager
            .begin(crate::transactional::TransactionOptions::default())
            .await
            .map_err(|e| Box::new(e) as Box<dyn std::error::Error + Send + Sync>)?;

        // 2. Wrap in Arc<Mutex> for shared ownership
        // - One reference goes into the Request Extensions for the handler
        // - One reference stays here for commit/rollback
        let shared_tx = Arc::new(Mutex::new(tx));
        let active_tx = ActiveTransaction(shared_tx.clone());

        request.extensions_mut().insert(active_tx);

        // 3. Run handler
        let result = next.run(request).await;

        // 4. Lock and finalize
        let mut tx_guard = shared_tx.lock().await;

        match result {
            Ok(response) => {
                if response.status().is_success() {
                    tx_guard
                        .commit()
                        .await
                        .map_err(|e| Box::new(e) as Box<dyn std::error::Error + Send + Sync>)?;
                    Ok(response)
                } else {
                    // For client errors (4xx), we usually assume the logic ran correctly but found an issue.
                    // However, dependent on design, one might want to rollback.
                    // Defaulting to commit for consistency, unless it's a 5xx.
                    if response.status().is_server_error() {
                        tx_guard
                            .rollback()
                            .await
                            .map_err(|e| Box::new(e) as Box<dyn std::error::Error + Send + Sync>)?;
                    } else {
                        tx_guard
                            .commit()
                            .await
                            .map_err(|e| Box::new(e) as Box<dyn std::error::Error + Send + Sync>)?;
                    }
                    Ok(response)
                }
            }
            Err(e) => {
                tx_guard
                    .rollback()
                    .await
                    .map_err(|xe| Box::new(xe) as Box<dyn std::error::Error + Send + Sync>)?;
                Err(e)
            }
        }
    }
}
