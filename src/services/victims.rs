use actix_web::{web, HttpRequest, HttpResponse};
use log::{error, info};
use uuid::Uuid;

use crate::core::contracts::repository::victims::VictimRepository;

use crate::core::entities::victims::{AddressData, CreateVictim, PhoneData, UpdateVictim};
use crate::repositories::victims::PgVictimRepository;
use crate::repositories::users::PgUserRepository;

use crate::utils::{
    errors::AppError,
    responses::ApiResponse,
    authorization::{check_policy, get_allowed_cities_for_policy},
    service_helpers::{extract_claims, get_user_policies_with_defaults}
};
use crate::validators::{
    common::{POLICY_CREATE_VICTIMS, POLICY_READ_VICTIMS, POLICY_UPDATE_VICTIMS, POLICY_DELETE_VICTIMS},
    victim_validator::VictimValidator
};

pub struct VictimService {
    victim_repository: web::Data<PgVictimRepository>,
    user_repository: web::Data<PgUserRepository>,
}

impl VictimService {
    pub fn new(victim_repository: web::Data<PgVictimRepository>, user_repository: web::Data<PgUserRepository>) -> Self {
        Self { victim_repository, user_repository }
    }

    pub async fn create_victim(
        &self,
        victim: CreateVictim,
        req: HttpRequest,
    ) -> Result<HttpResponse, AppError> {
        info!("[VictimService] Starting victim creation: {}", victim.full_name);

        let claims = extract_claims(&req)?;
        let policies = get_user_policies_with_defaults(&**self.user_repository, &claims).await?;

        check_policy(&claims, POLICY_CREATE_VICTIMS, victim.city_id, &policies)?;

        VictimValidator::validate_required_fields(&victim.full_name, "Error adding victim")?;

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
                if let sqlx::Error::Database(db_err) = &e {
                    if db_err.is_unique_violation() && db_err.constraint() == Some("idx_victims_cpf_unique") {
                        error!("[VictimService] Attempt to create victim with duplicate CPF");
                        return Err(AppError::Conflict("A victim with this CPF already exists.".to_string()));
                    }
                }
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

        let claims = extract_claims(&req)?;

        match self.victim_repository.get_victim_by_id(id).await {
            Ok(victim_with_address) => {
                let policies = get_user_policies_with_defaults(&**self.user_repository, &claims).await?;
                check_policy(&claims, POLICY_READ_VICTIMS, victim_with_address.city_id, &policies)?;

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

        let claims = extract_claims(&req)?;
        let policies = get_user_policies_with_defaults(&**self.user_repository, &claims).await?;

        let victims = if let Some(allowed_cities) = get_allowed_cities_for_policy(&claims, POLICY_READ_VICTIMS, &policies) {
            match self.victim_repository.get_all_victims().await {
                Ok(all) => {
                    let filtered: Vec<_> = all.into_iter().filter(|v| allowed_cities.contains(&v.city_id)).collect();
                    Ok(filtered)
                }
                Err(e) => Err(e),
            }
        } else {
            self.victim_repository.get_all_victims().await
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

        let claims = extract_claims(&req)?;
        let policies = get_user_policies_with_defaults(&**self.user_repository, &claims).await?;
        check_policy(&claims, POLICY_UPDATE_VICTIMS, data.city_id, &policies)?;

        VictimValidator::validate_required_fields(&data.full_name, "Error updating victim")?;

        match self.victim_repository.get_victim_by_id(id).await {
            Ok(existing_victim) => {
                check_policy(&claims, POLICY_UPDATE_VICTIMS, existing_victim.city_id, &policies)?;
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

        let claims = extract_claims(&req)?;

        match self.victim_repository.get_victim_by_id(id).await {
            Ok(victim) => {
                let policies = get_user_policies_with_defaults(&**self.user_repository, &claims).await?;
                check_policy(&claims, POLICY_DELETE_VICTIMS, victim.city_id, &policies)?;
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

    pub async fn create_phone(
        &self,
        victim_id: Uuid,
        phone_data: PhoneData,
        req: HttpRequest,
    ) -> Result<HttpResponse, AppError> {
        info!("[VictimService] Adding phone to victim: {}", victim_id);

        let claims = extract_claims(&req)?;
        let policies = get_user_policies_with_defaults(&**self.user_repository, &claims).await?;

        match self.victim_repository.get_victim_by_id(victim_id).await {
            Ok(victim) => {
                check_policy(&claims, POLICY_UPDATE_VICTIMS, victim.city_id, &policies)?;
            }
            Err(sqlx::Error::RowNotFound) => {
                return Err(AppError::NotFound(format!(
                    "Victim with id '{}' not found",
                    victim_id
                )));
            }
            Err(_) => return Err(AppError::InternalServerError),
        }

        match self.victim_repository.create_phone(victim_id, phone_data).await {
            Ok(phone) => Ok(ApiResponse::created(phone).into_response()),
            Err(_) => Err(AppError::InternalServerError),
        }
    }

    pub async fn update_phone(
        &self,
        phone_id: Uuid,
        phone_data: PhoneData,
        req: HttpRequest,
    ) -> Result<HttpResponse, AppError> {
        info!("[VictimService] Updating phone: {}", phone_id);

        let claims = extract_claims(&req)?;
        let policies = get_user_policies_with_defaults(&**self.user_repository, &claims).await?;

        match self.victim_repository.get_phone_by_id(phone_id).await {
            Ok(phone) => {
                match self.victim_repository.get_victim_by_id(phone.victim_id).await {
                    Ok(victim) => {
                        check_policy(&claims, POLICY_UPDATE_VICTIMS, victim.city_id, &policies)?;
                    }
                    Err(_) => return Err(AppError::InternalServerError),
                }
            }
            Err(sqlx::Error::RowNotFound) => {
                return Err(AppError::NotFound(format!(
                    "Phone with id '{}' not found",
                    phone_id
                )));
            }
            Err(_) => return Err(AppError::InternalServerError),
        }

        match self.victim_repository.update_phone_by_id(phone_id, phone_data).await {
            Ok(phone) => Ok(ApiResponse::success(phone).into_response()),
            Err(_) => Err(AppError::InternalServerError),
        }
    }

    pub async fn delete_phone(&self, phone_id: Uuid, req: HttpRequest) -> Result<HttpResponse, AppError> {
        info!("[VictimService] Deleting phone: {}", phone_id);

        let claims = extract_claims(&req)?;
        let policies = get_user_policies_with_defaults(&**self.user_repository, &claims).await?;

        match self.victim_repository.get_phone_by_id(phone_id).await {
            Ok(phone) => {
                match self.victim_repository.get_victim_by_id(phone.victim_id).await {
                    Ok(victim) => {
                        check_policy(&claims, POLICY_UPDATE_VICTIMS, victim.city_id, &policies)?;
                    }
                    Err(_) => return Err(AppError::InternalServerError),
                }
            }
            Err(sqlx::Error::RowNotFound) => {
                return Err(AppError::NotFound(format!(
                    "Phone with id '{}' not found",
                    phone_id
                )));
            }
            Err(_) => return Err(AppError::InternalServerError),
        }

        match self.victim_repository.delete_phone_by_id(phone_id).await {
            Ok(phone) => Ok(ApiResponse::success(phone).into_response()),
            Err(_) => Err(AppError::InternalServerError),
        }
    }

    pub async fn create_address(
        &self,
        victim_id: Uuid,
        address_data: AddressData,
        req: HttpRequest,
    ) -> Result<HttpResponse, AppError> {
        info!("[VictimService] Adding address to victim: {}", victim_id);

        let claims = extract_claims(&req)?;
        let policies = get_user_policies_with_defaults(&**self.user_repository, &claims).await?;

        match self.victim_repository.get_victim_by_id(victim_id).await {
            Ok(victim) => {
                check_policy(&claims, POLICY_UPDATE_VICTIMS, victim.city_id, &policies)?;
            }
            Err(sqlx::Error::RowNotFound) => {
                return Err(AppError::NotFound(format!(
                    "Victim with id '{}' not found",
                    victim_id
                )));
            }
            Err(_) => return Err(AppError::InternalServerError),
        }

        match self.victim_repository.create_address(victim_id, address_data).await {
            Ok(address) => Ok(ApiResponse::created(address).into_response()),
            Err(_) => Err(AppError::InternalServerError),
        }
    }

    pub async fn update_address(
        &self,
        address_id: Uuid,
        address_data: AddressData,
        req: HttpRequest,
    ) -> Result<HttpResponse, AppError> {
        info!("[VictimService] Updating address: {}", address_id);

        let claims = extract_claims(&req)?;
        let policies = get_user_policies_with_defaults(&**self.user_repository, &claims).await?;

        match self.victim_repository.get_address_by_id(address_id).await {
            Ok(address) => {
                match self.victim_repository.get_victim_by_id(address.victim_id).await {
                    Ok(victim) => {
                        check_policy(&claims, POLICY_UPDATE_VICTIMS, victim.city_id, &policies)?;
                    }
                    Err(_) => return Err(AppError::InternalServerError),
                }
            }
            Err(sqlx::Error::RowNotFound) => {
                return Err(AppError::NotFound(format!(
                    "Address with id '{}' not found",
                    address_id
                )));
            }
            Err(_) => return Err(AppError::InternalServerError),
        }

        match self.victim_repository.update_address_by_id(address_id, address_data).await {
            Ok(address) => Ok(ApiResponse::success(address).into_response()),
            Err(_) => Err(AppError::InternalServerError),
        }
    }

    pub async fn delete_address(&self, address_id: Uuid, req: HttpRequest) -> Result<HttpResponse, AppError> {
        info!("[VictimService] Deleting address: {}", address_id);

        let claims = extract_claims(&req)?;
        let policies = get_user_policies_with_defaults(&**self.user_repository, &claims).await?;

        match self.victim_repository.get_address_by_id(address_id).await {
            Ok(address) => {
                match self.victim_repository.get_victim_by_id(address.victim_id).await {
                    Ok(victim) => {
                        check_policy(&claims, POLICY_UPDATE_VICTIMS, victim.city_id, &policies)?;
                    }
                    Err(_) => return Err(AppError::InternalServerError),
                }
            }
            Err(sqlx::Error::RowNotFound) => {
                return Err(AppError::NotFound(format!(
                    "Address with id '{}' not found",
                    address_id
                )));
            }
            Err(_) => return Err(AppError::InternalServerError),
        }

        match self.victim_repository.delete_address_by_id(address_id).await {
            Ok(address) => Ok(ApiResponse::success(address).into_response()),
            Err(_) => Err(AppError::InternalServerError),
        }
    }

}
