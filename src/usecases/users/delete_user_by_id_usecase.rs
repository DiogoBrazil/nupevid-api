use log::{error, info};
use uuid::Uuid;

use crate::core::application_error::ApplicationError as AppError;
use crate::core::auth_context::AuthContext;
use crate::core::entities::auth::UserClaims;
use crate::core::entities::users::User;
use crate::core::value_objects::policies::Policy;
use crate::core::value_objects::profiles::Profile;
use crate::usecases::helpers_common::get_user_or_not_found;
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

        if existing.profile == Profile::Root && claims.profile != Profile::Root {
            return Err(AppError::Forbidden(
                "Only ROOT can modify ROOT users".to_string(),
            ));
        }

        if claims.profile != Profile::Root
            && let Some(user_city_id) = existing.city_id
        {
            let auth = AuthContext::load(self.deps.user_repository.as_ref(), claims).await?;
            auth.check_policy(&Policy::DeleteUsers, user_city_id)?;
        }

        match self.deps.user_repository.delete_user_by_id(id).await {
            Ok(deleted_user) => {
                info!(
                    "[DeleteUserByIdUseCase] User with id {} deleted successfully",
                    id
                );
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
