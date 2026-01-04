use crate::modules::user::domain::{CreateUserRequest, User};
use crate::modules::user::service::UserService;
use meshestra::prelude::*;
use std::sync::Arc;

#[controller(path = "/users")]
pub struct UserController {
    service: Arc<UserService>,
}

#[routes(UserController)]
impl UserController {
    #[post("/")]
    pub async fn create(&self, #[body] req: CreateUserRequest) -> Json<User> {
        let user = self.service.create(req).await.unwrap();
        Json(user)
    }

    #[get("/:id")]
    pub async fn get_one(&self, #[param] id: String) -> Json<User> {
        let user = self.service.get(id).await.unwrap();
        Json(user)
    }

    #[get("/all")]
    pub async fn list(&self) -> Json<Vec<User>> {
        let users = self.service.list().await;
        Json(users)
    }

    #[post("/transaction-test")]
    pub async fn create_transaction_test(
        &self,
        #[body] req: CreateUserRequest,
    ) -> Result<Json<User>> {
        let user = self.service.create_transaction_test(req).await?;
        Ok(Json(user))
    }
}
