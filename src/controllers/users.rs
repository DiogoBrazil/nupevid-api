use actix_web::{HttpRequest, HttpResponse, web};
use log::info;
use uuid::Uuid;

use crate::core::application_error::ApplicationError as AppError;
use crate::core::commands::users::{CreateUser, UpdateUser, UpdateUserPassword};
use crate::core::filters::users::UserSearchQuery;
use crate::core::value_objects::policies::Policy;
use crate::usecases::users::{
    AppendUserPolicyCitiesUseCase, CreateUserUseCase, DeleteUserByIdUseCase, GetAllUsersUseCase,
    GetUserByIdUseCase, RemoveUserPolicyCitiesUseCase, ResetUserPasswordByIdUseCase,
    SearchUsersUseCase, UpdateUserPasswordUseCase, UpdateUserUseCase,
};
use crate::utils::controller_helpers::{
    created, paginated, request_claims, request_pagination, success,
};
use crate::utils::pagination::PaginationParams;

#[derive(serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PolicyCityAssignment {
    pub city_ids: Vec<Uuid>,
}

pub async fn create_user(
    user: web::Json<CreateUser>,
    usecase: web::Data<CreateUserUseCase>,
    req: HttpRequest,
) -> Result<HttpResponse, AppError> {
    info!(
        "[Controller] Received request to create user with email: {}",
        user.email
    );
    let claims = request_claims(&req)?;
    let user = usecase.execute(user.into_inner(), &claims).await?;
    Ok(created(user))
}

pub async fn update_user_by_id(
    path: web::Path<Uuid>,
    data: web::Json<UpdateUser>,
    usecase: web::Data<UpdateUserUseCase>,
    req: HttpRequest,
) -> Result<HttpResponse, AppError> {
    let user_id = path.into_inner();
    info!(
        "[Controller] Received request to update user with id: {}",
        user_id
    );
    let claims = request_claims(&req)?;
    let user = usecase.execute(data.into_inner(), user_id, &claims).await?;
    Ok(success(user))
}

pub async fn get_user_by_id(
    path: web::Path<Uuid>,
    usecase: web::Data<GetUserByIdUseCase>,
    req: HttpRequest,
) -> Result<HttpResponse, AppError> {
    let user_id = path.into_inner();
    info!(
        "[Controller] Received request to get user with id: {}",
        user_id
    );
    let claims = request_claims(&req)?;
    let user = usecase.execute(user_id, &claims).await?;
    Ok(success(user))
}

pub async fn get_all_users(
    query: web::Query<PaginationParams>,
    usecase: web::Data<GetAllUsersUseCase>,
    req: HttpRequest,
) -> Result<HttpResponse, AppError> {
    info!("[Controller] Received request to get all users");
    let claims = request_claims(&req)?;
    let pagination = request_pagination(&query.into_inner());
    let result = usecase.execute(pagination, &claims).await?;
    Ok(paginated(result))
}

pub async fn search_users(
    query: web::Query<UserSearchQuery>,
    usecase: web::Data<SearchUsersUseCase>,
    req: HttpRequest,
) -> Result<HttpResponse, AppError> {
    let query = query.into_inner();
    info!("[Controller] Received request to search users");
    let claims = request_claims(&req)?;
    let users = usecase
        .execute(query.name, query.registration, &claims)
        .await?;
    Ok(success(users))
}

pub async fn delete_user_by_id(
    path: web::Path<Uuid>,
    usecase: web::Data<DeleteUserByIdUseCase>,
    req: HttpRequest,
) -> Result<HttpResponse, AppError> {
    let user_id = path.into_inner();
    info!(
        "[Controller] Received request to delete user with id: {}",
        user_id
    );
    let claims = request_claims(&req)?;
    let user = usecase.execute(user_id, &claims).await?;
    Ok(success(user))
}

pub async fn update_user_password_by_id(
    data: web::Json<UpdateUserPassword>,
    usecase: web::Data<UpdateUserPasswordUseCase>,
    req: HttpRequest,
) -> Result<HttpResponse, AppError> {
    info!("[Controller] Received request to update password for current user");
    let claims = request_claims(&req)?;
    let user = usecase.execute(data.into_inner(), &claims).await?;
    Ok(success(user))
}

pub async fn reset_user_password_by_id(
    path: web::Path<Uuid>,
    usecase: web::Data<ResetUserPasswordByIdUseCase>,
    req: HttpRequest,
) -> Result<HttpResponse, AppError> {
    let user_id = path.into_inner();
    info!(
        "[Controller] Received request to reset password for user with id: {}",
        user_id
    );
    let claims = request_claims(&req)?;
    let response = usecase.execute(user_id, &claims).await?;
    Ok(success(response))
}

fn parse_policy_or_bad_request(policy_str: &str) -> Result<Policy, AppError> {
    serde_json::from_value(serde_json::Value::String(policy_str.to_string())).map_err(|_| {
        AppError::BadRequest(format!(
            "Invalid policy name '{}'. Valid policies are: {:?}",
            policy_str,
            Policy::all().iter().map(|p| p.as_str()).collect::<Vec<_>>()
        ))
    })
}

pub async fn append_user_policy_cities(
    path: web::Path<(Uuid, String)>,
    body: web::Json<PolicyCityAssignment>,
    usecase: web::Data<AppendUserPolicyCitiesUseCase>,
    req: HttpRequest,
) -> Result<HttpResponse, AppError> {
    let (user_id, policy_str) = path.into_inner();
    info!(
        "[Controller] Append cities to user policy '{}' for user: {}",
        policy_str, user_id
    );
    let policy = parse_policy_or_bad_request(&policy_str)?;
    let claims = request_claims(&req)?;
    let user = usecase
        .execute(user_id, &policy, &body.city_ids, &claims)
        .await?;
    Ok(success(user))
}

pub async fn remove_user_policy_cities(
    path: web::Path<(Uuid, String)>,
    body: web::Json<PolicyCityAssignment>,
    usecase: web::Data<RemoveUserPolicyCitiesUseCase>,
    req: HttpRequest,
) -> Result<HttpResponse, AppError> {
    let (user_id, policy_str) = path.into_inner();
    info!(
        "[Controller] Remove cities from user policy '{}' for user: {}",
        policy_str, user_id
    );
    let policy = parse_policy_or_bad_request(&policy_str)?;
    let claims = request_claims(&req)?;
    let user = usecase
        .execute(user_id, &policy, &body.city_ids, &claims)
        .await?;
    Ok(success(user))
}
