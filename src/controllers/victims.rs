use actix_web::{web, HttpRequest, HttpResponse};
use log::info;
use serde::Deserialize;
use uuid::Uuid;

use crate::core::entities::victims::{AddressData, CreateVictim, PhoneData, UpdateVictim};
use crate::services::victims::VictimService;
use crate::utils::errors::AppError;
use crate::utils::pagination::PaginationParams;

#[derive(Deserialize)]
pub struct VictimSearchQuery {
    pub name: Option<String>,
    pub cpf: Option<String>,
}

pub async fn create_victim(
    victim_data: web::Json<CreateVictim>,
    victim_service: web::Data<VictimService>,
    req: HttpRequest,
) -> Result<HttpResponse, AppError> {
    info!("[Controller] Received request to create victim");
    victim_service
        .create_victim(victim_data.into_inner(), req)
        .await
}

pub async fn get_victim_by_id(
    path: web::Path<Uuid>,
    victim_service: web::Data<VictimService>,
    req: HttpRequest,
) -> Result<HttpResponse, AppError> {
    let victim_id = path.into_inner();
    info!(
        "[Controller] Received request to get victim with id: {}",
        victim_id
    );
    victim_service.get_victim_by_id(victim_id, req).await
}

pub async fn get_all_victims(
    query: web::Query<PaginationParams>,
    victim_service: web::Data<VictimService>,
    req: HttpRequest,
) -> Result<HttpResponse, AppError> {
    info!("[Controller] Received request to get all victims");
    victim_service.get_all_victims(query.into_inner(), req).await
}

pub async fn search_victims(
    query: web::Query<VictimSearchQuery>,
    victim_service: web::Data<VictimService>,
    req: HttpRequest,
) -> Result<HttpResponse, AppError> {
    let query = query.into_inner();
    info!("[Controller] Received request to search victims");
    victim_service.search_victims(query.name, query.cpf, req).await
}

pub async fn update_victim_by_id(
    path: web::Path<Uuid>,
    victim_data: web::Json<UpdateVictim>,
    victim_service: web::Data<VictimService>,
    req: HttpRequest,
) -> Result<HttpResponse, AppError> {
    let victim_id = path.into_inner();
    info!(
        "[Controller] Received request to update victim with id: {}",
        victim_id
    );
    victim_service
        .update_victim_by_id(victim_data.into_inner(), victim_id, req)
        .await
}

pub async fn delete_victim_by_id(
    path: web::Path<Uuid>,
    victim_service: web::Data<VictimService>,
    req: HttpRequest,
) -> Result<HttpResponse, AppError> {
    let victim_id = path.into_inner();
    info!(
        "[Controller] Received request to delete victim with id: {}",
        victim_id
    );
    victim_service.delete_victim_by_id(victim_id, req).await
}

pub async fn add_phone_to_victim(
    path: web::Path<Uuid>,
    phone_data: web::Json<PhoneData>,
    victim_service: web::Data<VictimService>,
    req: HttpRequest,
) -> Result<HttpResponse, AppError> {
    let victim_id = path.into_inner();
    info!("[Controller] Received request to add phone to victim {}", victim_id);
    victim_service
        .create_phone(victim_id, phone_data.into_inner(), req)
        .await
}

pub async fn update_victim_phone(
    path: web::Path<(Uuid, Uuid)>,
    phone_data: web::Json<PhoneData>,
    victim_service: web::Data<VictimService>,
    req: HttpRequest,
) -> Result<HttpResponse, AppError> {
    let (_victim_id, phone_id) = path.into_inner();
    info!("[Controller] Received request to update phone {}", phone_id);
    victim_service
        .update_phone(phone_id, phone_data.into_inner(), req)
        .await
}

pub async fn delete_victim_phone(
    path: web::Path<(Uuid, Uuid)>,
    victim_service: web::Data<VictimService>,
    req: HttpRequest,
) -> Result<HttpResponse, AppError> {
    let (_victim_id, phone_id) = path.into_inner();
    info!("[Controller] Received request to delete phone {}", phone_id);
    victim_service.delete_phone(phone_id, req).await
}

pub async fn add_address_to_victim(
    path: web::Path<Uuid>,
    address_data: web::Json<AddressData>,
    victim_service: web::Data<VictimService>,
    req: HttpRequest,
) -> Result<HttpResponse, AppError> {
    let victim_id = path.into_inner();
    info!("[Controller] Received request to add address to victim {}", victim_id);
    victim_service
        .create_address(victim_id, address_data.into_inner(), req)
        .await
}

pub async fn update_victim_address(
    path: web::Path<(Uuid, Uuid)>,
    address_data: web::Json<AddressData>,
    victim_service: web::Data<VictimService>,
    req: HttpRequest,
) -> Result<HttpResponse, AppError> {
    let (_victim_id, address_id) = path.into_inner();
    info!("[Controller] Received request to update address {}", address_id);
    victim_service
        .update_address(address_id, address_data.into_inner(), req)
        .await
}

pub async fn delete_victim_address(
    path: web::Path<(Uuid, Uuid)>,
    victim_service: web::Data<VictimService>,
    req: HttpRequest,
) -> Result<HttpResponse, AppError> {
    let (_victim_id, address_id) = path.into_inner();
    info!("[Controller] Received request to delete address {}", address_id);
    victim_service.delete_address(address_id, req).await
}
