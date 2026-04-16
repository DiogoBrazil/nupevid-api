use log::{error, info};

use crate::core::application_error::ApplicationError as AppError;
use crate::core::auth_context::AuthContext;
use crate::core::commands::victims::CreateVictim;
use crate::core::contracts::repository::error::RepositoryError;
use crate::core::entities::auth::UserClaims;
use crate::core::entities::common::{normalize_flag_from_list, resolve_city_id_from_addresses};
use crate::core::read_models::victims::VictimWithDetails;
use crate::core::value_objects::policies::Policy;
use crate::usecases::victims::deps::VictimUseCaseDependencies;
use crate::validators::{cpf_validator::validate_cpf, victim_validator::VictimValidator};

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
        let city_id = resolve_city_id_from_addresses(&victim.addresses, victim.city_id)
            .map_err(|e| AppError::BadRequest(format!("Error adding victim: {}", e)))?;
        victim.city_id = Some(city_id);

        let (has_special_needs, special_needs_type) =
            normalize_flag_from_list(&victim.special_needs_type);
        victim.has_special_needs = has_special_needs;
        victim.special_needs_type = special_needs_type;

        let (has_psychiatric_issues, psychiatric_issues_type) =
            normalize_flag_from_list(&victim.psychiatric_issues_type);
        victim.has_psychiatric_issues = has_psychiatric_issues;
        victim.psychiatric_issues_type = psychiatric_issues_type;
        victim.has_children = victim.children_count.is_some();

        if let Some(cpf) = victim.cpf.as_ref() {
            let normalized = validate_cpf(cpf, "Error adding victim")?;
            victim.cpf = Some(normalized);
        }

        info!(
            "[CreateVictimUseCase] Starting victim creation: {}",
            victim.full_name
        );

        let auth = AuthContext::load(&*self.deps.user_repository, claims).await?;

        auth.check_policy(&Policy::CreateVictims, city_id)?;

        VictimValidator::validate_required_fields(&victim.full_name, "Error adding victim")?;

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
