use log::{error, info};
use uuid::Uuid;

use crate::core::application_error::ApplicationError as AppError;
use crate::core::auth_context::AuthContext;
use crate::core::commands::offenders::UpdateOffender;
use crate::core::contracts::repository::error::RepositoryError;
use crate::core::entities::auth::UserClaims;
use crate::core::read_models::offenders::OffenderWithDetails;
use crate::core::value_objects::policies::Policy;
use crate::usecases::offenders::deps::OffenderUseCaseDependencies;
use crate::usecases::offenders::normalization::normalize_offender_input;
use crate::validators::common::validate_person_name;

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
        let city_id = normalize_offender_input(&mut data, "Error updating offender")?;

        auth.check_policy(&Policy::UpdateOffenders, city_id)?;

        validate_person_name(&data.full_name, "Error updating offender")?;

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
