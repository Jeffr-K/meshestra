use rayon::ThreadPool;
use std::sync::Arc;
use tokio::sync::oneshot;

/// Shared thread pool for CPU-bound tasks
#[derive(Clone)]
pub struct WorkerPool {
    pool: Arc<ThreadPool>,
}

impl Default for WorkerPool {
    fn default() -> Self {
        Self::new(num_cpus::get())
    }
}

impl WorkerPool {
    pub fn new(num_threads: usize) -> Self {
        let pool = rayon::ThreadPoolBuilder::new()
            .num_threads(num_threads)
            .build()
            .unwrap();
        Self {
            pool: Arc::new(pool),
        }
    }

    /// Execute a CPU-bound task in the thread pool and return result asynchronously
    pub async fn execute<F, R>(&self, f: F) -> R
    where
        F: FnOnce() -> R + Send + 'static,
        R: Send + 'static,
    {
        let (tx, rx) = oneshot::channel();
        self.pool.spawn(move || {
            let result = f();
            let _ = tx.send(result);
        });

        rx.await.expect("Worker task panicked")
    }
}
