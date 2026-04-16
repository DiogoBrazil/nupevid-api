use log::info;
use uuid::Uuid;

use crate::core::application_error::ApplicationError as AppError;
use crate::core::auth_context::AuthContext;
use crate::core::contracts::repository::error::RepositoryError;
use crate::core::entities::auth::UserClaims;
use crate::core::read_models::victims::VictimAddressResponse;
use crate::core::value_objects::policies::Policy;
use crate::usecases::victims::deps::VictimUseCaseDependencies;
use crate::usecases::victims::helpers::authorize_victim_access;

pub struct DeleteVictimAddressUseCase {
    deps: VictimUseCaseDependencies,
}

impl DeleteVictimAddressUseCase {
    pub fn new(deps: VictimUseCaseDependencies) -> Self {
        Self { deps }
    }

    pub async fn execute(
        &self,
        address_id: Uuid,
        claims: &UserClaims,
    ) -> Result<VictimAddressResponse, AppError> {
        info!(
            "[DeleteVictimAddressUseCase] Deleting address: {}",
            address_id
        );

        let auth = AuthContext::load(&*self.deps.user_repository, claims).await?;

        match self
            .deps
            .victim_write_repository
            .get_address_by_id(address_id)
            .await
        {
            Ok(address) => {
                authorize_victim_access(
                    &auth,
                    &*self.deps.victim_read_repository,
                    address.victim_id,
                    &Policy::UpdateVictims,
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
            .victim_write_repository
            .delete_address_by_id(address_id)
            .await
        {
            Ok(address) => Ok(address.into()),
            Err(_) => Err(AppError::InternalServerError),
        }
    }
}
