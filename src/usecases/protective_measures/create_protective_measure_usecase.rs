use chrono::Local;
use log::{error, info};

use crate::core::application_error::ApplicationError as AppError;
use crate::core::auth_context::AuthContext;
use crate::core::commands::protective_measures::CreateProtectiveMeasure;
use crate::core::contracts::repository::error::RepositoryError;
use crate::core::entities::auth::UserClaims;
use crate::core::entities::protective_measures::{ProtectiveMeasure, ProtectiveMeasureStatus};
use crate::core::value_objects::policies::Policy;
use crate::usecases::helpers_common::{get_offender_or_not_found, get_victim_or_not_found};
use crate::usecases::protective_measures::deps::ProtectiveMeasureUseCaseDependencies;
use crate::usecases::protective_measures::errors::map_reference_error;
use crate::validators::protective_measure_validator::ProtectiveMeasureValidator;

pub struct CreateProtectiveMeasureUseCase {
    deps: ProtectiveMeasureUseCaseDependencies,
}

impl CreateProtectiveMeasureUseCase {
    pub fn new(deps: ProtectiveMeasureUseCaseDependencies) -> Self {
        Self { deps }
    }

    pub async fn execute(
        &self,
        measure: CreateProtectiveMeasure,
        claims: &UserClaims,
    ) -> Result<ProtectiveMeasure, AppError> {
        info!(
            "[CreateProtectiveMeasureUseCase] Starting protective measure creation for victim: {}",
            measure.victim_id
        );

        let victim =
            get_victim_or_not_found(&*self.deps.victim_repository, measure.victim_id).await?;
        get_offender_or_not_found(&*self.deps.offender_repository, measure.offender_id).await?;

        let auth = AuthContext::load(&*self.deps.user_repository, claims).await?;
        auth.check_policy(&Policy::CreateProtectiveMeasures, victim.summary.city_id)?;

        ProtectiveMeasureValidator::validate_required_fields(
            &measure.process_number,
            &measure.judicial_authority,
            &measure.violence_types,
            "Error adding protective measure",
        )?;
        ProtectiveMeasureValidator::validate_issued_at_not_future(
            measure.issued_at,
            Local::now().date_naive(),
        )?;

        if measure.status == ProtectiveMeasureStatus::Valid {
            let active_exists = self
                .deps
                .measure_read_repository
                .check_active_measure_exists_for_victim(
                    measure.victim_id,
                    measure.offender_id,
                    None,
                )
                .await
                .map_err(|e| {
                    error!(
                        "[CreateProtectiveMeasureUseCase] Failed to check for active measure: {:?}",
                        e
                    );
                    AppError::InternalServerError
                })?;

            if active_exists {
                error!(
                    "[CreateProtectiveMeasureUseCase] Active measure already exists for victim: {} offender: {}",
                    measure.victim_id, measure.offender_id
                );
                return Err(AppError::Conflict(
                    "Victim and offender already have an active protective measure".to_string(),
                ));
            }
        }

        info!("[CreateProtectiveMeasureUseCase] Saving protective measure to database");

        let created = match self
            .deps
            .measure_write_repository
            .create_protective_measure(measure)
            .await
        {
            Ok(measure) => measure,
            Err(RepositoryError::ReferencedEntityNotFound(msg)) => {
                return Err(map_reference_error(msg, "Error adding protective measure"));
            }
            Err(RepositoryError::Conflict(msg)) | Err(RepositoryError::DuplicateEntry(msg)) => {
                return Err(AppError::Conflict(msg));
            }
            Err(e) => {
                error!(
                    "[CreateProtectiveMeasureUseCase] Failed to save protective measure: {:?}",
                    e
                );
                return Err(AppError::InternalServerError);
            }
        };

        info!(
            "[CreateProtectiveMeasureUseCase] Protective measure created successfully with ID: {}",
            created.id
        );
        Ok(created)
    }
}
