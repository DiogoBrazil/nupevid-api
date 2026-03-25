use actix_web::{HttpRequest, HttpResponse, web};
use log::info;
use uuid::Uuid;

use crate::core::commands::attendance_offenders::{
    CreateAttendanceOffender, UpdateAttendanceOffender,
};
use crate::core::entities::attendance_members::AddAttendanceMember;
use crate::services::attendance_offenders::AttendanceOffenderService;
use crate::utils::controller_helpers::{
    created, paginated, request_claims, request_pagination, success,
};
use crate::utils::errors::AppError;
use crate::utils::pagination::PaginationParams;

pub async fn create_attendance_offender(
    data: web::Json<CreateAttendanceOffender>,
    service: web::Data<AttendanceOffenderService>,
    req: HttpRequest,
) -> Result<HttpResponse, AppError> {
    info!("[Controller] Received request to create attendance offender");
    let claims = request_claims(&req)?;
    let attendance = service
        .create_attendance_offender(data.into_inner(), &claims)
        .await?;
    Ok(created(attendance))
}

pub async fn get_attendance_offender_by_id(
    path: web::Path<Uuid>,
    service: web::Data<AttendanceOffenderService>,
    req: HttpRequest,
) -> Result<HttpResponse, AppError> {
    let attendance_id = path.into_inner();
    info!(
        "[Controller] Received request to get attendance offender with id: {}",
        attendance_id
    );
    let claims = request_claims(&req)?;
    let attendance = service
        .get_attendance_offender_by_id(attendance_id, &claims)
        .await?;
    Ok(success(attendance))
}

pub async fn get_all_attendance_offenders(
    query: web::Query<PaginationParams>,
    service: web::Data<AttendanceOffenderService>,
    req: HttpRequest,
) -> Result<HttpResponse, AppError> {
    info!("[Controller] Received request to get all attendance offenders");
    let claims = request_claims(&req)?;
    let pagination = request_pagination(&query.into_inner());
    let result = service
        .get_all_attendance_offenders(pagination, &claims)
        .await?;
    Ok(paginated(result))
}

pub async fn get_attendance_offenders_by_offender(
    path: web::Path<Uuid>,
    service: web::Data<AttendanceOffenderService>,
    req: HttpRequest,
) -> Result<HttpResponse, AppError> {
    let offender_id = path.into_inner();
    info!(
        "[Controller] Received request to get attendance offenders for offender: {}",
        offender_id
    );
    let claims = request_claims(&req)?;
    let attendances = service
        .get_attendance_offenders_by_offender(offender_id, &claims)
        .await?;
    Ok(success(attendances))
}

pub async fn get_attendance_offenders_by_victim(
    path: web::Path<Uuid>,
    service: web::Data<AttendanceOffenderService>,
    req: HttpRequest,
) -> Result<HttpResponse, AppError> {
    let victim_id = path.into_inner();
    info!(
        "[Controller] Received request to get attendance offenders for victim: {}",
        victim_id
    );
    let claims = request_claims(&req)?;
    let attendances = service
        .get_attendance_offenders_by_victim(victim_id, &claims)
        .await?;
    Ok(success(attendances))
}

pub async fn update_attendance_offender_by_id(
    path: web::Path<Uuid>,
    data: web::Json<UpdateAttendanceOffender>,
    service: web::Data<AttendanceOffenderService>,
    req: HttpRequest,
) -> Result<HttpResponse, AppError> {
    let attendance_id = path.into_inner();
    info!(
        "[Controller] Received request to update attendance offender with id: {}",
        attendance_id
    );
    let claims = request_claims(&req)?;
    let attendance = service
        .update_attendance_offender_by_id(data.into_inner(), attendance_id, &claims)
        .await?;
    Ok(success(attendance))
}

pub async fn delete_attendance_offender_by_id(
    path: web::Path<Uuid>,
    service: web::Data<AttendanceOffenderService>,
    req: HttpRequest,
) -> Result<HttpResponse, AppError> {
    let attendance_id = path.into_inner();
    info!(
        "[Controller] Received request to delete attendance offender with id: {}",
        attendance_id
    );
    let claims = request_claims(&req)?;
    let attendance = service
        .delete_attendance_offender_by_id(attendance_id, &claims)
        .await?;
    Ok(success(attendance))
}

pub async fn get_attendance_members(
    path: web::Path<Uuid>,
    service: web::Data<AttendanceOffenderService>,
    req: HttpRequest,
) -> Result<HttpResponse, AppError> {
    let attendance_id = path.into_inner();
    info!(
        "[Controller] Received request to get members of attendance offender: {}",
        attendance_id
    );
    let claims = request_claims(&req)?;
    let members = service
        .get_attendance_members(attendance_id, &claims)
        .await?;
    Ok(success(members))
}

pub async fn add_attendance_member(
    path: web::Path<Uuid>,
    data: web::Json<AddAttendanceMember>,
    service: web::Data<AttendanceOffenderService>,
    req: HttpRequest,
) -> Result<HttpResponse, AppError> {
    let attendance_id = path.into_inner();
    info!(
        "[Controller] Received request to add member to attendance offender: {}",
        attendance_id
    );
    let claims = request_claims(&req)?;
    let message = service
        .add_attendance_member(attendance_id, data.into_inner(), &claims)
        .await?;
    Ok(success(message))
}

pub async fn remove_attendance_member(
    path: web::Path<(Uuid, Uuid)>,
    service: web::Data<AttendanceOffenderService>,
    req: HttpRequest,
) -> Result<HttpResponse, AppError> {
    let (attendance_id, user_id) = path.into_inner();
    info!(
        "[Controller] Received request to remove member {} from attendance offender: {}",
        user_id, attendance_id
    );
    let claims = request_claims(&req)?;
    let message = service
        .remove_attendance_member(attendance_id, user_id, &claims)
        .await?;
    Ok(success(message))
}
