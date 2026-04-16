use log::{error, info};
use uuid::Uuid;

use crate::core::application_error::ApplicationError as AppError;
use crate::core::auth_context::AuthContext;
use crate::core::contracts::repository::error::RepositoryError;
use crate::core::entities::auth::UserClaims;
use crate::core::read_models::victims::VictimWithDetails;
use crate::core::value_objects::policies::Policy;
use crate::usecases::victims::deps::VictimUseCaseDependencies;

pub struct DeleteVictimUseCase {
    deps: VictimUseCaseDependencies,
}

impl DeleteVictimUseCase {
    pub fn new(deps: VictimUseCaseDependencies) -> Self {
        Self { deps }
    }

    pub async fn execute(
        &self,
        id: Uuid,
        claims: &UserClaims,
    ) -> Result<VictimWithDetails, AppError> {
        info!(
            "[DeleteVictimUseCase] Starting process to delete victim with id: {}",
            id
        );

        match self.deps.victim_read_repository.get_victim_by_id(id).await {
            Ok(victim) => {
                let auth = AuthContext::load(&*self.deps.user_repository, claims).await?;
                auth.check_policy(&Policy::DeleteVictims, victim.city_id)?;
            }
            Err(RepositoryError::NotFound) => {
                return Err(AppError::NotFound(format!(
                    "Victim with id '{}' not found",
                    id
                )));
            }
            Err(e) => {
                error!("[DeleteVictimUseCase] Error checking victim: {:?}", e);
                return Err(AppError::InternalServerError);
            }
        }

        match self
            .deps
            .victim_write_repository
            .delete_victim_by_id(id)
            .await
        {
            Ok(deleted_victim) => {
                let deleted_victim = VictimWithDetails::from_write_result(deleted_victim);
                info!(
                    "[DeleteVictimUseCase] Victim with id {} deleted successfully",
                    id
                );
                Ok(deleted_victim)
            }
            Err(RepositoryError::NotFound) => {
                info!(
                    "[DeleteVictimUseCase] Victim with id {} not found for deletion",
                    id
                );
                Err(AppError::NotFound(format!(
                    "Victim with id '{}' not found",
                    id
                )))
            }
            Err(e) => {
                error!("[DeleteVictimUseCase] Failed to delete victim: {:?}", e);
                Err(AppError::InternalServerError)
            }
        }
    }
}
