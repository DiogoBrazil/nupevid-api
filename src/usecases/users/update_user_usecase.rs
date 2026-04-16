use log::{error, info};
use uuid::Uuid;

use crate::core::application_error::ApplicationError as AppError;
use crate::core::auth_helpers::get_user_policies_strict;
use crate::core::authorization::check_policy;
use crate::core::commands::users::UpdateUser;
use crate::core::contracts::repository::error::RepositoryError;
use crate::core::entities::auth::UserClaims;
use crate::core::entities::users::UserRecord;
use crate::core::value_objects::policies::Policy;
use crate::core::value_objects::profiles::Profile;
use crate::usecases::users::deps::UserUseCaseDependencies;
use crate::usecases::users::helpers::{get_claims_policies_or_empty, get_existing_user};
use crate::validators::{policy_validator::PolicyValidator, user_validator::UserValidator};

pub struct UpdateUserUseCase {
    deps: UserUseCaseDependencies,
}

impl UpdateUserUseCase {
    pub fn new(deps: UserUseCaseDependencies) -> Self {
        Self { deps }
    }

    pub async fn execute(
        &self,
        data: UpdateUser,
        id: Uuid,
        claims: &UserClaims,
    ) -> Result<UserRecord, AppError> {
        info!("[UpdateUserUseCase] Starting user update for id: {}", id);

        let mut data = data;
        data.email = data.email.trim().to_lowercase();

        if data.profile == Profile::Root && claims.profile != Profile::Root {
            return Err(AppError::Forbidden(
                "Only ROOT can assign ROOT profile".to_string(),
            ));
        }

        let existing = get_existing_user(self.deps.user_repository.as_ref(), id).await?;

        if existing.profile == Profile::Root && claims.profile != Profile::Root {
            return Err(AppError::Forbidden(
                "Only ROOT can modify ROOT users".to_string(),
            ));
        }

        if claims.profile != Profile::Root {
            let policies =
                get_claims_policies_or_empty(self.deps.user_repository.as_ref(), claims).await?;

            if let Some(existing_city_id) = existing.city_id {
                check_policy(claims, &Policy::UpdateUsers, existing_city_id, &policies)?;
            }

            if let Some(new_city_id) = data.city_id
                && Some(new_city_id) != existing.city_id
            {
                check_policy(claims, &Policy::UpdateUsers, new_city_id, &policies)?;
            }
        }

        UserValidator::validate_fields(
            &data.rank,
            &data.registration,
            &data.full_name,
            &data.profile,
            &data.email,
            None,
            "Error updating user: ",
        )?;

        UserValidator::validate_city_requirement(
            &data.profile,
            &data.city_id,
            "Error updating user",
        )?;

        if let Some(policies) = data.permission_policies.as_ref() {
            let claims_policies_json =
                get_user_policies_strict(self.deps.user_repository.as_ref(), claims).await?;
            PolicyValidator::validate_assignment_permission(
                claims,
                &data.profile,
                policies,
                claims_policies_json.as_ref(),
            )?;
        }

        if data.profile == Profile::CityAdmin
            && let Some(city_id) = data.city_id
        {
            let admin_exists = self
                .deps
                .user_repository
                .check_city_admin_exists_for_city(city_id, id)
                .await
                .map_err(|error| {
                    error!(
                        "[UpdateUserUseCase] Failed to check for existing CITY_ADMIN: {:?}",
                        error
                    );
                    AppError::InternalServerError
                })?;

            if admin_exists {
                return Err(AppError::BadRequest(
                    "Error updating user: a CITY_ADMIN already exists for this city".to_string(),
                ));
            }
        }

        if self
            .deps
            .user_repository
            .check_email_exists_for_other_user(&data.email, id)
            .await
            .map_err(|error| {
                error!(
                    "[UpdateUserUseCase] Failed to check if email exists for another user: {:?}",
                    error
                );
                AppError::InternalServerError
            })?
        {
            return Err(AppError::BadRequest(format!(
                "Email '{}' is already in use by another user",
                data.email
            )));
        }

        match self.deps.user_repository.update_user_by_id(data, id).await {
            Ok(user) => Ok(user),
            Err(RepositoryError::NotFound) => Err(AppError::NotFound(format!(
                "User with id '{}' not found",
                id
            ))),
            Err(RepositoryError::DuplicateEntry(msg)) => Err(AppError::BadRequest(msg)),
            Err(RepositoryError::ReferencedEntityNotFound(msg)) => Err(AppError::BadRequest(msg)),
            Err(error) => {
                error!(
                    "[UpdateUserUseCase] Error updating user in database: {:?}",
                    error
                );
                Err(AppError::InternalServerError)
            }
        }
    }
}
