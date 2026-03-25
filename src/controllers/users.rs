use actix_web::{HttpRequest, HttpResponse, web};
use log::info;
use uuid::Uuid;

use crate::core::commands::users::{CreateUser, UpdateUser, UpdateUserPassword};
use crate::core::queries::users::UserSearchQuery;
use crate::services::users::UserService;
use crate::utils::controller_helpers::{
    created, paginated, request_claims, request_pagination, success,
};
use crate::utils::errors::AppError;
use crate::utils::pagination::PaginationParams;

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
    let claims = request_claims(&req)?;
    let user = service.create_user(user.into_inner(), &claims).await?;
    Ok(created(user))
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
    let claims = request_claims(&req)?;
    let user = service
        .update_user_by_id(data.into_inner(), user_id, &claims)
        .await?;
    Ok(success(user))
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
    let claims = request_claims(&req)?;
    let user = service.get_user_by_id(user_id, &claims).await?;
    Ok(success(user))
}

pub async fn get_all_users(
    query: web::Query<PaginationParams>,
    service: web::Data<UserService>,
    req: HttpRequest,
) -> Result<HttpResponse, AppError> {
    info!("[Controller] Received request to get all users");
    let claims = request_claims(&req)?;
    let pagination = request_pagination(&query.into_inner());
    let result = service.get_all_users(pagination, &claims).await?;
    Ok(paginated(result))
}

pub async fn search_users(
    query: web::Query<UserSearchQuery>,
    service: web::Data<UserService>,
    req: HttpRequest,
) -> Result<HttpResponse, AppError> {
    let query = query.into_inner();
    info!("[Controller] Received request to search users");
    let claims = request_claims(&req)?;
    let users = service
        .search_users(query.name, query.registration, &claims)
        .await?;
    Ok(success(users))
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
    let claims = request_claims(&req)?;
    let user = service.delete_user_by_id(user_id, &claims).await?;
    Ok(success(user))
}

pub async fn update_user_password_by_id(
    data: web::Json<UpdateUserPassword>,
    service: web::Data<UserService>,
    req: HttpRequest,
) -> Result<HttpResponse, AppError> {
    info!("[Controller] Received request to update password for current user");
    let claims = request_claims(&req)?;
    let user = service
        .update_user_password_by_id(data.into_inner(), &claims)
        .await?;
    Ok(success(user))
}

pub async fn reset_user_password_by_id(
    path: web::Path<Uuid>,
    service: web::Data<UserService>,
    req: HttpRequest,
) -> Result<HttpResponse, AppError> {
    let user_id = path.into_inner();
    info!(
        "[Controller] Received request to reset password for user with id: {}",
        user_id
    );
    let claims = request_claims(&req)?;
    let response = service.reset_user_password_by_id(user_id, &claims).await?;
    Ok(success(response))
}

pub async fn append_user_policy_cities(
    path: web::Path<(Uuid, String)>,
    body: web::Json<PolicyCitiesPayload>,
    service: web::Data<UserService>,
    req: HttpRequest,
) -> Result<HttpResponse, AppError> {
    let (user_id, policy) = path.into_inner();
    info!(
        "[Controller] Append cities to user policy '{}' for user: {}",
        policy, user_id
    );
    let claims = request_claims(&req)?;
    let user = service
        .append_user_policy_cities(user_id, &policy, &body.city_ids, &claims)
        .await?;
    Ok(success(user))
}

pub async fn remove_user_policy_cities(
    path: web::Path<(Uuid, String)>,
    body: web::Json<PolicyCitiesPayload>,
    service: web::Data<UserService>,
    req: HttpRequest,
) -> Result<HttpResponse, AppError> {
    let (user_id, policy) = path.into_inner();
    info!(
        "[Controller] Remove cities from user policy '{}' for user: {}",
        policy, user_id
    );
    let claims = request_claims(&req)?;
    let user = service
        .remove_user_policy_cities(user_id, &policy, &body.city_ids, &claims)
        .await?;
    Ok(success(user))
}
