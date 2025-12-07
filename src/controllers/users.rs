use actix_web::{web, HttpRequest, HttpResponse};
use log::info;
use uuid::Uuid;

use crate::core::entities::users::{CreateUser, UpdateUser, UpdateUserPassword};
use crate::services::users::UserService;
use crate::utils::errors::AppError;

#[derive(serde::Deserialize)]
pub struct PolicyCitiesPayload {
    pub city_ids: Vec<Uuid>,
}

pub async fn create_user(
    user: web::Json<CreateUser>,
    service: web::Data<UserService>,
    req: HttpRequest,
) -> Result<HttpResponse, AppError> {
    info!(
        "[Controller] Received request to create user with email: {}",
        user.email
    );
    service.create_user(user.into_inner(), req).await
}

pub async fn update_user_by_id(
    path: web::Path<Uuid>,
    data: web::Json<UpdateUser>,
    service: web::Data<UserService>,
    req: HttpRequest,
) -> Result<HttpResponse, AppError> {
    let user_id = path.into_inner();
    info!(
        "[Controller] Received request to update user with id: {}",
        user_id
    );
    service.update_user_by_id(data.into_inner(), user_id, req).await
}

pub async fn get_user_by_id(
    path: web::Path<Uuid>,
    service: web::Data<UserService>,
    req: HttpRequest,
) -> Result<HttpResponse, AppError> {
    let user_id = path.into_inner();
    info!(
        "[Controller] Received request to get user with id: {}",
        user_id
    );
    service.get_user_by_id(user_id, req).await
}

pub async fn get_all_users(service: web::Data<UserService>, req: HttpRequest) -> Result<HttpResponse, AppError> {
    info!("[Controller] Received request to get all users");
    service.get_all_users(req).await
}

pub async fn delete_user_by_id(
    path: web::Path<Uuid>,
    service: web::Data<UserService>,
    req: HttpRequest,
) -> Result<HttpResponse, AppError> {
    let user_id = path.into_inner();
    info!(
        "[Controller] Received request to delete user with id: {}",
        user_id
    );
    service.delete_user_by_id(user_id, req).await
}

pub async fn update_user_password_by_id(
    path: web::Path<Uuid>,
    data: web::Json<UpdateUserPassword>,
    service: web::Data<UserService>,
    req: HttpRequest,
) -> Result<HttpResponse, AppError> {
    let user_id = path.into_inner();
    info!(
        "[Controller] Received request to update password for user with id: {}",
        user_id
    );
    service
        .update_user_password_by_id(user_id, data.into_inner(), req)
        .await
}

pub async fn append_user_policy_cities(
    path: web::Path<(Uuid, String)>,
    body: web::Json<PolicyCitiesPayload>,
    service: web::Data<UserService>,
    req: HttpRequest,
) -> Result<HttpResponse, AppError> {
    let (user_id, policy) = path.into_inner();
    info!("[Controller] Append cities to user policy '{}' for user: {}", policy, user_id);
    service
        .append_user_policy_cities(user_id, &policy, &body.city_ids, req)
        .await
}

pub async fn remove_user_policy_cities(
    path: web::Path<(Uuid, String)>,
    body: web::Json<PolicyCitiesPayload>,
    service: web::Data<UserService>,
    req: HttpRequest,
) -> Result<HttpResponse, AppError> {
    let (user_id, policy) = path.into_inner();
    info!("[Controller] Remove cities from user policy '{}' for user: {}", policy, user_id);
    service
        .remove_user_policy_cities(user_id, &policy, &body.city_ids, req)
        .await
}
