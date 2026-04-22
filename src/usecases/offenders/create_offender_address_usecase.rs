use log::info;
use uuid::Uuid;

use crate::core::application_error::ApplicationError as AppError;
use crate::core::entities::auth::UserClaims;
use crate::core::entities::common::AddressData;
use crate::core::read_models::offenders::OffenderAddressResponse;
use crate::core::value_objects::policies::Policy;
use crate::usecases::offenders::deps::OffenderUseCaseDependencies;
use crate::usecases::offenders::helpers::load_auth_and_check_offender;

pub struct CreateOffenderAddressUseCase {
    deps: OffenderUseCaseDependencies,
}

impl CreateOffenderAddressUseCase {
    pub fn new(deps: OffenderUseCaseDependencies) -> Self {
        Self { deps }
    }

    pub async fn execute(
        &self,
        offender_id: Uuid,
        address_data: AddressData,
        claims: &UserClaims,
    ) -> Result<OffenderAddressResponse, AppError> {
        info!(
            "[CreateOffenderAddressUseCase] Adding address to offender: {}",
            offender_id
        );

        load_auth_and_check_offender(
            &*self.deps.user_repository,
            claims,
            &*self.deps.offender_read_repository,
            offender_id,
            &Policy::UpdateOffenders,
        )
        .await?;

        match self
            .deps
            .offender_write_repository
            .create_address(offender_id, address_data)
            .await
        {
            Ok(address) => Ok(address.into()),
            Err(_) => Err(AppError::InternalServerError),
        }
    }
}
