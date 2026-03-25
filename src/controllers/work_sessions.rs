use actix_web::{HttpRequest, HttpResponse, web};
use log::info;
use serde::Deserialize;
use uuid::Uuid;

use crate::core::commands::work_session_members::UpdateMemberFunction;
use crate::core::commands::work_sessions::{CreateWorkSession, UpdateWorkSessionMembers};
use crate::core::entities::work_session_members::TeamMemberFunction;
use crate::core::queries::common::IncludeComplementQuery;
use crate::core::queries::work_sessions::ListWorkSessionsQuery;
use crate::services::work_sessions::WorkSessionService;
use crate::utils::controller_helpers::{
    created, include_complement, paginated, request_claims, request_pagination_from_parts, success,
};
use crate::utils::errors::AppError;

pub async fn create_work_session(
    data: web::Json<CreateWorkSession>,
    query: web::Query<IncludeComplementQuery>,
    service: web::Data<WorkSessionService>,
    req: HttpRequest,
) -> Result<HttpResponse, AppError> {
    info!("[Controller] Received request to create work session");
    let claims = request_claims(&req)?;
    let session = service
        .create_work_session(data.into_inner(), &claims, include_complement(&query))
        .await?;
    Ok(created(session))
}

pub async fn get_active_session(
    query: web::Query<IncludeComplementQuery>,
    service: web::Data<WorkSessionService>,
    req: HttpRequest,
) -> Result<HttpResponse, AppError> {
    info!("[Controller] Received request to get active work session");
    let claims = request_claims(&req)?;
    let session = service
        .get_active_session(&claims, include_complement(&query))
        .await?;
    Ok(success(session))
}

pub async fn get_session_by_id(
    path: web::Path<Uuid>,
    query: web::Query<IncludeComplementQuery>,
    service: web::Data<WorkSessionService>,
    req: HttpRequest,
) -> Result<HttpResponse, AppError> {
    let session_id = path.into_inner();
    info!(
        "[Controller] Received request to get work session: {}",
        session_id
    );
    let claims = request_claims(&req)?;
    let session = service
        .get_session_by_id(session_id, &claims, include_complement(&query))
        .await?;
    Ok(success(session))
}

pub async fn list_sessions(
    query: web::Query<ListWorkSessionsQuery>,
    service: web::Data<WorkSessionService>,
    req: HttpRequest,
) -> Result<HttpResponse, AppError> {
    info!("[Controller] Received request to list work sessions");
    let claims = request_claims(&req)?;
    let query = query.into_inner();
    let pagination = request_pagination_from_parts(query.page, query.page_size);
    let result = service.list_sessions(query, pagination, &claims).await?;
    Ok(paginated(result))
}

pub async fn end_session(
    service: web::Data<WorkSessionService>,
    req: HttpRequest,
) -> Result<HttpResponse, AppError> {
    info!("[Controller] Received request to end work session");
    let claims = request_claims(&req)?;
    let message = service.end_session(&claims).await?;
    Ok(success(message))
}

#[derive(Deserialize)]
pub struct AddMemberRequest {
    pub user_id: Uuid,
    pub function: Option<TeamMemberFunction>,
}

pub async fn add_member(
    path: web::Path<Uuid>,
    data: web::Json<AddMemberRequest>,
    service: web::Data<WorkSessionService>,
    req: HttpRequest,
) -> Result<HttpResponse, AppError> {
    let session_id = path.into_inner();
    info!(
        "[Controller] Received request to add member to session: {}",
        session_id
    );
    let claims = request_claims(&req)?;
    let member = service
        .add_member_to_session(session_id, data.user_id, data.function.clone(), &claims)
        .await?;
    Ok(success(member))
}

pub async fn remove_member(
    path: web::Path<(Uuid, Uuid)>,
    service: web::Data<WorkSessionService>,
    req: HttpRequest,
) -> Result<HttpResponse, AppError> {
    let (session_id, member_id) = path.into_inner();
    info!(
        "[Controller] Received request to remove member {} from session: {}",
        member_id, session_id
    );
    let claims = request_claims(&req)?;
    let message = service
        .remove_member_from_session(session_id, member_id, &claims)
        .await?;
    Ok(success(message))
}

pub async fn update_members(
    path: web::Path<Uuid>,
    data: web::Json<UpdateWorkSessionMembers>,
    service: web::Data<WorkSessionService>,
    req: HttpRequest,
) -> Result<HttpResponse, AppError> {
    let session_id = path.into_inner();
    info!(
        "[Controller] Received request to update members of session: {}",
        session_id
    );
    let claims = request_claims(&req)?;
    let session = service
        .update_members(session_id, data.into_inner(), &claims)
        .await?;
    Ok(success(session))
}

pub async fn update_member_function(
    path: web::Path<(Uuid, Uuid)>,
    data: web::Json<UpdateMemberFunction>,
    service: web::Data<WorkSessionService>,
    req: HttpRequest,
) -> Result<HttpResponse, AppError> {
    let (session_id, user_id) = path.into_inner();
    info!(
        "[Controller] Received request to update function of member {} in session: {}",
        user_id, session_id
    );
    let claims = request_claims(&req)?;
    let message = service
        .update_member_function(session_id, user_id, data.function.clone(), &claims)
        .await?;
    Ok(success(message))
}

pub async fn update_work_session(
    path: web::Path<Uuid>,
    data: web::Json<CreateWorkSession>,
    query: web::Query<IncludeComplementQuery>,
    service: web::Data<WorkSessionService>,
    req: HttpRequest,
) -> Result<HttpResponse, AppError> {
    let session_id = path.into_inner();
    info!(
        "[Controller] Received request to update work session: {}",
        session_id
    );
    let claims = request_claims(&req)?;
    let session = service
        .update_work_session(
            session_id,
            data.into_inner(),
            &claims,
            include_complement(&query),
        )
        .await?;
    Ok(success(session))
}
