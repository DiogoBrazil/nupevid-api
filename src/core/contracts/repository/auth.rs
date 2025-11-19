use async_trait::async_trait;
use crate::core::entities::auth::CompleteUserData;

#[async_trait]
pub trait AuthRepository {
    async fn get_complete_user_data_by_email(&self, email: String) -> Result<CompleteUserData, sqlx::Error>;
}
