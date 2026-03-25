use actix_web::{HttpRequest, HttpResponse, web};
use log::info;
use uuid::Uuid;

use crate::core::commands::cities::{CreateCity, UpdateCity};
use crate::services::cities::CityService;
use crate::utils::controller_helpers::{
    created, paginated, request_claims, request_pagination, success,
};
use crate::utils::errors::AppError;
use crate::utils::pagination::PaginationParams;

pub async fn create_city(
    city_data: web::Json<CreateCity>,
    city_service: web::Data<CityService>,
    req: HttpRequest,
) -> Result<HttpResponse, AppError> {
    info!("[Controller] Received request to create city");
    let claims = request_claims(&req)?;
    let city = city_service
        .create_city(city_data.into_inner(), &claims)
        .await?;
    Ok(created(city))
}

pub async fn get_city_by_id(
    path: web::Path<Uuid>,
    city_service: web::Data<CityService>,
    req: HttpRequest,
) -> Result<HttpResponse, AppError> {
    let city_id = path.into_inner();
    info!(
        "[Controller] Received request to get city with id: {}",
        city_id
    );
    let claims = request_claims(&req)?;
    let city = city_service.get_city_by_id(city_id, &claims).await?;
    Ok(success(city))
}

pub async fn get_all_cities(
    query: web::Query<PaginationParams>,
    city_service: web::Data<CityService>,
    req: HttpRequest,
) -> Result<HttpResponse, AppError> {
    info!("[Controller] Received request to get all cities");
    let claims = request_claims(&req)?;
    let pagination = request_pagination(&query.into_inner());
    let result = city_service.get_all_cities(pagination, &claims).await?;
    Ok(paginated(result))
}

pub async fn update_city_by_id(
    path: web::Path<Uuid>,
    city_data: web::Json<UpdateCity>,
    city_service: web::Data<CityService>,
    req: HttpRequest,
) -> Result<HttpResponse, AppError> {
    let city_id = path.into_inner();
    info!(
        "[Controller] Received request to update city with id: {}",
        city_id
    );
    let claims = request_claims(&req)?;
    let city = city_service
        .update_city_by_id(city_data.into_inner(), city_id, &claims)
        .await?;
    Ok(success(city))
}

pub async fn delete_city_by_id(
    path: web::Path<Uuid>,
    city_service: web::Data<CityService>,
    req: HttpRequest,
) -> Result<HttpResponse, AppError> {
    let city_id = path.into_inner();
    info!(
        "[Controller] Received request to delete city with id: {}",
        city_id
    );
    let claims = request_claims(&req)?;
    let city = city_service.delete_city_by_id(city_id, &claims).await?;
    Ok(success(city))
}
