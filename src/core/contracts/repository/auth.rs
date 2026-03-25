use async_trait::async_trait;

use super::error::RepositoryError;
use crate::core::entities::auth::CompleteUserData;

#[async_trait]
pub trait AuthRepository: Send + Sync {
    async fn get_complete_user_data_by_email(
        &self,
        email: &str,
    ) -> Result<CompleteUserData, RepositoryError>;
}
