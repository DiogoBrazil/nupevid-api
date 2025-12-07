use actix_web::{web, HttpRequest, HttpResponse};
use log::info;
use uuid::Uuid;

use crate::core::entities::protective_measures::{CreateExtension, UpdateExtension};
use crate::services::extensions::ExtensionService;
use crate::utils::errors::AppError;

pub async fn create_extension(
    path: web::Path<Uuid>,
    body: web::Json<CreateExtension>,
    service: web::Data<ExtensionService>,
    req: HttpRequest,
) -> Result<HttpResponse, AppError> {
    let protective_measure_id = path.into_inner();
    info!("[Controller] Creating extension for protective measure: {}", protective_measure_id);
    service.create_extension(protective_measure_id, body.into_inner(), req).await
}

pub async fn get_extension_by_id(
    path: web::Path<(Uuid, Uuid)>,
    service: web::Data<ExtensionService>,
    req: HttpRequest,
) -> Result<HttpResponse, AppError> {
    let (_protective_measure_id, extension_id) = path.into_inner();
    info!("[Controller] Getting extension with ID: {}", extension_id);
    service.get_extension_by_id(extension_id, req).await
}

pub async fn get_extensions_by_measure(
    path: web::Path<Uuid>,
    service: web::Data<ExtensionService>,
    req: HttpRequest,
) -> Result<HttpResponse, AppError> {
    let protective_measure_id = path.into_inner();
    info!("[Controller] Getting extensions for protective measure: {}", protective_measure_id);
    service.get_extensions_by_measure(protective_measure_id, req).await
}

pub async fn update_extension_by_id(
    path: web::Path<(Uuid, Uuid)>,
    body: web::Json<UpdateExtension>,
    service: web::Data<ExtensionService>,
    req: HttpRequest,
) -> Result<HttpResponse, AppError> {
    let (_protective_measure_id, extension_id) = path.into_inner();
    info!("[Controller] Updating extension with ID: {}", extension_id);
    service.update_extension_by_id(extension_id, body.into_inner(), req).await
}

pub async fn delete_extension_by_id(
    path: web::Path<(Uuid, Uuid)>,
    service: web::Data<ExtensionService>,
    req: HttpRequest,
) -> Result<HttpResponse, AppError> {
    let (_protective_measure_id, extension_id) = path.into_inner();
    info!("[Controller] Deleting extension with ID: {}", extension_id);
    service.delete_extension_by_id(extension_id, req).await
}
