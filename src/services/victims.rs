use actix_web::{web, HttpMessage, HttpRequest, HttpResponse};
use log::{error, info};
use uuid::Uuid;

use crate::core::contracts::repository::victims::VictimRepository;
use crate::core::entities::auth::ClaimsToUserToken;
use crate::core::entities::victims::{CreateVictim, UpdateVictim};
use crate::repositories::victims::PgVictimRepository;
use crate::utils::{errors::AppError, responses::ApiResponse, validations::{validate_required_fields, PROFILE_ROOT}};

pub struct VictimService {
    victim_repository: web::Data<PgVictimRepository>,
}

impl VictimService {
    pub fn new(victim_repository: web::Data<PgVictimRepository>) -> Self {
        Self { victim_repository }
    }

    pub async fn create_victim(
        &self,
        victim: CreateVictim,
        req: HttpRequest,
    ) -> Result<HttpResponse, AppError> {
        info!("[VictimService] Starting victim creation: {}", victim.full_name);

        let claims = self.get_claims(&req)?;

        self.validate_city_access(&claims, &victim.city_id)?;

        validate_required_fields(
            &[("full_name", victim.full_name.is_empty())],
            "Error adding victim: ",
        )?;

        info!("[VictimService] Saving victim to database");

        match self.victim_repository.create_victim(victim).await {
            Ok(victim_with_address) => {
                info!(
                    "[VictimService] Victim created successfully with ID: {}",
                    victim_with_address.id
                );
                Ok(ApiResponse::created(victim_with_address).into_response())
            }
            Err(e) => {
                error!("[VictimService] Failed to save victim: {:?}", e);
                Err(AppError::InternalServerError)
            }
        }
    }

    pub async fn get_victim_by_id(
        &self,
        id: Uuid,
        req: HttpRequest,
    ) -> Result<HttpResponse, AppError> {
        info!(
            "[VictimService] Starting find victim by id process for id: {}",
            id
        );

        let claims = self.get_claims(&req)?;

        match self.victim_repository.get_victim_by_id(id).await {
            Ok(victim_with_address) => {
                self.validate_city_access(&claims, &victim_with_address.city_id)?;

                info!("[VictimService] Victim with id {} found successfully", id);
                Ok(ApiResponse::success(victim_with_address).into_response())
            }
            Err(sqlx::Error::RowNotFound) => {
                info!("[VictimService] Victim with id {} not found", id);
                Err(AppError::NotFound(format!(
                    "Victim with id '{}' not found",
                    id
                )))
            }
            Err(e) => {
                error!(
                    "[VictimService] Database error while finding victim: {:?}",
                    e
                );
                Err(AppError::InternalServerError)
            }
        }
    }

    pub async fn get_all_victims(&self, req: HttpRequest) -> Result<HttpResponse, AppError> {
        info!("[VictimService] Starting process to get victims");

        let claims = self.get_claims(&req)?;

        let victims = if claims.profile == PROFILE_ROOT {
            info!("[VictimService] ROOT user - fetching all victims");
            self.victim_repository.get_all_victims().await
        } else {
            let city_id = self.get_user_city_id(&claims)?;
            info!("[VictimService] Fetching victims for city: {}", city_id);
            self.victim_repository.get_victims_by_city(city_id).await
        };

        match victims {
            Ok(victims_list) => {
                info!(
                    "[VictimService] Successfully retrieved {} victims",
                    victims_list.len()
                );
                Ok(ApiResponse::success(victims_list).into_response())
            }
            Err(e) => {
                error!("[VictimService] Failed to retrieve victims: {:?}", e);
                Err(AppError::InternalServerError)
            }
        }
    }

    pub async fn update_victim_by_id(
        &self,
        data: UpdateVictim,
        id: Uuid,
        req: HttpRequest,
    ) -> Result<HttpResponse, AppError> {
        info!("[VictimService] Starting victim update for id: {}", id);

        let claims = self.get_claims(&req)?;

        self.validate_city_access(&claims, &data.city_id)?;

        validate_required_fields(
            &[("full_name", data.full_name.is_empty())],
            "Error updating victim: ",
        )?;

        match self.victim_repository.get_victim_by_id(id).await {
            Ok(existing_victim) => {
                self.validate_city_access(&claims, &existing_victim.city_id)?;
            }
            Err(sqlx::Error::RowNotFound) => {
                return Err(AppError::NotFound(format!(
                    "Victim with id '{}' not found",
                    id
                )));
            }
            Err(e) => {
                error!("[VictimService] Error checking victim: {:?}", e);
                return Err(AppError::InternalServerError);
            }
        }

        info!("[VictimService] Updating victim in database");

        match self.victim_repository.update_victim_by_id(data, id).await {
            Ok(victim_with_address) => {
                info!(
                    "[VictimService] Victim updated successfully with ID: {}",
                    victim_with_address.id
                );
                Ok(ApiResponse::success(victim_with_address).into_response())
            }
            Err(sqlx::Error::RowNotFound) => {
                error!("[VictimService] Victim with id {} not found for update", id);
                Err(AppError::NotFound(format!(
                    "Victim with id '{}' not found",
                    id
                )))
            }
            Err(e) => {
                error!(
                    "[VictimService] Error updating victim in database: {:?}",
                    e
                );
                Err(AppError::InternalServerError)
            }
        }
    }

    pub async fn delete_victim_by_id(
        &self,
        id: Uuid,
        req: HttpRequest,
    ) -> Result<HttpResponse, AppError> {
        info!(
            "[VictimService] Starting process to delete victim with id: {}",
            id
        );

        let claims = self.get_claims(&req)?;

        match self.victim_repository.get_victim_by_id(id).await {
            Ok(victim) => {
                self.validate_city_access(&claims, &victim.city_id)?;
            }
            Err(sqlx::Error::RowNotFound) => {
                return Err(AppError::NotFound(format!(
                    "Victim with id '{}' not found",
                    id
                )));
            }
            Err(e) => {
                error!("[VictimService] Error checking victim: {:?}", e);
                return Err(AppError::InternalServerError);
            }
        }

        match self.victim_repository.delete_victim_by_id(id).await {
            Ok(deleted_victim) => {
                info!("[VictimService] Victim with id {} deleted successfully", id);
                Ok(ApiResponse::success(deleted_victim).into_response())
            }
            Err(sqlx::Error::RowNotFound) => {
                info!("[VictimService] Victim with id {} not found for deletion", id);
                Err(AppError::NotFound(format!(
                    "Victim with id '{}' not found",
                    id
                )))
            }
            Err(e) => {
                error!("[VictimService] Failed to delete victim: {:?}", e);
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
                error!("[VictimService] No claims found in request");
                AppError::Unauthorized("Unauthorized".to_string())
            })
    }

    fn get_user_city_id(&self, claims: &ClaimsToUserToken) -> Result<Uuid, AppError> {
        claims
            .city_id
            .as_ref()
            .and_then(|id| Uuid::parse_str(id).ok())
            .ok_or_else(|| {
                error!("[VictimService] User has no city_id in claims");
                AppError::Forbidden("User must be associated with a city".to_string())
            })
    }

    fn validate_city_access(&self, claims: &ClaimsToUserToken, city_id: &Uuid) -> Result<(), AppError> {
        if claims.profile == PROFILE_ROOT {
            return Ok(());
        }

        let user_city_id = self.get_user_city_id(claims)?;

        if &user_city_id != city_id {
            error!(
                "[VictimService] User city {} does not match requested city {}",
                user_city_id, city_id
            );
            return Err(AppError::Forbidden(
                "Access denied to this city's data".to_string(),
            ));
        }

        Ok(())
    }
}
