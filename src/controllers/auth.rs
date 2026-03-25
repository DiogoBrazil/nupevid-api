use actix_web::{HttpResponse, web};
use log::info;

use crate::core::commands::auth::Login;
use crate::services::auth::AuthService;
use crate::utils::controller_helpers::success;
use crate::utils::errors::AppError;

pub async fn login(
    data: web::Json<Login>,
    service: web::Data<AuthService>,
) -> Result<HttpResponse, AppError> {
    info!(
        "[Controller] Received request to login user with email: {}",
        data.email
    );
    let response = service.login(data.into_inner()).await?;
    Ok(success(response))
}
