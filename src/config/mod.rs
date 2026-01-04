use dashmap::DashMap;
use std::env;
use std::sync::Arc;

/// Configuration service
#[derive(Clone, Default)]
pub struct ConfigService {
    config: Arc<DashMap<String, String>>,
}

impl ConfigService {
    pub fn new() -> Self {
        let service = Self::default();
        // Load from env?
        for (key, value) in env::vars() {
            service.set(&key, &value);
        }
        service
    }

    pub fn get(&self, key: &str) -> Option<String> {
        self.config.get(key).map(|v| v.clone())
    }

    pub fn set(&self, key: &str, value: &str) {
        self.config.insert(key.to_string(), value.to_string());
    }
}
