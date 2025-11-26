use actix_web::{web, HttpResponse, HttpRequest, HttpMessage};
use log::{error, info};
use uuid::Uuid;

use crate::core::entities::cities::{CreateCity, UpdateCity};
use crate::core::contracts::repository::cities::CityRepository;
use crate::repositories::cities::PgCityRepository;
use crate::repositories::users::PgUserRepository;
use crate::core::contracts::repository::users::UserRepository;
use crate::utils::{
    errors::AppError,
    responses::ApiResponse,
    validations::{
        validate_required_fields, is_valid_city_name, is_valid_battalion, is_valid_state,
        VALID_CITIES, VALID_BATTALIONS, VALID_STATES, PROFILE_ROOT,
        POLICY_READ_CITIES
    }
};
use crate::core::entities::auth::ClaimsToUserToken;
use crate::utils::authorization::{check_policy, get_allowed_cities_for_policy};

pub struct CityService {
    city_repository: web::Data<PgCityRepository>,
    user_repository: web::Data<PgUserRepository>,
}

impl CityService {
    pub fn new(city_repository: web::Data<PgCityRepository>, user_repository: web::Data<PgUserRepository>) -> Self {
        Self { city_repository, user_repository }
    }

    pub async fn create_city(&self, city: CreateCity, req: HttpRequest) -> Result<HttpResponse, AppError> {
        info!("[CityService] Starting city creation: {}", city.name);

        let claims = self.get_claims(&req)?;
        if claims.profile != PROFILE_ROOT {
            error!("[CityService] Only ROOT can create cities");
            return Err(AppError::Forbidden("Only ROOT can create cities".to_string()));
        }

        validate_required_fields(&[
            ("name", city.name.is_empty()),
            ("state", city.state.is_empty()),
            ("battalion", city.battalion.is_empty()),
        ], "Error adding city: ")?;

        if !is_valid_city_name(&city.name) {
            error!("[CityService] Invalid city name: {}", city.name);
            return Err(AppError::BadRequest(
                format!("Error adding city: invalid city name '{}'. Valid cities: {:?}", city.name, VALID_CITIES)
            ));
        }

        if !is_valid_state(&city.state) {
            error!("[CityService] Invalid state: {}", city.state);
            return Err(AppError::BadRequest(
                format!("Error adding city: invalid state '{}'. Valid states: {:?}", city.state, VALID_STATES)
            ));
        }

        if !is_valid_battalion(&city.battalion) {
            error!("[CityService] Invalid battalion: {}", city.battalion);
            return Err(AppError::BadRequest(
                format!("Error adding city: invalid battalion '{}'. Valid battalions: {:?}", city.battalion, VALID_BATTALIONS)
            ));
        }

        // Check if city already exists with the same name and battalion
        match self.city_repository.get_city_by_name_and_battalion(&city.name, &city.battalion).await {
            Ok(Some(_existing_city)) => {
                error!("[CityService] City already exists with name '{}' and battalion '{}'", city.name, city.battalion);
                return Err(AppError::BadRequest(
                    format!("Error adding city: a city with name '{}' and battalion '{}' already exists", city.name, city.battalion)
                ));
            }
            Ok(None) => {
                info!("[CityService] No duplicate city found, proceeding with creation");
            }
            Err(e) => {
                error!("[CityService] Error checking for duplicate city: {:?}", e);
                return Err(AppError::InternalServerError);
            }
        }

        info!("[CityService] Saving city to database");

        match self.city_repository.create_city(city).await {
            Ok(city) => {
                info!("[CityService] City created successfully with ID: {}", city.id);
                Ok(ApiResponse::created(city).into_response())
            }
            Err(e) => {
                error!("[CityService] Failed to save city: {:?}", e);
                Err(AppError::InternalServerError)
            }
        }
    }

    pub async fn get_city_by_id(&self, id: Uuid, req: HttpRequest) -> Result<HttpResponse, AppError> {
        info!("[CityService] Starting find city by id process for id: {}", id);

        let claims = self.get_claims(&req)?;
        let policies = self.get_user_policies(&claims).await?;

        match self.city_repository.get_city_by_id(id).await {
            Ok(city) => {
                if claims.profile != PROFILE_ROOT {
                    check_policy(&claims, POLICY_READ_CITIES, city.id, &policies)?;
                }
                info!("[CityService] City with id {} found successfully", id);
                Ok(ApiResponse::success(city).into_response())
            }
            Err(sqlx::Error::RowNotFound) => {
                info!("[CityService] City with id {} not found", id);
                Err(AppError::NotFound(format!("City with id '{}' not found", id)))
            }
            Err(e) => {
                error!("[CityService] Database error while finding city: {:?}", e);
                Err(AppError::InternalServerError)
            }
        }
    }

    pub async fn get_all_cities(&self, req: HttpRequest) -> Result<HttpResponse, AppError> {
        info!("[CityService] Starting process to get all cities");

        let claims = self.get_claims(&req)?;
        let policies = self.get_user_policies(&claims).await?;

        match self.city_repository.get_all_cities().await {
            Ok(cities) => {
                if claims.profile == PROFILE_ROOT {
                    info!("[CityService] Successfully retrieved {} cities (ROOT)", cities.len());
                    return Ok(ApiResponse::success(cities).into_response());
                }

                let allowed = get_allowed_cities_for_policy(&claims, POLICY_READ_CITIES, &policies);
                let filtered = if let Some(allowed_cities) = allowed {
                    cities.into_iter().filter(|c| allowed_cities.contains(&c.id)).collect()
                } else {
                    cities
                };
                info!("[CityService] Successfully retrieved {} cities (filtered)", filtered.len());
                Ok(ApiResponse::success(filtered).into_response())
            }
            Err(e) => {
                error!("[CityService] Failed to retrieve cities: {:?}", e);
                Err(AppError::InternalServerError)
            }
        }
    }

    pub async fn update_city_by_id(&self, data: UpdateCity, id: Uuid, req: HttpRequest) -> Result<HttpResponse, AppError> {
        info!("[CityService] Starting city update for id: {}", id);

        let claims = self.get_claims(&req)?;
        if claims.profile != PROFILE_ROOT {
            error!("[CityService] Only ROOT can update cities");
            return Err(AppError::Forbidden("Only ROOT can update cities".to_string()));
        }

        validate_required_fields(&[
            ("name", data.name.is_empty()),
            ("state", data.state.is_empty()),
            ("battalion", data.battalion.is_empty()),
        ], "Error updating city: ")?;

        if !is_valid_city_name(&data.name) {
            error!("[CityService] Invalid city name: {}", data.name);
            return Err(AppError::BadRequest(
                format!("Error updating city: invalid city name '{}'. Valid cities: {:?}", data.name, VALID_CITIES)
            ));
        }

        if !is_valid_state(&data.state) {
            error!("[CityService] Invalid state: {}", data.state);
            return Err(AppError::BadRequest(
                format!("Error updating city: invalid state '{}'. Valid states: {:?}", data.state, VALID_STATES)
            ));
        }

        if !is_valid_battalion(&data.battalion) {
            error!("[CityService] Invalid battalion: {}", data.battalion);
            return Err(AppError::BadRequest(
                format!("Error updating city: invalid battalion '{}'. Valid battalions: {:?}", data.battalion, VALID_BATTALIONS)
            ));
        }

        // Check if another city already exists with the same name and battalion
        match self.city_repository.get_city_by_name_and_battalion(&data.name, &data.battalion).await {
            Ok(Some(existing_city)) => {
                // Only error if it's a different city (not the one being updated)
                if existing_city.id != id {
                    error!("[CityService] Another city already exists with name '{}' and battalion '{}'", data.name, data.battalion);
                    return Err(AppError::BadRequest(
                        format!("Error updating city: a city with name '{}' and battalion '{}' already exists", data.name, data.battalion)
                    ));
                }
            }
            Ok(None) => {
                info!("[CityService] No duplicate city found, proceeding with update");
            }
            Err(e) => {
                error!("[CityService] Error checking for duplicate city: {:?}", e);
                return Err(AppError::InternalServerError);
            }
        }

        info!("[CityService] Updating city in database");

        match self.city_repository.update_city_by_id(data, id).await {
            Ok(city) => {
                info!("[CityService] City updated successfully with ID: {}", city.id);
                Ok(ApiResponse::success(city).into_response())
            }
            Err(sqlx::Error::RowNotFound) => {
                error!("[CityService] City with id {} not found for update", id);
                Err(AppError::NotFound(
                    format!("City with id '{}' not found", id)
                ))
            }
            Err(e) => {
                error!("[CityService] Error updating city in database: {:?}", e);
                Err(AppError::InternalServerError)
            }
        }
    }

    pub async fn delete_city_by_id(&self, id: Uuid, req: HttpRequest) -> Result<HttpResponse, AppError> {
        info!("[CityService] Starting process to delete city with id: {}", id);

        let claims = self.get_claims(&req)?;
        if claims.profile != PROFILE_ROOT {
            error!("[CityService] Only ROOT can delete cities");
            return Err(AppError::Forbidden("Only ROOT can delete cities".to_string()));
        }

        match self.city_repository.delete_city_by_id(id).await {
            Ok(deleted_city) => {
                info!("[CityService] City with id {} deleted successfully", id);
                Ok(ApiResponse::success(deleted_city).into_response())
            }
            Err(sqlx::Error::RowNotFound) => {
                info!("[CityService] City with id {} not found for deletion", id);
                Err(AppError::NotFound(format!("City with id '{}' not found", id)))
            }
            Err(e) => {
                error!("[CityService] Failed to delete city: {:?}", e);
                Err(AppError::InternalServerError)
            }
        }
    }
}

impl CityService {
    fn get_claims(&self, req: &HttpRequest) -> Result<ClaimsToUserToken, AppError> {
        req.extensions()
            .get::<ClaimsToUserToken>()
            .cloned()
            .ok_or_else(|| {
                error!("[CityService] No claims found in request");
                AppError::Unauthorized("Unauthorized".to_string())
            })
    }

    async fn get_user_policies(&self, claims: &ClaimsToUserToken) -> Result<serde_json::Value, AppError> {
        if claims.profile == PROFILE_ROOT {
            return Ok(serde_json::json!({}));
        }
        if let Ok(uid) = Uuid::parse_str(&claims.id) {
            match self.user_repository.get_user_policies_json_by_id(uid).await {
                Ok(p) => return Ok(p),
                Err(sqlx::Error::RowNotFound) => {}
                Err(_) => return Err(AppError::InternalServerError),
            }
        }
        if let Some(cid_str) = &claims.city_id {
            if let Ok(cid) = Uuid::parse_str(cid_str) {
                let mut defaults = std::collections::HashMap::new();
                defaults.insert(POLICY_READ_CITIES.to_string(), vec![cid]);
                return Ok(serde_json::json!(defaults));
            }
        }
        Ok(serde_json::json!({}))
    }
}
