use log::{error, info};

use crate::core::application_error::ApplicationError as AppError;
use crate::core::contracts::repository::error::RepositoryError;
use crate::usecases::auth::deps::AuthUseCaseDependencies;
use crate::usecases::auth::helpers::parse_refresh_token;

pub struct LogoutUseCase {
    deps: AuthUseCaseDependencies,
}

impl LogoutUseCase {
    pub fn new(deps: AuthUseCaseDependencies) -> Self {
        Self { deps }
    }

    pub async fn execute(&self, refresh_token: String) -> Result<(), AppError> {
        info!("[LogoutUseCase] Processing logout / refresh token revocation");

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
                    "[LogoutUseCase] Failed to load refresh token {}: {:?}",
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

        self.deps
            .refresh_token_repository
            .revoke_refresh_token(stored.id)
            .await
            .map_err(|error| {
                error!(
                    "[LogoutUseCase] Failed to revoke refresh token {}: {:?}",
                    stored.id, error
                );
                AppError::InternalServerError
            })?;

        info!(
            "[LogoutUseCase] Refresh token {} revoked for user {}",
            stored.id, stored.user_id
        );

        Ok(())
    }
}
