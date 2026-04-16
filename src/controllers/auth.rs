use actix_web::{HttpResponse, web};
use log::info;

use crate::core::commands::auth::Login;
use crate::usecases::auth::LoginUseCase;
use crate::utils::controller_helpers::success;
use crate::core::application_error::ApplicationError as AppError;

pub async fn login(
    data: web::Json<Login>,
    usecase: web::Data<LoginUseCase>,
) -> Result<HttpResponse, AppError> {
    info!(
        "[Controller] Received request to login user with email: {}",
        data.email
    );
    let response = usecase.execute(data.into_inner()).await?;
    Ok(success(response))
}
