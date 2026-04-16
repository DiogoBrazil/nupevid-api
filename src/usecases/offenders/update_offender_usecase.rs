use log::{error, info};
use uuid::Uuid;

use crate::core::application_error::ApplicationError as AppError;
use crate::core::auth_context::AuthContext;
use crate::core::commands::offenders::UpdateOffender;
use crate::core::contracts::repository::error::RepositoryError;
use crate::core::entities::auth::UserClaims;
use crate::core::entities::common::{
    is_security_agent, normalize_flag_from_list, resolve_city_id_from_addresses,
};
use crate::core::read_models::offenders::OffenderWithDetails;
use crate::core::value_objects::policies::Policy;
use crate::usecases::offenders::deps::OffenderUseCaseDependencies;
use crate::validators::{cpf_validator::validate_cpf, offender_validator::OffenderValidator};

pub struct UpdateOffenderUseCase {
    deps: OffenderUseCaseDependencies,
}

impl UpdateOffenderUseCase {
    pub fn new(deps: OffenderUseCaseDependencies) -> Self {
        Self { deps }
    }

    pub async fn execute(
        &self,
        data: UpdateOffender,
        id: Uuid,
        claims: &UserClaims,
    ) -> Result<OffenderWithDetails, AppError> {
        info!(
            "[UpdateOffenderUseCase] Starting offender update for id: {}",
            id
        );

        let auth = AuthContext::load(&*self.deps.user_repository, claims).await?;
        let mut data = data;
        let city_id = resolve_city_id_from_addresses(&data.addresses, data.city_id)
            .map_err(|e| AppError::BadRequest(format!("Error updating offender: {}", e)))?;
        data.city_id = Some(city_id);
        data.is_public_security_agent = is_security_agent(&data.security_force);
        let (has_psychiatric_issues, psychiatric_issues_type) =
            normalize_flag_from_list(&data.psychiatric_issues_type);
        data.has_psychiatric_issues = has_psychiatric_issues;
        data.psychiatric_issues_type = psychiatric_issues_type;

        if let Some(cpf) = data.cpf.as_ref() {
            let normalized = validate_cpf(cpf, "Error updating offender")?;
            data.cpf = Some(normalized);
        }

        auth.check_policy(&Policy::UpdateOffenders, city_id)?;

        OffenderValidator::validate_required_fields(&data.full_name, "Error updating offender")?;

        match self
            .deps
            .offender_read_repository
            .get_offender_by_id(id)
            .await
        {
            Ok(existing_offender) => {
                auth.check_policy(&Policy::UpdateOffenders, existing_offender.city_id)?;
            }
            Err(RepositoryError::NotFound) => {
                return Err(AppError::NotFound(format!(
                    "Offender with id '{}' not found",
                    id
                )));
            }
            Err(e) => {
                error!("[UpdateOffenderUseCase] Error checking offender: {:?}", e);
                return Err(AppError::InternalServerError);
            }
        }

        info!("[UpdateOffenderUseCase] Updating offender in database");

        match self
            .deps
            .offender_write_repository
            .update_offender_by_id(data, id)
            .await
        {
            Ok(offender_with_details) => {
                let offender_with_details =
                    OffenderWithDetails::from_write_result(offender_with_details);
                info!(
                    "[UpdateOffenderUseCase] Offender updated successfully with ID: {}",
                    offender_with_details.id
                );
                Ok(offender_with_details)
            }
            Err(RepositoryError::NotFound) => {
                error!(
                    "[UpdateOffenderUseCase] Offender with id {} not found for update",
                    id
                );
                Err(AppError::NotFound(format!(
                    "Offender with id '{}' not found",
                    id
                )))
            }
            Err(RepositoryError::DuplicateEntry(msg)) => Err(AppError::Conflict(msg)),
            Err(RepositoryError::ReferencedEntityNotFound(msg)) => Err(AppError::BadRequest(msg)),
            Err(e) => {
                error!(
                    "[UpdateOffenderUseCase] Error updating offender in database: {:?}",
                    e
                );
                Err(AppError::InternalServerError)
            }
        }
    }
}
