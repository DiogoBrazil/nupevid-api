use actix_web::{HttpRequest, HttpResponse, web};
use log::info;
use uuid::Uuid;

use crate::core::commands::attendance_victims::{CreateAttendanceVictim, UpdateAttendanceVictim};
use crate::core::entities::attendance_members::AddAttendanceMember;
use crate::services::attendance_victims::AttendanceVictimService;
use crate::utils::controller_helpers::{
    created, paginated, request_claims, request_pagination, success,
};
use crate::utils::errors::AppError;
use crate::utils::pagination::PaginationParams;

pub async fn create_attendance_victim(
    data: web::Json<CreateAttendanceVictim>,
    service: web::Data<AttendanceVictimService>,
    req: HttpRequest,
) -> Result<HttpResponse, AppError> {
    info!("[Controller] Received request to create attendance victim");
    let claims = request_claims(&req)?;
    let attendance = service
        .create_attendance_victim(data.into_inner(), &claims)
        .await?;
    Ok(created(attendance))
}

pub async fn get_attendance_victim_by_id(
    path: web::Path<Uuid>,
    service: web::Data<AttendanceVictimService>,
    req: HttpRequest,
) -> Result<HttpResponse, AppError> {
    let attendance_id = path.into_inner();
    info!(
        "[Controller] Received request to get attendance victim with id: {}",
        attendance_id
    );
    let claims = request_claims(&req)?;
    let attendance = service
        .get_attendance_victim_by_id(attendance_id, &claims)
        .await?;
    Ok(success(attendance))
}

pub async fn get_all_attendance_victims(
    query: web::Query<PaginationParams>,
    service: web::Data<AttendanceVictimService>,
    req: HttpRequest,
) -> Result<HttpResponse, AppError> {
    info!("[Controller] Received request to get all attendance victims");
    let claims = request_claims(&req)?;
    let pagination = request_pagination(&query.into_inner());
    let result = service
        .get_all_attendance_victims(pagination, &claims)
        .await?;
    Ok(paginated(result))
}

pub async fn get_attendance_victims_by_victim(
    path: web::Path<Uuid>,
    service: web::Data<AttendanceVictimService>,
    req: HttpRequest,
) -> Result<HttpResponse, AppError> {
    let victim_id = path.into_inner();
    info!(
        "[Controller] Received request to get attendance victims for victim: {}",
        victim_id
    );
    let claims = request_claims(&req)?;
    let attendances = service
        .get_attendance_victims_by_victim(victim_id, &claims)
        .await?;
    Ok(success(attendances))
}

pub async fn update_attendance_victim_by_id(
    path: web::Path<Uuid>,
    data: web::Json<UpdateAttendanceVictim>,
    service: web::Data<AttendanceVictimService>,
    req: HttpRequest,
) -> Result<HttpResponse, AppError> {
    let attendance_id = path.into_inner();
    info!(
        "[Controller] Received request to update attendance victim with id: {}",
        attendance_id
    );
    let claims = request_claims(&req)?;
    let attendance = service
        .update_attendance_victim_by_id(data.into_inner(), attendance_id, &claims)
        .await?;
    Ok(success(attendance))
}

pub async fn delete_attendance_victim_by_id(
    path: web::Path<Uuid>,
    service: web::Data<AttendanceVictimService>,
    req: HttpRequest,
) -> Result<HttpResponse, AppError> {
    let attendance_id = path.into_inner();
    info!(
        "[Controller] Received request to delete attendance victim with id: {}",
        attendance_id
    );
    let claims = request_claims(&req)?;
    let attendance = service
        .delete_attendance_victim_by_id(attendance_id, &claims)
        .await?;
    Ok(success(attendance))
}

pub async fn get_attendance_members(
    path: web::Path<Uuid>,
    service: web::Data<AttendanceVictimService>,
    req: HttpRequest,
) -> Result<HttpResponse, AppError> {
    let attendance_id = path.into_inner();
    info!(
        "[Controller] Received request to get members of attendance victim: {}",
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
    service: web::Data<AttendanceVictimService>,
    req: HttpRequest,
) -> Result<HttpResponse, AppError> {
    let attendance_id = path.into_inner();
    info!(
        "[Controller] Received request to add member to attendance victim: {}",
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
    service: web::Data<AttendanceVictimService>,
    req: HttpRequest,
) -> Result<HttpResponse, AppError> {
    let (attendance_id, user_id) = path.into_inner();
    info!(
        "[Controller] Received request to remove member {} from attendance victim: {}",
        user_id, attendance_id
    );
    let claims = request_claims(&req)?;
    let message = service
        .remove_attendance_member(attendance_id, user_id, &claims)
        .await?;
    Ok(success(message))
}
