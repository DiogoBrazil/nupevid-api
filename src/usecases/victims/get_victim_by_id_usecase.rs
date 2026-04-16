use log::{error, info};
use uuid::Uuid;

use crate::core::application_error::ApplicationError as AppError;
use crate::core::auth_context::AuthContext;
use crate::core::contracts::repository::error::RepositoryError;
use crate::core::entities::auth::UserClaims;
use crate::core::read_models::victims::VictimWithDetails;
use crate::core::value_objects::policies::Policy;
use crate::usecases::victims::deps::VictimUseCaseDependencies;

pub struct GetVictimByIdUseCase {
    deps: VictimUseCaseDependencies,
}

impl GetVictimByIdUseCase {
    pub fn new(deps: VictimUseCaseDependencies) -> Self {
        Self { deps }
    }

    pub async fn execute(
        &self,
        id: Uuid,
        claims: &UserClaims,
    ) -> Result<VictimWithDetails, AppError> {
        info!(
            "[GetVictimByIdUseCase] Starting find victim by id process for id: {}",
            id
        );

        match self.deps.victim_read_repository.get_victim_by_id(id).await {
            Ok(victim_with_address) => {
                let auth = AuthContext::load(&*self.deps.user_repository, claims).await?;
                auth.check_policy(&Policy::ReadVictims, victim_with_address.city_id)?;

                info!(
                    "[GetVictimByIdUseCase] Victim with id {} found successfully",
                    id
                );
                Ok(victim_with_address)
            }
            Err(RepositoryError::NotFound) => {
                info!("[GetVictimByIdUseCase] Victim with id {} not found", id);
                Err(AppError::NotFound(format!(
                    "Victim with id '{}' not found",
                    id
                )))
            }
            Err(e) => {
                error!(
                    "[GetVictimByIdUseCase] Database error while finding victim: {:?}",
                    e
                );
                Err(AppError::InternalServerError)
            }
        }
    }
}
