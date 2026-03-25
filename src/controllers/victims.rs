use actix_web::{HttpRequest, HttpResponse, web};
use log::info;
use uuid::Uuid;

use crate::core::commands::victims::{CreateVictim, UpdateVictim};
use crate::core::entities::victims::{AddressData, PhoneData};
use crate::core::queries::victims::VictimSearchQuery;
use crate::services::victims::VictimService;
use crate::utils::controller_helpers::{
    created, paginated, request_claims, request_pagination, success,
};
use crate::utils::errors::AppError;
use crate::utils::pagination::PaginationParams;

pub async fn create_victim(
    victim_data: web::Json<CreateVictim>,
    victim_service: web::Data<VictimService>,
    req: HttpRequest,
) -> Result<HttpResponse, AppError> {
    info!("[Controller] Received request to create victim");
    let claims = request_claims(&req)?;
    let victim = victim_service
        .create_victim(victim_data.into_inner(), &claims)
        .await?;
    Ok(created(victim))
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
    let claims = request_claims(&req)?;
    let victim = victim_service.get_victim_by_id(victim_id, &claims).await?;
    Ok(success(victim))
}

pub async fn get_all_victims(
    query: web::Query<PaginationParams>,
    victim_service: web::Data<VictimService>,
    req: HttpRequest,
) -> Result<HttpResponse, AppError> {
    info!("[Controller] Received request to get all victims");
    let claims = request_claims(&req)?;
    let pagination = request_pagination(&query.into_inner());
    let result = victim_service.get_all_victims(pagination, &claims).await?;
    Ok(paginated(result))
}

pub async fn search_victims(
    query: web::Query<VictimSearchQuery>,
    victim_service: web::Data<VictimService>,
    req: HttpRequest,
) -> Result<HttpResponse, AppError> {
    let query = query.into_inner();
    info!("[Controller] Received request to search victims");
    let claims = request_claims(&req)?;
    let victims = victim_service
        .search_victims(query.name, query.cpf, &claims)
        .await?;
    Ok(success(victims))
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
    let claims = request_claims(&req)?;
    let victim = victim_service
        .update_victim_by_id(victim_data.into_inner(), victim_id, &claims)
        .await?;
    Ok(success(victim))
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
    let claims = request_claims(&req)?;
    let victim = victim_service
        .delete_victim_by_id(victim_id, &claims)
        .await?;
    Ok(success(victim))
}

pub async fn add_phone_to_victim(
    path: web::Path<Uuid>,
    phone_data: web::Json<PhoneData>,
    victim_service: web::Data<VictimService>,
    req: HttpRequest,
) -> Result<HttpResponse, AppError> {
    let victim_id = path.into_inner();
    info!(
        "[Controller] Received request to add phone to victim {}",
        victim_id
    );
    let claims = request_claims(&req)?;
    let phone = victim_service
        .create_phone(victim_id, phone_data.into_inner(), &claims)
        .await?;
    Ok(created(phone))
}

pub async fn update_victim_phone(
    path: web::Path<(Uuid, Uuid)>,
    phone_data: web::Json<PhoneData>,
    victim_service: web::Data<VictimService>,
    req: HttpRequest,
) -> Result<HttpResponse, AppError> {
    let (_victim_id, phone_id) = path.into_inner();
    info!("[Controller] Received request to update phone {}", phone_id);
    let claims = request_claims(&req)?;
    let phone = victim_service
        .update_phone(phone_id, phone_data.into_inner(), &claims)
        .await?;
    Ok(success(phone))
}

pub async fn delete_victim_phone(
    path: web::Path<(Uuid, Uuid)>,
    victim_service: web::Data<VictimService>,
    req: HttpRequest,
) -> Result<HttpResponse, AppError> {
    let (_victim_id, phone_id) = path.into_inner();
    info!("[Controller] Received request to delete phone {}", phone_id);
    let claims = request_claims(&req)?;
    let phone = victim_service.delete_phone(phone_id, &claims).await?;
    Ok(success(phone))
}

pub async fn add_address_to_victim(
    path: web::Path<Uuid>,
    address_data: web::Json<AddressData>,
    victim_service: web::Data<VictimService>,
    req: HttpRequest,
) -> Result<HttpResponse, AppError> {
    let victim_id = path.into_inner();
    info!(
        "[Controller] Received request to add address to victim {}",
        victim_id
    );
    let claims = request_claims(&req)?;
    let address = victim_service
        .create_address(victim_id, address_data.into_inner(), &claims)
        .await?;
    Ok(created(address))
}

pub async fn update_victim_address(
    path: web::Path<(Uuid, Uuid)>,
    address_data: web::Json<AddressData>,
    victim_service: web::Data<VictimService>,
    req: HttpRequest,
) -> Result<HttpResponse, AppError> {
    let (_victim_id, address_id) = path.into_inner();
    info!(
        "[Controller] Received request to update address {}",
        address_id
    );
    let claims = request_claims(&req)?;
    let address = victim_service
        .update_address(address_id, address_data.into_inner(), &claims)
        .await?;
    Ok(success(address))
}

pub async fn delete_victim_address(
    path: web::Path<(Uuid, Uuid)>,
    victim_service: web::Data<VictimService>,
    req: HttpRequest,
) -> Result<HttpResponse, AppError> {
    let (_victim_id, address_id) = path.into_inner();
    info!(
        "[Controller] Received request to delete address {}",
        address_id
    );
    let claims = request_claims(&req)?;
    let address = victim_service.delete_address(address_id, &claims).await?;
    Ok(success(address))
}
