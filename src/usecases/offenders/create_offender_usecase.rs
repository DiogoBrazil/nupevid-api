use log::{error, info};

use crate::core::application_error::ApplicationError as AppError;
use crate::core::auth_context::AuthContext;
use crate::core::commands::offenders::CreateOffender;
use crate::core::contracts::repository::error::RepositoryError;
use crate::core::entities::auth::UserClaims;
use crate::core::entities::common::{
    is_security_agent, normalize_flag_from_list, resolve_city_id_from_addresses,
};
use crate::core::read_models::offenders::OffenderWithDetails;
use crate::core::value_objects::policies::Policy;
use crate::usecases::offenders::deps::OffenderUseCaseDependencies;
use crate::validators::{cpf_validator::validate_cpf, offender_validator::OffenderValidator};

pub struct CreateOffenderUseCase {
    deps: OffenderUseCaseDependencies,
}

impl CreateOffenderUseCase {
    pub fn new(deps: OffenderUseCaseDependencies) -> Self {
        Self { deps }
    }

    pub async fn execute(
        &self,
        offender: CreateOffender,
        claims: &UserClaims,
    ) -> Result<OffenderWithDetails, AppError> {
        let mut offender = offender;
        let city_id = resolve_city_id_from_addresses(&offender.addresses, offender.city_id)
            .map_err(|e| AppError::BadRequest(format!("Error adding offender: {}", e)))?;
        offender.city_id = Some(city_id);
        offender.is_public_security_agent = is_security_agent(&offender.security_force);
        let (has_psychiatric_issues, psychiatric_issues_type) =
            normalize_flag_from_list(&offender.psychiatric_issues_type);
        offender.has_psychiatric_issues = has_psychiatric_issues;
        offender.psychiatric_issues_type = psychiatric_issues_type;

        if let Some(cpf) = offender.cpf.as_ref() {
            let normalized = validate_cpf(cpf, "Error adding offender")?;
            offender.cpf = Some(normalized);
        }

        info!(
            "[CreateOffenderUseCase] Starting offender creation: {}",
            offender.full_name
        );

        let auth = AuthContext::load(&*self.deps.user_repository, claims).await?;

        auth.check_policy(&Policy::CreateOffenders, city_id)?;

        OffenderValidator::validate_required_fields(&offender.full_name, "Error adding offender")?;

        info!("[CreateOffenderUseCase] Saving offender to database");

        match self
            .deps
            .offender_write_repository
            .create_offender(offender)
            .await
        {
            Ok(offender_with_details) => {
                let offender_with_details =
                    OffenderWithDetails::from_write_result(offender_with_details);
                info!(
                    "[CreateOffenderUseCase] Offender created successfully with ID: {}",
                    offender_with_details.id
                );
                Ok(offender_with_details)
            }
            Err(RepositoryError::DuplicateEntry(msg)) => Err(AppError::Conflict(msg)),
            Err(RepositoryError::ReferencedEntityNotFound(msg)) => Err(AppError::BadRequest(msg)),
            Err(e) => {
                error!("[CreateOffenderUseCase] Failed to save offender: {:?}", e);
                Err(AppError::InternalServerError)
            }
        }
    }
}
