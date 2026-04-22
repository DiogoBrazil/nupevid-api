use crate::core::{contracts::repository::auth::AuthRepository, entities::auth::CompleteUserData};
use async_trait::async_trait;
use log::info;
use sqlx::PgPool;

use crate::config::querys::auth::AuthQueries;

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
    ) -> Result<CompleteUserData, sqlx::Error> {
        info!(
            "[Repository] Executing SQL query to get user password with email: {}",
            email
        );

        let complete_user_data: CompleteUserData =
            sqlx::query_as(AuthQueries::GET_COMPLETE_USER_DATA_BY_EMAIL)
                .bind(email)
                .fetch_one(&self.pool)
                .await?;

        info!("[Repository] User found with ID: {}", complete_user_data.id);

        Ok(complete_user_data)
    }
}
