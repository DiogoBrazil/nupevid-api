use actix_web::{web, HttpRequest, HttpResponse};
use log::info;
use uuid::Uuid;

use crate::core::entities::attendances::{CreateAttendance, UpdateAttendance};
use crate::services::attendances::AttendanceService;
use crate::utils::errors::AppError;

pub async fn create_attendance(
    data: web::Json<CreateAttendance>,
    service: web::Data<AttendanceService>,
    req: HttpRequest,
) -> Result<HttpResponse, AppError> {
    info!("[Controller] Received request to create attendance");
    service.create_attendance(data.into_inner(), req).await
}

pub async fn get_attendance_by_id(
    path: web::Path<Uuid>,
    service: web::Data<AttendanceService>,
    req: HttpRequest,
) -> Result<HttpResponse, AppError> {
    let attendance_id = path.into_inner();
    info!(
        "[Controller] Received request to get attendance with id: {}",
        attendance_id
    );
    service.get_attendance_by_id(attendance_id, req).await
}

pub async fn get_all_attendances(
    service: web::Data<AttendanceService>,
    req: HttpRequest,
) -> Result<HttpResponse, AppError> {
    info!("[Controller] Received request to get all attendances");
    service.get_all_attendances(req).await
}

pub async fn get_attendances_by_victim(
    path: web::Path<Uuid>,
    service: web::Data<AttendanceService>,
    req: HttpRequest,
) -> Result<HttpResponse, AppError> {
    let victim_id = path.into_inner();
    info!(
        "[Controller] Received request to get attendances for victim: {}",
        victim_id
    );
    service.get_attendances_by_victim(victim_id, req).await
}

pub async fn update_attendance_by_id(
    path: web::Path<Uuid>,
    data: web::Json<UpdateAttendance>,
    service: web::Data<AttendanceService>,
    req: HttpRequest,
) -> Result<HttpResponse, AppError> {
    let attendance_id = path.into_inner();
    info!(
        "[Controller] Received request to update attendance with id: {}",
        attendance_id
    );
    service
        .update_attendance_by_id(data.into_inner(), attendance_id, req)
        .await
}

pub async fn delete_attendance_by_id(
    path: web::Path<Uuid>,
    service: web::Data<AttendanceService>,
    req: HttpRequest,
) -> Result<HttpResponse, AppError> {
    let attendance_id = path.into_inner();
    info!(
        "[Controller] Received request to delete attendance with id: {}",
        attendance_id
    );
    service.delete_attendance_by_id(attendance_id, req).await
}
