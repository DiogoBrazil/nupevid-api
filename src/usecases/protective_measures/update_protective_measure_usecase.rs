use log::{error, info};
use uuid::Uuid;

use crate::core::application_error::ApplicationError as AppError;
use crate::core::auth_context::AuthContext;
use crate::core::commands::protective_measures::{
    CreateExtension, UpdateExtension, UpdateProtectiveMeasure,
};
use crate::core::contracts::repository::error::RepositoryError;
use crate::core::entities::auth::UserClaims;
use crate::core::entities::protective_measures::{ProtectiveMeasure, ProtectiveMeasureStatus};
use crate::core::value_objects::policies::Policy;
use crate::usecases::protective_measures::deps::ProtectiveMeasureUseCaseDependencies;
use crate::validators::protective_measure_validator::ProtectiveMeasureValidator;

pub struct UpdateProtectiveMeasureUseCase {
    deps: ProtectiveMeasureUseCaseDependencies,
}

fn map_reference_error_for_update(msg: String) -> AppError {
    let detail = match msg.as_str() {
        "Court district not found" => "court_district_id not found".to_string(),
        _ => msg,
    };

    AppError::BadRequest(format!("Error updating protective measure: {}", detail))
}

impl UpdateProtectiveMeasureUseCase {
    pub fn new(deps: ProtectiveMeasureUseCaseDependencies) -> Self {
        Self { deps }
    }

    pub async fn execute(
        &self,
        data: UpdateProtectiveMeasure,
        id: Uuid,
        claims: &UserClaims,
    ) -> Result<ProtectiveMeasure, AppError> {
        info!(
            "[UpdateProtectiveMeasureUseCase] Updating protective measure: {}",
            id
        );

        let mut data = data;

        let existing_measure = match self
            .deps
            .measure_read_repository
            .get_protective_measure_by_id(id)
            .await
        {
            Ok(m) => m,
            Err(RepositoryError::NotFound) => {
                return Err(AppError::NotFound(format!(
                    "Protective measure with id '{}' not found",
                    id
                )));
            }
            Err(e) => {
                error!(
                    "[UpdateProtectiveMeasureUseCase] Error fetching measure: {:?}",
                    e
                );
                return Err(AppError::InternalServerError);
            }
        };

        let existing_victim = self
            .deps
            .victim_repository
            .get_victim_by_id(existing_measure.victim_id)
            .await
            .map_err(|e| {
                error!(
                    "[UpdateProtectiveMeasureUseCase] Error fetching existing victim: {:?}",
                    e
                );
                AppError::InternalServerError
            })?;

        let auth = AuthContext::load(&*self.deps.user_repository, claims).await?;
        auth.check_policy(&Policy::UpdateProtectiveMeasures, existing_victim.city_id)?;

        if data.victim_id != existing_measure.victim_id {
            let new_victim = match self
                .deps
                .victim_repository
                .get_victim_by_id(data.victim_id)
                .await
            {
                Ok(v) => v,
                Err(RepositoryError::NotFound) => {
                    return Err(AppError::NotFound(format!(
                        "Victim with id '{}' not found",
                        data.victim_id
                    )));
                }
                Err(e) => {
                    error!(
                        "[UpdateProtectiveMeasureUseCase] Error checking new victim: {:?}",
                        e
                    );
                    return Err(AppError::InternalServerError);
                }
            };

            auth.check_policy(&Policy::UpdateProtectiveMeasures, new_victim.city_id)?;
        }

        if data.offender_id != existing_measure.offender_id {
            match self
                .deps
                .offender_repository
                .get_offender_by_id(data.offender_id)
                .await
            {
                Ok(_) => {}
                Err(RepositoryError::NotFound) => {
                    return Err(AppError::NotFound(format!(
                        "Offender with id '{}' not found",
                        data.offender_id
                    )));
                }
                Err(e) => {
                    error!(
                        "[UpdateProtectiveMeasureUseCase] Error checking offender: {:?}",
                        e
                    );
                    return Err(AppError::InternalServerError);
                }
            }
        }

        ProtectiveMeasureValidator::validate_required_fields(
            &data.process_number,
            &data.judicial_authority,
            &data.violence_types,
            "Error updating protective measure",
        )?;

        if data.status == ProtectiveMeasureStatus::Valid {
            let active_exists = self
                .deps
                .measure_read_repository
                .check_active_measure_exists_for_victim(data.victim_id, id)
                .await
                .map_err(|e| {
                    error!(
                        "[UpdateProtectiveMeasureUseCase] Failed to check for active measure: {:?}",
                        e
                    );
                    AppError::InternalServerError
                })?;

            if active_exists {
                error!(
                    "[UpdateProtectiveMeasureUseCase] Active measure already exists for victim: {}",
                    data.victim_id
                );
                return Err(AppError::BadRequest(
                    "Error updating protective measure: victim already has an active protective measure".to_string()
                ));
            }
        }

        let extensions = data.extensions.take();
        if let Some(extensions_list) = extensions.as_ref() {
            for ext in extensions_list {
                if let Some(ext_id) = ext.id {
                    let existing_ext = match self
                        .deps
                        .extension_repository
                        .get_extension_by_id(ext_id)
                        .await
                    {
                        Ok(found) => found,
                        Err(RepositoryError::NotFound) => {
                            return Err(AppError::NotFound(format!(
                                "Extension with id '{}' not found",
                                ext_id
                            )));
                        }
                        Err(e) => {
                            error!(
                                "[UpdateProtectiveMeasureUseCase] Error fetching extension: {:?}",
                                e
                            );
                            return Err(AppError::InternalServerError);
                        }
                    };

                    if existing_ext.protective_measure_id != id {
                        return Err(AppError::BadRequest(
                            "Error updating protective measure: extension does not belong to this measure".to_string()
                        ));
                    }
                }
            }
        }

        let mut extensions_to_create = Vec::new();
        let mut extensions_to_update = Vec::new();

        if let Some(extensions_list) = extensions.as_ref() {
            for ext in extensions_list {
                if let Some(ext_id) = ext.id {
                    extensions_to_update.push((
                        ext_id,
                        UpdateExtension {
                            extension_number: ext.extension_number,
                            extension_date: ext.extension_date,
                            new_valid_until: ext.new_valid_until,
                            notes: ext.notes.clone(),
                        },
                    ));
                } else {
                    extensions_to_create.push(CreateExtension {
                        extension_number: ext.extension_number,
                        extension_date: ext.extension_date,
                        new_valid_until: ext.new_valid_until,
                        notes: ext.notes.clone(),
                    });
                }
            }
        }

        let updated = match self
            .deps
            .measure_write_repository
            .update_protective_measure_with_extensions(
                &data,
                id,
                &extensions_to_create,
                &extensions_to_update,
            )
            .await
        {
            Ok(measure) => measure,
            Err(RepositoryError::NotFound) => {
                return Err(AppError::NotFound(format!(
                    "Protective measure with id '{}' not found",
                    id
                )));
            }
            Err(RepositoryError::ReferencedEntityNotFound(msg)) => {
                return Err(map_reference_error_for_update(msg));
            }
            Err(e) => {
                error!(
                    "[UpdateProtectiveMeasureUseCase] Failed to update measure: {:?}",
                    e
                );
                return Err(AppError::InternalServerError);
            }
        };

        info!(
            "[UpdateProtectiveMeasureUseCase] Protective measure updated successfully: {}",
            id
        );
        Ok(updated)
    }
}
