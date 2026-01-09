use actix_web::{HttpRequest, HttpResponse, web};
use log::info;
use uuid::Uuid;

use crate::core::entities::work_session_members::{TeamMemberFunction, UpdateMemberFunction};
use crate::core::entities::work_sessions::{
    CreateWorkSession, ListWorkSessionsQuery, UpdateWorkSessionMembers,
};
use crate::services::work_sessions::WorkSessionService;
use crate::utils::errors::AppError;
use serde::Deserialize;

pub async fn create_work_session(
    data: web::Json<CreateWorkSession>,
    service: web::Data<WorkSessionService>,
    req: HttpRequest,
) -> Result<HttpResponse, AppError> {
    info!("[Controller] Received request to create work session");
    service.create_work_session(data.into_inner(), req).await
}

pub async fn get_active_session(
    service: web::Data<WorkSessionService>,
    req: HttpRequest,
) -> Result<HttpResponse, AppError> {
    info!("[Controller] Received request to get active work session");
    service.get_active_session(req).await
}

pub async fn get_session_by_id(
    path: web::Path<Uuid>,
    service: web::Data<WorkSessionService>,
    req: HttpRequest,
) -> Result<HttpResponse, AppError> {
    let session_id = path.into_inner();
    info!(
        "[Controller] Received request to get work session: {}",
        session_id
    );
    service.get_session_by_id(session_id, req).await
}

pub async fn list_sessions(
    query: web::Query<ListWorkSessionsQuery>,
    service: web::Data<WorkSessionService>,
    req: HttpRequest,
) -> Result<HttpResponse, AppError> {
    info!("[Controller] Received request to list work sessions");
    service.list_sessions(query.into_inner(), req).await
}

pub async fn end_session(
    service: web::Data<WorkSessionService>,
    req: HttpRequest,
) -> Result<HttpResponse, AppError> {
    info!("[Controller] Received request to end work session");
    service.end_session(req).await
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
    service
        .add_member_to_session(session_id, data.user_id, data.function.clone(), req)
        .await
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
    service
        .remove_member_from_session(session_id, member_id, req)
        .await
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
    service
        .update_members(session_id, data.into_inner(), req)
        .await
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
    service
        .update_member_function(session_id, user_id, data.function.clone(), req)
        .await
}
