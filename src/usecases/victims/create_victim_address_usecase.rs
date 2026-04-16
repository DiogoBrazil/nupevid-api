use log::info;
use uuid::Uuid;

use crate::core::application_error::ApplicationError as AppError;
use crate::core::entities::auth::UserClaims;
use crate::core::entities::common::AddressData;
use crate::core::read_models::victims::VictimAddressResponse;
use crate::core::value_objects::policies::Policy;
use crate::usecases::victims::deps::VictimUseCaseDependencies;
use crate::usecases::victims::helpers::load_auth_and_check_victim;

pub struct CreateVictimAddressUseCase {
    deps: VictimUseCaseDependencies,
}

impl CreateVictimAddressUseCase {
    pub fn new(deps: VictimUseCaseDependencies) -> Self {
        Self { deps }
    }

    pub async fn execute(
        &self,
        victim_id: Uuid,
        address_data: AddressData,
        claims: &UserClaims,
    ) -> Result<VictimAddressResponse, AppError> {
        info!(
            "[CreateVictimAddressUseCase] Adding address to victim: {}",
            victim_id
        );

        load_auth_and_check_victim(
            &*self.deps.user_repository,
            claims,
            &*self.deps.victim_read_repository,
            victim_id,
            &Policy::UpdateVictims,
        )
        .await?;

        match self
            .deps
            .victim_write_repository
            .create_address(victim_id, address_data)
            .await
        {
            Ok(address) => Ok(address.into()),
            Err(_) => Err(AppError::InternalServerError),
        }
    }
}
