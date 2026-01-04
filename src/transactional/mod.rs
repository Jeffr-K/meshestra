use crate::error::MeshestraError;
use crate::interceptor::{Interceptor, InterceptorResult, Next};
use async_trait::async_trait;
use axum::{body::Body, http::Request};
use std::sync::Arc;
use tokio::sync::Mutex;

/// Isolation levels for transactions
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IsolationLevel {
    ReadUncommitted,
    ReadCommitted,
    RepeatableRead,
    Serializable,
}

/// Propagation behaviors for transactions
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Propagation {
    Required,
    RequiresNew,
    Supports,
    Mandatory,
    Nested,
    Never,
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
pub trait Transaction: Send + Sync {
    /// Commit the transaction
    async fn commit(&mut self) -> Result<(), MeshestraError>;

    /// Rollback the transaction
    async fn rollback(&mut self) -> Result<(), MeshestraError>;
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
