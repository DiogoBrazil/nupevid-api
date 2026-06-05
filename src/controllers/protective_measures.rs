use actix_web::{HttpRequest, HttpResponse, web};
use log::info;
use uuid::Uuid;

use crate::core::application_error::ApplicationError as AppError;
use crate::core::commands::protective_measures::{
    CreateProtectiveMeasure, UpdateProtectiveMeasure,
};
use crate::core::filters::common::IncludeRelatedQuery;
use crate::core::filters::protective_measures::ProtectiveMeasuresQuery;
use crate::presenters::protective_measures::ProtectiveMeasurePresenter;
use crate::usecases::protective_measures::{
    CreateProtectiveMeasureUseCase, DeleteProtectiveMeasureUseCase,
    GetAllProtectiveMeasuresUseCase, GetMeasuresByVictimUseCase, GetProtectiveMeasureByIdUseCase,
    UpdateProtectiveMeasureUseCase,
};
use crate::utils::controller_helpers::{
    created, include_related, paginated, request_claims, request_pagination_from_parts, success,
};
use crate::utils::pagination::Pagination;

pub async fn create_protective_measure(
    measure_data: web::Json<CreateProtectiveMeasure>,
    query: web::Query<IncludeRelatedQuery>,
    usecase: web::Data<CreateProtectiveMeasureUseCase>,
    presenter: web::Data<ProtectiveMeasurePresenter>,
    req: HttpRequest,
) -> Result<HttpResponse, AppError> {
    info!("[Controller] Received request to create protective measure");
    let claims = request_claims(&req)?;
    let measure = usecase.execute(measure_data.into_inner(), &claims).await?;
    let response = presenter
        .build_response(measure, include_related(&query))
        .await?;
    Ok(created(response))
}

pub async fn get_protective_measure_by_id(
    path: web::Path<Uuid>,
    query: web::Query<IncludeRelatedQuery>,
    usecase: web::Data<GetProtectiveMeasureByIdUseCase>,
    presenter: web::Data<ProtectiveMeasurePresenter>,
    req: HttpRequest,
) -> Result<HttpResponse, AppError> {
    let measure_id = path.into_inner();
    info!(
        "[Controller] Received request to get protective measure with id: {}",
        measure_id
    );
    let claims = request_claims(&req)?;
    let measure = usecase.execute(measure_id, &claims).await?;
    let response = presenter
        .build_response(measure, include_related(&query))
        .await?;
    Ok(success(response))
}

pub async fn get_all_protective_measures(
    query: web::Query<ProtectiveMeasuresQuery>,
    usecase: web::Data<GetAllProtectiveMeasuresUseCase>,
    presenter: web::Data<ProtectiveMeasurePresenter>,
    req: HttpRequest,
) -> Result<HttpResponse, AppError> {
    info!("[Controller] Received request to get all protective measures");
    let query = query.into_inner();
    let include = query.include_related_entities.unwrap_or(false);
    let claims = request_claims(&req)?;
    let pagination = request_pagination_from_parts(query.page, query.page_size);
    let result = usecase
        .execute(pagination, query.victim_id, query.offender_id, &claims)
        .await?;
    let response = presenter
        .build_responses(
            result.items,
            include,
            Pagination {
                page: result.page,
                page_size: result.page_size,
                offset: 0,
            },
            result.total_items,
        )
        .await?;
    Ok(paginated(response))
}

pub async fn get_protective_measures_by_victim(
    path: web::Path<Uuid>,
    query: web::Query<IncludeRelatedQuery>,
    usecase: web::Data<GetMeasuresByVictimUseCase>,
    presenter: web::Data<ProtectiveMeasurePresenter>,
    req: HttpRequest,
) -> Result<HttpResponse, AppError> {
    let victim_id = path.into_inner();
    info!(
        "[Controller] Received request to get protective measures for victim: {}",
        victim_id
    );
    let claims = request_claims(&req)?;
    let measures = usecase.execute(victim_id, &claims).await?;
    let mut responses = Vec::with_capacity(measures.len());
    for measure in measures {
        responses.push(
            presenter
                .build_response(measure, include_related(&query))
                .await?,
        );
    }
    Ok(success(responses))
}

pub async fn update_protective_measure_by_id(
    path: web::Path<Uuid>,
    measure_data: web::Json<UpdateProtectiveMeasure>,
    query: web::Query<IncludeRelatedQuery>,
    usecase: web::Data<UpdateProtectiveMeasureUseCase>,
    presenter: web::Data<ProtectiveMeasurePresenter>,
    req: HttpRequest,
) -> Result<HttpResponse, AppError> {
    let measure_id = path.into_inner();
    info!(
        "[Controller] Received request to update protective measure with id: {}",
        measure_id
    );
    let claims = request_claims(&req)?;
    let measure = usecase
        .execute(measure_data.into_inner(), measure_id, &claims)
        .await?;
    let response = presenter
        .build_response(measure, include_related(&query))
        .await?;
    Ok(success(response))
}

pub async fn delete_protective_measure_by_id(
    path: web::Path<Uuid>,
    usecase: web::Data<DeleteProtectiveMeasureUseCase>,
    req: HttpRequest,
) -> Result<HttpResponse, AppError> {
    let measure_id = path.into_inner();
    info!(
        "[Controller] Received request to delete protective measure with id: {}",
        measure_id
    );
    let claims = request_claims(&req)?;
    let measure = usecase.execute(measure_id, &claims).await?;
    Ok(success(measure))
}
