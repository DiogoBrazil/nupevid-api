use actix_web::{web, HttpResponse};
use log::info;
use uuid::Uuid;
use crate::services::users::UserService;
use crate::core::entities::users::{CreateUser, UpdateUser, UpdateUserPassword};
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

pub async fn update_user_by_id(data: web::Json<UpdateUser>, id: web::Path<Uuid>, service: web::Data<UserService>) -> Result<HttpResponse, AppError> {
    info!("[Controller] Received request to update user with email: {}", data.email);
    let result = service.update_user_by_id(data.into_inner(), id.into_inner()).await;
    match &result {
        Ok(_) => info!("[Controller] User updated request completed successfully"),
        Err(e) => info!("[Controller] User updated request failed: {:?}", e)
    }
    result
}

pub async fn get_user_by_id(id: web::Path<Uuid>, service: web::Data<UserService>) -> Result<HttpResponse, AppError> {
    info!("[Controller] Received request to get user by id with id: {}", id);
    let result = service.get_user_by_id(id.into_inner()).await;
    match &result {
        Ok(_) => info!("[Controller] Get user by id request completed successfully"),
        Err(e) => info!("[Controller] Get user by id request failed: {:?}", e)
    }
    result
}

pub async fn get_all_users(service: web::Data<UserService>) -> Result<HttpResponse, AppError> {
    info!("[Controller] Received request to get all users");
    let result = service.get_all_users().await;
    match &result {
        Ok(_) => info!("[Controller] Get all users request completed successfully"),
        Err(e) => info!("[Controller] Get all users request failed: {:?}", e)
    }
    result
}

pub async fn delete_user_by_id(id: web::Path<Uuid>, service: web::Data<UserService>) -> Result<HttpResponse, AppError> {
    info!("[Controller] Received request to delete user with id: {}", id);
    let result = service.delete_user_by_id(id.into_inner()).await;
    match &result {
        Ok(_) => info!("[Controller] Delete user request completed successfully"),
        Err(e) => info!("[Controller] Delete user request failed: {:?}", e)
    }
    result
}

pub async fn update_user_password_by_id(data: web::Json<UpdateUserPassword>, id: web::Path<Uuid>, service: web::Data<UserService>) -> Result<HttpResponse, AppError> {
    info!("[Controller] Received request to update password for user with id: {}", id);
    let result = service.update_user_password_by_id(id.into_inner(), data.into_inner()).await;
    match &result {
        Ok(_) => info!("[Controller] Update password request completed successfully"),
        Err(e) => info!("[Controller] Update password request failed: {:?}", e)
    }
    result
}
