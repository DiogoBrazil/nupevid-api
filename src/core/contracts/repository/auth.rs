use crate::core::entities::auth::CompleteUserData;
use async_trait::async_trait;

#[async_trait]
pub trait AuthRepository {
    async fn get_complete_user_data_by_email(
        &self,
        email: &str,
    ) -> Result<CompleteUserData, sqlx::Error>;
}
