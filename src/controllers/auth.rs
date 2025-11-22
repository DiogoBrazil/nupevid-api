use actix_web::{web, HttpResponse};
use log::info;

use crate::core::entities::auth::Login;
use crate::services::auth::AuthService;
use crate::utils::errors::AppError;

pub async fn login(
    data: web::Json<Login>,
    service: web::Data<AuthService>,
) -> Result<HttpResponse, AppError> {
    info!(
        "[Controller] Received request to login user with email: {}",
        data.email
    );
    service.login(data.into_inner()).await
}
