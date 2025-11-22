use actix_web::{web, HttpResponse, HttpRequest, HttpMessage};
use log::{error, info};
use uuid::Uuid;

use crate::core::entities::victims::{
    CreateVictim,
    UpdateVictim,
    CreateVictimAddress,
    UpdateVictimAddress
};
use crate::core::entities::auth::ClaimsToUserToken;
use crate::core::contracts::repository::victims::{
    VictimRepository,
    VictimAddressRepository
};
use crate::repositories::victims::{
    PgVictimRepository,
    PgVictimAddressRepository
};
use crate::utils::{
    errors::AppError,
    responses::ApiResponse,
    validations::validate_required_fields
};

pub struct VictimService {
    victim_repository: web::Data<PgVictimRepository>,
    victim_address_repository: web::Data<PgVictimAddressRepository>,
}

impl VictimService {
    pub fn new(
        victim_repository: web::Data<PgVictimRepository>,
        victim_address_repository: web::Data<PgVictimAddressRepository>,
    ) -> Self {
        Self { victim_repository, victim_address_repository }
    }

    pub async fn create_victim(&self, victim: CreateVictim, req: HttpRequest) -> Result<HttpResponse, AppError> {
        info!("[VictimService] Starting victim creation: {}", victim.full_name);

        // Get claims from request
        let claims = self.get_claims(&req)?;

        // Validate city access
        self.validate_city_access(&claims, &victim.city_id)?;

        // Validate required fields
        validate_required_fields(&[
            ("full_name", victim.full_name.is_empty()),
        ], "Error adding victim: ")?;

        info!("[VictimService] Saving victim to database");

        match self.victim_repository.create_victim(victim).await {
            Ok(victim) => {
                info!("[VictimService] Victim created successfully with ID: {}", victim.id);
                Ok(ApiResponse::created(victim).into_response())
            }
            Err(e) => {
                error!("[VictimService] Failed to save victim: {:?}", e);
                Err(AppError::InternalServerError)
            }
        }
    }

    pub async fn get_victim_by_id(&self, id: Uuid, req: HttpRequest) -> Result<HttpResponse, AppError> {
        info!("[VictimService] Starting find victim by id process for id: {}", id);

        let claims = self.get_claims(&req)?;

        match self.victim_repository.get_victim_by_id(id).await {
            Ok(victim) => {
                // Verify user has access to this victim's city
                self.validate_city_access(&claims, &victim.city_id)?;

                info!("[VictimService] Victim with id {} found successfully", id);
                Ok(ApiResponse::success(victim).into_response())
            }
            Err(sqlx::Error::RowNotFound) => {
                info!("[VictimService] Victim with id {} not found", id);
                Err(AppError::NotFound(format!("Victim with id '{}' not found", id)))
            }
            Err(e) => {
                error!("[VictimService] Database error while finding victim: {:?}", e);
                Err(AppError::InternalServerError)
            }
        }
    }

    pub async fn get_all_victims(&self, req: HttpRequest) -> Result<HttpResponse, AppError> {
        info!("[VictimService] Starting process to get victims");

        let claims = self.get_claims(&req)?;

        let victims = if claims.profile == "ROOT" {
            // ROOT sees all victims
            info!("[VictimService] ROOT user - fetching all victims");
            self.victim_repository.get_all_victims().await
        } else {
            // CITY_ADMIN and CITY_USER see only their city's victims
            let city_id = self.get_user_city_id(&claims)?;
            info!("[VictimService] Fetching victims for city: {}", city_id);
            self.victim_repository.get_victims_by_city(city_id).await
        };

        match victims {
            Ok(victims) => {
                info!("[VictimService] Successfully retrieved {} victims", victims.len());
                Ok(ApiResponse::success(victims).into_response())
            }
            Err(e) => {
                error!("[VictimService] Failed to retrieve victims: {:?}", e);
                Err(AppError::InternalServerError)
            }
        }
    }

    pub async fn update_victim_by_id(&self, data: UpdateVictim, id: Uuid, req: HttpRequest) -> Result<HttpResponse, AppError> {
        info!("[VictimService] Starting victim update for id: {}", id);

        let claims = self.get_claims(&req)?;

        // Validate city access for new city_id
        self.validate_city_access(&claims, &data.city_id)?;

        // Validate required fields
        validate_required_fields(&[
            ("full_name", data.full_name.is_empty()),
        ], "Error updating victim: ")?;

        // Check if victim exists and user has access
        match self.victim_repository.get_victim_by_id(id).await {
            Ok(existing_victim) => {
                self.validate_city_access(&claims, &existing_victim.city_id)?;
            }
            Err(sqlx::Error::RowNotFound) => {
                return Err(AppError::NotFound(format!("Victim with id '{}' not found", id)));
            }
            Err(e) => {
                error!("[VictimService] Error checking victim: {:?}", e);
                return Err(AppError::InternalServerError);
            }
        }

        info!("[VictimService] Updating victim in database");

        match self.victim_repository.update_victim_by_id(data, id).await {
            Ok(victim) => {
                info!("[VictimService] Victim updated successfully with ID: {}", victim.id);
                Ok(ApiResponse::success(victim).into_response())
            }
            Err(sqlx::Error::RowNotFound) => {
                error!("[VictimService] Victim with id {} not found for update", id);
                Err(AppError::NotFound(format!("Victim with id '{}' not found", id)))
            }
            Err(e) => {
                error!("[VictimService] Error updating victim in database: {:?}", e);
                Err(AppError::InternalServerError)
            }
        }
    }

    pub async fn delete_victim_by_id(&self, id: Uuid, req: HttpRequest) -> Result<HttpResponse, AppError> {
        info!("[VictimService] Starting process to delete victim with id: {}", id);

        let claims = self.get_claims(&req)?;

        // Check if victim exists and user has access
        match self.victim_repository.get_victim_by_id(id).await {
            Ok(victim) => {
                self.validate_city_access(&claims, &victim.city_id)?;
            }
            Err(sqlx::Error::RowNotFound) => {
                return Err(AppError::NotFound(format!("Victim with id '{}' not found", id)));
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
                Err(AppError::NotFound(format!("Victim with id '{}' not found", id)))
            }
            Err(e) => {
                error!("[VictimService] Failed to delete victim: {:?}", e);
                Err(AppError::InternalServerError)
            }
        }
    }

    pub async fn create_victim_address(&self, address: CreateVictimAddress, req: HttpRequest) -> Result<HttpResponse, AppError> {
        info!("[VictimService] Starting victim address creation for victim: {}", address.victim_id);

        let claims = self.get_claims(&req)?;

        // Verify victim exists and user has access
        match self.victim_repository.get_victim_by_id(address.victim_id).await {
            Ok(victim) => {
                self.validate_city_access(&claims, &victim.city_id)?;
            }
            Err(sqlx::Error::RowNotFound) => {
                return Err(AppError::NotFound(format!("Victim with id '{}' not found", address.victim_id)));
            }
            Err(e) => {
                error!("[VictimService] Error checking victim: {:?}", e);
                return Err(AppError::InternalServerError);
            }
        }

        info!("[VictimService] Saving victim address to database");

        match self.victim_address_repository.create_victim_address(address).await {
            Ok(address) => {
                info!("[VictimService] Victim address created successfully with ID: {}", address.id);
                Ok(ApiResponse::created(address).into_response())
            }
            Err(e) => {
                error!("[VictimService] Failed to save victim address: {:?}", e);
                Err(AppError::InternalServerError)
            }
        }
    }

    pub async fn get_victim_address(&self, victim_id: Uuid, req: HttpRequest) -> Result<HttpResponse, AppError> {
        info!("[VictimService] Getting address for victim: {}", victim_id);

        let claims = self.get_claims(&req)?;

        // Verify victim exists and user has access
        match self.victim_repository.get_victim_by_id(victim_id).await {
            Ok(victim) => {
                self.validate_city_access(&claims, &victim.city_id)?;
            }
            Err(sqlx::Error::RowNotFound) => {
                return Err(AppError::NotFound(format!("Victim with id '{}' not found", victim_id)));
            }
            Err(e) => {
                error!("[VictimService] Error checking victim: {:?}", e);
                return Err(AppError::InternalServerError);
            }
        }

        match self.victim_address_repository.get_victim_address_by_victim_id(victim_id).await {
            Ok(Some(address)) => {
                info!("[VictimService] Address found for victim: {}", victim_id);
                Ok(ApiResponse::success(address).into_response())
            }
            Ok(None) => {
                info!("[VictimService] No address found for victim: {}", victim_id);
                Err(AppError::NotFound(format!("No address found for victim '{}'", victim_id)))
            }
            Err(e) => {
                error!("[VictimService] Failed to retrieve victim address: {:?}", e);
                Err(AppError::InternalServerError)
            }
        }
    }

    pub async fn update_victim_address(&self, data: UpdateVictimAddress, address_id: Uuid, req: HttpRequest) -> Result<HttpResponse, AppError> {
        info!("[VictimService] Updating victim address: {}", address_id);

        let claims = self.get_claims(&req)?;

        // Get address and verify access
        let address = match self.victim_address_repository.get_victim_address_by_id(address_id).await {
            Ok(addr) => addr,
            Err(sqlx::Error::RowNotFound) => {
                return Err(AppError::NotFound(format!("Address with id '{}' not found", address_id)));
            }
            Err(e) => {
                error!("[VictimService] Error fetching address: {:?}", e);
                return Err(AppError::InternalServerError);
            }
        };

        // Verify victim exists and user has access
        match self.victim_repository.get_victim_by_id(address.victim_id).await {
            Ok(victim) => {
                self.validate_city_access(&claims, &victim.city_id)?;
            }
            Err(e) => {
                error!("[VictimService] Error checking victim: {:?}", e);
                return Err(AppError::InternalServerError);
            }
        }

        match self.victim_address_repository.update_victim_address_by_id(data, address_id).await {
            Ok(updated_address) => {
                info!("[VictimService] Address updated successfully: {}", address_id);
                Ok(ApiResponse::success(updated_address).into_response())
            }
            Err(sqlx::Error::RowNotFound) => {
                Err(AppError::NotFound(format!("Address with id '{}' not found", address_id)))
            }
            Err(e) => {
                error!("[VictimService] Failed to update address: {:?}", e);
                Err(AppError::InternalServerError)
            }
        }
    }

    // Helper methods for authorization
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
        claims.city_id
            .as_ref()
            .and_then(|id| Uuid::parse_str(id).ok())
            .ok_or_else(|| {
                error!("[VictimService] User has no city_id in claims");
                AppError::Forbidden("User must be associated with a city".to_string())
            })
    }

    fn validate_city_access(&self, claims: &ClaimsToUserToken, city_id: &Uuid) -> Result<(), AppError> {
        // ROOT has access to all cities
        if claims.profile == "ROOT" {
            return Ok(());
        }

        // Non-ROOT users must have city_id and it must match
        let user_city_id = self.get_user_city_id(claims)?;

        if &user_city_id != city_id {
            error!("[VictimService] User city {} does not match requested city {}", user_city_id, city_id);
            return Err(AppError::Forbidden("Access denied to this city's data".to_string()));
        }

        Ok(())
    }
}
