use log::{error, info};
use std::sync::Arc;
use uuid::Uuid;

use crate::core::commands::offenders::{CreateOffender, UpdateOffender};
use crate::core::contracts::repository::offenders::{
    OffenderReadRepository, OffenderWriteRepository,
};
use crate::core::contracts::repository::users::UserRepository;
use crate::core::entities::auth::ClaimsToUserToken;
use crate::core::entities::common::{
    is_security_agent, normalize_flag_from_list, resolve_city_id_from_addresses, AddressData,
    PaginatedResult, PhoneData,
};
use crate::core::value_objects::search::SearchCriteria;
use crate::utils::errors::AppError;
use crate::core::contracts::repository::error::RepositoryError;
use crate::core::read_models::offenders::{
    OffenderAddressResponse, OffenderPhoneResponse, OffenderWithDetails,
};
use crate::services::auth_context::AuthContext;
use crate::services::error_mapping::map_constraint;
use crate::utils::pagination::Pagination;
use crate::core::value_objects::policies::Policy;
use crate::validators::{
    cpf_validator::validate_cpf,
    offender_validator::OffenderValidator,
};

pub struct OffenderService {
    offender_read_repository: Arc<dyn OffenderReadRepository>,
    offender_write_repository: Arc<dyn OffenderWriteRepository>,
    user_repository: Arc<dyn UserRepository>,
}

impl OffenderService {
    pub fn new(
        offender_read_repository: Arc<dyn OffenderReadRepository>,
        offender_write_repository: Arc<dyn OffenderWriteRepository>,
        user_repository: Arc<dyn UserRepository>,
    ) -> Self {
        Self {
            offender_read_repository,
            offender_write_repository,
            user_repository,
        }
    }

    pub async fn create_offender(
        &self,
        offender: CreateOffender,
        claims: &ClaimsToUserToken,
    ) -> Result<OffenderWithDetails, AppError> {
        let mut offender = offender;
        let city_id = resolve_city_id_from_addresses(&offender.addresses, offender.city_id)
            .map_err(|e| AppError::BadRequest(format!("Error adding offender: {}", e)))?;
        offender.city_id = Some(city_id);
        offender.is_public_security_agent = is_security_agent(&offender.security_force);
        let (has_psychiatric_issues, psychiatric_issues_type) =
            normalize_flag_from_list(&offender.psychiatric_issues_type);
        offender.has_psychiatric_issues = has_psychiatric_issues;
        offender.psychiatric_issues_type = psychiatric_issues_type;

        if let Some(cpf) = offender.cpf.as_ref() {
            let normalized = validate_cpf(cpf, "Error adding offender")?;
            offender.cpf = Some(normalized);
        }

        info!(
            "[OffenderService] Starting offender creation: {}",
            offender.full_name
        );

        let auth = AuthContext::load(&*self.user_repository, claims).await?;

        auth.check_policy(&Policy::CreateOffenders, city_id)?;

        OffenderValidator::validate_required_fields(&offender.full_name, "Error adding offender")?;

        info!("[OffenderService] Saving offender to database");

        match self
            .offender_write_repository
            .create_offender(offender)
            .await
        {
            Ok(offender_with_details) => {
                let offender_with_details = offender_with_details.into_details();
                info!(
                    "[OffenderService] Offender created successfully with ID: {}",
                    offender_with_details.id
                );
                Ok(offender_with_details)
            }
            Err(e) => {
                if let RepositoryError::UniqueViolation { constraint }
                | RepositoryError::ForeignKeyViolation { constraint } = &e
                    && let Some(app_err) = map_constraint(
                        constraint.as_deref(),
                        &[
                            (
                                "fk_offenders_city",
                                "Error adding offender: city_id not found",
                            ),
                            (
                                "fk_offender_addresses_city",
                                "Error adding offender: address city_id not found",
                            ),
                        ],
                    )
                {
                    return Err(app_err);
                }
                error!("[OffenderService] Failed to save offender: {:?}", e);
                Err(AppError::InternalServerError)
            }
        }
    }

    pub async fn get_offender_by_id(
        &self,
        id: Uuid,
        claims: &ClaimsToUserToken,
    ) -> Result<OffenderWithDetails, AppError> {
        info!(
            "[OffenderService] Starting find offender by id process for id: {}",
            id
        );

        match self.offender_read_repository.get_offender_by_id(id).await {
            Ok(offender_with_details) => {
                let auth = AuthContext::load(&*self.user_repository, claims).await?;
                auth.check_policy(&Policy::ReadOffenders, offender_with_details.city_id)?;

                info!(
                    "[OffenderService] Offender with id {} found successfully",
                    id
                );
                Ok(offender_with_details)
            }
            Err(RepositoryError::NotFound) => {
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
        pagination: Pagination,
        claims: &ClaimsToUserToken,
    ) -> Result<PaginatedResult<OffenderWithDetails>, AppError> {
        info!("[OffenderService] Starting process to get offenders");

        let auth = AuthContext::load(&*self.user_repository, claims).await?;
        let allowed_cities = auth.allowed_cities(&Policy::ReadOffenders);

        let total_items = self
            .offender_read_repository
            .count_offenders(allowed_cities.as_deref())
            .await
            .map_err(|e| {
                error!("[OffenderService] Failed to count offenders: {:?}", e);
                AppError::InternalServerError
            })?;

        let offenders_list = self
            .offender_read_repository
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
        Ok(PaginatedResult {
            items: offenders_list,
            page: pagination.page,
            page_size: pagination.page_size,
            total_items,
        })
    }

    pub async fn search_offenders(
        &self,
        name: Option<String>,
        cpf: Option<String>,
        claims: &ClaimsToUserToken,
    ) -> Result<Vec<OffenderWithDetails>, AppError> {
        info!("[OffenderService] Starting offender search");

        let search = SearchCriteria::parse(name, cpf)
            .map_err(|e| AppError::BadRequest(format!("Error searching offenders: {}", e)))?;

        let auth = AuthContext::load(&*self.user_repository, claims).await?;

        let offenders = match search {
            SearchCriteria::ByName(name) => {
                self.offender_read_repository
                    .get_offenders_by_name(&name)
                    .await
            }
            SearchCriteria::ByCpf(cpf) => {
                self.offender_read_repository
                    .get_offenders_by_cpf(&cpf)
                    .await
            }
        };

        let offenders = if let Some(allowed_cities) =
            auth.allowed_cities(&Policy::ReadOffenders)
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
                Ok(offenders_list)
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
        claims: &ClaimsToUserToken,
    ) -> Result<Vec<OffenderWithDetails>, AppError> {
        info!(
            "[OffenderService] Starting process to get offenders for victim: {}",
            victim_id
        );

        let auth = AuthContext::load(&*self.user_repository, claims).await?;

        let offenders = if let Some(allowed_cities) =
            auth.allowed_cities(&Policy::ReadOffenders)
        {
            match self
                .offender_read_repository
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
            self.offender_read_repository
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
                Ok(offenders_list)
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
        claims: &ClaimsToUserToken,
    ) -> Result<OffenderWithDetails, AppError> {
        info!("[OffenderService] Starting offender update for id: {}", id);

        let auth = AuthContext::load(&*self.user_repository, claims).await?;
        let mut data = data;
        let city_id = resolve_city_id_from_addresses(&data.addresses, data.city_id)
            .map_err(|e| AppError::BadRequest(format!("Error updating offender: {}", e)))?;
        data.city_id = Some(city_id);
        data.is_public_security_agent = is_security_agent(&data.security_force);
        let (has_psychiatric_issues, psychiatric_issues_type) =
            normalize_flag_from_list(&data.psychiatric_issues_type);
        data.has_psychiatric_issues = has_psychiatric_issues;
        data.psychiatric_issues_type = psychiatric_issues_type;

        if let Some(cpf) = data.cpf.as_ref() {
            let normalized = validate_cpf(cpf, "Error updating offender")?;
            data.cpf = Some(normalized);
        }

        auth.check_policy(&Policy::UpdateOffenders, city_id)?;

        OffenderValidator::validate_required_fields(&data.full_name, "Error updating offender")?;

        match self.offender_read_repository.get_offender_by_id(id).await {
            Ok(existing_offender) => {
                auth.check_policy(&Policy::UpdateOffenders, existing_offender.city_id)?;
            }
            Err(RepositoryError::NotFound) => {
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
            .offender_write_repository
            .update_offender_by_id(data, id)
            .await
        {
            Ok(offender_with_details) => {
                let offender_with_details = offender_with_details.into_details();
                info!(
                    "[OffenderService] Offender updated successfully with ID: {}",
                    offender_with_details.id
                );
                Ok(offender_with_details)
            }
            Err(RepositoryError::NotFound) => {
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
                if let RepositoryError::UniqueViolation { constraint }
                | RepositoryError::ForeignKeyViolation { constraint } = &e
                    && let Some(app_err) = map_constraint(
                        constraint.as_deref(),
                        &[
                            (
                                "fk_offenders_city",
                                "Error updating offender: city_id not found",
                            ),
                            (
                                "fk_offender_addresses_city",
                                "Error updating offender: address city_id not found",
                            ),
                        ],
                    )
                {
                    return Err(app_err);
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
        claims: &ClaimsToUserToken,
    ) -> Result<OffenderWithDetails, AppError> {
        info!(
            "[OffenderService] Starting process to delete offender with id: {}",
            id
        );

        match self.offender_read_repository.get_offender_by_id(id).await {
            Ok(offender) => {
                let auth = AuthContext::load(&*self.user_repository, claims).await?;
                auth.check_policy(&Policy::DeleteOffenders, offender.city_id)?;
            }
            Err(RepositoryError::NotFound) => {
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

        match self
            .offender_write_repository
            .delete_offender_by_id(id)
            .await
        {
            Ok(deleted_offender) => {
                let deleted_offender = deleted_offender.into_details();
                info!(
                    "[OffenderService] Offender with id {} deleted successfully",
                    id
                );
                Ok(deleted_offender)
            }
            Err(RepositoryError::NotFound) => {
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
        claims: &ClaimsToUserToken,
    ) -> Result<OffenderPhoneResponse, AppError> {
        info!(
            "[OffenderService] Adding phone to offender: {}",
            offender_id
        );

        let auth = AuthContext::load(&*self.user_repository, claims).await?;

        match self
            .offender_read_repository
            .get_offender_by_id(offender_id)
            .await
        {
            Ok(offender) => {
                auth.check_policy(&Policy::UpdateOffenders, offender.city_id)?;
            }
            Err(RepositoryError::NotFound) => {
                return Err(AppError::NotFound(format!(
                    "Offender with id '{}' not found",
                    offender_id
                )));
            }
            Err(_) => return Err(AppError::InternalServerError),
        }

        match self
            .offender_write_repository
            .create_phone(offender_id, phone_data)
            .await
        {
            Ok(phone) => Ok(phone.to_response()),
            Err(_) => Err(AppError::InternalServerError),
        }
    }

    pub async fn update_phone(
        &self,
        phone_id: Uuid,
        phone_data: PhoneData,
        claims: &ClaimsToUserToken,
    ) -> Result<OffenderPhoneResponse, AppError> {
        info!("[OffenderService] Updating phone: {}", phone_id);

        let auth = AuthContext::load(&*self.user_repository, claims).await?;

        match self
            .offender_write_repository
            .get_phone_by_id(phone_id)
            .await
        {
            Ok(phone) => {
                match self
                    .offender_read_repository
                    .get_offender_by_id(phone.offender_id)
                    .await
                {
                    Ok(offender) => {
                        auth.check_policy(&Policy::UpdateOffenders, offender.city_id)?;
                    }
                    Err(e) => {
                        return match e {
                            RepositoryError::NotFound => Err(AppError::NotFound(
                                format!("Offender with id '{}' not found", phone.offender_id),
                            )),
                            _ => Err(AppError::InternalServerError),
                        };
                    }
                }
            }
            Err(RepositoryError::NotFound) => {
                return Err(AppError::NotFound(format!(
                    "Phone with id '{}' not found",
                    phone_id
                )));
            }
            Err(_) => return Err(AppError::InternalServerError),
        }

        match self
            .offender_write_repository
            .update_phone_by_id(phone_id, phone_data)
            .await
        {
            Ok(phone) => Ok(phone.to_response()),
            Err(_) => Err(AppError::InternalServerError),
        }
    }

    pub async fn delete_phone(
        &self,
        phone_id: Uuid,
        claims: &ClaimsToUserToken,
    ) -> Result<OffenderPhoneResponse, AppError> {
        info!("[OffenderService] Deleting phone: {}", phone_id);

        let auth = AuthContext::load(&*self.user_repository, claims).await?;

        match self
            .offender_write_repository
            .get_phone_by_id(phone_id)
            .await
        {
            Ok(phone) => {
                match self
                    .offender_read_repository
                    .get_offender_by_id(phone.offender_id)
                    .await
                {
                    Ok(offender) => {
                        auth.check_policy(&Policy::UpdateOffenders, offender.city_id)?;
                    }
                    Err(e) => {
                        return match e {
                            RepositoryError::NotFound => Err(AppError::NotFound(
                                format!("Offender with id '{}' not found", phone.offender_id),
                            )),
                            _ => Err(AppError::InternalServerError),
                        };
                    }
                }
            }
            Err(RepositoryError::NotFound) => {
                return Err(AppError::NotFound(format!(
                    "Phone with id '{}' not found",
                    phone_id
                )));
            }
            Err(_) => return Err(AppError::InternalServerError),
        }

        match self
            .offender_write_repository
            .delete_phone_by_id(phone_id)
            .await
        {
            Ok(phone) => Ok(phone.to_response()),
            Err(_) => Err(AppError::InternalServerError),
        }
    }

    pub async fn create_address(
        &self,
        offender_id: Uuid,
        address_data: AddressData,
        claims: &ClaimsToUserToken,
    ) -> Result<OffenderAddressResponse, AppError> {
        info!(
            "[OffenderService] Adding address to offender: {}",
            offender_id
        );

        let auth = AuthContext::load(&*self.user_repository, claims).await?;

        match self
            .offender_read_repository
            .get_offender_by_id(offender_id)
            .await
        {
            Ok(offender) => {
                auth.check_policy(&Policy::UpdateOffenders, offender.city_id)?;
            }
            Err(RepositoryError::NotFound) => {
                return Err(AppError::NotFound(format!(
                    "Offender with id '{}' not found",
                    offender_id
                )));
            }
            Err(_) => return Err(AppError::InternalServerError),
        }

        match self
            .offender_write_repository
            .create_address(offender_id, address_data)
            .await
        {
            Ok(address) => Ok(address.to_response()),
            Err(_) => Err(AppError::InternalServerError),
        }
    }

    pub async fn update_address(
        &self,
        address_id: Uuid,
        address_data: AddressData,
        claims: &ClaimsToUserToken,
    ) -> Result<OffenderAddressResponse, AppError> {
        info!("[OffenderService] Updating address: {}", address_id);

        let auth = AuthContext::load(&*self.user_repository, claims).await?;

        match self
            .offender_write_repository
            .get_address_by_id(address_id)
            .await
        {
            Ok(address) => {
                match self
                    .offender_read_repository
                    .get_offender_by_id(address.offender_id)
                    .await
                {
                    Ok(offender) => {
                        auth.check_policy(&Policy::UpdateOffenders, offender.city_id)?;
                    }
                    Err(e) => {
                        return match e {
                            RepositoryError::NotFound => Err(AppError::NotFound(
                                format!("Offender with id '{}' not found", address.offender_id),
                            )),
                            _ => Err(AppError::InternalServerError),
                        };
                    }
                }
            }
            Err(RepositoryError::NotFound) => {
                return Err(AppError::NotFound(format!(
                    "Address with id '{}' not found",
                    address_id
                )));
            }
            Err(_) => return Err(AppError::InternalServerError),
        }

        match self
            .offender_write_repository
            .update_address_by_id(address_id, address_data)
            .await
        {
            Ok(address) => Ok(address.to_response()),
            Err(_) => Err(AppError::InternalServerError),
        }
    }

    pub async fn delete_address(
        &self,
        address_id: Uuid,
        claims: &ClaimsToUserToken,
    ) -> Result<OffenderAddressResponse, AppError> {
        info!("[OffenderService] Deleting address: {}", address_id);

        let auth = AuthContext::load(&*self.user_repository, claims).await?;

        match self
            .offender_write_repository
            .get_address_by_id(address_id)
            .await
        {
            Ok(address) => {
                match self
                    .offender_read_repository
                    .get_offender_by_id(address.offender_id)
                    .await
                {
                    Ok(offender) => {
                        auth.check_policy(&Policy::UpdateOffenders, offender.city_id)?;
                    }
                    Err(e) => {
                        return match e {
                            RepositoryError::NotFound => Err(AppError::NotFound(
                                format!("Offender with id '{}' not found", address.offender_id),
                            )),
                            _ => Err(AppError::InternalServerError),
                        };
                    }
                }
            }
            Err(RepositoryError::NotFound) => {
                return Err(AppError::NotFound(format!(
                    "Address with id '{}' not found",
                    address_id
                )));
            }
            Err(_) => return Err(AppError::InternalServerError),
        }

        match self
            .offender_write_repository
            .delete_address_by_id(address_id)
            .await
        {
            Ok(address) => Ok(address.to_response()),
            Err(_) => Err(AppError::InternalServerError),
        }
    }
}
