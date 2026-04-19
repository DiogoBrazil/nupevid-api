use log::{error, info};

use crate::core::application_error::ApplicationError as AppError;
use crate::core::auth_helpers::{extract_city_id_from_claims, get_user_policies_strict};
use crate::core::authorization::validate_user_creation_permission;
use crate::core::commands::users::CreateUser;
use crate::core::contracts::repository::error::RepositoryError;
use crate::core::entities::auth::UserClaims;
use crate::core::entities::users::User;
use crate::core::policy_defaults;
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

    pub async fn execute(&self, user: CreateUser, claims: &UserClaims) -> Result<User, AppError> {
        let mut data = user;
        data.email = data.email.trim().to_lowercase();

        info!(
            "[CreateUserUseCase] Starting user creation for email: {}",
            data.email
        );

        self.authorize_creation(claims, &data).await?;
        self.resolve_target_city(claims, &mut data)?;
        self.validate_input(&data)?;
        self.validate_unique_role_rules(&data).await?;
        self.check_email_uniqueness(&data.email).await?;
        self.hash_password(&mut data).await?;
        Self::apply_default_policies(&mut data);

        self.persist_user(data).await
    }

    async fn authorize_creation(
        &self,
        claims: &UserClaims,
        data: &CreateUser,
    ) -> Result<(), AppError> {
        validate_user_creation_permission(&claims.profile, &data.profile)?;

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

    fn validate_input(&self, data: &CreateUser) -> Result<(), AppError> {
        UserValidator::validate_fields(
            &data.registration,
            &data.full_name,
            &data.email,
            Some(&data.password),
            "Error adding user: ",
        )
    }

    fn resolve_target_city(
        &self,
        claims: &UserClaims,
        data: &mut CreateUser,
    ) -> Result<(), AppError> {
        if data.profile == Profile::CityAdmin || data.profile == Profile::CityUser {
            if claims.profile == Profile::CityAdmin {
                let admin_city_id = extract_city_id_from_claims(claims)?;
                data.city_id = Some(admin_city_id);
            } else if data.city_id.is_none() {
                return Err(AppError::BadRequest(
                    "Error adding user: city_id is required for CITY_ADMIN and CITY_USER profiles"
                        .to_string(),
                ));
            }
        }
        Ok(())
    }

    async fn validate_unique_role_rules(&self, data: &CreateUser) -> Result<(), AppError> {
        if data.profile == Profile::CityAdmin
            && let Some(city_id) = data.city_id
        {
            let admin_exists = self
                .deps
                .user_repository
                .check_city_admin_exists_for_city(city_id, None)
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
        Ok(())
    }

    async fn check_email_uniqueness(&self, email: &str) -> Result<(), AppError> {
        let user_exists = self
            .deps
            .user_repository
            .check_user_exists_by_email(email)
            .await
            .map_err(|error| {
                error!(
                    "[CreateUserUseCase] Failed to check user existence for {}: {:?}",
                    email, error
                );
                AppError::InternalServerError
            })?;

        if user_exists {
            return Err(AppError::BadRequest(format!(
                "Error adding user: email '{}' already exists",
                email
            )));
        }
        Ok(())
    }

    async fn hash_password(&self, data: &mut CreateUser) -> Result<(), AppError> {
        data.password = self
            .deps
            .password_hasher
            .hash_password(&data.password)
            .map_err(|error| {
                error!(
                    "[CreateUserUseCase] Failed to hash password for {}: {:?}",
                    data.email, error
                );
                AppError::InternalServerError
            })?;
        Ok(())
    }

    fn apply_default_policies(data: &mut CreateUser) {
        if data.permission_policies.is_none() {
            let default_policies =
                policy_defaults::default_for_profile(&data.profile, data.city_id);
            if !default_policies.is_empty() {
                data.permission_policies = Some(default_policies);
            }
        }
    }

    async fn persist_user(&self, data: CreateUser) -> Result<User, AppError> {
        match self.deps.user_repository.create_user(data).await {
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
