use actix_web::{web, HttpResponse};
use log::{error, info};
use uuid::Uuid;

use crate::core::entities::users::{CreateUser, UpdateUser, UpdateUserPassword};
use crate::adapters::password_hasher::PasswordHasherPort;
use crate::core::contracts::repository::users::UserRepository;
use crate::repositories::users::PgUserRepository;
use crate::utils::{
    errors::AppError,
    responses::ApiResponse,
    validations::{is_valid_email, validate_required_fields}
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

    pub async fn create_user(&self, user: CreateUser) -> Result<HttpResponse, AppError> {
        info!("[UserService] Starting user creation for email: {}", user.email);

        self.validate_user_fields(
            &user.rank,
            &user.registration,
            &user.full_name,
            &user.profile,
            &user.email,
            Some(&user.password),
            "Error adding user: ",
        )?;

        // Validate city_id requirement based on profile
        if user.profile == "CITY_ADMIN" || user.profile == "CITY_USER" {
            if user.city_id.is_none() {
                error!("[UserService] city_id is required for CITY_ADMIN and CITY_USER profiles");
                return Err(AppError::BadRequest(
                    "Error adding user: city_id is required for CITY_ADMIN and CITY_USER profiles".to_string()
                ));
            }
        }

        // Check if CITY_ADMIN already exists for this city
        if user.profile == "CITY_ADMIN" {
            if let Some(city_id) = user.city_id {
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

        info!("[UserService] Checking if email already exists: {}", user.email);

        let user_exists = self.user_repository
            .check_user_exists_by_email(user.email.clone())
            .await
            .map_err(|e| {
                error!("[UserService] Failed to check user existence for {}: {:?}", user.email, e);
                AppError::InternalServerError
            })?;

        if user_exists {
            error!("[UserService] Email already registered: {}", user.email);
            return Err(AppError::BadRequest(format!(
                "Error adding user: email '{}' already exists",
                user.email
            )));
        }

        info!("[UserService] Email not found. Proceeding with creation.");
        info!("[UserService] Hashing user password");

        let mut user_with_password_hash = user;
        user_with_password_hash.password = self.password_hasher
            .hash_password(&user_with_password_hash.password)
            .map_err(|e| {
                error!("[UserService] Failed to hash password: {:?}", e);
                AppError::InternalServerError
            })?;

        info!("[UserService] Password hashed successfully");
        info!("[UserService] Saving user to database");

        match self.user_repository.create_user(user_with_password_hash).await {
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

    pub async fn update_user_by_id(&self, data: UpdateUser, id: Uuid) -> Result<HttpResponse, AppError> {
        info!("[UserService] Starting user updation for id: {}", id);

        self.validate_user_fields(
            &data.rank,
            &data.registration,
            &data.full_name,
            &data.profile,
            &data.email,
            None,
            "Error updating user: "
        )?;

        // Validate city_id requirement based on profile
        if data.profile == "CITY_ADMIN" || data.profile == "CITY_USER" {
            if data.city_id.is_none() {
                error!("[UserService] city_id is required for CITY_ADMIN and CITY_USER profiles");
                return Err(AppError::BadRequest(
                    "Error updating user: city_id is required for CITY_ADMIN and CITY_USER profiles".to_string()
                ));
            }
        }

        // Check if CITY_ADMIN already exists for this city (excluding current user)
        if data.profile == "CITY_ADMIN" {
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

    pub async fn get_user_by_id(&self, id: Uuid) -> Result<HttpResponse, AppError> {
        info!("[Service] Starting find user by id process for id: {}", id);

        match self.user_repository.get_user_by_id(id).await {
            Ok(user) => {
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

    pub async fn get_all_users(&self) -> Result<HttpResponse, AppError> {
        info!("[UserService] Starting process to get all users");

        match self.user_repository.get_all_users().await {
            Ok(users) => {
                info!("[UserService] Successfully retrieved {} users", users.len());
                Ok(ApiResponse::success(users).into_response())
            }
            Err(e) => {
                error!("[UserService] Failed to retrieve users: {:?}", e);
                Err(AppError::InternalServerError)
            }
        }
    }

    pub async fn delete_user_by_id(&self, id: Uuid) -> Result<HttpResponse, AppError> {
        info!("[UserService] Starting process to delete user with id: {}", id);

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

    pub async fn update_user_password_by_id(&self, id: Uuid, data: UpdateUserPassword) -> Result<HttpResponse, AppError> {
        info!("[UserService] Starting password update for user with id: {}", id);

        // Validate required fields
        if data.current_password.is_empty() {
            return Err(AppError::BadRequest("Error updating password: current_password is required".to_string()));
        }
        if data.new_password.is_empty() {
            return Err(AppError::BadRequest("Error updating password: new_password is required".to_string()));
        }

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
            .verify_password(&stored_password_hash, &data.current_password)
            .map_err(|e| {
                error!("[UserService] Failed to verify password: {:?}", e);
                AppError::InternalServerError
            })?;

        if !password_matches {
            error!("[UserService] Current password is incorrect");
            return Err(AppError::BadRequest("Error updating password: current password is incorrect".to_string()));
        }

        info!("[UserService] Current password verified. Hashing new password");

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

    // Private validation helper function
    fn validate_user_fields(&self,
        rank: &str,
        registration: &str,
        full_name: &str,
        profile: &str,
        email: &str,
        password: Option<&str>,
        error_context: &str
    ) -> Result<(), AppError> {
        info!("[Service] Validating required fields");

        let mut fields_to_validate = vec![
            ("rank", rank.is_empty()),
            ("registration", registration.is_empty()),
            ("full_name", full_name.is_empty()),
            ("profile", profile.is_empty()),
            ("email", email.is_empty()),
        ];

        if let Some(p) = password {
            fields_to_validate.push(("password", p.is_empty()));
        }

        validate_required_fields(&fields_to_validate, error_context)?;
        info!("[Service] Required fields validation passed");

        info!("[Service] Validating email format: {}", email);
        if !is_valid_email(email) {
            return Err(AppError::BadRequest(
                format!("{}'{}' is not a valid email", error_context, email)
            ));
        }
        info!("[Service] Email format validation passed");

        Ok(())
    }

}
