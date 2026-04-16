use actix_web::{HttpRequest, HttpResponse, web};
use log::info;
use uuid::Uuid;

use crate::core::application_error::ApplicationError as AppError;
use crate::core::commands::protective_measures::{CreateExtension, UpdateExtension};
use crate::usecases::extensions::{
    CreateExtensionUseCase, DeleteExtensionByIdUseCase, GetExtensionByIdUseCase,
    GetExtensionsByMeasureUseCase, UpdateExtensionByIdUseCase,
};
use crate::utils::controller_helpers::{created, request_claims, success};

pub async fn create_extension(
    path: web::Path<Uuid>,
    body: web::Json<CreateExtension>,
    usecase: web::Data<CreateExtensionUseCase>,
    req: HttpRequest,
) -> Result<HttpResponse, AppError> {
    let protective_measure_id = path.into_inner();
    info!(
        "[Controller] Creating extension for protective measure: {}",
        protective_measure_id
    );
    let claims = request_claims(&req)?;
    let extension = usecase
        .execute(protective_measure_id, body.into_inner(), &claims)
        .await?;
    Ok(created(extension))
}

pub async fn get_extension_by_id(
    path: web::Path<(Uuid, Uuid)>,
    usecase: web::Data<GetExtensionByIdUseCase>,
    req: HttpRequest,
) -> Result<HttpResponse, AppError> {
    let (_protective_measure_id, extension_id) = path.into_inner();
    info!("[Controller] Getting extension with ID: {}", extension_id);
    let claims = request_claims(&req)?;
    let extension = usecase.execute(extension_id, &claims).await?;
    Ok(success(extension))
}

pub async fn get_extensions_by_measure(
    path: web::Path<Uuid>,
    usecase: web::Data<GetExtensionsByMeasureUseCase>,
    req: HttpRequest,
) -> Result<HttpResponse, AppError> {
    let protective_measure_id = path.into_inner();
    info!(
        "[Controller] Getting extensions for protective measure: {}",
        protective_measure_id
    );
    let claims = request_claims(&req)?;
    let extensions = usecase.execute(protective_measure_id, &claims).await?;
    Ok(success(extensions))
}

pub async fn update_extension_by_id(
    path: web::Path<(Uuid, Uuid)>,
    body: web::Json<UpdateExtension>,
    usecase: web::Data<UpdateExtensionByIdUseCase>,
    req: HttpRequest,
) -> Result<HttpResponse, AppError> {
    let (_protective_measure_id, extension_id) = path.into_inner();
    info!("[Controller] Updating extension with ID: {}", extension_id);
    let claims = request_claims(&req)?;
    let extension = usecase
        .execute(extension_id, body.into_inner(), &claims)
        .await?;
    Ok(success(extension))
}

pub async fn delete_extension_by_id(
    path: web::Path<(Uuid, Uuid)>,
    usecase: web::Data<DeleteExtensionByIdUseCase>,
    req: HttpRequest,
) -> Result<HttpResponse, AppError> {
    let (_protective_measure_id, extension_id) = path.into_inner();
    info!("[Controller] Deleting extension with ID: {}", extension_id);
    let claims = request_claims(&req)?;
    let extension = usecase.execute(extension_id, &claims).await?;
    Ok(success(extension))
}
