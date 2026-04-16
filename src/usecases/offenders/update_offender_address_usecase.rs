use log::info;
use uuid::Uuid;

use crate::core::contracts::repository::error::RepositoryError;
use crate::core::entities::auth::UserClaims;
use crate::core::entities::common::AddressData;
use crate::core::read_models::offenders::OffenderAddressResponse;
use crate::core::value_objects::policies::Policy;
use crate::core::application_error::ApplicationError as AppError;
use crate::core::auth_context::AuthContext;
use crate::usecases::offenders::deps::OffenderUseCaseDependencies;
use crate::usecases::offenders::helpers::authorize_offender_access;

pub struct UpdateOffenderAddressUseCase {
    deps: OffenderUseCaseDependencies,
}

impl UpdateOffenderAddressUseCase {
    pub fn new(deps: OffenderUseCaseDependencies) -> Self {
        Self { deps }
    }

    pub async fn execute(
        &self,
        address_id: Uuid,
        address_data: AddressData,
        claims: &UserClaims,
    ) -> Result<OffenderAddressResponse, AppError> {
        info!("[UpdateOffenderAddressUseCase] Updating address: {}", address_id);

        let auth = AuthContext::load(&*self.deps.user_repository, claims).await?;

        match self
            .deps
            .offender_write_repository
            .get_address_by_id(address_id)
            .await
        {
            Ok(address) => {
                authorize_offender_access(
                    &auth,
                    &*self.deps.offender_read_repository,
                    address.offender_id,
                    &Policy::UpdateOffenders,
                )
                .await?;
            }
            Err(RepositoryError::NotFound) => {
                return Err(AppError::NotFound(format!(
                    "Address with id '{}' not found",
                    address_id
                )));
            }
            Err(_) => return Err(AppError::InternalServerError),
        }

        match self
            .deps
            .offender_write_repository
            .update_address_by_id(address_id, address_data)
            .await
        {
            Ok(address) => Ok(address.into()),
            Err(_) => Err(AppError::InternalServerError),
        }
    }
}
