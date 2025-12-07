use actix_web::{web, HttpResponse, HttpRequest};
use log::{error, info};
use uuid::Uuid;

use crate::core::entities::protective_measures::{
    CreateProtectiveMeasure,
    UpdateProtectiveMeasure
};

use crate::core::contracts::repository::protective_measures::ProtectiveMeasureRepository;
use crate::core::contracts::repository::victims::VictimRepository;
use crate::repositories::protective_measures::PgProtectiveMeasureRepository;
use crate::repositories::victims::PgVictimRepository;
use crate::repositories::users::PgUserRepository;

use crate::utils::{
    errors::AppError,
    responses::ApiResponse,
    authorization::{check_policy, get_allowed_cities_for_policy},
    service_helpers::{extract_claims, get_user_policies_with_defaults}
};
use crate::validators::{
    common::{
        POLICY_CREATE_PROTECTIVE_MEASURES, POLICY_READ_PROTECTIVE_MEASURES,
        POLICY_UPDATE_PROTECTIVE_MEASURES, POLICY_DELETE_PROTECTIVE_MEASURES
    },
    protective_measure_validator::ProtectiveMeasureValidator
};

pub struct ProtectiveMeasureService {
    measure_repository: web::Data<PgProtectiveMeasureRepository>,
    victim_repository: web::Data<PgVictimRepository>,
    user_repository: web::Data<PgUserRepository>,
}

impl ProtectiveMeasureService {
    pub fn new(
        measure_repository: web::Data<PgProtectiveMeasureRepository>,
        victim_repository: web::Data<PgVictimRepository>,
        user_repository: web::Data<PgUserRepository>,
    ) -> Self {
        Self { measure_repository, victim_repository, user_repository }
    }

    pub async fn create_protective_measure(&self, measure: CreateProtectiveMeasure, req: HttpRequest) -> Result<HttpResponse, AppError> {
        info!("[ProtectiveMeasureService] Starting protective measure creation for victim: {}", measure.victim_id);

        let claims = extract_claims(&req)?;

        let victim = match self.victim_repository.get_victim_by_id(measure.victim_id).await {
            Ok(v) => v,
            Err(sqlx::Error::RowNotFound) => {
                return Err(AppError::NotFound(format!("Victim with id '{}' not found", measure.victim_id)));
            }
            Err(e) => {
                error!("[ProtectiveMeasureService] Error checking victim: {:?}", e);
                return Err(AppError::InternalServerError);
            }
        };

        let policies = get_user_policies_with_defaults(&**self.user_repository, &claims).await?;
        check_policy(&claims, POLICY_CREATE_PROTECTIVE_MEASURES, victim.city_id, &policies)?;

        ProtectiveMeasureValidator::validate_required_fields(
            &measure.process_number,
            &measure.judicial_authority,
            "Error adding protective measure"
        )?;

        if measure.is_active {
            let active_exists = self.measure_repository
                .check_active_measure_exists_for_victim(measure.victim_id, Uuid::nil())
                .await
                .map_err(|e| {
                    error!("[ProtectiveMeasureService] Failed to check for active measure: {:?}", e);
                    AppError::InternalServerError
                })?;

            if active_exists {
                error!("[ProtectiveMeasureService] Active measure already exists for victim: {}", measure.victim_id);
                return Err(AppError::BadRequest(
                    "Error adding protective measure: victim already has an active protective measure".to_string()
                ));
            }
        }

        info!("[ProtectiveMeasureService] Saving protective measure to database");

        match self.measure_repository.create_protective_measure(measure).await {
            Ok(measure) => {
                info!("[ProtectiveMeasureService] Protective measure created successfully with ID: {}", measure.id);
                Ok(ApiResponse::created(measure).into_response())
            }
            Err(e) => {
                error!("[ProtectiveMeasureService] Failed to save protective measure: {:?}", e);
                Err(AppError::InternalServerError)
            }
        }
    }

    pub async fn get_protective_measure_by_id(&self, id: Uuid, req: HttpRequest) -> Result<HttpResponse, AppError> {
        info!("[ProtectiveMeasureService] Getting protective measure by id: {}", id);

        let claims = extract_claims(&req)?;

        match self.measure_repository.get_protective_measure_by_id(id).await {
            Ok(measure) => {
                let victim = self.victim_repository.get_victim_by_id(measure.victim_id).await
                    .map_err(|e| {
                        error!("[ProtectiveMeasureService] Error fetching victim: {:?}", e);
                        AppError::InternalServerError
                    })?;

                let policies = get_user_policies_with_defaults(&**self.user_repository, &claims).await?;
                check_policy(&claims, POLICY_READ_PROTECTIVE_MEASURES, victim.city_id, &policies)?;

                info!("[ProtectiveMeasureService] Protective measure found: {}", id);
                Ok(ApiResponse::success(measure).into_response())
            }
            Err(sqlx::Error::RowNotFound) => {
                Err(AppError::NotFound(format!("Protective measure with id '{}' not found", id)))
            }
            Err(e) => {
                error!("[ProtectiveMeasureService] Database error: {:?}", e);
                Err(AppError::InternalServerError)
            }
        }
    }

    pub async fn get_all_protective_measures(&self, req: HttpRequest) -> Result<HttpResponse, AppError> {
        info!("[ProtectiveMeasureService] Getting all protective measures");

        let claims = extract_claims(&req)?;

        let policies = get_user_policies_with_defaults(&**self.user_repository, &claims).await?;
        let measures = if let Some(allowed_cities) = get_allowed_cities_for_policy(&claims, POLICY_READ_PROTECTIVE_MEASURES, &policies) {
            match self.measure_repository.get_all_protective_measures().await {
                Ok(all_measures) => {
                    let mut filtered = Vec::new();
                    for measure in all_measures {
                        if let Ok(victim) = self.victim_repository.get_victim_by_id(measure.victim_id).await {
                            if allowed_cities.contains(&victim.city_id) {
                                filtered.push(measure);
                            }
                        }
                    }
                    Ok(filtered)
                }
                Err(e) => Err(e),
            }
        } else {
            self.measure_repository.get_all_protective_measures().await
        };

        match measures {
            Ok(measures) => {
                info!("[ProtectiveMeasureService] Successfully retrieved {} measures", measures.len());
                Ok(ApiResponse::success(measures).into_response())
            }
            Err(e) => {
                error!("[ProtectiveMeasureService] Failed to retrieve measures: {:?}", e);
                Err(AppError::InternalServerError)
            }
        }
    }

    pub async fn get_protective_measures_by_victim(&self, victim_id: Uuid, req: HttpRequest) -> Result<HttpResponse, AppError> {
        info!("[ProtectiveMeasureService] Getting measures for victim: {}", victim_id);

        let claims = extract_claims(&req)?;

        let victim = match self.victim_repository.get_victim_by_id(victim_id).await {
            Ok(v) => v,
            Err(sqlx::Error::RowNotFound) => {
                return Err(AppError::NotFound(format!("Victim with id '{}' not found", victim_id)));
            }
            Err(e) => {
                error!("[ProtectiveMeasureService] Error checking victim: {:?}", e);
                return Err(AppError::InternalServerError);
            }
        };

        let policies = get_user_policies_with_defaults(&**self.user_repository, &claims).await?;
        check_policy(&claims, POLICY_READ_PROTECTIVE_MEASURES, victim.city_id, &policies)?;

        match self.measure_repository.get_protective_measures_by_victim(victim_id).await {
            Ok(measures) => {
                info!("[ProtectiveMeasureService] Found {} measures for victim", measures.len());
                Ok(ApiResponse::success(measures).into_response())
            }
            Err(e) => {
                error!("[ProtectiveMeasureService] Failed to retrieve measures: {:?}", e);
                Err(AppError::InternalServerError)
            }
        }
    }

    pub async fn update_protective_measure_by_id(&self, data: UpdateProtectiveMeasure, id: Uuid, req: HttpRequest) -> Result<HttpResponse, AppError> {
        info!("[ProtectiveMeasureService] Updating protective measure: {}", id);

        let claims = extract_claims(&req)?;

        let existing_measure = match self.measure_repository.get_protective_measure_by_id(id).await {
            Ok(m) => m,
            Err(sqlx::Error::RowNotFound) => {
                return Err(AppError::NotFound(format!("Protective measure with id '{}' not found", id)));
            }
            Err(e) => {
                error!("[ProtectiveMeasureService] Error fetching measure: {:?}", e);
                return Err(AppError::InternalServerError);
            }
        };

        let existing_victim = self.victim_repository.get_victim_by_id(existing_measure.victim_id).await
            .map_err(|e| {
                error!("[ProtectiveMeasureService] Error fetching existing victim: {:?}", e);
                AppError::InternalServerError
            })?;

        let policies = get_user_policies_with_defaults(&**self.user_repository, &claims).await?;
        check_policy(&claims, POLICY_UPDATE_PROTECTIVE_MEASURES, existing_victim.city_id, &policies)?;

        if data.victim_id != existing_measure.victim_id {
            let new_victim = match self.victim_repository.get_victim_by_id(data.victim_id).await {
                Ok(v) => v,
                Err(sqlx::Error::RowNotFound) => {
                    return Err(AppError::NotFound(format!("Victim with id '{}' not found", data.victim_id)));
                }
                Err(e) => {
                    error!("[ProtectiveMeasureService] Error checking new victim: {:?}", e);
                    return Err(AppError::InternalServerError);
                }
            };

            check_policy(&claims, POLICY_UPDATE_PROTECTIVE_MEASURES, new_victim.city_id, &policies)?;
        }

        ProtectiveMeasureValidator::validate_required_fields(
            &data.process_number,
            &data.judicial_authority,
            "Error updating protective measure"
        )?;

        if data.is_active {
            let active_exists = self.measure_repository
                .check_active_measure_exists_for_victim(data.victim_id, id)
                .await
                .map_err(|e| {
                    error!("[ProtectiveMeasureService] Failed to check for active measure: {:?}", e);
                    AppError::InternalServerError
                })?;

            if active_exists {
                error!("[ProtectiveMeasureService] Active measure already exists for victim: {}", data.victim_id);
                return Err(AppError::BadRequest(
                    "Error updating protective measure: victim already has an active protective measure".to_string()
                ));
            }
        }

        match self.measure_repository.update_protective_measure_by_id(data, id).await {
            Ok(measure) => {
                info!("[ProtectiveMeasureService] Protective measure updated successfully: {}", id);
                Ok(ApiResponse::success(measure).into_response())
            }
            Err(sqlx::Error::RowNotFound) => {
                Err(AppError::NotFound(format!("Protective measure with id '{}' not found", id)))
            }
            Err(e) => {
                error!("[ProtectiveMeasureService] Failed to update measure: {:?}", e);
                Err(AppError::InternalServerError)
            }
        }
    }

    pub async fn delete_protective_measure_by_id(&self, id: Uuid, req: HttpRequest) -> Result<HttpResponse, AppError> {
        info!("[ProtectiveMeasureService] Deleting protective measure: {}", id);

        let claims = extract_claims(&req)?;

        let measure = match self.measure_repository.get_protective_measure_by_id(id).await {
            Ok(m) => m,
            Err(sqlx::Error::RowNotFound) => {
                return Err(AppError::NotFound(format!("Protective measure with id '{}' not found", id)));
            }
            Err(e) => {
                error!("[ProtectiveMeasureService] Error fetching measure: {:?}", e);
                return Err(AppError::InternalServerError);
            }
        };

        let victim = self.victim_repository.get_victim_by_id(measure.victim_id).await
            .map_err(|e| {
                error!("[ProtectiveMeasureService] Error fetching victim: {:?}", e);
                AppError::InternalServerError
            })?;

        let policies = get_user_policies_with_defaults(&**self.user_repository, &claims).await?;
        check_policy(&claims, POLICY_DELETE_PROTECTIVE_MEASURES, victim.city_id, &policies)?;

        match self.measure_repository.delete_protective_measure_by_id(id).await {
            Ok(deleted_measure) => {
                info!("[ProtectiveMeasureService] Protective measure deleted successfully: {}", id);
                Ok(ApiResponse::success(deleted_measure).into_response())
            }
            Err(sqlx::Error::RowNotFound) => {
                Err(AppError::NotFound(format!("Protective measure with id '{}' not found", id)))
            }
            Err(e) => {
                error!("[ProtectiveMeasureService] Failed to delete measure: {:?}", e);
                Err(AppError::InternalServerError)
            }
        }
    }
}
