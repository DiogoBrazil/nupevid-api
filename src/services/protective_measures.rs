use actix_web::{web, HttpResponse, HttpRequest, HttpMessage};
use log::{error, info};
use uuid::Uuid;

use crate::core::entities::protective_measures::{
    CreateProtectiveMeasure,
    UpdateProtectiveMeasure
};
use crate::core::entities::auth::ClaimsToUserToken;
use crate::core::contracts::repository::protective_measures::ProtectiveMeasureRepository;
use crate::core::contracts::repository::victims::VictimRepository;
use crate::repositories::protective_measures::PgProtectiveMeasureRepository;
use crate::repositories::victims::PgVictimRepository;
use crate::utils::{
    errors::AppError,
    responses::ApiResponse,
    validations::{validate_required_fields, PROFILE_ROOT}
};

pub struct ProtectiveMeasureService {
    measure_repository: web::Data<PgProtectiveMeasureRepository>,
    victim_repository: web::Data<PgVictimRepository>,
}

impl ProtectiveMeasureService {
    pub fn new(
        measure_repository: web::Data<PgProtectiveMeasureRepository>,
        victim_repository: web::Data<PgVictimRepository>,
    ) -> Self {
        Self { measure_repository, victim_repository }
    }

    pub async fn create_protective_measure(&self, measure: CreateProtectiveMeasure, req: HttpRequest) -> Result<HttpResponse, AppError> {
        info!("[ProtectiveMeasureService] Starting protective measure creation for victim: {}", measure.victim_id);

        let claims = self.get_claims(&req)?;

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

        self.validate_city_access(&claims, &victim.city_id)?;

        validate_required_fields(&[
            ("process_number", measure.process_number.is_empty()),
            ("judicial_authority", measure.judicial_authority.is_empty()),
        ], "Error adding protective measure: ")?;

        // Business rule: Check if active measure already exists for this victim
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

        let claims = self.get_claims(&req)?;

        match self.measure_repository.get_protective_measure_by_id(id).await {
            Ok(measure) => {
                let victim = self.victim_repository.get_victim_by_id(measure.victim_id).await
                    .map_err(|e| {
                        error!("[ProtectiveMeasureService] Error fetching victim: {:?}", e);
                        AppError::InternalServerError
                    })?;

                self.validate_city_access(&claims, &victim.city_id)?;

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

        let claims = self.get_claims(&req)?;

        let measures = if claims.profile == PROFILE_ROOT {
            self.measure_repository.get_all_protective_measures().await
        } else {
            match self.measure_repository.get_all_protective_measures().await {
                Ok(all_measures) => {
                    let city_id = self.get_user_city_id(&claims)?;
                    let mut filtered = Vec::new();

                    for measure in all_measures {
                        if let Ok(victim) = self.victim_repository.get_victim_by_id(measure.victim_id).await {
                            if victim.city_id == city_id {
                                filtered.push(measure);
                            }
                        }
                    }

                    Ok(filtered)
                }
                Err(e) => Err(e),
            }
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

        let claims = self.get_claims(&req)?;

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

        self.validate_city_access(&claims, &victim.city_id)?;

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

        let claims = self.get_claims(&req)?;

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

        self.validate_city_access(&claims, &existing_victim.city_id)?;

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

            self.validate_city_access(&claims, &new_victim.city_id)?;
        }

        validate_required_fields(&[
            ("process_number", data.process_number.is_empty()),
            ("judicial_authority", data.judicial_authority.is_empty()),
        ], "Error updating protective measure: ")?;

        // Business rule: Check if setting to active when another active measure exists
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

        let claims = self.get_claims(&req)?;

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

        self.validate_city_access(&claims, &victim.city_id)?;

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

    // Helper methods
    fn get_claims(&self, req: &HttpRequest) -> Result<ClaimsToUserToken, AppError> {
        req.extensions()
            .get::<ClaimsToUserToken>()
            .cloned()
            .ok_or_else(|| {
                error!("[ProtectiveMeasureService] No claims found in request");
                AppError::Unauthorized("Unauthorized".to_string())
            })
    }

    fn get_user_city_id(&self, claims: &ClaimsToUserToken) -> Result<Uuid, AppError> {
        claims.city_id
            .as_ref()
            .and_then(|id| Uuid::parse_str(id).ok())
            .ok_or_else(|| {
                error!("[ProtectiveMeasureService] User has no city_id in claims");
                AppError::Forbidden("User must be associated with a city".to_string())
            })
    }

    fn validate_city_access(&self, claims: &ClaimsToUserToken, city_id: &Uuid) -> Result<(), AppError> {
        if claims.profile == PROFILE_ROOT {
            return Ok(());
        }

        let user_city_id = self.get_user_city_id(claims)?;

        if &user_city_id != city_id {
            error!("[ProtectiveMeasureService] User city {} does not match requested city {}", user_city_id, city_id);
            return Err(AppError::Forbidden("Access denied to this city's data".to_string()));
        }

        Ok(())
    }
}
