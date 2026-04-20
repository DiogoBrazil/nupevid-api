use log::{error, info};
use uuid::Uuid;

use crate::core::application_error::ApplicationError as AppError;
use crate::core::auth_context::AuthContext;
use crate::core::auth_helpers::get_user_policies_strict;
use crate::core::commands::users::UpdateUser;
use crate::core::contracts::repository::error::RepositoryError;
use crate::core::entities::auth::UserClaims;
use crate::core::entities::users::User;
use crate::core::value_objects::policies::Policy;
use crate::core::value_objects::profiles::Profile;
use crate::usecases::helpers_common::get_user_or_not_found;
use crate::usecases::users::deps::UserUseCaseDependencies;
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
    ) -> Result<User, AppError> {
        info!("[UpdateUserUseCase] Starting user update for id: {}", id);

        let mut data = data;
        data.email = data.email.trim().to_lowercase();

        self.authorize_update(claims, &data, id).await?;
        self.validate_input(&data)?;
        self.validate_policy_changes(claims, &data).await?;
        self.validate_unique_role_rules(&data, id).await?;
        self.check_email_uniqueness(&data.email, id).await?;

        self.persist_update(data, id).await
    }

    async fn authorize_update(
        &self,
        claims: &UserClaims,
        data: &UpdateUser,
        id: Uuid,
    ) -> Result<(), AppError> {
        if data.profile == Profile::Root && claims.profile != Profile::Root {
            return Err(AppError::Forbidden(
                "Only ROOT can assign ROOT profile".to_string(),
            ));
        }

        let existing = get_user_or_not_found(self.deps.user_repository.as_ref(), id).await?;

        if existing.profile == Profile::Root && claims.profile != Profile::Root {
            return Err(AppError::Forbidden(
                "Only ROOT can modify ROOT users".to_string(),
            ));
        }

        if claims.profile != Profile::Root {
            let auth = AuthContext::load(self.deps.user_repository.as_ref(), claims).await?;

            if let Some(existing_city_id) = existing.city_id {
                auth.check_policy(&Policy::UpdateUsers, existing_city_id)?;
            }

            if let Some(new_city_id) = data.city_id
                && Some(new_city_id) != existing.city_id
            {
                auth.check_policy(&Policy::UpdateUsers, new_city_id)?;
            }
        }

        Ok(())
    }

    fn validate_input(&self, data: &UpdateUser) -> Result<(), AppError> {
        UserValidator::validate_fields(
            &data.registration,
            &data.full_name,
            &data.email,
            None,
            "Error updating user: ",
        )?;

        UserValidator::validate_city_requirement(
            &data.profile,
            &data.city_id,
            "Error updating user",
        )
    }

    async fn validate_policy_changes(
        &self,
        claims: &UserClaims,
        data: &UpdateUser,
    ) -> Result<(), AppError> {
        if let Some(policies) = data.permission_policies.as_ref() {
            let claims_policies =
                get_user_policies_strict(self.deps.user_repository.as_ref(), claims).await?;
            PolicyValidator::validate_assignment_permission(
                claims,
                &data.profile,
                policies,
                claims_policies.as_ref(),
            )?;
        }
        Ok(())
    }

    async fn validate_unique_role_rules(
        &self,
        data: &UpdateUser,
        exclude_id: Uuid,
    ) -> Result<(), AppError> {
        if data.profile == Profile::CityAdmin
            && let Some(city_id) = data.city_id
        {
            let admin_exists = self
                .deps
                .user_repository
                .check_city_admin_exists_for_city(city_id, Some(exclude_id))
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
        Ok(())
    }

    async fn check_email_uniqueness(&self, email: &str, exclude_id: Uuid) -> Result<(), AppError> {
        if self
            .deps
            .user_repository
            .check_email_exists_for_other_user(email, exclude_id)
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
                email
            )));
        }
        Ok(())
    }

    async fn persist_update(&self, data: UpdateUser, id: Uuid) -> Result<User, AppError> {
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
