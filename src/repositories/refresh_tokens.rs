use async_trait::async_trait;
use log::info;
use sqlx::PgPool;
use uuid::Uuid;

use crate::core::contracts::repository::error::RepositoryError;
use crate::core::contracts::repository::refresh_tokens::RefreshTokenRepository;
use crate::core::entities::auth::{NewRefreshToken, RefreshToken};
use crate::repositories::error_mapper::map_sqlx_error;
use crate::repositories::queries::refresh_tokens::RefreshTokenQueries;

use super::models::refresh_tokens::RefreshTokenRow;

#[derive(Clone)]
pub struct PgRefreshTokenRepository {
    pool: PgPool,
}

impl PgRefreshTokenRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    async fn insert_with<'e, E>(
        executor: E,
        new_token: &NewRefreshToken,
    ) -> Result<RefreshToken, RepositoryError>
    where
        E: sqlx::PgExecutor<'e>,
    {
        let created: RefreshToken = sqlx::query_as::<_, RefreshTokenRow>(
            RefreshTokenQueries::CREATE_REFRESH_TOKEN,
        )
        .bind(new_token.id)
        .bind(new_token.user_id)
        .bind(&new_token.token_hash)
        .bind(new_token.expires_at)
        .bind(&new_token.user_agent)
        .bind(&new_token.ip_address)
        .fetch_one(executor)
        .await
        .map_err(map_sqlx_error)?
        .into();

        Ok(created)
    }
}

#[async_trait]
impl RefreshTokenRepository for PgRefreshTokenRepository {
    async fn create_refresh_token(
        &self,
        new_token: NewRefreshToken,
    ) -> Result<RefreshToken, RepositoryError> {
        info!(
            "[Repository] Creating refresh token {} for user {}",
            new_token.id, new_token.user_id
        );
        Self::insert_with(&self.pool, &new_token).await
    }

    async fn get_refresh_token_by_id(&self, id: Uuid) -> Result<RefreshToken, RepositoryError> {
        let token: RefreshToken =
            sqlx::query_as::<_, RefreshTokenRow>(RefreshTokenQueries::GET_REFRESH_TOKEN_BY_ID)
                .bind(id)
                .fetch_one(&self.pool)
                .await
                .map_err(map_sqlx_error)?
                .into();
        Ok(token)
    }

    async fn rotate_refresh_token(
        &self,
        old_id: Uuid,
        new_token: NewRefreshToken,
    ) -> Result<RefreshToken, RepositoryError> {
        info!(
            "[Repository] Rotating refresh token {} -> {}",
            old_id, new_token.id
        );

        let mut tx = self.pool.begin().await.map_err(map_sqlx_error)?;

        let created = Self::insert_with(&mut *tx, &new_token).await?;

        sqlx::query(RefreshTokenQueries::REVOKE_AND_REPLACE_REFRESH_TOKEN)
            .bind(old_id)
            .bind(new_token.id)
            .execute(&mut *tx)
            .await
            .map_err(map_sqlx_error)?;

        tx.commit().await.map_err(map_sqlx_error)?;

        Ok(created)
    }

    async fn revoke_refresh_token(&self, id: Uuid) -> Result<(), RepositoryError> {
        info!("[Repository] Revoking refresh token {}", id);
        sqlx::query(RefreshTokenQueries::REVOKE_REFRESH_TOKEN)
            .bind(id)
            .execute(&self.pool)
            .await
            .map_err(map_sqlx_error)?;
        Ok(())
    }
}
