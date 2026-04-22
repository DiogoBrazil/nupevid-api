use actix_web::{HttpRequest, HttpResponse, web};
use log::info;
use uuid::Uuid;

use crate::core::entities::protective_measures::{
    CreateProtectiveMeasure, UpdateProtectiveMeasure,
};
use crate::services::protective_measures::ProtectiveMeasureService;
use crate::utils::errors::AppError;
use crate::utils::pagination::PaginationParams;
use serde::Deserialize;

#[derive(Deserialize)]
pub struct IncludeComplementQuery {
    pub include_complement_for_entities: Option<bool>,
}

#[derive(Deserialize)]
pub struct ProtectiveMeasuresQuery {
    pub page: Option<i64>,
    pub page_size: Option<i64>,
    pub include_complement_for_entities: Option<bool>,
}

pub async fn create_protective_measure(
    measure_data: web::Json<CreateProtectiveMeasure>,
    query: web::Query<IncludeComplementQuery>,
    service: web::Data<ProtectiveMeasureService>,
    req: HttpRequest,
) -> Result<HttpResponse, AppError> {
    info!("[Controller] Received request to create protective measure");
    service
        .create_protective_measure(
            measure_data.into_inner(),
            req,
            query.include_complement_for_entities.unwrap_or(false),
        )
        .await
}

pub async fn get_protective_measure_by_id(
    path: web::Path<Uuid>,
    query: web::Query<IncludeComplementQuery>,
    service: web::Data<ProtectiveMeasureService>,
    req: HttpRequest,
) -> Result<HttpResponse, AppError> {
    let measure_id = path.into_inner();
    info!(
        "[Controller] Received request to get protective measure with id: {}",
        measure_id
    );
    service
        .get_protective_measure_by_id(
            measure_id,
            req,
            query.include_complement_for_entities.unwrap_or(false),
        )
        .await
}

pub async fn get_all_protective_measures(
    query: web::Query<ProtectiveMeasuresQuery>,
    service: web::Data<ProtectiveMeasureService>,
    req: HttpRequest,
) -> Result<HttpResponse, AppError> {
    info!("[Controller] Received request to get all protective measures");
    let query = query.into_inner();
    let pagination = PaginationParams {
        page: query.page,
        page_size: query.page_size,
    };
    service
        .get_all_protective_measures(
            pagination,
            req,
            query.include_complement_for_entities.unwrap_or(false),
        )
        .await
}

pub async fn get_protective_measures_by_victim(
    path: web::Path<Uuid>,
    query: web::Query<IncludeComplementQuery>,
    service: web::Data<ProtectiveMeasureService>,
    req: HttpRequest,
) -> Result<HttpResponse, AppError> {
    let victim_id = path.into_inner();
    info!(
        "[Controller] Received request to get protective measures for victim: {}",
        victim_id
    );
    service
        .get_protective_measures_by_victim(
            victim_id,
            req,
            query.include_complement_for_entities.unwrap_or(false),
        )
        .await
}

pub async fn update_protective_measure_by_id(
    path: web::Path<Uuid>,
    measure_data: web::Json<UpdateProtectiveMeasure>,
    service: web::Data<ProtectiveMeasureService>,
    req: HttpRequest,
) -> Result<HttpResponse, AppError> {
    let measure_id = path.into_inner();
    info!(
        "[Controller] Received request to update protective measure with id: {}",
        measure_id
    );
    service
        .update_protective_measure_by_id(measure_data.into_inner(), measure_id, req)
        .await
}

pub async fn delete_protective_measure_by_id(
    path: web::Path<Uuid>,
    service: web::Data<ProtectiveMeasureService>,
    req: HttpRequest,
) -> Result<HttpResponse, AppError> {
    let measure_id = path.into_inner();
    info!(
        "[Controller] Received request to delete protective measure with id: {}",
        measure_id
    );
    service
        .delete_protective_measure_by_id(measure_id, req)
        .await
}
