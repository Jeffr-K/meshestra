use meshestra::prelude::*;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

#[derive(Clone)]
pub struct Database {
    // Key: format!("{}:{}", table, id)
    storage: Arc<Mutex<HashMap<String, String>>>,
    connected: Arc<Mutex<bool>>,
}

impl Database {
    pub fn new() -> Self {
        Self {
            storage: Arc::new(Mutex::new(HashMap::new())),
            connected: Arc::new(Mutex::new(false)),
        }
    }

    pub fn insert(&self, table: &str, id: &str, data: String) {
        let mut storage = self.storage.lock().unwrap();
        storage.insert(format!("{}:{}", table, id), data);
    }

    pub fn get(&self, table: &str, id: &str) -> Option<String> {
        let storage = self.storage.lock().unwrap();
        storage.get(&format!("{}:{}", table, id)).cloned()
    }

    pub fn scan(&self, table: &str) -> Vec<String> {
        let storage = self.storage.lock().unwrap();
        let prefix = format!("{}:", table);
        storage
            .iter()
            .filter(|(k, _)| k.starts_with(&prefix))
            .map(|(_, v)| v.clone())
            .collect()
    }
}

// Allow injecting Arc<Database>
unsafe impl Send for Database {}
unsafe impl Sync for Database {}

// Injectable implementation (no dependencies)
impl meshestra::di::Injectable for Database {
    fn inject(_container: &meshestra::Container) -> meshestra::Result<Self> {
        Ok(Self::new())
    }
}

// Lifecycle Hooks Implementation
#[async_trait]
impl OnModuleInit for Database {
    async fn on_module_init(&mut self) -> std::result::Result<(), LifecycleError> {
        tracing::info!("📦 Database: Initializing connection pool...");

        // Simulate connection delay
        tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;

        *self.connected.lock().unwrap() = true;
        tracing::info!("✅ Database: Connection pool initialized");
        Ok(())
    }
}

#[async_trait]
impl OnModuleDestroy for Database {
    async fn on_module_destroy(&mut self) -> std::result::Result<(), LifecycleError> {
        tracing::info!("📦 Database: Closing connection pool...");

        // Simulate cleanup
        tokio::time::sleep(tokio::time::Duration::from_millis(50)).await;

        *self.connected.lock().unwrap() = false;
        tracing::info!("✅ Database: Connection pool closed");
        Ok(())
    }
}
