use log::{error, info};

use crate::core::application_error::ApplicationError as AppError;
use crate::core::auth_helpers::{extract_city_id_from_claims, get_user_policies_strict};
use crate::core::authorization::validate_user_creation_permission;
use crate::core::commands::users::CreateUser;
use crate::core::contracts::repository::error::RepositoryError;
use crate::core::entities::auth::UserClaims;
use crate::core::entities::users::UserRecord;
use crate::core::value_objects::policies::Policy;
use crate::core::value_objects::profiles::Profile;
use crate::usecases::users::deps::UserUseCaseDependencies;
use crate::validators::{policy_validator::PolicyValidator, user_validator::UserValidator};

pub struct CreateUserUseCase {
    deps: UserUseCaseDependencies,
}

impl CreateUserUseCase {
    pub fn new(deps: UserUseCaseDependencies) -> Self {
        Self { deps }
    }

    pub async fn execute(
        &self,
        user: CreateUser,
        claims: &UserClaims,
    ) -> Result<UserRecord, AppError> {
        let mut user_with_city = user;
        user_with_city.email = user_with_city.email.trim().to_lowercase();

        info!(
            "[CreateUserUseCase] Starting user creation for email: {}",
            user_with_city.email
        );

        validate_user_creation_permission(&claims.profile, &user_with_city.profile)?;

        if let Some(policies) = user_with_city.permission_policies.as_ref() {
            let claims_policies_json =
                get_user_policies_strict(self.deps.user_repository.as_ref(), claims).await?;
            PolicyValidator::validate_assignment_permission(
                claims,
                &user_with_city.profile,
                policies,
                claims_policies_json.as_ref(),
            )?;
        }

        UserValidator::validate_fields(
            &user_with_city.rank,
            &user_with_city.registration,
            &user_with_city.full_name,
            &user_with_city.profile,
            &user_with_city.email,
            Some(&user_with_city.password),
            "Error adding user: ",
        )?;

        if user_with_city.profile == Profile::CityAdmin
            || user_with_city.profile == Profile::CityUser
        {
            if claims.profile == Profile::CityAdmin {
                let admin_city_id = extract_city_id_from_claims(claims)?;
                user_with_city.city_id = Some(admin_city_id);
            } else if user_with_city.city_id.is_none() {
                return Err(AppError::BadRequest(
                    "Error adding user: city_id is required for CITY_ADMIN and CITY_USER profiles"
                        .to_string(),
                ));
            }
        }

        if user_with_city.profile == Profile::CityAdmin
            && let Some(city_id) = user_with_city.city_id
        {
            let admin_exists = self
                .deps
                .user_repository
                .check_city_admin_exists_for_city(city_id, uuid::Uuid::nil())
                .await
                .map_err(|error| {
                    error!(
                        "[CreateUserUseCase] Failed to check for existing CITY_ADMIN: {:?}",
                        error
                    );
                    AppError::InternalServerError
                })?;

            if admin_exists {
                return Err(AppError::BadRequest(
                    "Error adding user: a CITY_ADMIN already exists for this city".to_string(),
                ));
            }
        }

        let user_exists = self
            .deps
            .user_repository
            .check_user_exists_by_email(user_with_city.email.as_ref())
            .await
            .map_err(|error| {
                error!(
                    "[CreateUserUseCase] Failed to check user existence for {}: {:?}",
                    user_with_city.email, error
                );
                AppError::InternalServerError
            })?;

        if user_exists {
            return Err(AppError::BadRequest(format!(
                "Error adding user: email '{}' already exists",
                user_with_city.email
            )));
        }

        user_with_city.password = self
            .deps
            .password_hasher
            .hash_password(&user_with_city.password)
            .map_err(|error| {
                error!(
                    "[CreateUserUseCase] Failed to hash password for {}: {:?}",
                    user_with_city.email, error
                );
                AppError::InternalServerError
            })?;

        if user_with_city.permission_policies.is_none() {
            let default_policies =
                Policy::default_for_profile(&user_with_city.profile, user_with_city.city_id);
            if !default_policies.is_empty() {
                user_with_city.permission_policies = Some(default_policies);
            }
        }

        match self.deps.user_repository.create_user(user_with_city).await {
            Ok(user) => Ok(user),
            Err(RepositoryError::DuplicateEntry(msg)) => Err(AppError::BadRequest(msg)),
            Err(RepositoryError::ReferencedEntityNotFound(msg)) => Err(AppError::BadRequest(msg)),
            Err(error) => {
                error!("[CreateUserUseCase] Failed to save user: {:?}", error);
                Err(AppError::InternalServerError)
            }
        }
    }
}
