use actix_web::{HttpRequest, HttpResponse, web};
use log::info;
use serde::Deserialize;
use uuid::Uuid;

use crate::core::commands::attendance_offenders::{
    CreateAttendanceOffender, UpdateAttendanceOffender,
};
use crate::core::entities::attendance_members::AddAttendanceMember;
use crate::usecases::attendance_offenders::{
    AddAttendanceOffenderMemberUseCase, CreateAttendanceOffenderUseCase,
    DeleteAttendanceOffenderUseCase, GetAllAttendanceOffendersUseCase,
    GetAttendanceOffenderByIdUseCase, GetAttendanceOffenderMembersUseCase,
    GetAttendanceOffendersByOffenderUseCase, GetAttendanceOffendersByVictimUseCase,
    RemoveAttendanceOffenderMemberUseCase, UpdateAttendanceOffenderUseCase,
};
use crate::utils::controller_helpers::{
    created, paginated, request_claims, request_pagination, success,
};
use crate::core::application_error::ApplicationError as AppError;
use crate::utils::pagination::PaginationParams;

#[derive(Debug, Deserialize)]
pub struct AttendanceFilterParams {
    pub protective_measure_id: Option<Uuid>,
}

pub async fn create_attendance_offender(
    data: web::Json<CreateAttendanceOffender>,
    usecase: web::Data<CreateAttendanceOffenderUseCase>,
    req: HttpRequest,
) -> Result<HttpResponse, AppError> {
    info!("[Controller] Received request to create attendance offender");
    let claims = request_claims(&req)?;
    let attendance = usecase
        .execute(data.into_inner(), &claims)
        .await?;
    Ok(created(attendance))
}

pub async fn get_attendance_offender_by_id(
    path: web::Path<Uuid>,
    usecase: web::Data<GetAttendanceOffenderByIdUseCase>,
    req: HttpRequest,
) -> Result<HttpResponse, AppError> {
    let attendance_id = path.into_inner();
    info!(
        "[Controller] Received request to get attendance offender with id: {}",
        attendance_id
    );
    let claims = request_claims(&req)?;
    let attendance = usecase
        .execute(attendance_id, &claims)
        .await?;
    Ok(success(attendance))
}

pub async fn get_all_attendance_offenders(
    query: web::Query<PaginationParams>,
    usecase: web::Data<GetAllAttendanceOffendersUseCase>,
    req: HttpRequest,
) -> Result<HttpResponse, AppError> {
    info!("[Controller] Received request to get all attendance offenders");
    let claims = request_claims(&req)?;
    let pagination = request_pagination(&query.into_inner());
    let result = usecase
        .execute(pagination, &claims)
        .await?;
    Ok(paginated(result))
}

pub async fn get_attendance_offenders_by_offender(
    path: web::Path<Uuid>,
    query: web::Query<AttendanceFilterParams>,
    usecase: web::Data<GetAttendanceOffendersByOffenderUseCase>,
    req: HttpRequest,
) -> Result<HttpResponse, AppError> {
    let offender_id = path.into_inner();
    let filter = query.into_inner();
    info!(
        "[Controller] Received request to get attendance offenders for offender: {}",
        offender_id
    );
    let claims = request_claims(&req)?;
    let attendances = usecase
        .execute(offender_id, filter.protective_measure_id, &claims)
        .await?;
    Ok(success(attendances))
}

pub async fn get_attendance_offenders_by_victim(
    path: web::Path<Uuid>,
    query: web::Query<AttendanceFilterParams>,
    usecase: web::Data<GetAttendanceOffendersByVictimUseCase>,
    req: HttpRequest,
) -> Result<HttpResponse, AppError> {
    let victim_id = path.into_inner();
    let filter = query.into_inner();
    info!(
        "[Controller] Received request to get attendance offenders for victim: {}",
        victim_id
    );
    let claims = request_claims(&req)?;
    let attendances = usecase
        .execute(victim_id, filter.protective_measure_id, &claims)
        .await?;
    Ok(success(attendances))
}

pub async fn update_attendance_offender_by_id(
    path: web::Path<Uuid>,
    data: web::Json<UpdateAttendanceOffender>,
    usecase: web::Data<UpdateAttendanceOffenderUseCase>,
    req: HttpRequest,
) -> Result<HttpResponse, AppError> {
    let attendance_id = path.into_inner();
    info!(
        "[Controller] Received request to update attendance offender with id: {}",
        attendance_id
    );
    let claims = request_claims(&req)?;
    let attendance = usecase
        .execute(data.into_inner(), attendance_id, &claims)
        .await?;
    Ok(success(attendance))
}

pub async fn delete_attendance_offender_by_id(
    path: web::Path<Uuid>,
    usecase: web::Data<DeleteAttendanceOffenderUseCase>,
    req: HttpRequest,
) -> Result<HttpResponse, AppError> {
    let attendance_id = path.into_inner();
    info!(
        "[Controller] Received request to delete attendance offender with id: {}",
        attendance_id
    );
    let claims = request_claims(&req)?;
    let attendance = usecase
        .execute(attendance_id, &claims)
        .await?;
    Ok(success(attendance))
}

pub async fn get_attendance_members(
    path: web::Path<Uuid>,
    usecase: web::Data<GetAttendanceOffenderMembersUseCase>,
    req: HttpRequest,
) -> Result<HttpResponse, AppError> {
    let attendance_id = path.into_inner();
    info!(
        "[Controller] Received request to get members of attendance offender: {}",
        attendance_id
    );
    let claims = request_claims(&req)?;
    let members = usecase
        .execute(attendance_id, &claims)
        .await?;
    Ok(success(members))
}

pub async fn add_attendance_member(
    path: web::Path<Uuid>,
    data: web::Json<AddAttendanceMember>,
    usecase: web::Data<AddAttendanceOffenderMemberUseCase>,
    req: HttpRequest,
) -> Result<HttpResponse, AppError> {
    let attendance_id = path.into_inner();
    info!(
        "[Controller] Received request to add member to attendance offender: {}",
        attendance_id
    );
    let claims = request_claims(&req)?;
    let message = usecase
        .execute(attendance_id, data.into_inner(), &claims)
        .await?;
    Ok(success(message))
}

pub async fn remove_attendance_member(
    path: web::Path<(Uuid, Uuid)>,
    usecase: web::Data<RemoveAttendanceOffenderMemberUseCase>,
    req: HttpRequest,
) -> Result<HttpResponse, AppError> {
    let (attendance_id, user_id) = path.into_inner();
    info!(
        "[Controller] Received request to remove member {} from attendance offender: {}",
        user_id, attendance_id
    );
    let claims = request_claims(&req)?;
    let message = usecase
        .execute(attendance_id, user_id, &claims)
        .await?;
    Ok(success(message))
}
