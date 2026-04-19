use log::{error, info};

use crate::core::application_error::ApplicationError as AppError;
use crate::core::auth_context::AuthContext;
use crate::core::commands::protective_measures::{CreateExtension, CreateProtectiveMeasure};
use crate::core::contracts::repository::error::RepositoryError;
use crate::core::entities::auth::UserClaims;
use crate::core::entities::protective_measures::{ProtectiveMeasure, ProtectiveMeasureStatus};
use crate::core::value_objects::policies::Policy;
use crate::usecases::protective_measures::deps::ProtectiveMeasureUseCaseDependencies;
use crate::validators::protective_measure_validator::ProtectiveMeasureValidator;

pub struct CreateProtectiveMeasureUseCase {
    deps: ProtectiveMeasureUseCaseDependencies,
}

fn map_reference_error_for_create(msg: String) -> AppError {
    let detail = match msg.as_str() {
        "Court district not found" => "court_district_id not found".to_string(),
        _ => msg,
    };

    AppError::BadRequest(format!("Error adding protective measure: {}", detail))
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

        let victim = match self
            .deps
            .victim_repository
            .get_victim_by_id(measure.victim_id)
            .await
        {
            Ok(v) => v,
            Err(RepositoryError::NotFound) => {
                return Err(AppError::NotFound(format!(
                    "Victim with id '{}' not found",
                    measure.victim_id
                )));
            }
            Err(e) => {
                error!(
                    "[CreateProtectiveMeasureUseCase] Error checking victim: {:?}",
                    e
                );
                return Err(AppError::InternalServerError);
            }
        };

        match self
            .deps
            .offender_repository
            .get_offender_by_id(measure.offender_id)
            .await
        {
            Ok(_) => {}
            Err(RepositoryError::NotFound) => {
                return Err(AppError::NotFound(format!(
                    "Offender with id '{}' not found",
                    measure.offender_id
                )));
            }
            Err(e) => {
                error!(
                    "[CreateProtectiveMeasureUseCase] Error checking offender: {:?}",
                    e
                );
                return Err(AppError::InternalServerError);
            }
        }

        let auth = AuthContext::load(&*self.deps.user_repository, claims).await?;
        auth.check_policy(&Policy::CreateProtectiveMeasures, victim.city_id)?;

        ProtectiveMeasureValidator::validate_required_fields(
            &measure.process_number,
            &measure.judicial_authority,
            &measure.violence_types,
            "Error adding protective measure",
        )?;

        if measure.status == ProtectiveMeasureStatus::Valid {
            let active_exists = self
                .deps
                .measure_read_repository
                .check_active_measure_exists_for_victim(measure.victim_id, None)
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
                    "[CreateProtectiveMeasureUseCase] Active measure already exists for victim: {}",
                    measure.victim_id
                );
                return Err(AppError::BadRequest(
                    "Error adding protective measure: victim already has an active protective measure".to_string()
                ));
            }
        }

        info!("[CreateProtectiveMeasureUseCase] Saving protective measure to database");

        if let Some(extensions) = measure.extensions.as_ref()
            && extensions.iter().any(|ext| ext.id.is_some())
        {
            return Err(AppError::BadRequest(
                "Error adding protective measure: extension id is not allowed on create"
                    .to_string(),
            ));
        }

        let extensions_to_create: Vec<CreateExtension> = measure
            .extensions
            .as_ref()
            .map(|extensions| {
                extensions
                    .iter()
                    .map(|extension| CreateExtension {
                        extension_number: extension.extension_number,
                        extension_date: extension.extension_date,
                        new_valid_until: extension.new_valid_until,
                        notes: extension.notes.clone(),
                    })
                    .collect()
            })
            .unwrap_or_default();

        let created = match self
            .deps
            .measure_write_repository
            .create_protective_measure_with_extensions(&measure, &extensions_to_create)
            .await
        {
            Ok(measure) => measure,
            Err(RepositoryError::ReferencedEntityNotFound(msg)) => {
                return Err(map_reference_error_for_create(msg));
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
