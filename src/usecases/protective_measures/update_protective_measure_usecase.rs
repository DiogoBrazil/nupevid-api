use log::{error, info};
use uuid::Uuid;

use crate::core::application_error::ApplicationError as AppError;
use crate::core::auth_context::AuthContext;
use crate::core::commands::protective_measures::{
    CreateExtension, ExtensionUpsert, UpdateExtension, UpdateProtectiveMeasure,
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

        let existing_measure = self.load_existing_measure(id).await?;
        self.authorize_update(claims, &data, &existing_measure)
            .await?;
        self.validate_related_entities(&data, &existing_measure)
            .await?;

        ProtectiveMeasureValidator::validate_required_fields(
            &data.process_number,
            &data.judicial_authority,
            &data.violence_types,
            "Error updating protective measure",
        )?;

        self.ensure_unique_active_measure(&data, id).await?;

        let extensions = data.extensions.take();
        self.validate_extension_ownership(&extensions, id).await?;
        let (extensions_to_create, extensions_to_update) =
            Self::split_extensions_for_upsert(&extensions);

        self.persist_measure_and_extensions(&data, id, &extensions_to_create, &extensions_to_update)
            .await
    }

    async fn load_existing_measure(&self, id: Uuid) -> Result<ProtectiveMeasure, AppError> {
        match self
            .deps
            .measure_read_repository
            .get_protective_measure_by_id(id)
            .await
        {
            Ok(m) => Ok(m),
            Err(RepositoryError::NotFound) => Err(AppError::NotFound(format!(
                "Protective measure with id '{}' not found",
                id
            ))),
            Err(e) => {
                error!(
                    "[UpdateProtectiveMeasureUseCase] Error fetching measure: {:?}",
                    e
                );
                Err(AppError::InternalServerError)
            }
        }
    }

    async fn authorize_update(
        &self,
        claims: &UserClaims,
        data: &UpdateProtectiveMeasure,
        existing: &ProtectiveMeasure,
    ) -> Result<(), AppError> {
        let existing_victim = self
            .deps
            .victim_repository
            .get_victim_by_id(existing.victim_id)
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

        if data.victim_id != existing.victim_id {
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

        Ok(())
    }

    async fn validate_related_entities(
        &self,
        data: &UpdateProtectiveMeasure,
        existing: &ProtectiveMeasure,
    ) -> Result<(), AppError> {
        if data.offender_id != existing.offender_id {
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
        Ok(())
    }

    async fn ensure_unique_active_measure(
        &self,
        data: &UpdateProtectiveMeasure,
        exclude_id: Uuid,
    ) -> Result<(), AppError> {
        if data.status == ProtectiveMeasureStatus::Valid {
            let active_exists = self
                .deps
                .measure_read_repository
                .check_active_measure_exists_for_victim(data.victim_id, exclude_id)
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
        Ok(())
    }

    async fn validate_extension_ownership(
        &self,
        extensions: &Option<Vec<ExtensionUpsert>>,
        measure_id: Uuid,
    ) -> Result<(), AppError> {
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

                    if existing_ext.protective_measure_id != measure_id {
                        return Err(AppError::BadRequest(
                            "Error updating protective measure: extension does not belong to this measure".to_string()
                        ));
                    }
                }
            }
        }
        Ok(())
    }

    fn split_extensions_for_upsert(
        extensions: &Option<Vec<ExtensionUpsert>>,
    ) -> (Vec<CreateExtension>, Vec<(Uuid, UpdateExtension)>) {
        let mut to_create = Vec::new();
        let mut to_update = Vec::new();

        if let Some(extensions_list) = extensions.as_ref() {
            for ext in extensions_list {
                if let Some(ext_id) = ext.id {
                    to_update.push((
                        ext_id,
                        UpdateExtension {
                            extension_number: ext.extension_number,
                            extension_date: ext.extension_date,
                            new_valid_until: ext.new_valid_until,
                            notes: ext.notes.clone(),
                        },
                    ));
                } else {
                    to_create.push(CreateExtension {
                        extension_number: ext.extension_number,
                        extension_date: ext.extension_date,
                        new_valid_until: ext.new_valid_until,
                        notes: ext.notes.clone(),
                    });
                }
            }
        }

        (to_create, to_update)
    }

    async fn persist_measure_and_extensions(
        &self,
        data: &UpdateProtectiveMeasure,
        id: Uuid,
        extensions_to_create: &[CreateExtension],
        extensions_to_update: &[(Uuid, UpdateExtension)],
    ) -> Result<ProtectiveMeasure, AppError> {
        match self
            .deps
            .measure_write_repository
            .update_protective_measure_with_extensions(
                data,
                id,
                extensions_to_create,
                extensions_to_update,
            )
            .await
        {
            Ok(measure) => {
                info!(
                    "[UpdateProtectiveMeasureUseCase] Protective measure updated successfully: {}",
                    id
                );
                Ok(measure)
            }
            Err(RepositoryError::NotFound) => Err(AppError::NotFound(format!(
                "Protective measure with id '{}' not found",
                id
            ))),
            Err(RepositoryError::ReferencedEntityNotFound(msg)) => {
                Err(map_reference_error_for_update(msg))
            }
            Err(e) => {
                error!(
                    "[UpdateProtectiveMeasureUseCase] Failed to update measure: {:?}",
                    e
                );
                Err(AppError::InternalServerError)
            }
        }
    }
}
