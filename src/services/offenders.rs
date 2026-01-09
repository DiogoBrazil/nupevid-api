use actix_web::{web, HttpRequest, HttpResponse};
use log::{error, info};
use uuid::Uuid;

use crate::core::contracts::repository::offenders::OffenderRepository;

use crate::core::entities::offenders::{
    AddressData, AddressType, CreateOffender, PhoneData, UpdateOffender,
};
use crate::repositories::offenders::PgOffenderRepository;
use crate::repositories::users::PgUserRepository;

use crate::utils::{
    errors::AppError,
    responses::{ApiResponse, PaginatedResponse},
    authorization::{check_policy, get_allowed_cities_for_policy},
    service_helpers::{extract_claims, get_user_policies_with_defaults},
    db_error_mapper::map_constraint,
    pagination::{PaginationParams, normalize_pagination}
};
use crate::validators::{
    cpf_validator::validate_cpf,
    common::{POLICY_CREATE_OFFENDERS, POLICY_READ_OFFENDERS, POLICY_UPDATE_OFFENDERS, POLICY_DELETE_OFFENDERS},
    offender_validator::OffenderValidator
};

pub struct OffenderService {
    offender_repository: web::Data<PgOffenderRepository>,
    user_repository: web::Data<PgUserRepository>,
}

enum OffenderSearchCriteria {
    Name(String),
    Cpf(String),
}

impl OffenderService {
    pub fn new(
        offender_repository: web::Data<PgOffenderRepository>,
        user_repository: web::Data<PgUserRepository>,
    ) -> Self {
        Self {
            offender_repository,
            user_repository,
        }
    }

    fn derive_city_id_from_addresses(addresses: &Option<Vec<AddressData>>) -> Option<Uuid> {
        let addresses = addresses.as_ref()?;

        for address in addresses {
            if address.address_type == AddressType::Residential {
                return Some(address.city_id);
            }
        }

        for address in addresses {
            if address.address_type == AddressType::Work {
                return Some(address.city_id);
            }
        }

        None
    }

    fn resolve_city_id(
        addresses: &Option<Vec<AddressData>>,
        fallback_city_id: Option<Uuid>,
        error_context: &str,
    ) -> Result<Uuid, AppError> {
        if let Some(city_id) = Self::derive_city_id_from_addresses(addresses) {
            return Ok(city_id);
        }

        if let Some(city_id) = fallback_city_id {
            return Ok(city_id);
        }

        Err(AppError::BadRequest(format!(
            "{}: no Residential or Work address provided; please send city_id in the request body",
            error_context
        )))
    }

    fn parse_search_criteria(
        name: Option<String>,
        cpf: Option<String>,
        error_context: &str,
    ) -> Result<OffenderSearchCriteria, AppError> {
        match (name, cpf) {
            (Some(_), Some(_)) => Err(AppError::BadRequest(format!(
                "{}: provide either 'name' or 'cpf', not both",
                error_context
            ))),
            (None, None) => Err(AppError::BadRequest(format!(
                "{}: query parameter 'name' or 'cpf' is required",
                error_context
            ))),
            (Some(name), None) => {
                let trimmed = name.trim();
                if trimmed.is_empty() {
                    return Err(AppError::BadRequest(format!(
                        "{}: query parameter 'name' cannot be empty",
                        error_context
                    )));
                }
                Ok(OffenderSearchCriteria::Name(trimmed.to_string()))
            }
            (None, Some(cpf)) => {
                let trimmed = cpf.trim();
                if trimmed.is_empty() {
                    return Err(AppError::BadRequest(format!(
                        "{}: query parameter 'cpf' cannot be empty",
                        error_context
                    )));
                }
                let normalized = validate_cpf(trimmed, error_context)?;
                Ok(OffenderSearchCriteria::Cpf(normalized))
            }
        }
    }

    pub async fn create_offender(
        &self,
        offender: CreateOffender,
        req: HttpRequest,
    ) -> Result<HttpResponse, AppError> {
        let mut offender = offender;
        let city_id =
            Self::resolve_city_id(&offender.addresses, offender.city_id, "Error adding offender")?;
        offender.city_id = Some(city_id);
        offender.is_public_security_agent = offender.security_force.is_some();

        if let Some(cpf) = offender.cpf.as_ref() {
            let normalized = validate_cpf(cpf, "Error adding offender")?;
            offender.cpf = Some(normalized);
        }

        info!(
            "[OffenderService] Starting offender creation: {}",
            offender.full_name
        );

        let claims = extract_claims(&req)?;
        let policies = get_user_policies_with_defaults(&**self.user_repository, &claims).await?;

        check_policy(
            &claims,
            POLICY_CREATE_OFFENDERS,
            city_id,
            &policies,
        )?;

        OffenderValidator::validate_required_fields(&offender.full_name, "Error adding offender")?;

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
                if let sqlx::Error::Database(db_err) = &e {
                    if let Some(app_err) = map_constraint(db_err.constraint(), &[
                        ("fk_offenders_city", "Error adding offender: city_id not found"),
                        ("fk_offender_addresses_city", "Error adding offender: address city_id not found"),
                    ]) {
                        return Err(app_err);
                    }
                }
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

    pub async fn get_all_offenders(
        &self,
        params: PaginationParams,
        req: HttpRequest,
    ) -> Result<HttpResponse, AppError> {
        info!("[OffenderService] Starting process to get offenders");

        let claims = extract_claims(&req)?;
        let policies = get_user_policies_with_defaults(&**self.user_repository, &claims).await?;
        let pagination = normalize_pagination(&params);
        let allowed_cities = get_allowed_cities_for_policy(&claims, POLICY_READ_OFFENDERS, &policies);

        let total_items = self.offender_repository
            .count_offenders(allowed_cities.as_deref())
            .await
            .map_err(|e| {
                error!("[OffenderService] Failed to count offenders: {:?}", e);
                AppError::InternalServerError
            })?;

        let offenders_list = self.offender_repository
            .get_offenders_paginated(
                allowed_cities.as_deref(),
                pagination.page_size,
                pagination.offset,
            )
            .await
            .map_err(|e| {
                error!("[OffenderService] Failed to retrieve offenders: {:?}", e);
                AppError::InternalServerError
            })?;

        info!(
            "[OffenderService] Successfully retrieved {} offenders (paged)",
            offenders_list.len()
        );
        Ok(PaginatedResponse::success(
            offenders_list,
            pagination.page,
            pagination.page_size,
            total_items,
        ).into_response())
    }

    pub async fn search_offenders(
        &self,
        name: Option<String>,
        cpf: Option<String>,
        req: HttpRequest,
    ) -> Result<HttpResponse, AppError> {
        info!("[OffenderService] Starting offender search");

        let search = Self::parse_search_criteria(name, cpf, "Error searching offenders")?;

        let claims = extract_claims(&req)?;
        let policies = get_user_policies_with_defaults(&**self.user_repository, &claims).await?;

        let offenders = match search {
            OffenderSearchCriteria::Name(name) => {
                self.offender_repository.get_offenders_by_name(&name).await
            }
            OffenderSearchCriteria::Cpf(cpf) => {
                self.offender_repository.get_offenders_by_cpf(&cpf).await
            }
        };

        let offenders = if let Some(allowed_cities) =
            get_allowed_cities_for_policy(&claims, POLICY_READ_OFFENDERS, &policies)
        {
            match offenders {
                Ok(list) => {
                    let filtered: Vec<_> = list
                        .into_iter()
                        .filter(|o| allowed_cities.contains(&o.city_id))
                        .collect();
                    Ok(filtered)
                }
                Err(e) => Err(e),
            }
        } else {
            offenders
        };

        match offenders {
            Ok(offenders_list) => {
                info!(
                    "[OffenderService] Successfully retrieved {} offenders from search",
                    offenders_list.len()
                );
                Ok(ApiResponse::success(offenders_list).into_response())
            }
            Err(e) => {
                error!("[OffenderService] Failed to search offenders: {:?}", e);
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
        let mut data = data;
        let city_id = Self::resolve_city_id(&data.addresses, data.city_id, "Error updating offender")?;
        data.city_id = Some(city_id);
        data.is_public_security_agent = data.security_force.is_some();

        if let Some(cpf) = data.cpf.as_ref() {
            let normalized = validate_cpf(cpf, "Error updating offender")?;
            data.cpf = Some(normalized);
        }

        check_policy(&claims, POLICY_UPDATE_OFFENDERS, city_id, &policies)?;

        OffenderValidator::validate_required_fields(&data.full_name, "Error updating offender")?;

        match self.offender_repository.get_offender_by_id(id).await {
            Ok(existing_offender) => {
                check_policy(
                    &claims,
                    POLICY_UPDATE_OFFENDERS,
                    existing_offender.city_id,
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
                if let sqlx::Error::Database(db_err) = &e {
                    if let Some(app_err) = map_constraint(db_err.constraint(), &[
                        ("fk_offenders_city", "Error updating offender: city_id not found"),
                        ("fk_offender_addresses_city", "Error updating offender: address city_id not found"),
                    ]) {
                        return Err(app_err);
                    }
                }
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
                    Err(e) => {
                        return match e {
                            sqlx::Error::RowNotFound => Err(AppError::NotFound(
                                format!("Offender with id '{}' not found", phone.offender_id)
                            )),
                            _ => Err(AppError::InternalServerError),
                        }
                    }
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
                    Err(e) => {
                        return match e {
                            sqlx::Error::RowNotFound => Err(AppError::NotFound(
                                format!("Offender with id '{}' not found", phone.offender_id)
                            )),
                            _ => Err(AppError::InternalServerError),
                        }
                    }
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
                    Err(e) => {
                        return match e {
                            sqlx::Error::RowNotFound => Err(AppError::NotFound(
                                format!("Offender with id '{}' not found", address.offender_id)
                            )),
                            _ => Err(AppError::InternalServerError),
                        }
                    }
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
                    Err(e) => {
                        return match e {
                            sqlx::Error::RowNotFound => Err(AppError::NotFound(
                                format!("Offender with id '{}' not found", address.offender_id)
                            )),
                            _ => Err(AppError::InternalServerError),
                        }
                    }
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
}
