use actix_web::{web, HttpRequest, HttpResponse};
use log::{error, info};
use uuid::Uuid;

use crate::core::contracts::repository::offenders::OffenderRepository;
use crate::core::contracts::repository::victims::VictimRepository;

use crate::core::entities::offenders::{
    AddressData, CreateOffender, PhoneData, UpdateOffender, WorkAddressData,
};
use crate::repositories::offenders::PgOffenderRepository;
use crate::repositories::users::PgUserRepository;
use crate::repositories::victims::PgVictimRepository;

use crate::utils::{
    errors::AppError,
    responses::ApiResponse,
    authorization::{check_policy, get_allowed_cities_for_policy},
    service_helpers::{extract_claims, get_user_policies_with_defaults}
};
use crate::validators::{
    common::{POLICY_CREATE_OFFENDERS, POLICY_READ_OFFENDERS, POLICY_UPDATE_OFFENDERS, POLICY_DELETE_OFFENDERS},
    offender_validator::OffenderValidator
};

pub struct OffenderService {
    offender_repository: web::Data<PgOffenderRepository>,
    victim_repository: web::Data<PgVictimRepository>,
    user_repository: web::Data<PgUserRepository>,
}

impl OffenderService {
    pub fn new(
        offender_repository: web::Data<PgOffenderRepository>,
        victim_repository: web::Data<PgVictimRepository>,
        user_repository: web::Data<PgUserRepository>,
    ) -> Self {
        Self {
            offender_repository,
            victim_repository,
            user_repository,
        }
    }

    pub async fn create_offender(
        &self,
        offender: CreateOffender,
        req: HttpRequest,
    ) -> Result<HttpResponse, AppError> {
        info!(
            "[OffenderService] Starting offender creation: {}",
            offender.full_name
        );

        let claims = extract_claims(&req)?;
        let policies = get_user_policies_with_defaults(&**self.user_repository, &claims).await?;

        check_policy(
            &claims,
            POLICY_CREATE_OFFENDERS,
            offender.city_id,
            &policies,
        )?;

        OffenderValidator::validate_required_fields(&offender.full_name, "Error adding offender")?;

        match self.victim_repository.get_victim_by_id(offender.victim_id).await {
            Ok(_) => {},
            Err(sqlx::Error::RowNotFound) => {
                return Err(AppError::NotFound(format!(
                    "Victim with id '{}' not found",
                    offender.victim_id
                )));
            }
            Err(e) => {
                error!("[OffenderService] Error checking victim: {:?}", e);
                return Err(AppError::InternalServerError);
            }
        }

        info!("[OffenderService] Saving offender to database");

        match self.offender_repository.create_offender(offender).await {
            Ok(offender_with_details) => {
                info!(
                    "[OffenderService] Offender created successfully with ID: {}",
                    offender_with_details.id
                );
                Ok(ApiResponse::created(offender_with_details).into_response())
            }
            Err(e) => {
                error!("[OffenderService] Failed to save offender: {:?}", e);
                Err(AppError::InternalServerError)
            }
        }
    }

    pub async fn get_offender_by_id(
        &self,
        id: Uuid,
        req: HttpRequest,
    ) -> Result<HttpResponse, AppError> {
        info!(
            "[OffenderService] Starting find offender by id process for id: {}",
            id
        );

        let claims = extract_claims(&req)?;

        match self.offender_repository.get_offender_by_id(id).await {
            Ok(offender_with_details) => {
                let policies = get_user_policies_with_defaults(&**self.user_repository, &claims).await?;
                check_policy(
                    &claims,
                    POLICY_READ_OFFENDERS,
                    offender_with_details.city_id,
                    &policies,
                )?;

                info!(
                    "[OffenderService] Offender with id {} found successfully",
                    id
                );
                Ok(ApiResponse::success(offender_with_details).into_response())
            }
            Err(sqlx::Error::RowNotFound) => {
                info!("[OffenderService] Offender with id {} not found", id);
                Err(AppError::NotFound(format!(
                    "Offender with id '{}' not found",
                    id
                )))
            }
            Err(e) => {
                error!(
                    "[OffenderService] Database error while finding offender: {:?}",
                    e
                );
                Err(AppError::InternalServerError)
            }
        }
    }

    pub async fn get_all_offenders(&self, req: HttpRequest) -> Result<HttpResponse, AppError> {
        info!("[OffenderService] Starting process to get offenders");

        let claims = extract_claims(&req)?;
        let policies = get_user_policies_with_defaults(&**self.user_repository, &claims).await?;

        let offenders = if let Some(allowed_cities) =
            get_allowed_cities_for_policy(&claims, POLICY_READ_OFFENDERS, &policies)
        {
            match self.offender_repository.get_all_offenders().await {
                Ok(all) => {
                    let filtered: Vec<_> = all
                        .into_iter()
                        .filter(|o| allowed_cities.contains(&o.city_id))
                        .collect();
                    Ok(filtered)
                }
                Err(e) => Err(e),
            }
        } else {
            self.offender_repository.get_all_offenders().await
        };

        match offenders {
            Ok(offenders_list) => {
                info!(
                    "[OffenderService] Successfully retrieved {} offenders",
                    offenders_list.len()
                );
                Ok(ApiResponse::success(offenders_list).into_response())
            }
            Err(e) => {
                error!("[OffenderService] Failed to retrieve offenders: {:?}", e);
                Err(AppError::InternalServerError)
            }
        }
    }

    pub async fn get_offenders_by_victim_id(
        &self,
        victim_id: Uuid,
        req: HttpRequest,
    ) -> Result<HttpResponse, AppError> {
        info!(
            "[OffenderService] Starting process to get offenders for victim: {}",
            victim_id
        );

        let claims = extract_claims(&req)?;
        let policies = get_user_policies_with_defaults(&**self.user_repository, &claims).await?;

        let offenders = if let Some(allowed_cities) =
            get_allowed_cities_for_policy(&claims, POLICY_READ_OFFENDERS, &policies)
        {
            match self
                .offender_repository
                .get_offenders_by_victim_id(victim_id)
                .await
            {
                Ok(all) => {
                    let filtered: Vec<_> = all
                        .into_iter()
                        .filter(|o| allowed_cities.contains(&o.city_id))
                        .collect();
                    Ok(filtered)
                }
                Err(e) => Err(e),
            }
        } else {
            self.offender_repository
                .get_offenders_by_victim_id(victim_id)
                .await
        };

        match offenders {
            Ok(offenders_list) => {
                info!(
                    "[OffenderService] Successfully retrieved {} offenders for victim: {}",
                    offenders_list.len(),
                    victim_id
                );
                Ok(ApiResponse::success(offenders_list).into_response())
            }
            Err(e) => {
                error!(
                    "[OffenderService] Failed to retrieve offenders for victim: {:?}",
                    e
                );
                Err(AppError::InternalServerError)
            }
        }
    }

    pub async fn update_offender_by_id(
        &self,
        data: UpdateOffender,
        id: Uuid,
        req: HttpRequest,
    ) -> Result<HttpResponse, AppError> {
        info!("[OffenderService] Starting offender update for id: {}", id);

        let claims = extract_claims(&req)?;
        let policies = get_user_policies_with_defaults(&**self.user_repository, &claims).await?;
        check_policy(&claims, POLICY_UPDATE_OFFENDERS, data.city_id, &policies)?;

        OffenderValidator::validate_required_fields(&data.full_name, "Error updating offender")?;

        match self.offender_repository.get_offender_by_id(id).await {
            Ok(existing_offender) => {
                check_policy(
                    &claims,
                    POLICY_UPDATE_OFFENDERS,
                    existing_offender.city_id,
                    &policies,
                )?;

                if existing_offender.victim_id != data.victim_id {
                    match self.victim_repository.get_victim_by_id(data.victim_id).await {
                        Ok(_) => {},
                        Err(sqlx::Error::RowNotFound) => {
                            return Err(AppError::NotFound(format!(
                                "Victim with id '{}' not found",
                                data.victim_id
                            )));
                        }
                        Err(e) => {
                            error!("[OffenderService] Error checking victim: {:?}", e);
                            return Err(AppError::InternalServerError);
                        }
                    }
                }
            }
            Err(sqlx::Error::RowNotFound) => {
                return Err(AppError::NotFound(format!(
                    "Offender with id '{}' not found",
                    id
                )));
            }
            Err(e) => {
                error!("[OffenderService] Error checking offender: {:?}", e);
                return Err(AppError::InternalServerError);
            }
        }

        info!("[OffenderService] Updating offender in database");

        match self
            .offender_repository
            .update_offender_by_id(data, id)
            .await
        {
            Ok(offender_with_details) => {
                info!(
                    "[OffenderService] Offender updated successfully with ID: {}",
                    offender_with_details.id
                );
                Ok(ApiResponse::success(offender_with_details).into_response())
            }
            Err(sqlx::Error::RowNotFound) => {
                error!(
                    "[OffenderService] Offender with id {} not found for update",
                    id
                );
                Err(AppError::NotFound(format!(
                    "Offender with id '{}' not found",
                    id
                )))
            }
            Err(e) => {
                error!(
                    "[OffenderService] Error updating offender in database: {:?}",
                    e
                );
                Err(AppError::InternalServerError)
            }
        }
    }

    pub async fn delete_offender_by_id(
        &self,
        id: Uuid,
        req: HttpRequest,
    ) -> Result<HttpResponse, AppError> {
        info!(
            "[OffenderService] Starting process to delete offender with id: {}",
            id
        );

        let claims = extract_claims(&req)?;

        match self.offender_repository.get_offender_by_id(id).await {
            Ok(offender) => {
                let policies = get_user_policies_with_defaults(&**self.user_repository, &claims).await?;
                check_policy(
                    &claims,
                    POLICY_DELETE_OFFENDERS,
                    offender.city_id,
                    &policies,
                )?;
            }
            Err(sqlx::Error::RowNotFound) => {
                return Err(AppError::NotFound(format!(
                    "Offender with id '{}' not found",
                    id
                )));
            }
            Err(e) => {
                error!("[OffenderService] Error checking offender: {:?}", e);
                return Err(AppError::InternalServerError);
            }
        }

        match self.offender_repository.delete_offender_by_id(id).await {
            Ok(deleted_offender) => {
                info!(
                    "[OffenderService] Offender with id {} deleted successfully",
                    id
                );
                Ok(ApiResponse::success(deleted_offender).into_response())
            }
            Err(sqlx::Error::RowNotFound) => {
                info!(
                    "[OffenderService] Offender with id {} not found for deletion",
                    id
                );
                Err(AppError::NotFound(format!(
                    "Offender with id '{}' not found",
                    id
                )))
            }
            Err(e) => {
                error!("[OffenderService] Failed to delete offender: {:?}", e);
                Err(AppError::InternalServerError)
            }
        }
    }

    pub async fn create_phone(
        &self,
        offender_id: Uuid,
        phone_data: PhoneData,
        req: HttpRequest,
    ) -> Result<HttpResponse, AppError> {
        info!("[OffenderService] Adding phone to offender: {}", offender_id);

        let claims = extract_claims(&req)?;
        let policies = get_user_policies_with_defaults(&**self.user_repository, &claims).await?;

        match self.offender_repository.get_offender_by_id(offender_id).await {
            Ok(offender) => {
                check_policy(
                    &claims,
                    POLICY_UPDATE_OFFENDERS,
                    offender.city_id,
                    &policies,
                )?;
            }
            Err(sqlx::Error::RowNotFound) => {
                return Err(AppError::NotFound(format!(
                    "Offender with id '{}' not found",
                    offender_id
                )));
            }
            Err(_) => return Err(AppError::InternalServerError),
        }

        match self
            .offender_repository
            .create_phone(offender_id, phone_data)
            .await
        {
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
        info!("[OffenderService] Updating phone: {}", phone_id);

        let claims = extract_claims(&req)?;
        let policies = get_user_policies_with_defaults(&**self.user_repository, &claims).await?;

        match self.offender_repository.get_phone_by_id(phone_id).await {
            Ok(phone) => {
                match self
                    .offender_repository
                    .get_offender_by_id(phone.offender_id)
                    .await
                {
                    Ok(offender) => {
                        check_policy(
                            &claims,
                            POLICY_UPDATE_OFFENDERS,
                            offender.city_id,
                            &policies,
                        )?;
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

        match self
            .offender_repository
            .update_phone_by_id(phone_id, phone_data)
            .await
        {
            Ok(phone) => Ok(ApiResponse::success(phone).into_response()),
            Err(_) => Err(AppError::InternalServerError),
        }
    }

    pub async fn delete_phone(
        &self,
        phone_id: Uuid,
        req: HttpRequest,
    ) -> Result<HttpResponse, AppError> {
        info!("[OffenderService] Deleting phone: {}", phone_id);

        let claims = extract_claims(&req)?;
        let policies = get_user_policies_with_defaults(&**self.user_repository, &claims).await?;

        match self.offender_repository.get_phone_by_id(phone_id).await {
            Ok(phone) => {
                match self
                    .offender_repository
                    .get_offender_by_id(phone.offender_id)
                    .await
                {
                    Ok(offender) => {
                        check_policy(
                            &claims,
                            POLICY_UPDATE_OFFENDERS,
                            offender.city_id,
                            &policies,
                        )?;
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

        match self.offender_repository.delete_phone_by_id(phone_id).await {
            Ok(phone) => Ok(ApiResponse::success(phone).into_response()),
            Err(_) => Err(AppError::InternalServerError),
        }
    }

    pub async fn create_address(
        &self,
        offender_id: Uuid,
        address_data: AddressData,
        req: HttpRequest,
    ) -> Result<HttpResponse, AppError> {
        info!(
            "[OffenderService] Adding address to offender: {}",
            offender_id
        );

        let claims = extract_claims(&req)?;
        let policies = get_user_policies_with_defaults(&**self.user_repository, &claims).await?;

        match self.offender_repository.get_offender_by_id(offender_id).await {
            Ok(offender) => {
                check_policy(
                    &claims,
                    POLICY_UPDATE_OFFENDERS,
                    offender.city_id,
                    &policies,
                )?;
            }
            Err(sqlx::Error::RowNotFound) => {
                return Err(AppError::NotFound(format!(
                    "Offender with id '{}' not found",
                    offender_id
                )));
            }
            Err(_) => return Err(AppError::InternalServerError),
        }

        match self
            .offender_repository
            .create_address(offender_id, address_data)
            .await
        {
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
        info!("[OffenderService] Updating address: {}", address_id);

        let claims = extract_claims(&req)?;
        let policies = get_user_policies_with_defaults(&**self.user_repository, &claims).await?;

        match self.offender_repository.get_address_by_id(address_id).await {
            Ok(address) => {
                match self
                    .offender_repository
                    .get_offender_by_id(address.offender_id)
                    .await
                {
                    Ok(offender) => {
                        check_policy(
                            &claims,
                            POLICY_UPDATE_OFFENDERS,
                            offender.city_id,
                            &policies,
                        )?;
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

        match self
            .offender_repository
            .update_address_by_id(address_id, address_data)
            .await
        {
            Ok(address) => Ok(ApiResponse::success(address).into_response()),
            Err(_) => Err(AppError::InternalServerError),
        }
    }

    pub async fn delete_address(
        &self,
        address_id: Uuid,
        req: HttpRequest,
    ) -> Result<HttpResponse, AppError> {
        info!("[OffenderService] Deleting address: {}", address_id);

        let claims = extract_claims(&req)?;
        let policies = get_user_policies_with_defaults(&**self.user_repository, &claims).await?;

        match self.offender_repository.get_address_by_id(address_id).await {
            Ok(address) => {
                match self
                    .offender_repository
                    .get_offender_by_id(address.offender_id)
                    .await
                {
                    Ok(offender) => {
                        check_policy(
                            &claims,
                            POLICY_UPDATE_OFFENDERS,
                            offender.city_id,
                            &policies,
                        )?;
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

        match self
            .offender_repository
            .delete_address_by_id(address_id)
            .await
        {
            Ok(address) => Ok(ApiResponse::success(address).into_response()),
            Err(_) => Err(AppError::InternalServerError),
        }
    }

    pub async fn create_work_address(
        &self,
        offender_id: Uuid,
        work_address_data: WorkAddressData,
        req: HttpRequest,
    ) -> Result<HttpResponse, AppError> {
        info!(
            "[OffenderService] Adding work address to offender: {}",
            offender_id
        );

        let claims = extract_claims(&req)?;
        let policies = get_user_policies_with_defaults(&**self.user_repository, &claims).await?;

        match self.offender_repository.get_offender_by_id(offender_id).await {
            Ok(offender) => {
                check_policy(
                    &claims,
                    POLICY_UPDATE_OFFENDERS,
                    offender.city_id,
                    &policies,
                )?;
            }
            Err(sqlx::Error::RowNotFound) => {
                return Err(AppError::NotFound(format!(
                    "Offender with id '{}' not found",
                    offender_id
                )));
            }
            Err(_) => return Err(AppError::InternalServerError),
        }

        match self
            .offender_repository
            .create_work_address(offender_id, work_address_data)
            .await
        {
            Ok(work_address) => Ok(ApiResponse::created(work_address).into_response()),
            Err(_) => Err(AppError::InternalServerError),
        }
    }

    pub async fn update_work_address(
        &self,
        work_address_id: Uuid,
        work_address_data: WorkAddressData,
        req: HttpRequest,
    ) -> Result<HttpResponse, AppError> {
        info!(
            "[OffenderService] Updating work address: {}",
            work_address_id
        );

        let claims = extract_claims(&req)?;
        let policies = get_user_policies_with_defaults(&**self.user_repository, &claims).await?;

        match self
            .offender_repository
            .get_work_address_by_id(work_address_id)
            .await
        {
            Ok(work_address) => {
                match self
                    .offender_repository
                    .get_offender_by_id(work_address.offender_id)
                    .await
                {
                    Ok(offender) => {
                        check_policy(
                            &claims,
                            POLICY_UPDATE_OFFENDERS,
                            offender.city_id,
                            &policies,
                        )?;
                    }
                    Err(_) => return Err(AppError::InternalServerError),
                }
            }
            Err(sqlx::Error::RowNotFound) => {
                return Err(AppError::NotFound(format!(
                    "Work address with id '{}' not found",
                    work_address_id
                )));
            }
            Err(_) => return Err(AppError::InternalServerError),
        }

        match self
            .offender_repository
            .update_work_address_by_id(work_address_id, work_address_data)
            .await
        {
            Ok(work_address) => Ok(ApiResponse::success(work_address).into_response()),
            Err(_) => Err(AppError::InternalServerError),
        }
    }

    pub async fn delete_work_address(
        &self,
        work_address_id: Uuid,
        req: HttpRequest,
    ) -> Result<HttpResponse, AppError> {
        info!(
            "[OffenderService] Deleting work address: {}",
            work_address_id
        );

        let claims = extract_claims(&req)?;
        let policies = get_user_policies_with_defaults(&**self.user_repository, &claims).await?;

        match self
            .offender_repository
            .get_work_address_by_id(work_address_id)
            .await
        {
            Ok(work_address) => {
                match self
                    .offender_repository
                    .get_offender_by_id(work_address.offender_id)
                    .await
                {
                    Ok(offender) => {
                        check_policy(
                            &claims,
                            POLICY_UPDATE_OFFENDERS,
                            offender.city_id,
                            &policies,
                        )?;
                    }
                    Err(_) => return Err(AppError::InternalServerError),
                }
            }
            Err(sqlx::Error::RowNotFound) => {
                return Err(AppError::NotFound(format!(
                    "Work address with id '{}' not found",
                    work_address_id
                )));
            }
            Err(_) => return Err(AppError::InternalServerError),
        }

        match self
            .offender_repository
            .delete_work_address_by_id(work_address_id)
            .await
        {
            Ok(work_address) => Ok(ApiResponse::success(work_address).into_response()),
            Err(_) => Err(AppError::InternalServerError),
        }
    }

}
