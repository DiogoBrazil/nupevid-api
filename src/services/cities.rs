use actix_web::{web, HttpResponse};
use log::{error, info};
use uuid::Uuid;

use crate::core::entities::cities::{CreateCity, UpdateCity};
use crate::core::contracts::repository::cities::CityRepository;
use crate::repositories::cities::PgCityRepository;
use crate::utils::{
    errors::AppError,
    responses::ApiResponse,
    validations::{
        validate_required_fields, is_valid_city_name, is_valid_battalion, is_valid_state,
        VALID_CITIES, VALID_BATTALIONS, VALID_STATES
    }
};

pub struct CityService {
    city_repository: web::Data<PgCityRepository>,
}

impl CityService {
    pub fn new(city_repository: web::Data<PgCityRepository>) -> Self {
        Self { city_repository }
    }

    pub async fn create_city(&self, city: CreateCity) -> Result<HttpResponse, AppError> {
        info!("[CityService] Starting city creation: {}", city.name);

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

    pub async fn get_city_by_id(&self, id: Uuid) -> Result<HttpResponse, AppError> {
        info!("[CityService] Starting find city by id process for id: {}", id);

        match self.city_repository.get_city_by_id(id).await {
            Ok(city) => {
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

    pub async fn get_all_cities(&self) -> Result<HttpResponse, AppError> {
        info!("[CityService] Starting process to get all cities");

        match self.city_repository.get_all_cities().await {
            Ok(cities) => {
                info!("[CityService] Successfully retrieved {} cities", cities.len());
                Ok(ApiResponse::success(cities).into_response())
            }
            Err(e) => {
                error!("[CityService] Failed to retrieve cities: {:?}", e);
                Err(AppError::InternalServerError)
            }
        }
    }

    pub async fn update_city_by_id(&self, data: UpdateCity, id: Uuid) -> Result<HttpResponse, AppError> {
        info!("[CityService] Starting city update for id: {}", id);

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

    pub async fn delete_city_by_id(&self, id: Uuid) -> Result<HttpResponse, AppError> {
        info!("[CityService] Starting process to delete city with id: {}", id);

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
