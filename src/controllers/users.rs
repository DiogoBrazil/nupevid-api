use actix_web::{web, HttpResponse};
use log::info;
use crate::services::users::UserService;
use crate::core::entities::users::CreateUser;
use crate::utils::errors::AppError;

pub async fn create_user(user: web::Json<CreateUser>, service: web::Data<UserService>) -> Result<HttpResponse, AppError> {
    info!("[Controller] Received request to create user with email: {}", user.email);
    let result = service.create_user(user.into_inner()).await;
    match &result {
        Ok(_) => info!("[Controller] User creation request completed successfully"),
        Err(e) => info!("[Controller] User creation request failed: {:?}", e)
    }
    result
}
