use actix_web::{HttpRequest, HttpResponse, web};
use log::info;

use crate::core::application_error::ApplicationError as AppError;
use crate::usecases::machine_information::GetMachineInformationUseCase;
use crate::utils::controller_helpers::{request_claims, success};

pub async fn get_machine_information(
    usecase: web::Data<GetMachineInformationUseCase>,
    req: HttpRequest,
) -> Result<HttpResponse, AppError> {
    info!("[Controller] Received request to get machine information");
    let _claims = request_claims(&req)?;
    let info = usecase.execute().await?;
    Ok(success(info))
}
