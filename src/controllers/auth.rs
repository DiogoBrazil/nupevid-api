use actix_web::{HttpRequest, HttpResponse, web};
use log::info;

use crate::core::application_error::ApplicationError as AppError;
use crate::core::commands::auth::{Login, LogoutRequest, RefreshTokenRequest};
use crate::core::entities::auth::ClientMetadata;
use crate::usecases::auth::{LoginUseCase, LogoutUseCase, RefreshTokenUseCase};
use crate::utils::controller_helpers::success;

fn extract_client_metadata(req: &HttpRequest) -> ClientMetadata {
    let user_agent = req
        .headers()
        .get("User-Agent")
        .and_then(|value| value.to_str().ok())
        .map(|value| value.to_string());

    let ip_address = req
        .connection_info()
        .realip_remote_addr()
        .map(|value| value.to_string());

    ClientMetadata {
        user_agent,
        ip_address,
    }
}

pub async fn login(
    data: web::Json<Login>,
    usecase: web::Data<LoginUseCase>,
    req: HttpRequest,
) -> Result<HttpResponse, AppError> {
    info!(
        "[Controller] Received request to login user with email: {}",
        data.email
    );
    let metadata = extract_client_metadata(&req);
    let response = usecase.execute(data.into_inner(), metadata).await?;
    Ok(success(response))
}

pub async fn refresh(
    data: web::Json<RefreshTokenRequest>,
    usecase: web::Data<RefreshTokenUseCase>,
    req: HttpRequest,
) -> Result<HttpResponse, AppError> {
    info!("[Controller] Received request to refresh access token");
    let metadata = extract_client_metadata(&req);
    let response = usecase
        .execute(data.into_inner().refresh_token, metadata)
        .await?;
    Ok(success(response))
}

pub async fn logout(
    data: web::Json<LogoutRequest>,
    usecase: web::Data<LogoutUseCase>,
) -> Result<HttpResponse, AppError> {
    info!("[Controller] Received request to logout");
    let response = usecase.execute(data.into_inner().refresh_token).await?;
    Ok(success(response))
}
