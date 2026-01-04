use crate::infrastructure::database::Database;
use crate::modules::user::domain::User;
use async_trait::async_trait;
use meshestra::prelude::*;
use std::sync::Arc;

#[async_trait]
pub trait UserRepository: Send + Sync {
    async fn find_by_id(&self, id: &str) -> Option<User>;
    async fn save(&self, user: User);
    async fn find_all(&self) -> Vec<User>;
}

#[derive(Injectable)]
pub struct UserRepositoryImpl {
    db: Arc<Database>,
}

#[async_trait]
impl UserRepository for UserRepositoryImpl {
    async fn find_by_id(&self, id: &str) -> Option<User> {
        self.db
            .get("users", id)
            .and_then(|data| serde_json::from_str(&data).ok())
    }

    async fn save(&self, user: User) {
        if let Ok(data) = serde_json::to_string(&user) {
            self.db.insert("users", &user.id, data);
        }
    }

    async fn find_all(&self) -> Vec<User> {
        self.db
            .scan("users")
            .into_iter()
            .filter_map(|json| serde_json::from_str(&json).ok())
            .collect()
    }
}
