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
use crate::usecases::helpers_common::{
    get_extension_or_not_found, get_offender_or_not_found, get_protective_measure_or_not_found,
    get_victim_or_not_found,
};
use crate::usecases::protective_measures::deps::ProtectiveMeasureUseCaseDependencies;
use crate::usecases::protective_measures::errors::map_reference_error;
use crate::validators::protective_measure_validator::ProtectiveMeasureValidator;

pub struct UpdateProtectiveMeasureUseCase {
    deps: ProtectiveMeasureUseCaseDependencies,
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
        get_protective_measure_or_not_found(&*self.deps.measure_read_repository, id).await
    }

    async fn authorize_update(
        &self,
        claims: &UserClaims,
        data: &UpdateProtectiveMeasure,
        existing: &ProtectiveMeasure,
    ) -> Result<(), AppError> {
        let existing_victim =
            get_victim_or_not_found(&*self.deps.victim_repository, existing.victim_id).await?;

        let auth = AuthContext::load(&*self.deps.user_repository, claims).await?;
        auth.check_policy(&Policy::UpdateProtectiveMeasures, existing_victim.city_id)?;

        if data.victim_id != existing.victim_id {
            let new_victim =
                get_victim_or_not_found(&*self.deps.victim_repository, data.victim_id).await?;
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
            get_offender_or_not_found(&*self.deps.offender_repository, data.offender_id).await?;
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
                .check_active_measure_exists_for_victim(data.victim_id, Some(exclude_id))
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
                    let existing_ext =
                        get_extension_or_not_found(&*self.deps.extension_repository, ext_id)
                            .await?;

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
            Err(RepositoryError::ReferencedEntityNotFound(msg)) => Err(map_reference_error(
                msg,
                "Error updating protective measure",
            )),
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
