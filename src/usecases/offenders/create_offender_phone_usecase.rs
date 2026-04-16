use log::info;
use uuid::Uuid;

use crate::core::entities::auth::UserClaims;
use crate::core::entities::common::PhoneData;
use crate::core::read_models::offenders::OffenderPhoneResponse;
use crate::core::value_objects::policies::Policy;
use crate::core::application_error::ApplicationError as AppError;
use crate::usecases::offenders::deps::OffenderUseCaseDependencies;
use crate::usecases::offenders::helpers::load_auth_and_check_offender;

pub struct CreateOffenderPhoneUseCase {
    deps: OffenderUseCaseDependencies,
}

impl CreateOffenderPhoneUseCase {
    pub fn new(deps: OffenderUseCaseDependencies) -> Self {
        Self { deps }
    }

    pub async fn execute(
        &self,
        offender_id: Uuid,
        phone_data: PhoneData,
        claims: &UserClaims,
    ) -> Result<OffenderPhoneResponse, AppError> {
        info!(
            "[CreateOffenderPhoneUseCase] Adding phone to offender: {}",
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
            .create_phone(offender_id, phone_data)
            .await
        {
            Ok(phone) => Ok(phone.into()),
            Err(_) => Err(AppError::InternalServerError),
        }
    }
}
