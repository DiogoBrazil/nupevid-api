use log::{error, info};
use uuid::Uuid;

use crate::core::application_error::ApplicationError as AppError;
use crate::core::authorization::check_policy;
use crate::core::entities::auth::UserClaims;
use crate::core::entities::users::UserRecord;
use crate::core::value_objects::policies::Policy;
use crate::core::value_objects::profiles::Profile;
use crate::usecases::users::deps::UserUseCaseDependencies;
use crate::usecases::users::helpers::{get_claims_policies_or_empty, get_existing_user};

pub struct DeleteUserByIdUseCase {
    deps: UserUseCaseDependencies,
}

impl DeleteUserByIdUseCase {
    pub fn new(deps: UserUseCaseDependencies) -> Self {
        Self { deps }
    }

    pub async fn execute(&self, id: Uuid, claims: &UserClaims) -> Result<UserRecord, AppError> {
        let existing = get_existing_user(self.deps.user_repository.as_ref(), id).await?;

        if existing.profile == Profile::Root && claims.profile != Profile::Root {
            return Err(AppError::Forbidden(
                "Only ROOT can modify ROOT users".to_string(),
            ));
        }

        if claims.profile != Profile::Root
            && let Some(user_city_id) = existing.city_id
        {
            let policies =
                get_claims_policies_or_empty(self.deps.user_repository.as_ref(), claims).await?;
            check_policy(claims, &Policy::DeleteUsers, user_city_id, &policies)?;
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
