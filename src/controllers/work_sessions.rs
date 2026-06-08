use actix_web::{HttpRequest, HttpResponse, web};
use log::info;
use serde::Deserialize;
use uuid::Uuid;

use crate::core::application_error::ApplicationError as AppError;
use crate::core::commands::work_session_members::UpdateMemberFunction;
use crate::core::commands::work_sessions::{
    CreateWorkSession, UpdateWorkSession, UpdateWorkSessionMembers,
};
use crate::core::entities::work_session_members::TeamMemberFunction;
use crate::core::filters::common::IncludeRelatedQuery;
use crate::core::filters::work_sessions::ListWorkSessionsQuery;
use crate::presenters::work_sessions::WorkSessionPresenter;
use crate::usecases::work_sessions::{
    AddMemberToSessionUseCase, CreateWorkSessionUseCase, EndSessionUseCase,
    GetActiveSessionUseCase, GetSessionByIdUseCase, ListSessionsUseCase,
    RemoveMemberFromSessionUseCase, UpdateMemberFunctionUseCase, UpdateMembersUseCase,
    UpdateWorkSessionUseCase,
};
use crate::utils::controller_helpers::{
    created, include_related, paginated, request_claims, request_pagination_from_parts, success,
};
use crate::utils::pagination::Pagination;

pub async fn create_work_session(
    data: web::Json<CreateWorkSession>,
    query: web::Query<IncludeRelatedQuery>,
    usecase: web::Data<CreateWorkSessionUseCase>,
    presenter: web::Data<WorkSessionPresenter>,
    req: HttpRequest,
) -> Result<HttpResponse, AppError> {
    info!("[Controller] Received request to create work session");
    let claims = request_claims(&req)?;
    let session = usecase.execute(data.into_inner(), &claims).await?;
    let response = presenter
        .build_response(session, include_related(&query))
        .await?;
    Ok(created(response))
}

pub async fn get_active_session(
    query: web::Query<IncludeRelatedQuery>,
    usecase: web::Data<GetActiveSessionUseCase>,
    presenter: web::Data<WorkSessionPresenter>,
    req: HttpRequest,
) -> Result<HttpResponse, AppError> {
    info!("[Controller] Received request to get active work session");
    let claims = request_claims(&req)?;
    let session = usecase.execute(&claims).await?;
    let response = presenter
        .build_response(session, include_related(&query))
        .await?;
    Ok(success(response))
}

pub async fn get_session_by_id(
    path: web::Path<Uuid>,
    query: web::Query<IncludeRelatedQuery>,
    usecase: web::Data<GetSessionByIdUseCase>,
    presenter: web::Data<WorkSessionPresenter>,
    req: HttpRequest,
) -> Result<HttpResponse, AppError> {
    let session_id = path.into_inner();
    info!(
        "[Controller] Received request to get work session: {}",
        session_id
    );
    let claims = request_claims(&req)?;
    let session = usecase.execute(session_id, &claims).await?;
    let response = presenter
        .build_response(session, include_related(&query))
        .await?;
    Ok(success(response))
}

pub async fn list_sessions(
    query: web::Query<ListWorkSessionsQuery>,
    usecase: web::Data<ListSessionsUseCase>,
    presenter: web::Data<WorkSessionPresenter>,
    req: HttpRequest,
) -> Result<HttpResponse, AppError> {
    info!("[Controller] Received request to list work sessions");
    let claims = request_claims(&req)?;
    let query = query.into_inner();
    let include = query.include_related_entities.unwrap_or(false);
    let pagination = request_pagination_from_parts(query.page, query.page_size);
    let result = usecase.execute(query, pagination, &claims).await?;
    let response = presenter
        .build_responses(
            result.items,
            include,
            Pagination {
                page: result.page,
                page_size: result.page_size,
                offset: 0,
            },
            result.total_items,
        )
        .await?;
    Ok(paginated(response))
}

pub async fn end_session(
    usecase: web::Data<EndSessionUseCase>,
    req: HttpRequest,
) -> Result<HttpResponse, AppError> {
    info!("[Controller] Received request to end work session");
    let claims = request_claims(&req)?;
    usecase.execute(&claims).await?;
    Ok(success(()))
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AddMemberRequest {
    pub user_id: Uuid,
    pub function: Option<TeamMemberFunction>,
}

pub async fn add_member(
    path: web::Path<Uuid>,
    data: web::Json<AddMemberRequest>,
    usecase: web::Data<AddMemberToSessionUseCase>,
    req: HttpRequest,
) -> Result<HttpResponse, AppError> {
    let session_id = path.into_inner();
    info!(
        "[Controller] Received request to add member to session: {}",
        session_id
    );
    let claims = request_claims(&req)?;
    let member = usecase
        .execute(session_id, data.user_id, data.function.clone(), &claims)
        .await?;
    Ok(success(member))
}

pub async fn remove_member(
    path: web::Path<(Uuid, Uuid)>,
    usecase: web::Data<RemoveMemberFromSessionUseCase>,
    req: HttpRequest,
) -> Result<HttpResponse, AppError> {
    let (session_id, member_id) = path.into_inner();
    info!(
        "[Controller] Received request to remove member {} from session: {}",
        member_id, session_id
    );
    let claims = request_claims(&req)?;
    let message = usecase.execute(session_id, member_id, &claims).await?;
    Ok(success(message))
}

pub async fn update_members(
    path: web::Path<Uuid>,
    data: web::Json<UpdateWorkSessionMembers>,
    usecase: web::Data<UpdateMembersUseCase>,
    req: HttpRequest,
) -> Result<HttpResponse, AppError> {
    let session_id = path.into_inner();
    info!(
        "[Controller] Received request to update members of session: {}",
        session_id
    );
    let claims = request_claims(&req)?;
    let session = usecase
        .execute(session_id, data.into_inner(), &claims)
        .await?;
    Ok(success(session))
}

pub async fn update_member_function(
    path: web::Path<(Uuid, Uuid)>,
    data: web::Json<UpdateMemberFunction>,
    usecase: web::Data<UpdateMemberFunctionUseCase>,
    req: HttpRequest,
) -> Result<HttpResponse, AppError> {
    let (session_id, user_id) = path.into_inner();
    info!(
        "[Controller] Received request to update function of member {} in session: {}",
        user_id, session_id
    );
    let claims = request_claims(&req)?;
    let message = usecase
        .execute(session_id, user_id, data.function.clone(), &claims)
        .await?;
    Ok(success(message))
}

pub async fn update_work_session(
    path: web::Path<Uuid>,
    data: web::Json<UpdateWorkSession>,
    query: web::Query<IncludeRelatedQuery>,
    usecase: web::Data<UpdateWorkSessionUseCase>,
    presenter: web::Data<WorkSessionPresenter>,
    req: HttpRequest,
) -> Result<HttpResponse, AppError> {
    let session_id = path.into_inner();
    info!(
        "[Controller] Received request to update work session: {}",
        session_id
    );
    let claims = request_claims(&req)?;
    let session = usecase
        .execute(session_id, data.into_inner(), &claims)
        .await?;
    let response = presenter
        .build_response(session, include_related(&query))
        .await?;
    Ok(success(response))
}
