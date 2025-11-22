use actix_web::{web, HttpResponse, HttpRequest};
use log::info;
use uuid::Uuid;

use crate::core::entities::attendances::{
    CreateAttendance,
    UpdateAttendance,
    CreateAttendanceAddress,
    UpdateAttendanceAddress
};
use crate::services::attendances::AttendanceService;
use crate::utils::errors::AppError;

pub async fn create_attendance(
    data: web::Json<CreateAttendance>,
    service: web::Data<AttendanceService>,
    req: HttpRequest
) -> Result<HttpResponse, AppError> {
    info!("[Controller] Create attendance");
    service.create_attendance(data.into_inner(), req).await
}

pub async fn get_attendance_by_id(
    path: web::Path<Uuid>,
    service: web::Data<AttendanceService>,
    req: HttpRequest
) -> Result<HttpResponse, AppError> {
    service.get_attendance_by_id(path.into_inner(), req).await
}

pub async fn get_all_attendances(
    service: web::Data<AttendanceService>,
    req: HttpRequest
) -> Result<HttpResponse, AppError> {
    service.get_all_attendances(req).await
}

pub async fn get_attendances_by_victim(
    path: web::Path<Uuid>,
    service: web::Data<AttendanceService>,
    req: HttpRequest
) -> Result<HttpResponse, AppError> {
    service.get_attendances_by_victim(path.into_inner(), req).await
}

pub async fn update_attendance_by_id(
    path: web::Path<Uuid>,
    data: web::Json<UpdateAttendance>,
    service: web::Data<AttendanceService>,
    req: HttpRequest
) -> Result<HttpResponse, AppError> {
    service.update_attendance_by_id(data.into_inner(), path.into_inner(), req).await
}

pub async fn delete_attendance_by_id(
    path: web::Path<Uuid>,
    service: web::Data<AttendanceService>,
    req: HttpRequest
) -> Result<HttpResponse, AppError> {
    service.delete_attendance_by_id(path.into_inner(), req).await
}

pub async fn create_attendance_address(
    data: web::Json<CreateAttendanceAddress>,
    service: web::Data<AttendanceService>,
    req: HttpRequest
) -> Result<HttpResponse, AppError> {
    service.create_attendance_address(data.into_inner(), req).await
}

pub async fn get_attendance_address(
    path: web::Path<Uuid>,
    service: web::Data<AttendanceService>,
    req: HttpRequest
) -> Result<HttpResponse, AppError> {
    service.get_attendance_address(path.into_inner(), req).await
}

pub async fn update_attendance_address(
    path: web::Path<Uuid>,
    data: web::Json<UpdateAttendanceAddress>,
    service: web::Data<AttendanceService>,
    req: HttpRequest
) -> Result<HttpResponse, AppError> {
    service.update_attendance_address(data.into_inner(), path.into_inner(), req).await
}
