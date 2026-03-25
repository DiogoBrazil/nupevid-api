use actix_web::{HttpRequest, HttpResponse, web};
use log::info;
use uuid::Uuid;

use crate::core::commands::protective_measures::{
    CreateProtectiveMeasure, UpdateProtectiveMeasure,
};
use crate::core::queries::common::IncludeComplementQuery;
use crate::core::queries::protective_measures::ProtectiveMeasuresQuery;
use crate::services::protective_measures::ProtectiveMeasureService;
use crate::utils::controller_helpers::{
    created, include_complement, paginated, request_claims, request_pagination_from_parts, success,
};
use crate::utils::errors::AppError;

pub async fn create_protective_measure(
    measure_data: web::Json<CreateProtectiveMeasure>,
    query: web::Query<IncludeComplementQuery>,
    service: web::Data<ProtectiveMeasureService>,
    req: HttpRequest,
) -> Result<HttpResponse, AppError> {
    info!("[Controller] Received request to create protective measure");
    let claims = request_claims(&req)?;
    let measure = service
        .create_protective_measure(
            measure_data.into_inner(),
            &claims,
            include_complement(&query),
        )
        .await?;
    Ok(created(measure))
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
    let claims = request_claims(&req)?;
    let measure = service
        .get_protective_measure_by_id(measure_id, &claims, include_complement(&query))
        .await?;
    Ok(success(measure))
}

pub async fn get_all_protective_measures(
    query: web::Query<ProtectiveMeasuresQuery>,
    service: web::Data<ProtectiveMeasureService>,
    req: HttpRequest,
) -> Result<HttpResponse, AppError> {
    info!("[Controller] Received request to get all protective measures");
    let query = query.into_inner();
    let claims = request_claims(&req)?;
    let pagination = request_pagination_from_parts(query.page, query.page_size);
    let result = service
        .get_all_protective_measures(
            pagination,
            &claims,
            query.include_complement_for_entities.unwrap_or(false),
        )
        .await?;
    Ok(paginated(result))
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
    let claims = request_claims(&req)?;
    let measures = service
        .get_protective_measures_by_victim(victim_id, &claims, include_complement(&query))
        .await?;
    Ok(success(measures))
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
    let claims = request_claims(&req)?;
    let measure = service
        .update_protective_measure_by_id(measure_data.into_inner(), measure_id, &claims)
        .await?;
    Ok(success(measure))
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
    let claims = request_claims(&req)?;
    let measure = service
        .delete_protective_measure_by_id(measure_id, &claims)
        .await?;
    Ok(success(measure))
}
