use actix_web::{HttpRequest, HttpResponse, web};
use log::info;
use uuid::Uuid;

use crate::core::application_error::ApplicationError as AppError;
use crate::core::commands::cities::{CreateCity, UpdateCity};
use crate::usecases::cities::{
    CreateCityUseCase, DeleteCityByIdUseCase, GetAllCitiesUseCase, GetCityByIdUseCase,
    UpdateCityByIdUseCase,
};
use crate::utils::controller_helpers::{
    created, paginated, request_claims, request_pagination, success,
};
use crate::utils::pagination::PaginationParams;

pub async fn create_city(
    city_data: web::Json<CreateCity>,
    usecase: web::Data<CreateCityUseCase>,
    req: HttpRequest,
) -> Result<HttpResponse, AppError> {
    info!("[Controller] Received request to create city");
    let claims = request_claims(&req)?;
    let city = usecase.execute(city_data.into_inner(), &claims).await?;
    Ok(created(city))
}

pub async fn get_city_by_id(
    path: web::Path<Uuid>,
    usecase: web::Data<GetCityByIdUseCase>,
    req: HttpRequest,
) -> Result<HttpResponse, AppError> {
    let city_id = path.into_inner();
    info!(
        "[Controller] Received request to get city with id: {}",
        city_id
    );
    let claims = request_claims(&req)?;
    let city = usecase.execute(city_id, &claims).await?;
    Ok(success(city))
}

pub async fn get_all_cities(
    query: web::Query<PaginationParams>,
    usecase: web::Data<GetAllCitiesUseCase>,
    req: HttpRequest,
) -> Result<HttpResponse, AppError> {
    info!("[Controller] Received request to get all cities");
    let claims = request_claims(&req)?;
    let pagination = request_pagination(&query.into_inner());
    let result = usecase.execute(pagination, &claims).await?;
    Ok(paginated(result))
}

pub async fn update_city_by_id(
    path: web::Path<Uuid>,
    city_data: web::Json<UpdateCity>,
    usecase: web::Data<UpdateCityByIdUseCase>,
    req: HttpRequest,
) -> Result<HttpResponse, AppError> {
    let city_id = path.into_inner();
    info!(
        "[Controller] Received request to update city with id: {}",
        city_id
    );
    let claims = request_claims(&req)?;
    let city = usecase
        .execute(city_data.into_inner(), city_id, &claims)
        .await?;
    Ok(success(city))
}

pub async fn delete_city_by_id(
    path: web::Path<Uuid>,
    usecase: web::Data<DeleteCityByIdUseCase>,
    req: HttpRequest,
) -> Result<HttpResponse, AppError> {
    let city_id = path.into_inner();
    info!(
        "[Controller] Received request to delete city with id: {}",
        city_id
    );
    let claims = request_claims(&req)?;
    let city = usecase.execute(city_id, &claims).await?;
    Ok(success(city))
}
