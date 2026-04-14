use log::{error, info};
use std::sync::Arc;
use uuid::Uuid;

use crate::core::commands::cities::{CreateCity, UpdateCity};
use crate::core::contracts::repository::cities::CityRepository;
use crate::core::contracts::repository::users::UserRepository;
use crate::core::entities::auth::ClaimsToUserToken;
use crate::core::entities::cities::City;
use crate::core::entities::common::PaginatedResult;
use crate::utils::errors::AppError;
use crate::core::contracts::repository::error::RepositoryError;
use crate::services::auth_context::AuthContext;
use crate::utils::pagination::Pagination;
use crate::core::value_objects::policies::Policy;
use crate::core::value_objects::profiles::Profile;
use crate::validators::city_validator::CityValidator;

pub struct CityService {
    city_repository: Arc<dyn CityRepository>,
    user_repository: Arc<dyn UserRepository>,
}

impl CityService {
    pub fn new(
        city_repository: Arc<dyn CityRepository>,
        user_repository: Arc<dyn UserRepository>,
    ) -> Self {
        Self {
            city_repository,
            user_repository,
        }
    }

    pub async fn create_city(
        &self,
        city: CreateCity,
        claims: &ClaimsToUserToken,
    ) -> Result<City, AppError> {
        info!("[CityService] Starting city creation: {}", city.name);

        if claims.profile != Profile::Root {
            error!("[CityService] Only ROOT can create cities");
            return Err(AppError::Forbidden(
                "Only ROOT can create cities".to_string(),
            ));
        }

        CityValidator::validate_fields(
            &city.name,
            &city.state,
            &city.battalion,
            "Error adding city",
        )?;

        match self
            .city_repository
            .get_city_by_name_and_battalion(&city.name, &city.battalion)
            .await
        {
            Ok(Some(_existing_city)) => {
                error!(
                    "[CityService] City already exists with name '{}' and battalion '{}'",
                    city.name, city.battalion
                );
                return Err(AppError::BadRequest(format!(
                    "Error adding city: a city with name '{}' and battalion '{}' already exists",
                    city.name, city.battalion
                )));
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
                info!(
                    "[CityService] City created successfully with ID: {}",
                    city.id
                );
                Ok(city)
            }
            Err(e) => {
                error!("[CityService] Failed to save city: {:?}", e);
                Err(AppError::InternalServerError)
            }
        }
    }

    pub async fn get_city_by_id(
        &self,
        id: Uuid,
        claims: &ClaimsToUserToken,
    ) -> Result<City, AppError> {
        info!(
            "[CityService] Starting find city by id process for id: {}",
            id
        );

        let auth = AuthContext::load(&*self.user_repository, claims).await?;

        match self.city_repository.get_city_by_id(id).await {
            Ok(city) => {
                if claims.profile != Profile::Root {
                    auth.check_policy(&Policy::ReadCities, city.id)?;
                }
                info!("[CityService] City with id {} found successfully", id);
                Ok(city)
            }
            Err(RepositoryError::NotFound) => {
                info!("[CityService] City with id {} not found", id);
                Err(AppError::NotFound(format!(
                    "City with id '{}' not found",
                    id
                )))
            }
            Err(e) => {
                error!("[CityService] Database error while finding city: {:?}", e);
                Err(AppError::InternalServerError)
            }
        }
    }

    pub async fn get_all_cities(
        &self,
        pagination: Pagination,
        claims: &ClaimsToUserToken,
    ) -> Result<PaginatedResult<City>, AppError> {
        info!("[CityService] Starting process to get all cities");

        let auth = AuthContext::load(&*self.user_repository, claims).await?;
        let allowed_cities = auth.allowed_cities(&Policy::ReadCities);

        let total_items = self
            .city_repository
            .count_cities(allowed_cities.as_deref())
            .await
            .map_err(|_| AppError::InternalServerError)?;

        let cities = self
            .city_repository
            .get_cities_paginated(
                allowed_cities.as_deref(),
                pagination.page_size,
                pagination.offset,
            )
            .await
            .map_err(|_| AppError::InternalServerError)?;

        info!(
            "[CityService] Successfully retrieved {} cities",
            cities.len()
        );

        Ok(PaginatedResult {
            items: cities,
            page: pagination.page,
            page_size: pagination.page_size,
            total_items,
        })
    }

    pub async fn update_city_by_id(
        &self,
        data: UpdateCity,
        id: Uuid,
        claims: &ClaimsToUserToken,
    ) -> Result<City, AppError> {
        info!("[CityService] Starting city update for id: {}", id);

        if claims.profile != Profile::Root {
            error!("[CityService] Only ROOT can update cities");
            return Err(AppError::Forbidden(
                "Only ROOT can update cities".to_string(),
            ));
        }

        CityValidator::validate_fields(
            &data.name,
            &data.state,
            &data.battalion,
            "Error updating city",
        )?;

        match self
            .city_repository
            .get_city_by_name_and_battalion(&data.name, &data.battalion)
            .await
        {
            Ok(Some(existing_city)) => {
                if existing_city.id != id {
                    error!(
                        "[CityService] Another city already exists with name '{}' and battalion '{}'",
                        data.name, data.battalion
                    );
                    return Err(AppError::BadRequest(format!(
                        "Error updating city: a city with name '{}' and battalion '{}' already exists",
                        data.name, data.battalion
                    )));
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
                info!(
                    "[CityService] City updated successfully with ID: {}",
                    city.id
                );
                Ok(city)
            }
            Err(RepositoryError::NotFound) => {
                error!("[CityService] City with id {} not found for update", id);
                Err(AppError::NotFound(format!(
                    "City with id '{}' not found",
                    id
                )))
            }
            Err(e) => {
                error!("[CityService] Error updating city in database: {:?}", e);
                Err(AppError::InternalServerError)
            }
        }
    }

    pub async fn delete_city_by_id(
        &self,
        id: Uuid,
        claims: &ClaimsToUserToken,
    ) -> Result<City, AppError> {
        info!(
            "[CityService] Starting process to delete city with id: {}",
            id
        );

        if claims.profile != Profile::Root {
            error!("[CityService] Only ROOT can delete cities");
            return Err(AppError::Forbidden(
                "Only ROOT can delete cities".to_string(),
            ));
        }

        match self.city_repository.delete_city_by_id(id).await {
            Ok(deleted_city) => {
                info!("[CityService] City with id {} deleted successfully", id);
                Ok(deleted_city)
            }
            Err(RepositoryError::NotFound) => {
                info!("[CityService] City with id {} not found for deletion", id);
                Err(AppError::NotFound(format!(
                    "City with id '{}' not found",
                    id
                )))
            }
            Err(e) => {
                error!("[CityService] Failed to delete city: {:?}", e);
                Err(AppError::InternalServerError)
            }
        }
    }
}
