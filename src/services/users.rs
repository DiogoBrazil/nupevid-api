use actix_web::{web, HttpRequest, HttpResponse};
use log::{error, info};
use uuid::Uuid;

use crate::core::entities::users::{CreateUser, UpdateUser, UpdateUserPassword, UserDataCreatedWithoutPassword};
use crate::adapters::password_hasher::PasswordHasherPort;
use crate::core::contracts::repository::users::UserRepository;
use crate::repositories::users::PgUserRepository;
use crate::utils::{
    errors::AppError,
    responses::ApiResponse,
    authorization::{check_policy, validate_user_creation_permission},
    service_helpers::{extract_claims, extract_city_id_from_claims, get_user_policies_strict}
};
use crate::validators::{
    common::{
        generate_default_policies, is_valid_policy, is_assignable_policy,
        PROFILE_ROOT, PROFILE_CITY_ADMIN, PROFILE_CITY_USER,
        POLICY_READ_USERS, POLICY_UPDATE_USERS, POLICY_DELETE_USERS,
        VALID_POLICIES
    },
    user_validator::UserValidator,
    policy_validator::PolicyValidator,
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
        Self { user_repository, password_hasher }
    }

    pub async fn create_user(&self, user: CreateUser, req: HttpRequest) -> Result<HttpResponse, AppError> {
        info!("[UserService] Starting user creation for email: {}", user.email);

        let claims = extract_claims(&req)?;

        validate_user_creation_permission(&claims.profile, &user.profile)?;

        if let Some(policies) = user.permission_policies.as_ref() {
            PolicyValidator::validate_policy_names(policies)?;
            let claims_policies_json = get_user_policies_strict(self.user_repository.as_ref(), &claims).await?;
            PolicyValidator::validate_assignment_permission(&claims, &user.profile, policies, claims_policies_json.as_ref())?;
        }

        UserValidator::validate_fields(
            &user.rank,
            &user.registration,
            &user.full_name,
            &user.profile,
            &user.email,
            Some(&user.password),
            "Error adding user: ",
        )?;

        let mut user_with_city = user;

        if user_with_city.profile == PROFILE_CITY_ADMIN || user_with_city.profile == PROFILE_CITY_USER {
            if claims.profile == PROFILE_CITY_ADMIN {
                let admin_city_id = extract_city_id_from_claims(&claims)?;

                info!("[UserService] CITY_ADMIN creating user - using city_id from token: {}", admin_city_id);

                user_with_city.city_id = Some(admin_city_id);
            } else if user_with_city.city_id.is_none() {
                error!("[UserService] city_id is required for CITY_ADMIN and CITY_USER profiles");
                return Err(AppError::BadRequest(
                    "Error adding user: city_id is required for CITY_ADMIN and CITY_USER profiles".to_string()
                ));
            }
        }

        if user_with_city.profile == PROFILE_CITY_ADMIN {
            if let Some(city_id) = user_with_city.city_id {
                let admin_exists = self.user_repository
                    .check_city_admin_exists_for_city(city_id, Uuid::nil())
                    .await
                    .map_err(|e| {
                        error!("[UserService] Failed to check for existing CITY_ADMIN: {:?}", e);
                        AppError::InternalServerError
                    })?;

                if admin_exists {
                    error!("[UserService] CITY_ADMIN already exists for city: {}", city_id);
                    return Err(AppError::BadRequest(
                        "Error adding user: a CITY_ADMIN already exists for this city".to_string()
                    ));
                }
            }
        }

        info!("[UserService] Checking if email already exists: {}", user_with_city.email);

        let user_exists = self.user_repository
            .check_user_exists_by_email(user_with_city.email.as_ref())
            .await
            .map_err(|e| {
                error!("[UserService] Failed to check user existence for {}: {:?}", user_with_city.email, e);
                AppError::InternalServerError
            })?;

        if user_exists {
            error!("[UserService] Email already registered: {}", user_with_city.email);
            return Err(AppError::BadRequest(format!(
                "Error adding user: email '{}' already exists",
                user_with_city.email
            )));
        }

        info!("[UserService] Email not found. Proceeding with creation.");
        info!("[UserService] Hashing user password");

        user_with_city.password = self.password_hasher
            .hash_password(&user_with_city.password)
            .map_err(|e| {
                error!("[UserService] Failed to hash password: {:?}", e);
                AppError::InternalServerError
            })?;

        info!("[UserService] Password hashed successfully");

        if user_with_city.permission_policies.is_none() {
            let default_policies = generate_default_policies(&user_with_city.profile, user_with_city.city_id);
            if !default_policies.is_empty() {
                info!("[UserService] Generated default policies for profile '{}': {:?}",
                    user_with_city.profile, default_policies.keys().collect::<Vec<_>>());
                user_with_city.permission_policies = Some(default_policies);
            } else {
                info!("[UserService] No default policies for profile '{}'", user_with_city.profile);
            }
        } else {
            info!("[UserService] Using permission_policies provided in request");
        }

        info!("[UserService] Saving user to database");

        match self.user_repository.create_user(user_with_city).await {
            Ok(user) => {
                info!("[UserService] User created successfully with ID: {}", user.id);
                Ok(ApiResponse::created(user).into_response())
            }
            Err(sqlx::Error::Database(db_err)) => {
                if let Some(constraint) = db_err.constraint() {
                    if constraint.contains("registration") {
                        error!("[UserService] Registration already exists");
                        return Err(AppError::BadRequest(
                            "Error adding user: registration already exists".to_string()
                        ));
                    }
                }
                error!("[UserService] Database error while saving user: {:?}", db_err);
                Err(AppError::InternalServerError)
            }
            Err(e) => {
                error!("[UserService] Failed to save user: {:?}", e);
                Err(AppError::InternalServerError)
            }
        }
    }

    pub async fn update_user_by_id(&self, data: UpdateUser, id: Uuid, req: HttpRequest) -> Result<HttpResponse, AppError> {
        info!("[UserService] Starting user updation for id: {}", id);

        let claims = extract_claims(&req)?;

        if data.profile == PROFILE_ROOT && claims.profile != PROFILE_ROOT {
            return Err(AppError::Forbidden("Only ROOT can assign ROOT profile".to_string()));
        }

        let existing = match self.user_repository.get_user_by_id(id).await {
            Ok(user) => user,
            Err(sqlx::Error::RowNotFound) => {
                return Err(AppError::NotFound(format!("User with id '{}' not found", id)));
            }
            Err(e) => {
                error!("[UserService] Error fetching user: {:?}", e);
                return Err(AppError::InternalServerError);
            }
        };

        if existing.profile == PROFILE_ROOT && claims.profile != PROFILE_ROOT {
            return Err(AppError::Forbidden("Only ROOT can modify ROOT users".to_string()));
        }

        if claims.profile != PROFILE_ROOT {
            let policies = get_user_policies_strict(self.user_repository.as_ref(), &claims).await?.unwrap_or(serde_json::json!({}));

            if let Some(existing_city_id) = existing.city_id {
                check_policy(&claims, POLICY_UPDATE_USERS, existing_city_id, &policies)?;
            }

            if let Some(new_city_id) = data.city_id {
                if Some(new_city_id) != existing.city_id {
                    check_policy(&claims, POLICY_UPDATE_USERS, new_city_id, &policies)?;
                }
            }
        }

        UserValidator::validate_fields(
            &data.rank,
            &data.registration,
            &data.full_name,
            &data.profile,
            &data.email,
            None,
            "Error updating user: "
        )?;

        UserValidator::validate_city_requirement(&data.profile, &data.city_id, "Error updating user")?;

        if let Some(policies) = data.permission_policies.as_ref() {
            PolicyValidator::validate_policy_names(policies)?;
            let claims_policies_json = get_user_policies_strict(self.user_repository.as_ref(), &claims).await?;
            PolicyValidator::validate_assignment_permission(&claims, &data.profile, policies, claims_policies_json.as_ref())?;
        }

        if data.profile == PROFILE_CITY_ADMIN {
            if let Some(city_id) = data.city_id {
                let admin_exists = self.user_repository
                    .check_city_admin_exists_for_city(city_id, id)
                    .await
                    .map_err(|e| {
                        error!("[UserService] Failed to check for existing CITY_ADMIN: {:?}", e);
                        AppError::InternalServerError
                    })?;

                if admin_exists {
                    error!("[UserService] CITY_ADMIN already exists for city: {}", city_id);
                    return Err(AppError::BadRequest(
                        "Error updating user: a CITY_ADMIN already exists for this city".to_string()
                    ));
                }
            }
        }

        info!("[Service] Checking if the email is already in use by another user");

        if self.user_repository.check_email_exists_for_other_user(&data.email, id).await? {
            return Err(AppError::BadRequest(
                format!("Email '{}' is already in use by another user", data.email)
            ));
        }

        info!("[Service] Email is available, proceeding with the update");

        info!("[Service] Saving user to database");
        match self.user_repository.update_user_by_id(data, id).await {
            Ok(user) => {
                info!("[Service] User updated successfully with ID: {}", user.id);
                Ok(ApiResponse::success(user).into_response())
            },
            Err(sqlx::Error::RowNotFound) => {
                error!("[Service] User with id {} not found for update", id);
                Err(AppError::NotFound(
                    format!("User with id '{}' not found", id)
                ))
            }
            Err(sqlx::Error::Database(db_err)) => {
                if let Some(constraint) = db_err.constraint() {
                    if constraint.contains("registration") {
                        error!("[UserService] Registration already exists");
                        return Err(AppError::BadRequest(
                            "Error updating user: registration already exists".to_string()
                        ));
                    }
                }
                error!("[UserService] Database error while updating user: {:?}", db_err);
                Err(AppError::InternalServerError)
            }
            Err(e) => {
                error!("[Service] Error updating user in database: {:?}", e);
                Err(AppError::InternalServerError)
            }
        }
    }

    pub async fn get_user_by_id(&self, id: Uuid, req: HttpRequest) -> Result<HttpResponse, AppError> {
        info!("[Service] Starting find user by id process for id: {}", id);

        let claims = extract_claims(&req)?;

        match self.user_repository.get_user_by_id(id).await {
            Ok(user) => {
                if user.profile == PROFILE_ROOT && claims.profile != PROFILE_ROOT {
                    return Err(AppError::Forbidden("Only ROOT can access ROOT users".to_string()));
                }

                if claims.profile != PROFILE_ROOT {
                    if let Some(user_city_id) = user.city_id {
                        let policies = get_user_policies_strict(self.user_repository.as_ref(), &claims).await?.unwrap_or(serde_json::json!({}));
                        check_policy(&claims, POLICY_READ_USERS, user_city_id, &policies)?;
                    }
                }

                info!("[Service] User with id {} found successfully", id);
                Ok(ApiResponse::success(user).into_response())
            }
            Err(sqlx::Error::RowNotFound) => {
                info!("[Service] User with id {} not found", id);
                Err(AppError::NotFound(format!("User with id '{}' not found", id)))
            }
            Err(e) => {
                error!("[Service] Database error while finding user: {:?}", e);
                Err(AppError::InternalServerError)
            }
        }
    }

    pub async fn get_all_users(&self, req: HttpRequest) -> Result<HttpResponse, AppError> {
        info!("[UserService] Starting process to get all users");

        let claims = extract_claims(&req)?;

        match self.user_repository.get_all_users().await {
            Ok(users) => {
                let filtered = if claims.profile == PROFILE_ROOT {
                    users
                } else if claims.profile == PROFILE_CITY_ADMIN {
                    let claims_policies = match get_user_policies_strict(self.user_repository.as_ref(), &claims).await? {
                        Some(p) => p,
                        None => {
                            info!("[UserService] No policies found for CITY_ADMIN");
                            return Ok(ApiResponse::success(Vec::<UserDataCreatedWithoutPassword>::new()).into_response());
                        }
                    };

                    let allowed_cities: Vec<Uuid> = claims_policies
                        .get("read_users")
                        .and_then(|v| v.as_array())
                        .map(|arr| {
                            arr.iter()
                                .filter_map(|v| v.as_str())
                                .filter_map(|s| Uuid::parse_str(s).ok())
                                .collect()
                        })
                        .unwrap_or_default();

                    info!("[UserService] CITY_ADMIN can read users from {} cities", allowed_cities.len());

                    users.into_iter()
                        .filter(|u| {
                            if u.profile == PROFILE_ROOT {
                                return false;
                            }
                            if let Some(user_city_id) = u.city_id {
                                allowed_cities.contains(&user_city_id)
                            } else {
                                false
                            }
                        })
                        .collect()
                } else {
                    users.into_iter().filter(|u| u.profile != PROFILE_ROOT).collect()
                };
                info!("[UserService] Successfully retrieved {} users", filtered.len());
                Ok(ApiResponse::success(filtered).into_response())
            }
            Err(e) => {
                error!("[UserService] Failed to retrieve users: {:?}", e);
                Err(AppError::InternalServerError)
            }
        }
    }

    pub async fn delete_user_by_id(&self, id: Uuid, req: HttpRequest) -> Result<HttpResponse, AppError> {
        info!("[UserService] Starting process to delete user with id: {}", id);

        let claims = extract_claims(&req)?;

        let existing = match self.user_repository.get_user_by_id(id).await {
            Ok(user) => user,
            Err(sqlx::Error::RowNotFound) => {
                return Err(AppError::NotFound(format!("User with id '{}' not found", id)));
            }
            Err(e) => {
                error!("[UserService] Error fetching user: {:?}", e);
                return Err(AppError::InternalServerError);
            }
        };

        if existing.profile == PROFILE_ROOT && claims.profile != PROFILE_ROOT {
            return Err(AppError::Forbidden("Only ROOT can modify ROOT users".to_string()));
        }

        if claims.profile != PROFILE_ROOT {
            if let Some(user_city_id) = existing.city_id {
                let policies = get_user_policies_strict(self.user_repository.as_ref(), &claims).await?.unwrap_or(serde_json::json!({}));
                check_policy(&claims, POLICY_DELETE_USERS, user_city_id, &policies)?;
            }
        }

        match self.user_repository.delete_user_by_id(id).await {
            Ok(deleted_user) => {
                info!("[UserService] User with id {} deleted successfully", id);
                Ok(ApiResponse::success(deleted_user).into_response())
            }
            Err(sqlx::Error::RowNotFound) => {
                info!("[UserService] User with id {} not found for deletion", id);
                Err(AppError::NotFound(format!("User with id '{}' not found", id)))
            }
            Err(e) => {
                error!("[UserService] Failed to delete user: {:?}", e);
                Err(AppError::InternalServerError)
            }
        }
    }

    pub async fn update_user_password_by_id(&self, id: Uuid, data: UpdateUserPassword, req: HttpRequest) -> Result<HttpResponse, AppError> {
        info!("[UserService] Starting password update for user with id: {}", id);

        let claims = extract_claims(&req)?;

        let requester_id = Uuid::parse_str(&claims.id)
            .map_err(|_| AppError::Unauthorized("Invalid user id in token".to_string()))?;

        if requester_id != id && claims.profile != PROFILE_ROOT {
            error!("[UserService] User {} attempted to change password of user {}", requester_id, id);
            return Err(AppError::Forbidden("You can only change your own password".to_string()));
        }

        if let Ok(existing) = self.user_repository.get_user_by_id(id).await {
            if existing.profile == PROFILE_ROOT && claims.profile != PROFILE_ROOT {
                return Err(AppError::Forbidden("Only ROOT can modify ROOT users".to_string()));
            }
        }

        if claims.profile != PROFILE_ROOT {
            let current_pwd = data.current_password.as_ref()
                .filter(|pwd| !pwd.is_empty())
                .ok_or_else(|| AppError::BadRequest("Error updating password: current_password is required".to_string()))?;

            info!("[UserService] Retrieving current password hash from database");

            let stored_password_hash = match self.user_repository.get_user_password_by_id(id).await {
                Ok(hash) => hash,
                Err(sqlx::Error::RowNotFound) => {
                    info!("[UserService] User with id {} not found", id);
                    return Err(AppError::NotFound(format!("User with id '{}' not found", id)));
                }
                Err(e) => {
                    error!("[UserService] Failed to retrieve user password: {:?}", e);
                    return Err(AppError::InternalServerError);
                }
            };

            info!("[UserService] Verifying current password");

            let password_matches = self.password_hasher
                .verify_password(&stored_password_hash, current_pwd)
                .map_err(|e| {
                    error!("[UserService] Failed to verify password: {:?}", e);
                    AppError::InternalServerError
                })?;

            if !password_matches {
                error!("[UserService] Current password is incorrect");
                return Err(AppError::BadRequest("Error updating password: current password is incorrect".to_string()));
            }

            info!("[UserService] Current password verified with success.");

        } else {
            info!("[UserService] ROOT user changing password - skipping current password verification");
        }

        if data.new_password.is_empty() {
            return Err(AppError::BadRequest("Error updating password: new_password is required".to_string()));
        }

        info!("[UserService] Starting hashing a new password");

        let new_password_hash = self.password_hasher
            .hash_password(&data.new_password)
            .map_err(|e| {
                error!("[UserService] Failed to hash new password: {:?}", e);
                AppError::InternalServerError
            })?;

        info!("[UserService] New password hashed. Updating in database");

        match self.user_repository.update_user_password_by_id(id, new_password_hash).await {
            Ok(user) => {
                info!("[UserService] Password updated successfully for user with id: {}", id);
                Ok(ApiResponse::success(user).into_response())
            }
            Err(sqlx::Error::RowNotFound) => {
                info!("[UserService] User with id {} not found during update", id);
                Err(AppError::NotFound(format!("User with id '{}' not found", id)))
            }
            Err(e) => {
                error!("[UserService] Failed to update password: {:?}", e);
                Err(AppError::InternalServerError)
            }
        }
    }

    pub async fn append_user_policy_cities(
        &self,
        target_user_id: Uuid,
        policy: &str,
        city_ids: &[Uuid],
        req: HttpRequest
    ) -> Result<HttpResponse, AppError> {
        let claims = extract_claims(&req)?;
        if !is_valid_policy(policy) {
            return Err(AppError::BadRequest(format!("Invalid policy name '{}'. Valid policies are: {:?}", policy, VALID_POLICIES)));
        }
        if !is_assignable_policy(policy) {
            return Err(AppError::BadRequest(format!(
                "Policy '{}' is not assignable; only ROOT can perform this action",
                policy
            )));
        }

        let target_user = self.user_repository.get_user_by_id(target_user_id).await.map_err(|e| {
            if matches!(e, sqlx::Error::RowNotFound) {
                AppError::NotFound(format!("User with id '{}' not found", target_user_id))
            } else { AppError::InternalServerError }
        })?;

        if target_user.profile == PROFILE_ROOT && claims.profile != PROFILE_ROOT {
            return Err(AppError::Forbidden("Only ROOT can modify ROOT users".to_string()));
        }

        if claims.profile == PROFILE_CITY_USER {
            error!("[UserService] CITY_USER cannot assign policies");
            return Err(AppError::Forbidden("CITY_USER profile is not allowed to assign permission policies".to_string()));
        }

        if claims.profile != PROFILE_ROOT {
            let policies = get_user_policies_strict(self.user_repository.as_ref(), &claims).await?
                .ok_or_else(|| AppError::Forbidden("User has no policies".to_string()))?;

            if let Some(target_city_id) = target_user.city_id {
                check_policy(&claims, POLICY_UPDATE_USERS, target_city_id, &policies)?;
            }

            let arr = policies.get(policy).and_then(|v| v.as_array()).cloned().unwrap_or_default();
            for cid in city_ids {
                let has = arr.iter().filter_map(|v| v.as_str()).any(|s| Uuid::parse_str(s).ok() == Some(*cid));
                if !has {
                    return Err(AppError::Forbidden(format!("You cannot assign policy '{}' for city '{}' you do not possess", policy, cid)));
                }
            }
        }

        let mut policies = target_user.permission_policies.clone();
        let mut list = policies.get(policy).and_then(|v| v.as_array()).cloned().unwrap_or_default();
        for cid in city_ids {
            let cid_str = cid.to_string();
            if !list.iter().any(|v| v.as_str() == Some(&cid_str)) {
                list.push(serde_json::Value::String(cid_str));
            }
        }
        policies[policy] = serde_json::Value::Array(list);

        let updated = self.user_repository.update_user_policies_by_id(target_user_id, policies).await.map_err(|_| AppError::InternalServerError)?;
        Ok(ApiResponse::success(updated).into_response())
    }

    pub async fn remove_user_policy_cities(&self, target_user_id: Uuid, policy: &str, city_ids: &[Uuid], req: HttpRequest) -> Result<HttpResponse, AppError> {
        let claims = extract_claims(&req)?;
        if !is_valid_policy(policy) {
            return Err(AppError::BadRequest(format!("Invalid policy name '{}'. Valid policies are: {:?}", policy, VALID_POLICIES)));
        }
        if !is_assignable_policy(policy) {
            return Err(AppError::BadRequest(format!(
                "Policy '{}' is not assignable; only ROOT can perform this action",
                policy
            )));
        }

        let target_user = self.user_repository.get_user_by_id(target_user_id).await.map_err(|e| {
            if matches!(e, sqlx::Error::RowNotFound) {
                AppError::NotFound(format!("User with id '{}' not found", target_user_id))
            } else { AppError::InternalServerError }
        })?;

        if target_user.profile == PROFILE_ROOT && claims.profile != PROFILE_ROOT {
            return Err(AppError::Forbidden("Only ROOT can modify ROOT users".to_string()));
        }

        if claims.profile == PROFILE_CITY_USER {
            error!("[UserService] CITY_USER cannot modify policies");
            return Err(AppError::Forbidden("CITY_USER profile is not allowed to modify permission policies".to_string()));
        }

        if claims.profile != PROFILE_ROOT {
            if target_user.profile != PROFILE_CITY_USER {
                return Err(AppError::Forbidden("CITY_ADMIN can only modify policies of CITY_USER profiles".to_string()));
            }

            let policies = get_user_policies_strict(&**self.user_repository, &claims).await?
                .ok_or_else(|| AppError::Forbidden("User has no policies".to_string()))?;

            if let Some(target_city_id) = target_user.city_id {
                check_policy(&claims, POLICY_UPDATE_USERS, target_city_id, &policies)?;
            }

            let arr = policies.get(policy).and_then(|v| v.as_array()).cloned().unwrap_or_default();
            for cid in city_ids {
                let has = arr.iter().filter_map(|v| v.as_str()).any(|s| Uuid::parse_str(s).ok() == Some(*cid));
                if !has {
                    return Err(AppError::Forbidden(format!("You cannot modify policy '{}' for city '{}' you do not possess", policy, cid)));
                }
            }
        }

        let mut policies = target_user.permission_policies.clone();
        if let Some(arr) = policies.get_mut(policy) {
            if let Some(vec) = arr.as_array_mut() {
                vec.retain(|v| {
                    if let Some(s) = v.as_str() {
                        if let Ok(cid) = Uuid::parse_str(s) {
                            return !city_ids.contains(&cid);
                        }
                    }
                    true
                });
            }
        }

        let updated = self.user_repository.update_user_policies_by_id(target_user_id, policies).await.map_err(|_| AppError::InternalServerError)?;
        Ok(ApiResponse::success(updated).into_response())
    }

}
