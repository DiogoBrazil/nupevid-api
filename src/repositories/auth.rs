use crate::core::{
    contracts::repository::{auth::AuthRepository, error::RepositoryError},
    entities::auth::CompleteUserData,
};
use async_trait::async_trait;
use log::info;
use sqlx::PgPool;

use super::models::auth::CompleteUserDataRow;
use crate::repositories::queries::auth::AuthQueries;

#[derive(Clone)]
pub struct PgAuthRepository {
    pool: PgPool,
}

impl PgAuthRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl AuthRepository for PgAuthRepository {
    async fn get_complete_user_data_by_email(
        &self,
        email: &str,
    ) -> Result<CompleteUserData, RepositoryError> {
        info!(
            "[Repository] Executing SQL query to get user password with email: {}",
            email
        );

        let complete_user_data: CompleteUserData =
            sqlx::query_as::<_, CompleteUserDataRow>(AuthQueries::GET_COMPLETE_USER_DATA_BY_EMAIL)
                .bind(email)
                .fetch_one(&self.pool)
                .await
                .map_err(crate::repositories::error_mapper::map_sqlx_error)?
                .into();

        info!("[Repository] User found with ID: {}", complete_user_data.id);

        Ok(complete_user_data)
    }
}
