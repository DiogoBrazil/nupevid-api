use chrono::Utc;
use log::{error, info};

use crate::core::application_error::ApplicationError as AppError;
use crate::core::contracts::repository::error::RepositoryError;
use crate::core::entities::auth::ClientMetadata;
use crate::core::responses::auth::RefreshResponse;
use crate::usecases::auth::deps::AuthUseCaseDependencies;
use crate::usecases::auth::helpers::{
    AccessTokenSubject, build_new_refresh_token, issue_access_token, parse_refresh_token,
};

pub struct RefreshTokenUseCase {
    deps: AuthUseCaseDependencies,
}

impl RefreshTokenUseCase {
    pub fn new(deps: AuthUseCaseDependencies) -> Self {
        Self { deps }
    }

    pub async fn execute(
        &self,
        refresh_token: String,
        metadata: ClientMetadata,
    ) -> Result<RefreshResponse, AppError> {
        info!("[RefreshTokenUseCase] Processing refresh token rotation");

        let (token_id, secret) = parse_refresh_token(&refresh_token)?;

        let stored = match self
            .deps
            .refresh_token_repository
            .get_refresh_token_by_id(token_id)
            .await
        {
            Ok(token) => token,
            Err(RepositoryError::NotFound) => {
                return Err(AppError::Unauthorized("Invalid refresh token".to_string()));
            }
            Err(error) => {
                error!(
                    "[RefreshTokenUseCase] Failed to load refresh token {}: {:?}",
                    token_id, error
                );
                return Err(AppError::InternalServerError);
            }
        };

        let secret_matches = self
            .deps
            .password_hasher
            .verify_password(&stored.token_hash, &secret)
            .map_err(|_| AppError::InternalServerError)?;

        if !secret_matches {
            return Err(AppError::Unauthorized("Invalid refresh token".to_string()));
        }

        if stored.revoked_at.is_some() {
            // Reuse of a revoked/rotated token: reject as invalid.
            return Err(AppError::Unauthorized("Refresh token revoked".to_string()));
        }

        if stored.expires_at <= Utc::now() {
            return Err(AppError::Unauthorized("Refresh token expired".to_string()));
        }

        let user = match self
            .deps
            .user_repository
            .get_user_by_id(stored.user_id)
            .await
        {
            Ok(user) => user,
            Err(RepositoryError::NotFound) => {
                return Err(AppError::Unauthorized("Invalid refresh token".to_string()));
            }
            Err(error) => {
                error!(
                    "[RefreshTokenUseCase] Failed to load user {}: {:?}",
                    stored.user_id, error
                );
                return Err(AppError::InternalServerError);
            }
        };

        let (access_token, expires_in) = issue_access_token(
            &*self.deps.token_generator,
            &self.deps.config,
            AccessTokenSubject {
                id: user.id,
                rank: &user.rank,
                registration: &user.registration,
                full_name: &user.full_name,
                profile: &user.profile,
                email: &user.email,
                city_id: user.city_id,
            },
        )?;

        let (new_refresh, refresh_token) = build_new_refresh_token(
            &*self.deps.password_hasher,
            user.id,
            self.deps.config.refresh_token_ttl_seconds,
            &metadata,
        )?;

        self.deps
            .refresh_token_repository
            .rotate_refresh_token(stored.id, new_refresh)
            .await
            .map_err(|error| {
                error!(
                    "[RefreshTokenUseCase] Failed to rotate refresh token {}: {:?}",
                    stored.id, error
                );
                AppError::InternalServerError
            })?;

        info!(
            "[RefreshTokenUseCase] Refresh token rotated successfully for user {}",
            user.id
        );

        Ok(RefreshResponse {
            access_token,
            refresh_token,
            token_type: "Bearer".to_string(),
            expires_in,
        })
    }
}
