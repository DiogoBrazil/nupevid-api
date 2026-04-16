use log::{error, info};
use uuid::Uuid;

use crate::core::commands::users::UpdateUserPassword;
use crate::core::contracts::repository::error::RepositoryError;
use crate::core::entities::auth::UserClaims;
use crate::core::entities::users::UserRecord;
use crate::core::application_error::ApplicationError as AppError;
use crate::usecases::users::deps::UserUseCaseDependencies;

pub struct UpdateUserPasswordUseCase {
    deps: UserUseCaseDependencies,
}

impl UpdateUserPasswordUseCase {
    pub fn new(deps: UserUseCaseDependencies) -> Self {
        Self { deps }
    }

    pub async fn execute(
        &self,
        data: UpdateUserPassword,
        claims: &UserClaims,
    ) -> Result<UserRecord, AppError> {
        let id = Uuid::parse_str(&claims.id)
            .map_err(|_| AppError::Unauthorized("Invalid user id in token".to_string()))?;

        let current_password = data
            .current_password
            .as_ref()
            .filter(|password| !password.is_empty())
            .ok_or_else(|| {
                AppError::BadRequest(
                    "Error updating password: current_password is required".to_string(),
                )
            })?;

        let stored_password_hash = match self.deps.user_repository.get_user_password_by_id(id).await
        {
            Ok(hash) => hash,
            Err(RepositoryError::NotFound) => {
                return Err(AppError::NotFound(format!(
                    "User with id '{}' not found",
                    id
                )));
            }
            Err(error) => {
                error!(
                    "[UpdateUserPasswordUseCase] Failed to retrieve user password: {:?}",
                    error
                );
                return Err(AppError::InternalServerError);
            }
        };

        let password_matches = self
            .deps
            .password_hasher
            .verify_password(&stored_password_hash, current_password)
            .map_err(|error| {
                error!(
                    "[UpdateUserPasswordUseCase] Failed to verify password: {:?}",
                    error
                );
                AppError::InternalServerError
            })?;

        if !password_matches {
            return Err(AppError::BadRequest(
                "Error updating password: current password is incorrect".to_string(),
            ));
        }

        if data.new_password.is_empty() {
            return Err(AppError::BadRequest(
                "Error updating password: new_password is required".to_string(),
            ));
        }

        let new_password_hash = self
            .deps
            .password_hasher
            .hash_password(&data.new_password)
            .map_err(|error| {
                error!(
                    "[UpdateUserPasswordUseCase] Failed to hash new password: {:?}",
                    error
                );
                AppError::InternalServerError
            })?;

        match self
            .deps
            .user_repository
            .update_user_password_by_id(id, new_password_hash)
            .await
        {
            Ok(user) => {
                info!(
                    "[UpdateUserPasswordUseCase] Password updated successfully for user {}",
                    id
                );
                Ok(user)
            }
            Err(RepositoryError::NotFound) => {
                Err(AppError::NotFound(format!("User with id '{}' not found", id)))
            }
            Err(error) => {
                error!(
                    "[UpdateUserPasswordUseCase] Failed to update password: {:?}",
                    error
                );
                Err(AppError::InternalServerError)
            }
        }
    }
}
