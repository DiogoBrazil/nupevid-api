use log::{error, info};
use uuid::Uuid;

use crate::core::application_error::ApplicationError as AppError;
use crate::core::auth_helpers::extract_city_id_from_claims;
use crate::core::contracts::repository::error::RepositoryError;
use crate::core::entities::auth::UserClaims;
use crate::core::read_models::users::ResetUserPasswordResponse;
use crate::core::value_objects::profiles::Profile;
use crate::usecases::helpers_common::get_user_or_not_found;
use crate::usecases::users::deps::UserUseCaseDependencies;
use crate::usecases::users::helpers::generate_temporary_password;

pub struct ResetUserPasswordByIdUseCase {
    deps: UserUseCaseDependencies,
}

impl ResetUserPasswordByIdUseCase {
    pub fn new(deps: UserUseCaseDependencies) -> Self {
        Self { deps }
    }

    pub async fn execute(
        &self,
        id: Uuid,
        claims: &UserClaims,
    ) -> Result<ResetUserPasswordResponse, AppError> {
        if claims.profile != Profile::Root && claims.profile != Profile::CityAdmin {
            return Err(AppError::Forbidden(
                "Only ROOT or CITY_ADMIN can reset passwords".to_string(),
            ));
        }

        let target_user = get_user_or_not_found(self.deps.user_repository.as_ref(), id).await?;

        if claims.profile == Profile::CityAdmin {
            let admin_city_id = extract_city_id_from_claims(claims)?;
            if target_user.city_id != Some(admin_city_id) {
                return Err(AppError::Forbidden(
                    "CITY_ADMIN can only reset passwords for users in the same city".to_string(),
                ));
            }
        }

        let temporary_password = generate_temporary_password();
        let password_hash = self
            .deps
            .password_hasher
            .hash_password(&temporary_password)
            .map_err(|error| {
                error!(
                    "[ResetUserPasswordByIdUseCase] Failed to hash temporary password: {:?}",
                    error
                );
                AppError::InternalServerError
            })?;

        let expires_at = chrono::Utc::now() + chrono::Duration::hours(1);

        match self
            .deps
            .user_repository
            .reset_user_password_by_id(id, password_hash, expires_at)
            .await
        {
            Ok(_) => {
                info!(
                    "[ResetUserPasswordByIdUseCase] Password reset successfully for user {}",
                    id
                );
                Ok(ResetUserPasswordResponse { temporary_password })
            }
            Err(RepositoryError::NotFound) => Err(AppError::NotFound(format!(
                "User with id '{}' not found",
                id
            ))),
            Err(error) => {
                error!(
                    "[ResetUserPasswordByIdUseCase] Failed to reset password: {:?}",
                    error
                );
                Err(AppError::InternalServerError)
            }
        }
    }
}
