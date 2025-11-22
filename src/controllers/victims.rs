use actix_web::{web, HttpResponse, HttpRequest};
use log::info;
use uuid::Uuid;

use crate::core::entities::victims::{
    CreateVictim,
    UpdateVictim,
    CreateVictimAddress,
    UpdateVictimAddress
};
use crate::services::victims::VictimService;
use crate::utils::errors::AppError;

pub async fn create_victim(
    victim_data: web::Json<CreateVictim>,
    victim_service: web::Data<VictimService>,
    req: HttpRequest,
) -> Result<HttpResponse, AppError> {
    info!("[Controller] Received request to create victim");
    victim_service.create_victim(victim_data.into_inner(), req).await
}

pub async fn get_victim_by_id(
    path: web::Path<Uuid>,
    victim_service: web::Data<VictimService>,
    req: HttpRequest,
) -> Result<HttpResponse, AppError> {
    let victim_id = path.into_inner();
    info!("[Controller] Received request to get victim with id: {}", victim_id);
    victim_service.get_victim_by_id(victim_id, req).await
}

pub async fn get_all_victims(
    victim_service: web::Data<VictimService>,
    req: HttpRequest,
) -> Result<HttpResponse, AppError> {
    info!("[Controller] Received request to get all victims");
    victim_service.get_all_victims(req).await
}

pub async fn update_victim_by_id(
    path: web::Path<Uuid>,
    victim_data: web::Json<UpdateVictim>,
    victim_service: web::Data<VictimService>,
    req: HttpRequest,
) -> Result<HttpResponse, AppError> {
    let victim_id = path.into_inner();
    info!("[Controller] Received request to update victim with id: {}", victim_id);
    victim_service.update_victim_by_id(victim_data.into_inner(), victim_id, req).await
}

pub async fn delete_victim_by_id(
    path: web::Path<Uuid>,
    victim_service: web::Data<VictimService>,
    req: HttpRequest,
) -> Result<HttpResponse, AppError> {
    let victim_id = path.into_inner();
    info!("[Controller] Received request to delete victim with id: {}", victim_id);
    victim_service.delete_victim_by_id(victim_id, req).await
}

pub async fn create_victim_address(
    address_data: web::Json<CreateVictimAddress>,
    victim_service: web::Data<VictimService>,
    req: HttpRequest,
) -> Result<HttpResponse, AppError> {
    info!("[Controller] Received request to create victim address");
    victim_service.create_victim_address(address_data.into_inner(), req).await
}

pub async fn get_victim_address(
    path: web::Path<Uuid>,
    victim_service: web::Data<VictimService>,
    req: HttpRequest,
) -> Result<HttpResponse, AppError> {
    let victim_id = path.into_inner();
    info!("[Controller] Received request to get address for victim: {}", victim_id);
    victim_service.get_victim_address(victim_id, req).await
}

pub async fn update_victim_address(
    path: web::Path<Uuid>,
    address_data: web::Json<UpdateVictimAddress>,
    victim_service: web::Data<VictimService>,
    req: HttpRequest,
) -> Result<HttpResponse, AppError> {
    let address_id = path.into_inner();
    info!("[Controller] Received request to update victim address: {}", address_id);
    victim_service.update_victim_address(address_data.into_inner(), address_id, req).await
}
