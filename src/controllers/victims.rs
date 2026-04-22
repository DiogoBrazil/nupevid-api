use actix_web::{HttpRequest, HttpResponse, web};
use log::info;
use uuid::Uuid;

use crate::core::application_error::ApplicationError as AppError;
use crate::core::commands::victims::{CreateVictim, UpdateVictim};
use crate::core::entities::victims::{AddressData, PhoneData};
use crate::core::filters::victims::VictimSearchQuery;
use crate::usecases::victims::{
    CreateVictimAddressUseCase, CreateVictimPhoneUseCase, CreateVictimUseCase,
    DeleteVictimAddressUseCase, DeleteVictimPhoneUseCase, DeleteVictimUseCase,
    GetAllVictimsUseCase, GetVictimByIdUseCase, SearchVictimsUseCase, UpdateVictimAddressUseCase,
    UpdateVictimPhoneUseCase, UpdateVictimUseCase,
};
use crate::utils::controller_helpers::{
    created, paginated, request_claims, request_pagination, success,
};
use crate::utils::pagination::PaginationParams;

pub async fn create_victim(
    victim_data: web::Json<CreateVictim>,
    usecase: web::Data<CreateVictimUseCase>,
    req: HttpRequest,
) -> Result<HttpResponse, AppError> {
    info!("[Controller] Received request to create victim");
    let claims = request_claims(&req)?;
    let victim = usecase.execute(victim_data.into_inner(), &claims).await?;
    Ok(created(victim))
}

pub async fn get_victim_by_id(
    path: web::Path<Uuid>,
    usecase: web::Data<GetVictimByIdUseCase>,
    req: HttpRequest,
) -> Result<HttpResponse, AppError> {
    let victim_id = path.into_inner();
    info!(
        "[Controller] Received request to get victim with id: {}",
        victim_id
    );
    let claims = request_claims(&req)?;
    let victim = usecase.execute(victim_id, &claims).await?;
    Ok(success(victim))
}

pub async fn get_all_victims(
    query: web::Query<PaginationParams>,
    usecase: web::Data<GetAllVictimsUseCase>,
    req: HttpRequest,
) -> Result<HttpResponse, AppError> {
    info!("[Controller] Received request to get all victims");
    let claims = request_claims(&req)?;
    let pagination = request_pagination(&query.into_inner());
    let result = usecase.execute(pagination, &claims).await?;
    Ok(paginated(result))
}

pub async fn search_victims(
    query: web::Query<VictimSearchQuery>,
    usecase: web::Data<SearchVictimsUseCase>,
    req: HttpRequest,
) -> Result<HttpResponse, AppError> {
    let query = query.into_inner();
    info!("[Controller] Received request to search victims");
    let claims = request_claims(&req)?;
    let victims = usecase.execute(query.name, query.cpf, &claims).await?;
    Ok(success(victims))
}

pub async fn update_victim_by_id(
    path: web::Path<Uuid>,
    victim_data: web::Json<UpdateVictim>,
    usecase: web::Data<UpdateVictimUseCase>,
    req: HttpRequest,
) -> Result<HttpResponse, AppError> {
    let victim_id = path.into_inner();
    info!(
        "[Controller] Received request to update victim with id: {}",
        victim_id
    );
    let claims = request_claims(&req)?;
    let victim = usecase
        .execute(victim_data.into_inner(), victim_id, &claims)
        .await?;
    Ok(success(victim))
}

pub async fn delete_victim_by_id(
    path: web::Path<Uuid>,
    usecase: web::Data<DeleteVictimUseCase>,
    req: HttpRequest,
) -> Result<HttpResponse, AppError> {
    let victim_id = path.into_inner();
    info!(
        "[Controller] Received request to delete victim with id: {}",
        victim_id
    );
    let claims = request_claims(&req)?;
    let victim = usecase.execute(victim_id, &claims).await?;
    Ok(success(victim))
}

pub async fn add_phone_to_victim(
    path: web::Path<Uuid>,
    phone_data: web::Json<PhoneData>,
    usecase: web::Data<CreateVictimPhoneUseCase>,
    req: HttpRequest,
) -> Result<HttpResponse, AppError> {
    let victim_id = path.into_inner();
    info!(
        "[Controller] Received request to add phone to victim {}",
        victim_id
    );
    let claims = request_claims(&req)?;
    let phone = usecase
        .execute(victim_id, phone_data.into_inner(), &claims)
        .await?;
    Ok(created(phone))
}

pub async fn update_victim_phone(
    path: web::Path<(Uuid, Uuid)>,
    phone_data: web::Json<PhoneData>,
    usecase: web::Data<UpdateVictimPhoneUseCase>,
    req: HttpRequest,
) -> Result<HttpResponse, AppError> {
    let (_victim_id, phone_id) = path.into_inner();
    info!("[Controller] Received request to update phone {}", phone_id);
    let claims = request_claims(&req)?;
    let phone = usecase
        .execute(phone_id, phone_data.into_inner(), &claims)
        .await?;
    Ok(success(phone))
}

pub async fn delete_victim_phone(
    path: web::Path<(Uuid, Uuid)>,
    usecase: web::Data<DeleteVictimPhoneUseCase>,
    req: HttpRequest,
) -> Result<HttpResponse, AppError> {
    let (_victim_id, phone_id) = path.into_inner();
    info!("[Controller] Received request to delete phone {}", phone_id);
    let claims = request_claims(&req)?;
    let phone = usecase.execute(phone_id, &claims).await?;
    Ok(success(phone))
}

pub async fn add_address_to_victim(
    path: web::Path<Uuid>,
    address_data: web::Json<AddressData>,
    usecase: web::Data<CreateVictimAddressUseCase>,
    req: HttpRequest,
) -> Result<HttpResponse, AppError> {
    let victim_id = path.into_inner();
    info!(
        "[Controller] Received request to add address to victim {}",
        victim_id
    );
    let claims = request_claims(&req)?;
    let address = usecase
        .execute(victim_id, address_data.into_inner(), &claims)
        .await?;
    Ok(created(address))
}

pub async fn update_victim_address(
    path: web::Path<(Uuid, Uuid)>,
    address_data: web::Json<AddressData>,
    usecase: web::Data<UpdateVictimAddressUseCase>,
    req: HttpRequest,
) -> Result<HttpResponse, AppError> {
    let (_victim_id, address_id) = path.into_inner();
    info!(
        "[Controller] Received request to update address {}",
        address_id
    );
    let claims = request_claims(&req)?;
    let address = usecase
        .execute(address_id, address_data.into_inner(), &claims)
        .await?;
    Ok(success(address))
}

pub async fn delete_victim_address(
    path: web::Path<(Uuid, Uuid)>,
    usecase: web::Data<DeleteVictimAddressUseCase>,
    req: HttpRequest,
) -> Result<HttpResponse, AppError> {
    let (_victim_id, address_id) = path.into_inner();
    info!(
        "[Controller] Received request to delete address {}",
        address_id
    );
    let claims = request_claims(&req)?;
    let address = usecase.execute(address_id, &claims).await?;
    Ok(success(address))
}
