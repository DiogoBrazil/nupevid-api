use log::{error, info};
use uuid::Uuid;

use crate::core::application_error::ApplicationError as AppError;
use crate::core::auth_context::AuthContext;
use crate::core::entities::auth::UserClaims;
use crate::core::entities::users::User;
use crate::core::value_objects::policies::Policy;
use crate::core::value_objects::profiles::Profile;
use crate::usecases::helpers_common::{get_user_or_not_found, user_not_found_error};
use crate::usecases::users::deps::UserUseCaseDependencies;

pub struct DeleteUserByIdUseCase {
    deps: UserUseCaseDependencies,
}

impl DeleteUserByIdUseCase {
    pub fn new(deps: UserUseCaseDependencies) -> Self {
        Self { deps }
    }

    pub async fn execute(&self, id: Uuid, claims: &UserClaims) -> Result<User, AppError> {
        let existing = get_user_or_not_found(self.deps.user_repository.as_ref(), id).await?;

        // Out-of-scope access is answered with the same "not found" as a
        // missing user, so user existence cannot be probed by ID.
        if existing.profile == Profile::Root && claims.profile != Profile::Root {
            return Err(user_not_found_error(id));
        }

        if claims.profile != Profile::Root
            && let Some(user_city_id) = existing.city_id
        {
            let auth = AuthContext::load(self.deps.user_repository.as_ref(), claims).await?;
            auth.check_policy_or_not_found(&Policy::DeleteUsers, user_city_id, || {
                user_not_found_error(id)
            })?;
        }

        match self.deps.user_repository.delete_user_by_id(id).await {
            Ok(deleted_user) => {
                info!(
                    "[DeleteUserByIdUseCase] User with id {} deleted successfully",
                    id
                );
                // Invalidate every session of the removed user.
                if let Err(error) = self
                    .deps
                    .refresh_token_repository
                    .revoke_all_refresh_tokens_for_user(id)
                    .await
                {
                    error!(
                        "[DeleteUserByIdUseCase] Failed to revoke refresh tokens for user {}: {:?}",
                        id, error
                    );
                }
                Ok(deleted_user)
            }
            Err(crate::core::contracts::repository::error::RepositoryError::NotFound) => Err(
                AppError::NotFound(format!("User with id '{}' not found", id)),
            ),
            Err(error) => {
                error!("[DeleteUserByIdUseCase] Failed to delete user: {:?}", error);
                Err(AppError::InternalServerError)
            }
        }
    }
}
