use log::{error, info};
use uuid::Uuid;

use crate::core::application_error::ApplicationError as AppError;
use crate::core::auth_context::AuthContext;
use crate::core::commands::victims::UpdateVictim;
use crate::core::contracts::repository::error::RepositoryError;
use crate::core::entities::auth::UserClaims;
use crate::core::read_models::victims::VictimWithDetails;
use crate::core::value_objects::policies::Policy;
use crate::usecases::victims::deps::VictimUseCaseDependencies;
use crate::usecases::victims::normalization::normalize_victim_input;
use crate::validators::common::validate_person_name;

pub struct UpdateVictimUseCase {
    deps: VictimUseCaseDependencies,
}

impl UpdateVictimUseCase {
    pub fn new(deps: VictimUseCaseDependencies) -> Self {
        Self { deps }
    }

    pub async fn execute(
        &self,
        data: UpdateVictim,
        id: Uuid,
        claims: &UserClaims,
    ) -> Result<VictimWithDetails, AppError> {
        info!(
            "[UpdateVictimUseCase] Starting victim update for id: {}",
            id
        );

        let auth = AuthContext::load(&*self.deps.user_repository, claims).await?;
        let mut data = data;
        let city_id = normalize_victim_input(&mut data, "Error updating victim")?;

        auth.check_policy(&Policy::UpdateVictims, city_id)?;

        validate_person_name(&data.full_name, "Error updating victim")?;

        match self.deps.victim_read_repository.get_victim_by_id(id).await {
            Ok(existing_victim) => {
                auth.check_policy(&Policy::UpdateVictims, existing_victim.city_id)?;
            }
            Err(RepositoryError::NotFound) => {
                return Err(AppError::NotFound(format!(
                    "Victim with id '{}' not found",
                    id
                )));
            }
            Err(e) => {
                error!("[UpdateVictimUseCase] Error checking victim: {:?}", e);
                return Err(AppError::InternalServerError);
            }
        }

        info!("[UpdateVictimUseCase] Updating victim in database");

        match self
            .deps
            .victim_write_repository
            .update_victim_by_id(data, id)
            .await
        {
            Ok(victim_with_address) => {
                let victim_with_address = VictimWithDetails::from_write_result(victim_with_address);
                info!(
                    "[UpdateVictimUseCase] Victim updated successfully with ID: {}",
                    victim_with_address.id
                );
                Ok(victim_with_address)
            }
            Err(RepositoryError::NotFound) => {
                error!(
                    "[UpdateVictimUseCase] Victim with id {} not found for update",
                    id
                );
                Err(AppError::NotFound(format!(
                    "Victim with id '{}' not found",
                    id
                )))
            }
            Err(RepositoryError::DuplicateEntry(msg)) => Err(AppError::Conflict(msg)),
            Err(RepositoryError::ReferencedEntityNotFound(msg)) => Err(AppError::BadRequest(msg)),
            Err(e) => {
                error!(
                    "[UpdateVictimUseCase] Error updating victim in database: {:?}",
                    e
                );
                Err(AppError::InternalServerError)
            }
        }
    }
}
