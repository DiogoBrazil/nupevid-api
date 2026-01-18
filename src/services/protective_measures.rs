use actix_web::{HttpRequest, HttpResponse, web};
use log::{error, info};
use uuid::Uuid;

use crate::core::entities::protective_measures::{
    CreateExtension, CreateProtectiveMeasure, ProtectiveMeasureStatus,
    ProtectiveMeasureWithExtensions, ProtectiveMeasureWithExtensionsAndEntities, UpdateExtension,
    UpdateProtectiveMeasure,
};

use crate::core::contracts::repository::cities::CityRepository;
use crate::core::contracts::repository::extensions::ExtensionRepository;
use crate::core::contracts::repository::offenders::OffenderRepository;
use crate::core::contracts::repository::protective_measures::ProtectiveMeasureRepository;
use crate::core::contracts::repository::victims::VictimRepository;
use crate::repositories::cities::PgCityRepository;
use crate::repositories::extensions::PgExtensionRepository;
use crate::repositories::offenders::PgOffenderRepository;
use crate::repositories::protective_measures::PgProtectiveMeasureRepository;
use crate::repositories::users::PgUserRepository;
use crate::repositories::victims::PgVictimRepository;

use crate::utils::{
    authorization::{check_policy, get_allowed_cities_for_policy},
    db_error_mapper::map_constraint,
    errors::AppError,
    pagination::{PaginationParams, normalize_pagination},
    responses::{ApiResponse, PaginatedResponse},
    service_helpers::{extract_claims, get_user_policies_with_defaults},
};
use crate::validators::{
    common::{
        POLICY_CREATE_PROTECTIVE_MEASURES, POLICY_DELETE_PROTECTIVE_MEASURES,
        POLICY_READ_PROTECTIVE_MEASURES, POLICY_UPDATE_PROTECTIVE_MEASURES,
    },
    protective_measure_validator::ProtectiveMeasureValidator,
};

pub struct ProtectiveMeasureService {
    measure_repository: web::Data<PgProtectiveMeasureRepository>,
    victim_repository: web::Data<PgVictimRepository>,
    offender_repository: web::Data<PgOffenderRepository>,
    user_repository: web::Data<PgUserRepository>,
    extension_repository: web::Data<PgExtensionRepository>,
    city_repository: web::Data<PgCityRepository>,
}

impl ProtectiveMeasureService {
    pub fn new(
        measure_repository: web::Data<PgProtectiveMeasureRepository>,
        victim_repository: web::Data<PgVictimRepository>,
        offender_repository: web::Data<PgOffenderRepository>,
        user_repository: web::Data<PgUserRepository>,
        extension_repository: web::Data<PgExtensionRepository>,
        city_repository: web::Data<PgCityRepository>,
    ) -> Self {
        Self {
            measure_repository,
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
        req: HttpRequest,
        include_complement_for_entities: bool,
    ) -> Result<HttpResponse, AppError> {
        info!(
            "[ProtectiveMeasureService] Starting protective measure creation for victim: {}",
            measure.victim_id
        );

        let claims = extract_claims(&req)?;

        let victim = match self
            .victim_repository
            .get_victim_by_id(measure.victim_id)
            .await
        {
            Ok(v) => v,
            Err(sqlx::Error::RowNotFound) => {
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
            Err(sqlx::Error::RowNotFound) => {
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

        let policies = get_user_policies_with_defaults(&**self.user_repository, &claims).await?;
        check_policy(
            &claims,
            POLICY_CREATE_PROTECTIVE_MEASURES,
            victim.city_id,
            &policies,
        )?;

        ProtectiveMeasureValidator::validate_required_fields(
            &measure.process_number,
            &measure.judicial_authority,
            &measure.violence_types,
            "Error adding protective measure",
        )?;

        if measure.status == ProtectiveMeasureStatus::Valid {
            let active_exists = self
                .measure_repository
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

        let mut tx = self.measure_repository.begin_tx().await.map_err(|e| {
            error!(
                "[ProtectiveMeasureService] Failed to begin transaction: {:?}",
                e
            );
            AppError::InternalServerError
        })?;

        let created = match self
            .measure_repository
            .create_protective_measure_with_tx(&mut tx, &measure)
            .await
        {
            Ok(measure) => measure,
            Err(e) => {
                if let sqlx::Error::Database(db_err) = &e
                    && let Some(app_err) = map_constraint(
                        db_err.constraint(),
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

        if let Some(extensions) = measure.extensions.as_ref() {
            for ext in extensions {
                let create = CreateExtension {
                    extension_number: ext.extension_number,
                    extension_date: ext.extension_date,
                    new_valid_until: ext.new_valid_until,
                    notes: ext.notes.clone(),
                };

                if let Err(e) = self
                    .extension_repository
                    .create_extension_with_tx(&mut tx, created.id, &create)
                    .await
                {
                    if let sqlx::Error::Database(db_err) = &e
                        && let Some(app_err) = map_constraint(
                            db_err.constraint(),
                            &[(
                                "fk_extensions_protective_measure",
                                "Error adding protective measure: extension protective_measure_id not found",
                            )],
                        )
                    {
                        return Err(app_err);
                    }
                    error!(
                        "[ProtectiveMeasureService] Failed to create extension: {:?}",
                        e
                    );
                    return Err(AppError::InternalServerError);
                }
            }
        }

        tx.commit().await.map_err(|e| {
            error!(
                "[ProtectiveMeasureService] Failed to commit transaction: {:?}",
                e
            );
            AppError::InternalServerError
        })?;

        info!(
            "[ProtectiveMeasureService] Protective measure created successfully with ID: {}",
            created.id
        );
        if include_complement_for_entities {
            let extensions = self
                .extension_repository
                .get_extensions_by_measure(created.id)
                .await
                .map_err(|e| {
                    error!(
                        "[ProtectiveMeasureService] Error fetching extensions: {:?}",
                        e
                    );
                    AppError::InternalServerError
                })?;

            let response = self
                .build_measure_with_entities(created, extensions)
                .await?;
            Ok(ApiResponse::created(response).into_response())
        } else {
            Ok(ApiResponse::created(created).into_response())
        }
    }

    pub async fn get_protective_measure_by_id(
        &self,
        id: Uuid,
        req: HttpRequest,
        include_complement_for_entities: bool,
    ) -> Result<HttpResponse, AppError> {
        info!(
            "[ProtectiveMeasureService] Getting protective measure by id: {}",
            id
        );

        let claims = extract_claims(&req)?;

        match self
            .measure_repository
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

                let policies =
                    get_user_policies_with_defaults(&**self.user_repository, &claims).await?;
                check_policy(
                    &claims,
                    POLICY_READ_PROTECTIVE_MEASURES,
                    victim.city_id,
                    &policies,
                )?;

                let extensions = self
                    .extension_repository
                    .get_extensions_by_measure(id)
                    .await
                    .map_err(|e| {
                        error!(
                            "[ProtectiveMeasureService] Error fetching extensions: {:?}",
                            e
                        );
                        AppError::InternalServerError
                    })?;

                info!(
                    "[ProtectiveMeasureService] Protective measure found: {}",
                    id
                );
                if include_complement_for_entities {
                    let response = self
                        .build_measure_with_entities(measure, extensions)
                        .await?;
                    Ok(ApiResponse::success(response).into_response())
                } else {
                    let response = ProtectiveMeasureWithExtensions {
                        measure,
                        extensions,
                    };
                    Ok(ApiResponse::success(response).into_response())
                }
            }
            Err(sqlx::Error::RowNotFound) => Err(AppError::NotFound(format!(
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
        params: PaginationParams,
        req: HttpRequest,
        include_complement_for_entities: bool,
    ) -> Result<HttpResponse, AppError> {
        info!("[ProtectiveMeasureService] Getting all protective measures");

        let claims = extract_claims(&req)?;

        let policies = get_user_policies_with_defaults(&**self.user_repository, &claims).await?;
        let pagination = normalize_pagination(&params);
        let allowed_cities =
            get_allowed_cities_for_policy(&claims, POLICY_READ_PROTECTIVE_MEASURES, &policies);

        let total_items = self
            .measure_repository
            .count_protective_measures(allowed_cities.as_deref())
            .await
            .map_err(|_| AppError::InternalServerError)?;

        let measures = self
            .measure_repository
            .get_protective_measures_paginated(
                allowed_cities.as_deref(),
                pagination.page_size,
                pagination.offset,
            )
            .await
            .map_err(|_| AppError::InternalServerError)?;

        if include_complement_for_entities {
            let mut measures_with_entities = Vec::new();
            for measure in measures {
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

                let response = self
                    .build_measure_with_entities(measure, extensions)
                    .await?;
                measures_with_entities.push(response);
            }

            info!(
                "[ProtectiveMeasureService] Successfully retrieved {} measures",
                measures_with_entities.len()
            );
            Ok(PaginatedResponse::success(
                measures_with_entities,
                pagination.page,
                pagination.page_size,
                total_items,
            )
            .into_response())
        } else {
            let mut measures_with_extensions = Vec::new();
            for measure in measures {
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

                measures_with_extensions.push(ProtectiveMeasureWithExtensions {
                    measure,
                    extensions,
                });
            }

            info!(
                "[ProtectiveMeasureService] Successfully retrieved {} measures",
                measures_with_extensions.len()
            );
            Ok(PaginatedResponse::success(
                measures_with_extensions,
                pagination.page,
                pagination.page_size,
                total_items,
            )
            .into_response())
        }
    }

    pub async fn get_protective_measures_by_victim(
        &self,
        victim_id: Uuid,
        req: HttpRequest,
        include_complement_for_entities: bool,
    ) -> Result<HttpResponse, AppError> {
        info!(
            "[ProtectiveMeasureService] Getting measures for victim: {}",
            victim_id
        );

        let claims = extract_claims(&req)?;

        let victim = match self.victim_repository.get_victim_by_id(victim_id).await {
            Ok(v) => v,
            Err(sqlx::Error::RowNotFound) => {
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

        let policies = get_user_policies_with_defaults(&**self.user_repository, &claims).await?;
        check_policy(
            &claims,
            POLICY_READ_PROTECTIVE_MEASURES,
            victim.city_id,
            &policies,
        )?;

        match self
            .measure_repository
            .get_protective_measures_by_victim(victim_id)
            .await
        {
            Ok(measures) => {
                if include_complement_for_entities {
                    let mut measures_with_entities = Vec::new();
                    for measure in measures {
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

                        let response = self
                            .build_measure_with_entities(measure, extensions)
                            .await?;
                        measures_with_entities.push(response);
                    }

                    info!(
                        "[ProtectiveMeasureService] Found {} measures for victim",
                        measures_with_entities.len()
                    );
                    Ok(ApiResponse::success(measures_with_entities).into_response())
                } else {
                    let mut measures_with_extensions = Vec::new();
                    for measure in measures {
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

                        measures_with_extensions.push(ProtectiveMeasureWithExtensions {
                            measure,
                            extensions,
                        });
                    }

                    info!(
                        "[ProtectiveMeasureService] Found {} measures for victim",
                        measures_with_extensions.len()
                    );
                    Ok(ApiResponse::success(measures_with_extensions).into_response())
                }
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
        req: HttpRequest,
    ) -> Result<HttpResponse, AppError> {
        info!(
            "[ProtectiveMeasureService] Updating protective measure: {}",
            id
        );

        let claims = extract_claims(&req)?;
        let mut data = data;

        let existing_measure = match self
            .measure_repository
            .get_protective_measure_by_id(id)
            .await
        {
            Ok(m) => m,
            Err(sqlx::Error::RowNotFound) => {
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

        let policies = get_user_policies_with_defaults(&**self.user_repository, &claims).await?;
        check_policy(
            &claims,
            POLICY_UPDATE_PROTECTIVE_MEASURES,
            existing_victim.city_id,
            &policies,
        )?;

        if data.victim_id != existing_measure.victim_id {
            let new_victim = match self
                .victim_repository
                .get_victim_by_id(data.victim_id)
                .await
            {
                Ok(v) => v,
                Err(sqlx::Error::RowNotFound) => {
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

            check_policy(
                &claims,
                POLICY_UPDATE_PROTECTIVE_MEASURES,
                new_victim.city_id,
                &policies,
            )?;
        }

        if data.offender_id != existing_measure.offender_id {
            match self
                .offender_repository
                .get_offender_by_id(data.offender_id)
                .await
            {
                Ok(_) => {}
                Err(sqlx::Error::RowNotFound) => {
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
                .measure_repository
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
                            Err(sqlx::Error::RowNotFound) => {
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

        let mut tx = self.measure_repository.begin_tx().await.map_err(|e| {
            error!(
                "[ProtectiveMeasureService] Failed to begin transaction: {:?}",
                e
            );
            AppError::InternalServerError
        })?;

        let updated = match self
            .measure_repository
            .update_protective_measure_by_id_with_tx(&mut tx, &data, id)
            .await
        {
            Ok(measure) => measure,
            Err(sqlx::Error::RowNotFound) => {
                return Err(AppError::NotFound(format!(
                    "Protective measure with id '{}' not found",
                    id
                )));
            }
            Err(e) => {
                if let sqlx::Error::Database(db_err) = &e
                    && let Some(app_err) = map_constraint(
                        db_err.constraint(),
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

        if let Some(extensions_list) = extensions {
            for ext in extensions_list {
                if let Some(ext_id) = ext.id {
                    let update = UpdateExtension {
                        extension_number: ext.extension_number,
                        extension_date: ext.extension_date,
                        new_valid_until: ext.new_valid_until,
                        notes: ext.notes.clone(),
                    };

                    if let Err(e) = self
                        .extension_repository
                        .update_extension_by_id_with_tx(&mut tx, &update, ext_id)
                        .await
                    {
                        if let sqlx::Error::Database(db_err) = &e
                            && let Some(app_err) = map_constraint(
                                db_err.constraint(),
                                &[(
                                    "fk_extensions_protective_measure",
                                    "Error updating protective measure: extension protective_measure_id not found",
                                )],
                            )
                        {
                            return Err(app_err);
                        }
                        if let sqlx::Error::RowNotFound = e {
                            return Err(AppError::NotFound(format!(
                                "Extension with id '{}' not found",
                                ext_id
                            )));
                        }
                        error!(
                            "[ProtectiveMeasureService] Failed to update extension: {:?}",
                            e
                        );
                        return Err(AppError::InternalServerError);
                    }
                } else {
                    let create = CreateExtension {
                        extension_number: ext.extension_number,
                        extension_date: ext.extension_date,
                        new_valid_until: ext.new_valid_until,
                        notes: ext.notes.clone(),
                    };

                    if let Err(e) = self
                        .extension_repository
                        .create_extension_with_tx(&mut tx, id, &create)
                        .await
                    {
                        if let sqlx::Error::Database(db_err) = &e
                            && let Some(app_err) = map_constraint(
                                db_err.constraint(),
                                &[(
                                    "fk_extensions_protective_measure",
                                    "Error updating protective measure: extension protective_measure_id not found",
                                )],
                            )
                        {
                            return Err(app_err);
                        }
                        error!(
                            "[ProtectiveMeasureService] Failed to create extension: {:?}",
                            e
                        );
                        return Err(AppError::InternalServerError);
                    }
                }
            }
        }

        tx.commit().await.map_err(|e| {
            error!(
                "[ProtectiveMeasureService] Failed to commit transaction: {:?}",
                e
            );
            AppError::InternalServerError
        })?;

        info!(
            "[ProtectiveMeasureService] Protective measure updated successfully: {}",
            id
        );
        Ok(ApiResponse::success(updated).into_response())
    }

    pub async fn delete_protective_measure_by_id(
        &self,
        id: Uuid,
        req: HttpRequest,
    ) -> Result<HttpResponse, AppError> {
        info!(
            "[ProtectiveMeasureService] Deleting protective measure: {}",
            id
        );

        let claims = extract_claims(&req)?;

        let measure = match self
            .measure_repository
            .get_protective_measure_by_id(id)
            .await
        {
            Ok(m) => m,
            Err(sqlx::Error::RowNotFound) => {
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

        let policies = get_user_policies_with_defaults(&**self.user_repository, &claims).await?;
        check_policy(
            &claims,
            POLICY_DELETE_PROTECTIVE_MEASURES,
            victim.city_id,
            &policies,
        )?;

        match self
            .measure_repository
            .delete_protective_measure_by_id(id)
            .await
        {
            Ok(deleted_measure) => {
                info!(
                    "[ProtectiveMeasureService] Protective measure deleted successfully: {}",
                    id
                );
                Ok(ApiResponse::success(deleted_measure).into_response())
            }
            Err(sqlx::Error::RowNotFound) => Err(AppError::NotFound(format!(
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
        measure: crate::core::entities::protective_measures::ProtectiveMeasure,
        extensions: Vec<crate::core::entities::protective_measures::ProtectiveMeasureExtension>,
    ) -> Result<ProtectiveMeasureWithExtensionsAndEntities, AppError> {
        let victim = self
            .victim_repository
            .get_victim_by_id(measure.victim_id)
            .await
            .map_err(|e| match e {
                sqlx::Error::RowNotFound => AppError::NotFound(format!(
                    "Victim with id '{}' not found",
                    measure.victim_id
                )),
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
                sqlx::Error::RowNotFound => AppError::NotFound(format!(
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
                sqlx::Error::RowNotFound => AppError::NotFound(format!(
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
}
