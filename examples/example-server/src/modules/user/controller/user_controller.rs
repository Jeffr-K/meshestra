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
    pub async fn create(&self, #[body] req: CreateUserRequest) -> Result<Json<User>> {
        // [수정] state.container를 쓸 필요 없이 주입된 self.service를 바로 사용합니다.
        let user = self.service.create_user(req).await?;
        Ok(Json(user))
    }

    #[get("/{id}")]
    pub async fn get_one(&self, #[param] id: String) -> Result<Json<User>> {
        let user = self.service.get(id).await?;
        Ok(Json(user))
    }

    #[get("/all")]
    pub async fn list(&self) -> Result<Json<Vec<User>>> {
        let users = self.service.list().await?;
        Ok(Json(users))
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
