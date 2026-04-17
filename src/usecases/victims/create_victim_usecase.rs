use log::{error, info};

use crate::core::application_error::ApplicationError as AppError;
use crate::core::auth_context::AuthContext;
use crate::core::commands::victims::CreateVictim;
use crate::core::contracts::repository::error::RepositoryError;
use crate::core::entities::auth::UserClaims;
use crate::core::read_models::victims::VictimWithDetails;
use crate::core::value_objects::policies::Policy;
use crate::usecases::victims::deps::VictimUseCaseDependencies;
use crate::usecases::victims::normalization::normalize_victim_input;
use crate::validators::common::validate_person_name;

pub struct CreateVictimUseCase {
    deps: VictimUseCaseDependencies,
}

impl CreateVictimUseCase {
    pub fn new(deps: VictimUseCaseDependencies) -> Self {
        Self { deps }
    }

    pub async fn execute(
        &self,
        victim: CreateVictim,
        claims: &UserClaims,
    ) -> Result<VictimWithDetails, AppError> {
        let mut victim = victim;
        let city_id = normalize_victim_input(&mut victim, "Error adding victim")?;

        info!(
            "[CreateVictimUseCase] Starting victim creation: {}",
            victim.full_name
        );

        let auth = AuthContext::load(&*self.deps.user_repository, claims).await?;

        auth.check_policy(&Policy::CreateVictims, city_id)?;

        validate_person_name(&victim.full_name, "Error adding victim")?;

        info!("[CreateVictimUseCase] Saving victim to database");

        match self
            .deps
            .victim_write_repository
            .create_victim(victim)
            .await
        {
            Ok(victim_with_address) => {
                let victim_with_address = VictimWithDetails::from_write_result(victim_with_address);
                info!(
                    "[CreateVictimUseCase] Victim created successfully with ID: {}",
                    victim_with_address.id
                );
                Ok(victim_with_address)
            }
            Err(RepositoryError::DuplicateEntry(msg)) => Err(AppError::Conflict(msg)),
            Err(RepositoryError::ReferencedEntityNotFound(msg)) => Err(AppError::BadRequest(msg)),
            Err(e) => {
                error!("[CreateVictimUseCase] Failed to save victim: {:?}", e);
                Err(AppError::InternalServerError)
            }
        }
    }
}
