use async_trait::async_trait;
use uuid::Uuid;

use super::error::RepositoryError;
use crate::core::entities::auth::{NewRefreshToken, RefreshToken};

#[async_trait]
pub trait RefreshTokenRepository: Send + Sync {
    async fn create_refresh_token(
        &self,
        new_token: NewRefreshToken,
    ) -> Result<RefreshToken, RepositoryError>;

    async fn get_refresh_token_by_id(&self, id: Uuid) -> Result<RefreshToken, RepositoryError>;

    /// Rotates a refresh token: inserts the new token and, in the same transaction,
    /// marks the old one as revoked, pointing `replaced_by_token_id` to the new token.
    async fn rotate_refresh_token(
        &self,
        old_id: Uuid,
        new_token: NewRefreshToken,
    ) -> Result<RefreshToken, RepositoryError>;

    /// Idempotently revokes a refresh token (no-op if already revoked).
    async fn revoke_refresh_token(&self, id: Uuid) -> Result<(), RepositoryError>;

    /// Revokes every active refresh token belonging to a user. Used after
    /// password changes/resets, account removal and on refresh-token reuse
    /// detection, so that no previously issued session survives.
    async fn revoke_all_refresh_tokens_for_user(
        &self,
        user_id: Uuid,
    ) -> Result<(), RepositoryError>;
}
