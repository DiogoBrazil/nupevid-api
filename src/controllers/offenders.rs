use actix_web::{web, HttpRequest, HttpResponse};
use log::info;
use uuid::Uuid;

use crate::core::entities::offenders::{
    AddressData, CreateOffender, PhoneData, UpdateOffender, WorkAddressData,
};
use crate::services::offenders::OffenderService;
use crate::utils::errors::AppError;

pub async fn create_offender(
    offender_data: web::Json<CreateOffender>,
    offender_service: web::Data<OffenderService>,
    req: HttpRequest,
) -> Result<HttpResponse, AppError> {
    info!("[Controller] Received request to create offender");
    offender_service
        .create_offender(offender_data.into_inner(), req)
        .await
}

pub async fn get_offender_by_id(
    path: web::Path<Uuid>,
    offender_service: web::Data<OffenderService>,
    req: HttpRequest,
) -> Result<HttpResponse, AppError> {
    let offender_id = path.into_inner();
    info!(
        "[Controller] Received request to get offender with id: {}",
        offender_id
    );
    offender_service.get_offender_by_id(offender_id, req).await
}

pub async fn get_all_offenders(
    offender_service: web::Data<OffenderService>,
    req: HttpRequest,
) -> Result<HttpResponse, AppError> {
    info!("[Controller] Received request to get all offenders");
    offender_service.get_all_offenders(req).await
}

pub async fn get_offenders_by_victim_id(
    path: web::Path<Uuid>,
    offender_service: web::Data<OffenderService>,
    req: HttpRequest,
) -> Result<HttpResponse, AppError> {
    let victim_id = path.into_inner();
    info!(
        "[Controller] Received request to get offenders for victim: {}",
        victim_id
    );
    offender_service
        .get_offenders_by_victim_id(victim_id, req)
        .await
}

pub async fn update_offender_by_id(
    path: web::Path<Uuid>,
    offender_data: web::Json<UpdateOffender>,
    offender_service: web::Data<OffenderService>,
    req: HttpRequest,
) -> Result<HttpResponse, AppError> {
    let offender_id = path.into_inner();
    info!(
        "[Controller] Received request to update offender with id: {}",
        offender_id
    );
    offender_service
        .update_offender_by_id(offender_data.into_inner(), offender_id, req)
        .await
}

pub async fn delete_offender_by_id(
    path: web::Path<Uuid>,
    offender_service: web::Data<OffenderService>,
    req: HttpRequest,
) -> Result<HttpResponse, AppError> {
    let offender_id = path.into_inner();
    info!(
        "[Controller] Received request to delete offender with id: {}",
        offender_id
    );
    offender_service.delete_offender_by_id(offender_id, req).await
}

pub async fn add_phone_to_offender(
    path: web::Path<Uuid>,
    phone_data: web::Json<PhoneData>,
    offender_service: web::Data<OffenderService>,
    req: HttpRequest,
) -> Result<HttpResponse, AppError> {
    let offender_id = path.into_inner();
    info!(
        "[Controller] Received request to add phone to offender {}",
        offender_id
    );
    offender_service
        .create_phone(offender_id, phone_data.into_inner(), req)
        .await
}

pub async fn update_offender_phone(
    path: web::Path<(Uuid, Uuid)>,
    phone_data: web::Json<PhoneData>,
    offender_service: web::Data<OffenderService>,
    req: HttpRequest,
) -> Result<HttpResponse, AppError> {
    let (_offender_id, phone_id) = path.into_inner();
    info!(
        "[Controller] Received request to update phone {}",
        phone_id
    );
    offender_service
        .update_phone(phone_id, phone_data.into_inner(), req)
        .await
}

pub async fn delete_offender_phone(
    path: web::Path<(Uuid, Uuid)>,
    offender_service: web::Data<OffenderService>,
    req: HttpRequest,
) -> Result<HttpResponse, AppError> {
    let (_offender_id, phone_id) = path.into_inner();
    info!(
        "[Controller] Received request to delete phone {}",
        phone_id
    );
    offender_service.delete_phone(phone_id, req).await
}

pub async fn add_address_to_offender(
    path: web::Path<Uuid>,
    address_data: web::Json<AddressData>,
    offender_service: web::Data<OffenderService>,
    req: HttpRequest,
) -> Result<HttpResponse, AppError> {
    let offender_id = path.into_inner();
    info!(
        "[Controller] Received request to add address to offender {}",
        offender_id
    );
    offender_service
        .create_address(offender_id, address_data.into_inner(), req)
        .await
}

pub async fn update_offender_address(
    path: web::Path<(Uuid, Uuid)>,
    address_data: web::Json<AddressData>,
    offender_service: web::Data<OffenderService>,
    req: HttpRequest,
) -> Result<HttpResponse, AppError> {
    let (_offender_id, address_id) = path.into_inner();
    info!(
        "[Controller] Received request to update address {}",
        address_id
    );
    offender_service
        .update_address(address_id, address_data.into_inner(), req)
        .await
}

pub async fn delete_offender_address(
    path: web::Path<(Uuid, Uuid)>,
    offender_service: web::Data<OffenderService>,
    req: HttpRequest,
) -> Result<HttpResponse, AppError> {
    let (_offender_id, address_id) = path.into_inner();
    info!(
        "[Controller] Received request to delete address {}",
        address_id
    );
    offender_service.delete_address(address_id, req).await
}

pub async fn add_work_address_to_offender(
    path: web::Path<Uuid>,
    work_address_data: web::Json<WorkAddressData>,
    offender_service: web::Data<OffenderService>,
    req: HttpRequest,
) -> Result<HttpResponse, AppError> {
    let offender_id = path.into_inner();
    info!(
        "[Controller] Received request to add work address to offender {}",
        offender_id
    );
    offender_service
        .create_work_address(offender_id, work_address_data.into_inner(), req)
        .await
}

pub async fn update_offender_work_address(
    path: web::Path<(Uuid, Uuid)>,
    work_address_data: web::Json<WorkAddressData>,
    offender_service: web::Data<OffenderService>,
    req: HttpRequest,
) -> Result<HttpResponse, AppError> {
    let (_offender_id, work_address_id) = path.into_inner();
    info!(
        "[Controller] Received request to update work address {}",
        work_address_id
    );
    offender_service
        .update_work_address(work_address_id, work_address_data.into_inner(), req)
        .await
}

pub async fn delete_offender_work_address(
    path: web::Path<(Uuid, Uuid)>,
    offender_service: web::Data<OffenderService>,
    req: HttpRequest,
) -> Result<HttpResponse, AppError> {
    let (_offender_id, work_address_id) = path.into_inner();
    info!(
        "[Controller] Received request to delete work address {}",
        work_address_id
    );
    offender_service
        .delete_work_address(work_address_id, req)
        .await
}
