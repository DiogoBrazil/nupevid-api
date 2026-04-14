use log::{error, info};
use std::sync::Arc;
use uuid::Uuid;

use crate::core::commands::protective_measures::{
    CreateExtension, CreateProtectiveMeasure, UpdateExtension, UpdateProtectiveMeasure,
};
use crate::core::entities::auth::ClaimsToUserToken;
use crate::core::entities::common::PaginatedResult;
use crate::core::entities::protective_measures::{ProtectiveMeasure, ProtectiveMeasureStatus};
use crate::utils::errors::AppError;
use crate::core::contracts::repository::error::RepositoryError;
use crate::core::read_models::protective_measures::{
    ProtectiveMeasureWithExtensions, ProtectiveMeasureWithExtensionsAndEntities,
};
use crate::core::responses::protective_measures::ProtectiveMeasureResponse;

use crate::core::contracts::repository::cities::CityRepository;
use crate::core::contracts::repository::extensions::ExtensionRepository;
use crate::core::contracts::repository::offenders::OffenderReadRepository;
use crate::core::contracts::repository::protective_measures::{
    ProtectiveMeasureReadRepository, ProtectiveMeasureWriteRepository,
};
use crate::core::contracts::repository::users::UserRepository;
use crate::core::contracts::repository::victims::VictimReadRepository;

use crate::services::auth_context::AuthContext;
use crate::services::error_mapping::map_constraint;
use crate::utils::pagination::Pagination;
use crate::core::value_objects::policies::Policy;
use crate::validators::protective_measure_validator::ProtectiveMeasureValidator;

pub struct ProtectiveMeasureService {
    measure_read_repository: Arc<dyn ProtectiveMeasureReadRepository>,
    measure_write_repository: Arc<dyn ProtectiveMeasureWriteRepository>,
    victim_repository: Arc<dyn VictimReadRepository>,
    offender_repository: Arc<dyn OffenderReadRepository>,
    user_repository: Arc<dyn UserRepository>,
    extension_repository: Arc<dyn ExtensionRepository>,
    city_repository: Arc<dyn CityRepository>,
}

impl ProtectiveMeasureService {
    pub fn new(
        measure_read_repository: Arc<dyn ProtectiveMeasureReadRepository>,
        measure_write_repository: Arc<dyn ProtectiveMeasureWriteRepository>,
        victim_repository: Arc<dyn VictimReadRepository>,
        offender_repository: Arc<dyn OffenderReadRepository>,
        user_repository: Arc<dyn UserRepository>,
        extension_repository: Arc<dyn ExtensionRepository>,
        city_repository: Arc<dyn CityRepository>,
    ) -> Self {
        Self {
            measure_read_repository,
            measure_write_repository,
            victim_repository,
            offender_repository,
            user_repository,
            extension_repository,
            city_repository,
        }
    }

    pub async fn create_protective_measure(
        &self,
        measure: CreateProtectiveMeasure,
        claims: &ClaimsToUserToken,
        include_complement_for_entities: bool,
    ) -> Result<ProtectiveMeasureResponse, AppError> {
        info!(
            "[ProtectiveMeasureService] Starting protective measure creation for victim: {}",
            measure.victim_id
        );

        let victim = match self
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
                error!("[ProtectiveMeasureService] Error checking victim: {:?}", e);
                return Err(AppError::InternalServerError);
            }
        };

        match self
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
                    "[ProtectiveMeasureService] Error checking offender: {:?}",
                    e
                );
                return Err(AppError::InternalServerError);
            }
        }

        let auth = AuthContext::load(&*self.user_repository, claims).await?;
        auth.check_policy(&Policy::CreateProtectiveMeasures, victim.city_id)?;

        ProtectiveMeasureValidator::validate_required_fields(
            &measure.process_number,
            &measure.judicial_authority,
            &measure.violence_types,
            "Error adding protective measure",
        )?;

        if measure.status == ProtectiveMeasureStatus::Valid {
            let active_exists = self
                .measure_read_repository
                .check_active_measure_exists_for_victim(measure.victim_id, Uuid::nil())
                .await
                .map_err(|e| {
                    error!(
                        "[ProtectiveMeasureService] Failed to check for active measure: {:?}",
                        e
                    );
                    AppError::InternalServerError
                })?;

            if active_exists {
                error!(
                    "[ProtectiveMeasureService] Active measure already exists for victim: {}",
                    measure.victim_id
                );
                return Err(AppError::BadRequest(
                    "Error adding protective measure: victim already has an active protective measure".to_string()
                ));
            }
        }

        info!("[ProtectiveMeasureService] Saving protective measure to database");

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
            .measure_write_repository
            .create_protective_measure_with_extensions(&measure, &extensions_to_create)
            .await
        {
            Ok(measure) => measure,
            Err(e) => {
                if let RepositoryError::UniqueViolation { constraint }
                | RepositoryError::ForeignKeyViolation { constraint } = &e
                    && let Some(app_err) = map_constraint(
                        constraint.as_deref(),
                        &[
                            (
                                "fk_protective_measures_court_district",
                                "Error adding protective measure: court_district_id not found",
                            ),
                            (
                                "fk_protective_measures_victim",
                                "Error adding protective measure: victim_id not found",
                            ),
                            (
                                "fk_protective_measures_offender",
                                "Error adding protective measure: offender_id not found",
                            ),
                        ],
                    )
                {
                    return Err(app_err);
                }
                error!(
                    "[ProtectiveMeasureService] Failed to save protective measure: {:?}",
                    e
                );
                return Err(AppError::InternalServerError);
            }
        };

        info!(
            "[ProtectiveMeasureService] Protective measure created successfully with ID: {}",
            created.id
        );
        self.build_measure_response(created, include_complement_for_entities)
            .await
    }

    pub async fn get_protective_measure_by_id(
        &self,
        id: Uuid,
        claims: &ClaimsToUserToken,
        include_complement_for_entities: bool,
    ) -> Result<ProtectiveMeasureResponse, AppError> {
        info!(
            "[ProtectiveMeasureService] Getting protective measure by id: {}",
            id
        );

        match self
            .measure_read_repository
            .get_protective_measure_by_id(id)
            .await
        {
            Ok(measure) => {
                let victim = self
                    .victim_repository
                    .get_victim_by_id(measure.victim_id)
                    .await
                    .map_err(|e| {
                        error!("[ProtectiveMeasureService] Error fetching victim: {:?}", e);
                        AppError::InternalServerError
                    })?;

                let auth = AuthContext::load(&*self.user_repository, claims).await?;
                auth.check_policy(&Policy::ReadProtectiveMeasures, victim.city_id)?;

                info!(
                    "[ProtectiveMeasureService] Protective measure found: {}",
                    id
                );
                self.build_measure_response(measure, include_complement_for_entities)
                    .await
            }
            Err(RepositoryError::NotFound) => Err(AppError::NotFound(format!(
                "Protective measure with id '{}' not found",
                id
            ))),
            Err(e) => {
                error!("[ProtectiveMeasureService] Database error: {:?}", e);
                Err(AppError::InternalServerError)
            }
        }
    }

    pub async fn get_all_protective_measures(
        &self,
        pagination: Pagination,
        claims: &ClaimsToUserToken,
        include_complement_for_entities: bool,
    ) -> Result<PaginatedResult<ProtectiveMeasureResponse>, AppError> {
        info!("[ProtectiveMeasureService] Getting all protective measures");

        let auth = AuthContext::load(&*self.user_repository, claims).await?;
        let allowed_cities = auth.allowed_cities(&Policy::ReadProtectiveMeasures);

        let total_items = self
            .measure_read_repository
            .count_protective_measures(allowed_cities.as_deref())
            .await
            .map_err(|_| AppError::InternalServerError)?;

        let measures = self
            .measure_read_repository
            .get_protective_measures_paginated(
                allowed_cities.as_deref(),
                pagination.page_size,
                pagination.offset,
            )
            .await
            .map_err(|_| AppError::InternalServerError)?;

        let mut items = Vec::with_capacity(measures.len());
        for measure in measures {
            items.push(
                self.build_measure_response(measure, include_complement_for_entities)
                    .await?,
            );
        }

        info!(
            "[ProtectiveMeasureService] Successfully retrieved {} measures",
            items.len()
        );
        Ok(PaginatedResult {
            items,
            page: pagination.page,
            page_size: pagination.page_size,
            total_items,
        })
    }

    pub async fn get_protective_measures_by_victim(
        &self,
        victim_id: Uuid,
        claims: &ClaimsToUserToken,
        include_complement_for_entities: bool,
    ) -> Result<Vec<ProtectiveMeasureResponse>, AppError> {
        info!(
            "[ProtectiveMeasureService] Getting measures for victim: {}",
            victim_id
        );

        let victim = match self.victim_repository.get_victim_by_id(victim_id).await {
            Ok(v) => v,
            Err(RepositoryError::NotFound) => {
                return Err(AppError::NotFound(format!(
                    "Victim with id '{}' not found",
                    victim_id
                )));
            }
            Err(e) => {
                error!("[ProtectiveMeasureService] Error checking victim: {:?}", e);
                return Err(AppError::InternalServerError);
            }
        };

        let auth = AuthContext::load(&*self.user_repository, claims).await?;
        auth.check_policy(&Policy::ReadProtectiveMeasures, victim.city_id)?;

        match self
            .measure_read_repository
            .get_protective_measures_by_victim(victim_id)
            .await
        {
            Ok(measures) => {
                let mut responses = Vec::with_capacity(measures.len());
                for measure in measures {
                    responses.push(
                        self.build_measure_response(measure, include_complement_for_entities)
                            .await?,
                    );
                }

                info!(
                    "[ProtectiveMeasureService] Found {} measures for victim",
                    responses.len()
                );
                Ok(responses)
            }
            Err(e) => {
                error!(
                    "[ProtectiveMeasureService] Failed to retrieve measures: {:?}",
                    e
                );
                Err(AppError::InternalServerError)
            }
        }
    }

    pub async fn update_protective_measure_by_id(
        &self,
        data: UpdateProtectiveMeasure,
        id: Uuid,
        claims: &ClaimsToUserToken,
    ) -> Result<ProtectiveMeasureResponse, AppError> {
        info!(
            "[ProtectiveMeasureService] Updating protective measure: {}",
            id
        );

        let mut data = data;

        let existing_measure = match self
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
                error!("[ProtectiveMeasureService] Error fetching measure: {:?}", e);
                return Err(AppError::InternalServerError);
            }
        };

        let existing_victim = self
            .victim_repository
            .get_victim_by_id(existing_measure.victim_id)
            .await
            .map_err(|e| {
                error!(
                    "[ProtectiveMeasureService] Error fetching existing victim: {:?}",
                    e
                );
                AppError::InternalServerError
            })?;

        let auth = AuthContext::load(&*self.user_repository, claims).await?;
        auth.check_policy(&Policy::UpdateProtectiveMeasures, existing_victim.city_id)?;

        if data.victim_id != existing_measure.victim_id {
            let new_victim = match self
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
                        "[ProtectiveMeasureService] Error checking new victim: {:?}",
                        e
                    );
                    return Err(AppError::InternalServerError);
                }
            };

            auth.check_policy(&Policy::UpdateProtectiveMeasures, new_victim.city_id)?;
        }

        if data.offender_id != existing_measure.offender_id {
            match self
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
                        "[ProtectiveMeasureService] Error checking offender: {:?}",
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
                .measure_read_repository
                .check_active_measure_exists_for_victim(data.victim_id, id)
                .await
                .map_err(|e| {
                    error!(
                        "[ProtectiveMeasureService] Failed to check for active measure: {:?}",
                        e
                    );
                    AppError::InternalServerError
                })?;

            if active_exists {
                error!(
                    "[ProtectiveMeasureService] Active measure already exists for victim: {}",
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
                    let existing_ext =
                        match self.extension_repository.get_extension_by_id(ext_id).await {
                            Ok(found) => found,
                            Err(RepositoryError::NotFound) => {
                                return Err(AppError::NotFound(format!(
                                    "Extension with id '{}' not found",
                                    ext_id
                                )));
                            }
                            Err(e) => {
                                error!(
                                    "[ProtectiveMeasureService] Error fetching extension: {:?}",
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
            Err(e) => {
                if let RepositoryError::UniqueViolation { constraint }
                | RepositoryError::ForeignKeyViolation { constraint } = &e
                    && let Some(app_err) = map_constraint(
                        constraint.as_deref(),
                        &[
                            (
                                "fk_protective_measures_court_district",
                                "Error updating protective measure: court_district_id not found",
                            ),
                            (
                                "fk_protective_measures_victim",
                                "Error updating protective measure: victim_id not found",
                            ),
                            (
                                "fk_protective_measures_offender",
                                "Error updating protective measure: offender_id not found",
                            ),
                        ],
                    )
                {
                    return Err(app_err);
                }
                error!(
                    "[ProtectiveMeasureService] Failed to update measure: {:?}",
                    e
                );
                return Err(AppError::InternalServerError);
            }
        };

        info!(
            "[ProtectiveMeasureService] Protective measure updated successfully: {}",
            id
        );
        self.build_measure_response(updated, false).await
    }

    pub async fn delete_protective_measure_by_id(
        &self,
        id: Uuid,
        claims: &ClaimsToUserToken,
    ) -> Result<ProtectiveMeasure, AppError> {
        info!(
            "[ProtectiveMeasureService] Deleting protective measure: {}",
            id
        );

        let measure = match self
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
                error!("[ProtectiveMeasureService] Error fetching measure: {:?}", e);
                return Err(AppError::InternalServerError);
            }
        };

        let victim = self
            .victim_repository
            .get_victim_by_id(measure.victim_id)
            .await
            .map_err(|e| {
                error!("[ProtectiveMeasureService] Error fetching victim: {:?}", e);
                AppError::InternalServerError
            })?;

        let auth = AuthContext::load(&*self.user_repository, claims).await?;
        auth.check_policy(&Policy::DeleteProtectiveMeasures, victim.city_id)?;

        match self
            .measure_write_repository
            .delete_protective_measure_by_id(id)
            .await
        {
            Ok(deleted_measure) => {
                info!(
                    "[ProtectiveMeasureService] Protective measure deleted successfully: {}",
                    id
                );
                Ok(deleted_measure)
            }
            Err(RepositoryError::NotFound) => Err(AppError::NotFound(format!(
                "Protective measure with id '{}' not found",
                id
            ))),
            Err(e) => {
                error!(
                    "[ProtectiveMeasureService] Failed to delete measure: {:?}",
                    e
                );
                Err(AppError::InternalServerError)
            }
        }
    }

    async fn build_measure_with_entities(
        &self,
        measure: ProtectiveMeasure,
        extensions: Vec<crate::core::entities::protective_measures::ProtectiveMeasureExtension>,
    ) -> Result<ProtectiveMeasureWithExtensionsAndEntities, AppError> {
        let victim = self
            .victim_repository
            .get_victim_by_id(measure.victim_id)
            .await
            .map_err(|e| match e {
                RepositoryError::NotFound => {
                    AppError::NotFound(format!("Victim with id '{}' not found", measure.victim_id))
                }
                _ => {
                    error!("[ProtectiveMeasureService] Error fetching victim: {:?}", e);
                    AppError::InternalServerError
                }
            })?;

        let offender = self
            .offender_repository
            .get_offender_by_id(measure.offender_id)
            .await
            .map_err(|e| match e {
                RepositoryError::NotFound => AppError::NotFound(format!(
                    "Offender with id '{}' not found",
                    measure.offender_id
                )),
                _ => {
                    error!(
                        "[ProtectiveMeasureService] Error fetching offender: {:?}",
                        e
                    );
                    AppError::InternalServerError
                }
            })?;

        let court_district = self
            .city_repository
            .get_city_by_id(measure.court_district_id)
            .await
            .map_err(|e| match e {
                RepositoryError::NotFound => AppError::NotFound(format!(
                    "City with id '{}' not found",
                    measure.court_district_id
                )),
                _ => {
                    error!(
                        "[ProtectiveMeasureService] Error fetching court district: {:?}",
                        e
                    );
                    AppError::InternalServerError
                }
            })?;

        Ok(ProtectiveMeasureWithExtensionsAndEntities {
            measure,
            extensions,
            victim: victim.to_complement(),
            offender: offender.to_complement(),
            court_district: court_district.into(),
        })
    }

    async fn build_measure_response(
        &self,
        measure: ProtectiveMeasure,
        include_complement_for_entities: bool,
    ) -> Result<ProtectiveMeasureResponse, AppError> {
        let extensions = self
            .extension_repository
            .get_extensions_by_measure(measure.id)
            .await
            .map_err(|e| {
                error!(
                    "[ProtectiveMeasureService] Error fetching extensions: {:?}",
                    e
                );
                AppError::InternalServerError
            })?;

        if include_complement_for_entities {
            Ok(ProtectiveMeasureResponse::WithEntities(
                self.build_measure_with_entities(measure, extensions)
                    .await?,
            ))
        } else {
            Ok(ProtectiveMeasureResponse::Simple(
                ProtectiveMeasureWithExtensions {
                    measure,
                    extensions,
                },
            ))
        }
    }
}
