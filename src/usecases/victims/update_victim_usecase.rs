use log::{error, info};
use uuid::Uuid;

use crate::core::commands::victims::UpdateVictim;
use crate::core::contracts::repository::error::RepositoryError;
use crate::core::entities::auth::UserClaims;
use crate::core::entities::common::{normalize_flag_from_list, resolve_city_id_from_addresses};
use crate::core::read_models::victims::VictimWithDetails;
use crate::core::value_objects::policies::Policy;
use crate::core::application_error::ApplicationError as AppError;
use crate::core::auth_context::AuthContext;
use crate::usecases::victims::deps::VictimUseCaseDependencies;
use crate::validators::{cpf_validator::validate_cpf, victim_validator::VictimValidator};

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
        info!("[UpdateVictimUseCase] Starting victim update for id: {}", id);

        let auth = AuthContext::load(&*self.deps.user_repository, claims).await?;
        let mut data = data;
        let city_id = resolve_city_id_from_addresses(&data.addresses, data.city_id)
            .map_err(|e| AppError::BadRequest(format!("Error updating victim: {}", e)))?;
        data.city_id = Some(city_id);

        let (has_special_needs, special_needs_type) =
            normalize_flag_from_list(&data.special_needs_type);
        data.has_special_needs = has_special_needs;
        data.special_needs_type = special_needs_type;

        let (has_psychiatric_issues, psychiatric_issues_type) =
            normalize_flag_from_list(&data.psychiatric_issues_type);
        data.has_psychiatric_issues = has_psychiatric_issues;
        data.psychiatric_issues_type = psychiatric_issues_type;
        data.has_children = data.children_count.is_some();

        if let Some(cpf) = data.cpf.as_ref() {
            let normalized = validate_cpf(cpf, "Error updating victim")?;
            data.cpf = Some(normalized);
        }

        auth.check_policy(&Policy::UpdateVictims, city_id)?;

        VictimValidator::validate_required_fields(&data.full_name, "Error updating victim")?;

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
                error!("[UpdateVictimUseCase] Victim with id {} not found for update", id);
                Err(AppError::NotFound(format!(
                    "Victim with id '{}' not found",
                    id
                )))
            }
            Err(RepositoryError::DuplicateEntry(msg)) => Err(AppError::Conflict(msg)),
            Err(RepositoryError::ReferencedEntityNotFound(msg)) => {
                Err(AppError::BadRequest(msg))
            }
            Err(e) => {
                error!("[UpdateVictimUseCase] Error updating victim in database: {:?}", e);
                Err(AppError::InternalServerError)
            }
        }
    }
}
