use actix_web::{web, HttpResponse};
use log::{error, info};

use crate::core::entities::users::CreateUser;
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
        info!("[Service] Starting user creation process for email: {}", user.email);

        self.validate_user_fields(&user.rank, &user.registration, &user.full_name, &user.email, Some(&user.password), "Error adding user: ")?;

        // info!("[Service] Checking if user already exists with email: {}", data.email);
        // if self.user_repo.find_user_by_email(data.email.clone()).await.is_ok() {
        //     info!("[Service] User already exists with email: {}", data.email);
        //     return Err(errors::AppError::BadRequest(
        //         format!("Error adding user: email '{}' already exists", data.email)
        //     ));
        // }
        // info!("[Service] User does not exist, proceeding with creation");

        info!("[Service] Hashing user password");

        let mut user_with_password_hash: CreateUser = user;
        user_with_password_hash.password = self.password_hasher
            .hash_password(&user_with_password_hash.password)
            .map_err(|e | {
                error!("[Service] Error hashing password: {:?}", e);
                AppError::InternalServerError
            })?;

        info!("[Service] Password hashed successfully");

        info!("[Service] Saving user to database");

        match self.user_repository.create_user(user_with_password_hash).await {
            Ok(user) => {
                info!("[Service] User created successfully with ID: {}", user.id);
                Ok(ApiResponse::created(user).into_response())
            },
            Err(e) => {
                error!("[Service] Error creating user in database: {:?}", e);
                Err(AppError::InternalServerError)
            }
        }
    }

    // Private validation helper function
    fn validate_user_fields(&self,
        rank: &str,
        registration: &str,
        full_name: &str,
        email: &str,
        password: Option<&str>,
        error_context: &str
    ) -> Result<(), AppError> {
        info!("[Service] Validating required fields");

        let mut fields_to_validate = vec![
            ("rank", rank.is_empty()),
            ("registration", registration.is_empty()),
            ("full_name", full_name.is_empty()),
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
