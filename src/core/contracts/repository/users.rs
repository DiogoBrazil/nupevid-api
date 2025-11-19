use async_trait::async_trait;
//use uuid::Uuid;
use crate::core::entities::users::{
    CreateUser,
    //UpdateUser,
    //UpdatePasswordUser,
    UserDataCreatedWithoutPassword,
    //CompleteUserData
};


#[async_trait]
pub trait UserRepository: Send + Sync + 'static {
    async fn create_user(&self, data: CreateUser) -> Result<UserDataCreatedWithoutPassword, sqlx::Error>;
}
