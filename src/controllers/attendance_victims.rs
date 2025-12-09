use actix_web::{web, HttpRequest, HttpResponse};
use log::info;
use uuid::Uuid;

use crate::core::entities::attendance_victims::{CreateAttendanceVictim, UpdateAttendanceVictim};
use crate::services::attendance_victims::AttendanceVictimService;
use crate::utils::errors::AppError;

pub async fn create_attendance_victim(
    data: web::Json<CreateAttendanceVictim>,
    service: web::Data<AttendanceVictimService>,
    req: HttpRequest,
) -> Result<HttpResponse, AppError> {
    info!("[Controller] Received request to create attendance victim");
    service.create_attendance_victim(data.into_inner(), req).await
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
    service.get_attendance_victim_by_id(attendance_id, req).await
}

pub async fn get_all_attendance_victims(
    service: web::Data<AttendanceVictimService>,
    req: HttpRequest,
) -> Result<HttpResponse, AppError> {
    info!("[Controller] Received request to get all attendance victims");
    service.get_all_attendance_victims(req).await
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
    service.get_attendance_victims_by_victim(victim_id, req).await
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
    service
        .update_attendance_victim_by_id(data.into_inner(), attendance_id, req)
        .await
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
    service.delete_attendance_victim_by_id(attendance_id, req).await
}
