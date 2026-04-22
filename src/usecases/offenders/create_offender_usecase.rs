use log::{error, info};

use crate::core::application_error::ApplicationError as AppError;
use crate::core::auth_context::AuthContext;
use crate::core::commands::offenders::CreateOffender;
use crate::core::contracts::repository::error::RepositoryError;
use crate::core::entities::auth::UserClaims;
use crate::core::read_models::offenders::OffenderWithDetails;
use crate::core::value_objects::policies::Policy;
use crate::usecases::offenders::deps::OffenderUseCaseDependencies;
use crate::usecases::offenders::normalization::normalize_offender_input;
use crate::validators::common::validate_person_name;

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
        let city_id = normalize_offender_input(&mut offender, "Error adding offender")?;

        info!(
            "[CreateOffenderUseCase] Starting offender creation: {}",
            offender.full_name
        );

        let auth = AuthContext::load(&*self.deps.user_repository, claims).await?;

        auth.check_policy(&Policy::CreateOffenders, city_id)?;

        validate_person_name(&offender.full_name, "Error adding offender")?;

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
