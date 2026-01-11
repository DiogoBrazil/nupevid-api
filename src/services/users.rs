use actix_web::{HttpRequest, HttpResponse, web};
use log::{error, info};
use uuid::Uuid;

use crate::adapters::password_hasher::PasswordHasherPort;
use crate::core::contracts::repository::users::UserRepository;
use crate::core::entities::users::{
    CreateUser, ResetUserPasswordResponse, UpdateUser, UpdateUserPassword,
    UserDataCreatedWithoutPassword,
};
use crate::repositories::users::PgUserRepository;
use crate::utils::{
    authorization::{check_policy, validate_user_creation_permission},
    db_error_mapper::map_constraint,
    errors::AppError,
    pagination::{PaginationParams, normalize_pagination},
    responses::{ApiResponse, PaginatedResponse},
    service_helpers::{extract_city_id_from_claims, extract_claims, get_user_policies_strict},
};
use crate::validators::{
    common::{
        POLICY_DELETE_USERS, POLICY_READ_USERS, POLICY_UPDATE_USERS, PROFILE_CITY_ADMIN,
        PROFILE_CITY_USER, PROFILE_ROOT, VALID_POLICIES, generate_default_policies,
        is_assignable_policy, is_valid_policy,
    },
    policy_validator::PolicyValidator,
    user_validator::UserValidator,
};

pub struct UserService {
    user_repository: web::Data<PgUserRepository>,
    password_hasher: Box<dyn PasswordHasherPort>,
}

impl UserService {
    pub fn new(
        user_repository: web::Data<PgUserRepository>,
        password_hasher: Box<dyn PasswordHasherPort>,
    ) -> Self {
        Self {
            user_repository,
            password_hasher,
        }
    }

    pub async fn create_user(
        &self,
        user: CreateUser,
        req: HttpRequest,
    ) -> Result<HttpResponse, AppError> {
        let mut user_with_city = user;
        user_with_city.email = user_with_city.email.trim().to_lowercase();

        info!(
            "[UserService] Starting user creation for email: {}",
            user_with_city.email
        );

        let claims = extract_claims(&req)?;

        validate_user_creation_permission(&claims.profile, &user_with_city.profile)?;

        if let Some(policies) = user_with_city.permission_policies.as_ref() {
            PolicyValidator::validate_policy_names(policies)?;
            let claims_policies_json =
                get_user_policies_strict(self.user_repository.as_ref(), &claims).await?;
            PolicyValidator::validate_assignment_permission(
                &claims,
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

        if user_with_city.profile == PROFILE_CITY_ADMIN
            || user_with_city.profile == PROFILE_CITY_USER
        {
            if claims.profile == PROFILE_CITY_ADMIN {
                let admin_city_id = extract_city_id_from_claims(&claims)?;

                info!(
                    "[UserService] CITY_ADMIN creating user - using city_id from token: {}",
                    admin_city_id
                );

                user_with_city.city_id = Some(admin_city_id);
            } else if user_with_city.city_id.is_none() {
                error!("[UserService] city_id is required for CITY_ADMIN and CITY_USER profiles");
                return Err(AppError::BadRequest(
                    "Error adding user: city_id is required for CITY_ADMIN and CITY_USER profiles"
                        .to_string(),
                ));
            }
        }

        if user_with_city.profile == PROFILE_CITY_ADMIN
            && let Some(city_id) = user_with_city.city_id
        {
            let admin_exists = self
                .user_repository
                .check_city_admin_exists_for_city(city_id, Uuid::nil())
                .await
                .map_err(|e| {
                    error!(
                        "[UserService] Failed to check for existing CITY_ADMIN: {:?}",
                        e
                    );
                    AppError::InternalServerError
                })?;

            if admin_exists {
                error!(
                    "[UserService] CITY_ADMIN already exists for city: {}",
                    city_id
                );
                return Err(AppError::BadRequest(
                    "Error adding user: a CITY_ADMIN already exists for this city".to_string(),
                ));
            }
        }

        info!(
            "[UserService] Checking if email already exists: {}",
            user_with_city.email
        );

        let user_exists = self
            .user_repository
            .check_user_exists_by_email(user_with_city.email.as_ref())
            .await
            .map_err(|e| {
                error!(
                    "[UserService] Failed to check user existence for {}: {:?}",
                    user_with_city.email, e
                );
                AppError::InternalServerError
            })?;

        if user_exists {
            error!(
                "[UserService] Email already registered: {}",
                user_with_city.email
            );
            return Err(AppError::BadRequest(format!(
                "Error adding user: email '{}' already exists",
                user_with_city.email
            )));
        }

        info!("[UserService] Email not found. Proceeding with creation.");
        info!("[UserService] Hashing user password");

        user_with_city.password = self
            .password_hasher
            .hash_password(&user_with_city.password)
            .map_err(|e| {
                error!("[UserService] Failed to hash password: {:?}", e);
                AppError::InternalServerError
            })?;

        info!("[UserService] Password hashed successfully");

        if user_with_city.permission_policies.is_none() {
            let default_policies =
                generate_default_policies(&user_with_city.profile, user_with_city.city_id);
            if !default_policies.is_empty() {
                info!(
                    "[UserService] Generated default policies for profile '{}': {:?}",
                    user_with_city.profile,
                    default_policies.keys().collect::<Vec<_>>()
                );
                user_with_city.permission_policies = Some(default_policies);
            } else {
                info!(
                    "[UserService] No default policies for profile '{}'",
                    user_with_city.profile
                );
            }
        } else {
            info!("[UserService] Using permission_policies provided in request");
        }

        info!("[UserService] Saving user to database");

        match self.user_repository.create_user(user_with_city).await {
            Ok(user) => {
                info!(
                    "[UserService] User created successfully with ID: {}",
                    user.id
                );
                Ok(ApiResponse::created(user).into_response())
            }
            Err(sqlx::Error::Database(db_err)) => {
                if let Some(constraint) = db_err.constraint() {
                    if constraint.contains("registration") {
                        error!("[UserService] Registration already exists");
                        return Err(AppError::BadRequest(
                            "Error adding user: registration already exists".to_string(),
                        ));
                    }
                    if let Some(app_err) = map_constraint(
                        Some(constraint),
                        &[("fk_users_city", "Error adding user: city_id not found")],
                    ) {
                        return Err(app_err);
                    }
                }
                error!(
                    "[UserService] Database error while saving user: {:?}",
                    db_err
                );
                Err(AppError::InternalServerError)
            }
            Err(e) => {
                error!("[UserService] Failed to save user: {:?}", e);
                Err(AppError::InternalServerError)
            }
        }
    }

    pub async fn update_user_by_id(
        &self,
        data: UpdateUser,
        id: Uuid,
        req: HttpRequest,
    ) -> Result<HttpResponse, AppError> {
        info!("[UserService] Starting user updation for id: {}", id);

        let claims = extract_claims(&req)?;

        let mut data = data;
        data.email = data.email.trim().to_lowercase();

        if data.profile == PROFILE_ROOT && claims.profile != PROFILE_ROOT {
            return Err(AppError::Forbidden(
                "Only ROOT can assign ROOT profile".to_string(),
            ));
        }

        let existing = match self.user_repository.get_user_by_id(id).await {
            Ok(user) => user,
            Err(sqlx::Error::RowNotFound) => {
                return Err(AppError::NotFound(format!(
                    "User with id '{}' not found",
                    id
                )));
            }
            Err(e) => {
                error!("[UserService] Error fetching user: {:?}", e);
                return Err(AppError::InternalServerError);
            }
        };

        if existing.profile == PROFILE_ROOT && claims.profile != PROFILE_ROOT {
            return Err(AppError::Forbidden(
                "Only ROOT can modify ROOT users".to_string(),
            ));
        }

        if claims.profile != PROFILE_ROOT {
            let policies = get_user_policies_strict(self.user_repository.as_ref(), &claims)
                .await?
                .unwrap_or(serde_json::json!({}));

            if let Some(existing_city_id) = existing.city_id {
                check_policy(&claims, POLICY_UPDATE_USERS, existing_city_id, &policies)?;
            }

            if let Some(new_city_id) = data.city_id
                && Some(new_city_id) != existing.city_id
            {
                check_policy(&claims, POLICY_UPDATE_USERS, new_city_id, &policies)?;
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
            PolicyValidator::validate_policy_names(policies)?;
            let claims_policies_json =
                get_user_policies_strict(self.user_repository.as_ref(), &claims).await?;
            PolicyValidator::validate_assignment_permission(
                &claims,
                &data.profile,
                policies,
                claims_policies_json.as_ref(),
            )?;
        }

        if data.profile == PROFILE_CITY_ADMIN
            && let Some(city_id) = data.city_id
        {
            let admin_exists = self
                .user_repository
                .check_city_admin_exists_for_city(city_id, id)
                .await
                .map_err(|e| {
                    error!(
                        "[UserService] Failed to check for existing CITY_ADMIN: {:?}",
                        e
                    );
                    AppError::InternalServerError
                })?;

            if admin_exists {
                error!(
                    "[UserService] CITY_ADMIN already exists for city: {}",
                    city_id
                );
                return Err(AppError::BadRequest(
                    "Error updating user: a CITY_ADMIN already exists for this city".to_string(),
                ));
            }
        }

        info!("[Service] Checking if the email is already in use by another user");

        if self
            .user_repository
            .check_email_exists_for_other_user(&data.email, id)
            .await?
        {
            return Err(AppError::BadRequest(format!(
                "Email '{}' is already in use by another user",
                data.email
            )));
        }

        info!("[Service] Email is available, proceeding with the update");

        info!("[Service] Saving user to database");
        match self.user_repository.update_user_by_id(data, id).await {
            Ok(user) => {
                info!("[Service] User updated successfully with ID: {}", user.id);
                Ok(ApiResponse::success(user).into_response())
            }
            Err(sqlx::Error::RowNotFound) => {
                error!("[Service] User with id {} not found for update", id);
                Err(AppError::NotFound(format!(
                    "User with id '{}' not found",
                    id
                )))
            }
            Err(sqlx::Error::Database(db_err)) => {
                if let Some(constraint) = db_err.constraint() {
                    if constraint.contains("registration") {
                        error!("[UserService] Registration already exists");
                        return Err(AppError::BadRequest(
                            "Error updating user: registration already exists".to_string(),
                        ));
                    }
                    if let Some(app_err) = map_constraint(
                        Some(constraint),
                        &[("fk_users_city", "Error updating user: city_id not found")],
                    ) {
                        return Err(app_err);
                    }
                }
                error!(
                    "[UserService] Database error while updating user: {:?}",
                    db_err
                );
                Err(AppError::InternalServerError)
            }
            Err(e) => {
                error!("[Service] Error updating user in database: {:?}", e);
                Err(AppError::InternalServerError)
            }
        }
    }

    pub async fn get_user_by_id(
        &self,
        id: Uuid,
        req: HttpRequest,
    ) -> Result<HttpResponse, AppError> {
        info!("[Service] Starting find user by id process for id: {}", id);

        let claims = extract_claims(&req)?;

        match self.user_repository.get_user_by_id(id).await {
            Ok(user) => {
                if user.profile == PROFILE_ROOT && claims.profile != PROFILE_ROOT {
                    return Err(AppError::Forbidden(
                        "Only ROOT can access ROOT users".to_string(),
                    ));
                }

                if claims.profile != PROFILE_ROOT
                    && let Some(user_city_id) = user.city_id
                {
                    let policies = get_user_policies_strict(self.user_repository.as_ref(), &claims)
                        .await?
                        .unwrap_or(serde_json::json!({}));
                    check_policy(&claims, POLICY_READ_USERS, user_city_id, &policies)?;
                }

                info!("[Service] User with id {} found successfully", id);
                Ok(ApiResponse::success(user).into_response())
            }
            Err(sqlx::Error::RowNotFound) => {
                info!("[Service] User with id {} not found", id);
                Err(AppError::NotFound(format!(
                    "User with id '{}' not found",
                    id
                )))
            }
            Err(e) => {
                error!("[Service] Database error while finding user: {:?}", e);
                Err(AppError::InternalServerError)
            }
        }
    }

    pub async fn get_all_users(
        &self,
        params: PaginationParams,
        req: HttpRequest,
    ) -> Result<HttpResponse, AppError> {
        info!("[UserService] Starting process to get all users");

        let claims = extract_claims(&req)?;
        let pagination = normalize_pagination(&params);

        let (allowed_cities, exclude_root) = if claims.profile == PROFILE_ROOT {
            (None, false)
        } else if claims.profile == PROFILE_CITY_ADMIN {
            let claims_policies =
                match get_user_policies_strict(self.user_repository.as_ref(), &claims).await? {
                    Some(p) => p,
                    None => {
                        info!("[UserService] No policies found for CITY_ADMIN");
                        return Ok(PaginatedResponse::success(
                            Vec::<UserDataCreatedWithoutPassword>::new(),
                            pagination.page,
                            pagination.page_size,
                            0,
                        )
                        .into_response());
                    }
                };

            let allowed: Vec<Uuid> = claims_policies
                .get("read_users")
                .and_then(|v| v.as_array())
                .map(|arr| {
                    arr.iter()
                        .filter_map(|v| v.as_str())
                        .filter_map(|s| Uuid::parse_str(s).ok())
                        .collect()
                })
                .unwrap_or_default();

            info!(
                "[UserService] CITY_ADMIN can read users from {} cities",
                allowed.len()
            );
            (Some(allowed), true)
        } else {
            (None, true)
        };

        let total_items = self
            .user_repository
            .count_users(allowed_cities.as_deref(), exclude_root)
            .await
            .map_err(|_| AppError::InternalServerError)?;

        let users = self
            .user_repository
            .get_users_paginated(
                allowed_cities.as_deref(),
                exclude_root,
                pagination.page_size,
                pagination.offset,
            )
            .await
            .map_err(|_| AppError::InternalServerError)?;

        info!("[UserService] Successfully retrieved {} users", users.len());
        Ok(
            PaginatedResponse::success(users, pagination.page, pagination.page_size, total_items)
                .into_response(),
        )
    }

    pub async fn delete_user_by_id(
        &self,
        id: Uuid,
        req: HttpRequest,
    ) -> Result<HttpResponse, AppError> {
        info!(
            "[UserService] Starting process to delete user with id: {}",
            id
        );

        let claims = extract_claims(&req)?;

        let existing = match self.user_repository.get_user_by_id(id).await {
            Ok(user) => user,
            Err(sqlx::Error::RowNotFound) => {
                return Err(AppError::NotFound(format!(
                    "User with id '{}' not found",
                    id
                )));
            }
            Err(e) => {
                error!("[UserService] Error fetching user: {:?}", e);
                return Err(AppError::InternalServerError);
            }
        };

        if existing.profile == PROFILE_ROOT && claims.profile != PROFILE_ROOT {
            return Err(AppError::Forbidden(
                "Only ROOT can modify ROOT users".to_string(),
            ));
        }

        if claims.profile != PROFILE_ROOT
            && let Some(user_city_id) = existing.city_id
        {
            let policies = get_user_policies_strict(self.user_repository.as_ref(), &claims)
                .await?
                .unwrap_or(serde_json::json!({}));
            check_policy(&claims, POLICY_DELETE_USERS, user_city_id, &policies)?;
        }

        match self.user_repository.delete_user_by_id(id).await {
            Ok(deleted_user) => {
                info!("[UserService] User with id {} deleted successfully", id);
                Ok(ApiResponse::success(deleted_user).into_response())
            }
            Err(sqlx::Error::RowNotFound) => {
                info!("[UserService] User with id {} not found for deletion", id);
                Err(AppError::NotFound(format!(
                    "User with id '{}' not found",
                    id
                )))
            }
            Err(e) => {
                error!("[UserService] Failed to delete user: {:?}", e);
                Err(AppError::InternalServerError)
            }
        }
    }

    pub async fn update_user_password_by_id(
        &self,
        data: UpdateUserPassword,
        req: HttpRequest,
    ) -> Result<HttpResponse, AppError> {
        let claims = extract_claims(&req)?;
        let id = Uuid::parse_str(&claims.id)
            .map_err(|_| AppError::Unauthorized("Invalid user id in token".to_string()))?;

        info!(
            "[UserService] Starting password update for user with id: {}",
            id
        );

        let current_pwd = data
            .current_password
            .as_ref()
            .filter(|pwd| !pwd.is_empty())
            .ok_or_else(|| {
                AppError::BadRequest(
                    "Error updating password: current_password is required".to_string(),
                )
            })?;

        info!("[UserService] Retrieving current password hash from database");

        let stored_password_hash = match self.user_repository.get_user_password_by_id(id).await {
            Ok(hash) => hash,
            Err(sqlx::Error::RowNotFound) => {
                info!("[UserService] User with id {} not found", id);
                return Err(AppError::NotFound(format!(
                    "User with id '{}' not found",
                    id
                )));
            }
            Err(e) => {
                error!("[UserService] Failed to retrieve user password: {:?}", e);
                return Err(AppError::InternalServerError);
            }
        };

        info!("[UserService] Verifying current password");

        let password_matches = self
            .password_hasher
            .verify_password(&stored_password_hash, current_pwd)
            .map_err(|e| {
                error!("[UserService] Failed to verify password: {:?}", e);
                AppError::InternalServerError
            })?;

        if !password_matches {
            error!("[UserService] Current password is incorrect");
            return Err(AppError::BadRequest(
                "Error updating password: current password is incorrect".to_string(),
            ));
        }

        info!("[UserService] Current password verified with success.");

        if data.new_password.is_empty() {
            return Err(AppError::BadRequest(
                "Error updating password: new_password is required".to_string(),
            ));
        }

        info!("[UserService] Starting hashing a new password");

        let new_password_hash = self
            .password_hasher
            .hash_password(&data.new_password)
            .map_err(|e| {
                error!("[UserService] Failed to hash new password: {:?}", e);
                AppError::InternalServerError
            })?;

        info!("[UserService] New password hashed. Updating in database");

        match self
            .user_repository
            .update_user_password_by_id(id, new_password_hash)
            .await
        {
            Ok(user) => {
                info!(
                    "[UserService] Password updated successfully for user with id: {}",
                    id
                );
                Ok(ApiResponse::success(user).into_response())
            }
            Err(sqlx::Error::RowNotFound) => {
                info!("[UserService] User with id {} not found during update", id);
                Err(AppError::NotFound(format!(
                    "User with id '{}' not found",
                    id
                )))
            }
            Err(e) => {
                error!("[UserService] Failed to update password: {:?}", e);
                Err(AppError::InternalServerError)
            }
        }
    }

    pub async fn reset_user_password_by_id(
        &self,
        id: Uuid,
        req: HttpRequest,
    ) -> Result<HttpResponse, AppError> {
        info!(
            "[UserService] Starting password reset for user with id: {}",
            id
        );

        let claims = extract_claims(&req)?;

        if claims.profile != PROFILE_ROOT && claims.profile != PROFILE_CITY_ADMIN {
            return Err(AppError::Forbidden(
                "Only ROOT or CITY_ADMIN can reset passwords".to_string(),
            ));
        }

        let target_user = match self.user_repository.get_user_by_id(id).await {
            Ok(user) => user,
            Err(sqlx::Error::RowNotFound) => {
                return Err(AppError::NotFound(format!(
                    "User with id '{}' not found",
                    id
                )));
            }
            Err(e) => {
                error!("[UserService] Failed to retrieve user: {:?}", e);
                return Err(AppError::InternalServerError);
            }
        };

        if claims.profile == PROFILE_CITY_ADMIN {
            let admin_city_id = extract_city_id_from_claims(&claims)?;
            if target_user.city_id != Some(admin_city_id) {
                return Err(AppError::Forbidden(
                    "CITY_ADMIN can only reset passwords for users in the same city".to_string(),
                ));
            }
        }

        let uuid_str = Uuid::new_v4().to_string();
        let digits: String = uuid_str.chars().filter(|c| c.is_ascii_digit()).collect();
        let raw_suffix = digits
            .chars()
            .rev()
            .take(6)
            .collect::<String>()
            .chars()
            .rev()
            .collect::<String>();
        let suffix = format!("{:0>6}", raw_suffix);
        let temporary_password = format!("prov{}", suffix);

        let password_hash = self
            .password_hasher
            .hash_password(&temporary_password)
            .map_err(|e| {
                error!("[UserService] Failed to hash temporary password: {:?}", e);
                AppError::InternalServerError
            })?;

        let expires_at = chrono::Utc::now() + chrono::Duration::hours(1);

        match self
            .user_repository
            .reset_user_password_by_id(id, password_hash, expires_at)
            .await
        {
            Ok(_) => {
                info!(
                    "[UserService] Password reset successfully for user with id: {}",
                    id
                );
                Ok(
                    ApiResponse::success(ResetUserPasswordResponse { temporary_password })
                        .into_response(),
                )
            }
            Err(sqlx::Error::RowNotFound) => Err(AppError::NotFound(format!(
                "User with id '{}' not found",
                id
            ))),
            Err(e) => {
                error!("[UserService] Failed to reset password: {:?}", e);
                Err(AppError::InternalServerError)
            }
        }
    }

    pub async fn append_user_policy_cities(
        &self,
        target_user_id: Uuid,
        policy: &str,
        city_ids: &[Uuid],
        req: HttpRequest,
    ) -> Result<HttpResponse, AppError> {
        let claims = extract_claims(&req)?;
        if !is_valid_policy(policy) {
            return Err(AppError::BadRequest(format!(
                "Invalid policy name '{}'. Valid policies are: {:?}",
                policy, VALID_POLICIES
            )));
        }
        if !is_assignable_policy(policy) {
            return Err(AppError::BadRequest(format!(
                "Policy '{}' is not assignable; only ROOT can perform this action",
                policy
            )));
        }

        let target_user = self
            .user_repository
            .get_user_by_id(target_user_id)
            .await
            .map_err(|e| {
                if matches!(e, sqlx::Error::RowNotFound) {
                    AppError::NotFound(format!("User with id '{}' not found", target_user_id))
                } else {
                    AppError::InternalServerError
                }
            })?;

        if target_user.profile == PROFILE_ROOT && claims.profile != PROFILE_ROOT {
            return Err(AppError::Forbidden(
                "Only ROOT can modify ROOT users".to_string(),
            ));
        }

        if claims.profile == PROFILE_CITY_USER {
            error!("[UserService] CITY_USER cannot assign policies");
            return Err(AppError::Forbidden(
                "CITY_USER profile is not allowed to assign permission policies".to_string(),
            ));
        }

        if claims.profile != PROFILE_ROOT {
            let policies = get_user_policies_strict(self.user_repository.as_ref(), &claims)
                .await?
                .ok_or_else(|| AppError::Forbidden("User has no policies".to_string()))?;

            if let Some(target_city_id) = target_user.city_id {
                check_policy(&claims, POLICY_UPDATE_USERS, target_city_id, &policies)?;
            }

            let arr = policies
                .get(policy)
                .and_then(|v| v.as_array())
                .cloned()
                .unwrap_or_default();
            for cid in city_ids {
                let has = arr
                    .iter()
                    .filter_map(|v| v.as_str())
                    .any(|s| Uuid::parse_str(s).ok() == Some(*cid));
                if !has {
                    return Err(AppError::Forbidden(format!(
                        "You cannot assign policy '{}' for city '{}' you do not possess",
                        policy, cid
                    )));
                }
            }
        }

        let mut policies = target_user.permission_policies.clone();
        let mut list = policies
            .get(policy)
            .and_then(|v| v.as_array())
            .cloned()
            .unwrap_or_default();
        for cid in city_ids {
            let cid_str = cid.to_string();
            if !list.iter().any(|v| v.as_str() == Some(&cid_str)) {
                list.push(serde_json::Value::String(cid_str));
            }
        }
        policies[policy] = serde_json::Value::Array(list);

        let updated = self
            .user_repository
            .update_user_policies_by_id(target_user_id, policies)
            .await
            .map_err(|_| AppError::InternalServerError)?;
        Ok(ApiResponse::success(updated).into_response())
    }

    pub async fn remove_user_policy_cities(
        &self,
        target_user_id: Uuid,
        policy: &str,
        city_ids: &[Uuid],
        req: HttpRequest,
    ) -> Result<HttpResponse, AppError> {
        let claims = extract_claims(&req)?;
        if !is_valid_policy(policy) {
            return Err(AppError::BadRequest(format!(
                "Invalid policy name '{}'. Valid policies are: {:?}",
                policy, VALID_POLICIES
            )));
        }
        if !is_assignable_policy(policy) {
            return Err(AppError::BadRequest(format!(
                "Policy '{}' is not assignable; only ROOT can perform this action",
                policy
            )));
        }

        let target_user = self
            .user_repository
            .get_user_by_id(target_user_id)
            .await
            .map_err(|e| {
                if matches!(e, sqlx::Error::RowNotFound) {
                    AppError::NotFound(format!("User with id '{}' not found", target_user_id))
                } else {
                    AppError::InternalServerError
                }
            })?;

        if target_user.profile == PROFILE_ROOT && claims.profile != PROFILE_ROOT {
            return Err(AppError::Forbidden(
                "Only ROOT can modify ROOT users".to_string(),
            ));
        }

        if claims.profile == PROFILE_CITY_USER {
            error!("[UserService] CITY_USER cannot modify policies");
            return Err(AppError::Forbidden(
                "CITY_USER profile is not allowed to modify permission policies".to_string(),
            ));
        }

        if claims.profile != PROFILE_ROOT {
            if target_user.profile != PROFILE_CITY_USER {
                return Err(AppError::Forbidden(
                    "CITY_ADMIN can only modify policies of CITY_USER profiles".to_string(),
                ));
            }

            let policies = get_user_policies_strict(&**self.user_repository, &claims)
                .await?
                .ok_or_else(|| AppError::Forbidden("User has no policies".to_string()))?;

            if let Some(target_city_id) = target_user.city_id {
                check_policy(&claims, POLICY_UPDATE_USERS, target_city_id, &policies)?;
            }

            let arr = policies
                .get(policy)
                .and_then(|v| v.as_array())
                .cloned()
                .unwrap_or_default();
            for cid in city_ids {
                let has = arr
                    .iter()
                    .filter_map(|v| v.as_str())
                    .any(|s| Uuid::parse_str(s).ok() == Some(*cid));
                if !has {
                    return Err(AppError::Forbidden(format!(
                        "You cannot modify policy '{}' for city '{}' you do not possess",
                        policy, cid
                    )));
                }
            }
        }

        let mut policies = target_user.permission_policies.clone();
        if let Some(arr) = policies.get_mut(policy)
            && let Some(vec) = arr.as_array_mut()
        {
            vec.retain(|v| {
                if let Some(s) = v.as_str()
                    && let Ok(cid) = Uuid::parse_str(s)
                {
                    return !city_ids.contains(&cid);
                }
                true
            });
        }

        let updated = self
            .user_repository
            .update_user_policies_by_id(target_user_id, policies)
            .await
            .map_err(|_| AppError::InternalServerError)?;
        Ok(ApiResponse::success(updated).into_response())
    }
}
