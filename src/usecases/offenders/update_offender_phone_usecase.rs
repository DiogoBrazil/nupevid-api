use log::info;
use uuid::Uuid;

use crate::core::application_error::ApplicationError as AppError;
use crate::core::auth_context::AuthContext;
use crate::core::contracts::repository::error::RepositoryError;
use crate::core::entities::auth::UserClaims;
use crate::core::entities::common::PhoneData;
use crate::core::read_models::offenders::OffenderPhoneResponse;
use crate::core::value_objects::policies::Policy;
use crate::usecases::offenders::deps::OffenderUseCaseDependencies;
use crate::usecases::offenders::helpers::authorize_offender_access;

pub struct UpdateOffenderPhoneUseCase {
    deps: OffenderUseCaseDependencies,
}

impl UpdateOffenderPhoneUseCase {
    pub fn new(deps: OffenderUseCaseDependencies) -> Self {
        Self { deps }
    }

    pub async fn execute(
        &self,
        phone_id: Uuid,
        phone_data: PhoneData,
        claims: &UserClaims,
    ) -> Result<OffenderPhoneResponse, AppError> {
        info!("[UpdateOffenderPhoneUseCase] Updating phone: {}", phone_id);

        let auth = AuthContext::load(&*self.deps.user_repository, claims).await?;

        match self
            .deps
            .offender_write_repository
            .get_phone_by_id(phone_id)
            .await
        {
            Ok(phone) => {
                authorize_offender_access(
                    &auth,
                    &*self.deps.offender_read_repository,
                    phone.offender_id,
                    &Policy::UpdateOffenders,
                )
                .await?;
            }
            Err(RepositoryError::NotFound) => {
                return Err(AppError::NotFound(format!(
                    "Phone with id '{}' not found",
                    phone_id
                )));
            }
            Err(_) => return Err(AppError::InternalServerError),
        }

        match self
            .deps
            .offender_write_repository
            .update_phone_by_id(phone_id, phone_data)
            .await
        {
            Ok(phone) => Ok(phone.into()),
            Err(_) => Err(AppError::InternalServerError),
        }
    }
}
