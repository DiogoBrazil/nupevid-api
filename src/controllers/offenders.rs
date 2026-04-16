use actix_web::{HttpRequest, HttpResponse, web};
use log::info;
use uuid::Uuid;

use crate::core::application_error::ApplicationError as AppError;
use crate::core::commands::offenders::{CreateOffender, UpdateOffender};
use crate::core::entities::offenders::{AddressData, PhoneData};
use crate::core::filters::offenders::OffenderSearchQuery;
use crate::usecases::offenders::{
    CreateOffenderAddressUseCase, CreateOffenderPhoneUseCase, CreateOffenderUseCase,
    DeleteOffenderAddressUseCase, DeleteOffenderPhoneUseCase, DeleteOffenderUseCase,
    GetAllOffendersUseCase, GetOffenderByIdUseCase, GetOffendersByVictimUseCase,
    SearchOffendersUseCase, UpdateOffenderAddressUseCase, UpdateOffenderPhoneUseCase,
    UpdateOffenderUseCase,
};
use crate::utils::controller_helpers::{
    created, paginated, request_claims, request_pagination, success,
};
use crate::utils::pagination::PaginationParams;

pub async fn create_offender(
    offender_data: web::Json<CreateOffender>,
    usecase: web::Data<CreateOffenderUseCase>,
    req: HttpRequest,
) -> Result<HttpResponse, AppError> {
    info!("[Controller] Received request to create offender");
    let claims = request_claims(&req)?;
    let offender = usecase.execute(offender_data.into_inner(), &claims).await?;
    Ok(created(offender))
}

pub async fn get_offender_by_id(
    path: web::Path<Uuid>,
    usecase: web::Data<GetOffenderByIdUseCase>,
    req: HttpRequest,
) -> Result<HttpResponse, AppError> {
    let offender_id = path.into_inner();
    info!(
        "[Controller] Received request to get offender with id: {}",
        offender_id
    );
    let claims = request_claims(&req)?;
    let offender = usecase.execute(offender_id, &claims).await?;
    Ok(success(offender))
}

pub async fn get_all_offenders(
    query: web::Query<PaginationParams>,
    usecase: web::Data<GetAllOffendersUseCase>,
    req: HttpRequest,
) -> Result<HttpResponse, AppError> {
    info!("[Controller] Received request to get all offenders");
    let claims = request_claims(&req)?;
    let pagination = request_pagination(&query.into_inner());
    let result = usecase.execute(pagination, &claims).await?;
    Ok(paginated(result))
}

pub async fn search_offenders(
    query: web::Query<OffenderSearchQuery>,
    usecase: web::Data<SearchOffendersUseCase>,
    req: HttpRequest,
) -> Result<HttpResponse, AppError> {
    let query = query.into_inner();
    info!("[Controller] Received request to search offenders");
    let claims = request_claims(&req)?;
    let offenders = usecase.execute(query.name, query.cpf, &claims).await?;
    Ok(success(offenders))
}

pub async fn get_offenders_by_victim_id(
    path: web::Path<Uuid>,
    usecase: web::Data<GetOffendersByVictimUseCase>,
    req: HttpRequest,
) -> Result<HttpResponse, AppError> {
    let victim_id = path.into_inner();
    info!(
        "[Controller] Received request to get offenders for victim: {}",
        victim_id
    );
    let claims = request_claims(&req)?;
    let offenders = usecase.execute(victim_id, &claims).await?;
    Ok(success(offenders))
}

pub async fn update_offender_by_id(
    path: web::Path<Uuid>,
    offender_data: web::Json<UpdateOffender>,
    usecase: web::Data<UpdateOffenderUseCase>,
    req: HttpRequest,
) -> Result<HttpResponse, AppError> {
    let offender_id = path.into_inner();
    info!(
        "[Controller] Received request to update offender with id: {}",
        offender_id
    );
    let claims = request_claims(&req)?;
    let offender = usecase
        .execute(offender_data.into_inner(), offender_id, &claims)
        .await?;
    Ok(success(offender))
}

pub async fn delete_offender_by_id(
    path: web::Path<Uuid>,
    usecase: web::Data<DeleteOffenderUseCase>,
    req: HttpRequest,
) -> Result<HttpResponse, AppError> {
    let offender_id = path.into_inner();
    info!(
        "[Controller] Received request to delete offender with id: {}",
        offender_id
    );
    let claims = request_claims(&req)?;
    let offender = usecase.execute(offender_id, &claims).await?;
    Ok(success(offender))
}

pub async fn add_phone_to_offender(
    path: web::Path<Uuid>,
    phone_data: web::Json<PhoneData>,
    usecase: web::Data<CreateOffenderPhoneUseCase>,
    req: HttpRequest,
) -> Result<HttpResponse, AppError> {
    let offender_id = path.into_inner();
    info!(
        "[Controller] Received request to add phone to offender {}",
        offender_id
    );
    let claims = request_claims(&req)?;
    let phone = usecase
        .execute(offender_id, phone_data.into_inner(), &claims)
        .await?;
    Ok(created(phone))
}

pub async fn update_offender_phone(
    path: web::Path<(Uuid, Uuid)>,
    phone_data: web::Json<PhoneData>,
    usecase: web::Data<UpdateOffenderPhoneUseCase>,
    req: HttpRequest,
) -> Result<HttpResponse, AppError> {
    let (_offender_id, phone_id) = path.into_inner();
    info!("[Controller] Received request to update phone {}", phone_id);
    let claims = request_claims(&req)?;
    let phone = usecase
        .execute(phone_id, phone_data.into_inner(), &claims)
        .await?;
    Ok(success(phone))
}

pub async fn delete_offender_phone(
    path: web::Path<(Uuid, Uuid)>,
    usecase: web::Data<DeleteOffenderPhoneUseCase>,
    req: HttpRequest,
) -> Result<HttpResponse, AppError> {
    let (_offender_id, phone_id) = path.into_inner();
    info!("[Controller] Received request to delete phone {}", phone_id);
    let claims = request_claims(&req)?;
    let phone = usecase.execute(phone_id, &claims).await?;
    Ok(success(phone))
}

pub async fn add_address_to_offender(
    path: web::Path<Uuid>,
    address_data: web::Json<AddressData>,
    usecase: web::Data<CreateOffenderAddressUseCase>,
    req: HttpRequest,
) -> Result<HttpResponse, AppError> {
    let offender_id = path.into_inner();
    info!(
        "[Controller] Received request to add address to offender {}",
        offender_id
    );
    let claims = request_claims(&req)?;
    let address = usecase
        .execute(offender_id, address_data.into_inner(), &claims)
        .await?;
    Ok(created(address))
}

pub async fn update_offender_address(
    path: web::Path<(Uuid, Uuid)>,
    address_data: web::Json<AddressData>,
    usecase: web::Data<UpdateOffenderAddressUseCase>,
    req: HttpRequest,
) -> Result<HttpResponse, AppError> {
    let (_offender_id, address_id) = path.into_inner();
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

pub async fn delete_offender_address(
    path: web::Path<(Uuid, Uuid)>,
    usecase: web::Data<DeleteOffenderAddressUseCase>,
    req: HttpRequest,
) -> Result<HttpResponse, AppError> {
    let (_offender_id, address_id) = path.into_inner();
    info!(
        "[Controller] Received request to delete address {}",
        address_id
    );
    let claims = request_claims(&req)?;
    let address = usecase.execute(address_id, &claims).await?;
    Ok(success(address))
}
