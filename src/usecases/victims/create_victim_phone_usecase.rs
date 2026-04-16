use log::info;
use uuid::Uuid;

use crate::core::application_error::ApplicationError as AppError;
use crate::core::entities::auth::UserClaims;
use crate::core::entities::common::PhoneData;
use crate::core::read_models::victims::VictimPhoneResponse;
use crate::core::value_objects::policies::Policy;
use crate::usecases::victims::deps::VictimUseCaseDependencies;
use crate::usecases::victims::helpers::load_auth_and_check_victim;

pub struct CreateVictimPhoneUseCase {
    deps: VictimUseCaseDependencies,
}

impl CreateVictimPhoneUseCase {
    pub fn new(deps: VictimUseCaseDependencies) -> Self {
        Self { deps }
    }

    pub async fn execute(
        &self,
        victim_id: Uuid,
        phone_data: PhoneData,
        claims: &UserClaims,
    ) -> Result<VictimPhoneResponse, AppError> {
        info!(
            "[CreateVictimPhoneUseCase] Adding phone to victim: {}",
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
            .create_phone(victim_id, phone_data)
            .await
        {
            Ok(phone) => Ok(phone.into()),
            Err(_) => Err(AppError::InternalServerError),
        }
    }
}
