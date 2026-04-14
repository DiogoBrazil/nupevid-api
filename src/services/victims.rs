use log::{error, info};
use std::sync::Arc;
use uuid::Uuid;

use crate::core::commands::victims::{CreateVictim, UpdateVictim};
use crate::core::contracts::repository::users::UserRepository;
use crate::core::contracts::repository::victims::{VictimReadRepository, VictimWriteRepository};
use crate::core::entities::auth::ClaimsToUserToken;
use crate::core::entities::common::{
    normalize_flag_from_list, resolve_city_id_from_addresses, AddressData, PaginatedResult,
    PhoneData,
};
use crate::core::value_objects::search::SearchCriteria;
use crate::utils::errors::AppError;
use crate::core::contracts::repository::error::RepositoryError;
use crate::core::read_models::victims::{
    VictimAddressResponse, VictimPhoneResponse, VictimWithDetails,
};
use crate::core::value_objects::policies::Policy;
use crate::services::auth_context::AuthContext;
use crate::services::error_mapping::map_constraint;
use crate::utils::pagination::Pagination;
use crate::validators::{cpf_validator::validate_cpf, victim_validator::VictimValidator};

pub struct VictimService {
    victim_read_repository: Arc<dyn VictimReadRepository>,
    victim_write_repository: Arc<dyn VictimWriteRepository>,
    user_repository: Arc<dyn UserRepository>,
}

impl VictimService {
    pub fn new(
        victim_read_repository: Arc<dyn VictimReadRepository>,
        victim_write_repository: Arc<dyn VictimWriteRepository>,
        user_repository: Arc<dyn UserRepository>,
    ) -> Self {
        Self {
            victim_read_repository,
            victim_write_repository,
            user_repository,
        }
    }

    pub async fn create_victim(
        &self,
        victim: CreateVictim,
        claims: &ClaimsToUserToken,
    ) -> Result<VictimWithDetails, AppError> {
        let mut victim = victim;
        let city_id = resolve_city_id_from_addresses(&victim.addresses, victim.city_id)
            .map_err(|e| AppError::BadRequest(format!("Error adding victim: {}", e)))?;
        victim.city_id = Some(city_id);

        let (has_special_needs, special_needs_type) =
            normalize_flag_from_list(&victim.special_needs_type);
        victim.has_special_needs = has_special_needs;
        victim.special_needs_type = special_needs_type;

        let (has_psychiatric_issues, psychiatric_issues_type) =
            normalize_flag_from_list(&victim.psychiatric_issues_type);
        victim.has_psychiatric_issues = has_psychiatric_issues;
        victim.psychiatric_issues_type = psychiatric_issues_type;
        victim.has_children = victim.children_count.is_some();

        if let Some(cpf) = victim.cpf.as_ref() {
            let normalized = validate_cpf(cpf, "Error adding victim")?;
            victim.cpf = Some(normalized);
        }

        info!(
            "[VictimService] Starting victim creation: {}",
            victim.full_name
        );

        let auth = AuthContext::load(&*self.user_repository, claims).await?;

        auth.check_policy(&Policy::CreateVictims, city_id)?;

        VictimValidator::validate_required_fields(&victim.full_name, "Error adding victim")?;

        info!("[VictimService] Saving victim to database");

        match self.victim_write_repository.create_victim(victim).await {
            Ok(victim_with_address) => {
                let victim_with_address = victim_with_address.into_details();
                info!(
                    "[VictimService] Victim created successfully with ID: {}",
                    victim_with_address.id
                );
                Ok(victim_with_address)
            }
            Err(e) => {
                if let RepositoryError::UniqueViolation { constraint } = &e
                    && constraint.as_deref() == Some("idx_victims_cpf_unique")
                {
                    error!("[VictimService] Attempt to create victim with duplicate CPF");
                    return Err(AppError::Conflict(
                        "A victim with this CPF already exists.".to_string(),
                    ));
                }
                if let RepositoryError::UniqueViolation { constraint }
                | RepositoryError::ForeignKeyViolation { constraint } = &e
                    && let Some(app_err) = map_constraint(
                        constraint.as_deref(),
                        &[
                            ("fk_victims_city", "Error adding victim: city_id not found"),
                            (
                                "fk_victim_addresses_city",
                                "Error adding victim: address city_id not found",
                            ),
                        ],
                    )
                {
                    return Err(app_err);
                }
                error!("[VictimService] Failed to save victim: {:?}", e);
                Err(AppError::InternalServerError)
            }
        }
    }

    pub async fn get_victim_by_id(
        &self,
        id: Uuid,
        claims: &ClaimsToUserToken,
    ) -> Result<VictimWithDetails, AppError> {
        info!(
            "[VictimService] Starting find victim by id process for id: {}",
            id
        );

        match self.victim_read_repository.get_victim_by_id(id).await {
            Ok(victim_with_address) => {
                let auth = AuthContext::load(&*self.user_repository, claims).await?;
                auth.check_policy(&Policy::ReadVictims, victim_with_address.city_id)?;

                info!("[VictimService] Victim with id {} found successfully", id);
                Ok(victim_with_address)
            }
            Err(RepositoryError::NotFound) => {
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

    pub async fn get_all_victims(
        &self,
        pagination: Pagination,
        claims: &ClaimsToUserToken,
    ) -> Result<PaginatedResult<VictimWithDetails>, AppError> {
        info!("[VictimService] Starting process to get victims");

        let auth = AuthContext::load(&*self.user_repository, claims).await?;
        let allowed_cities = auth.allowed_cities(&Policy::ReadVictims);

        let total_items = self
            .victim_read_repository
            .count_victims(allowed_cities.as_deref())
            .await
            .map_err(|e| {
                error!("[VictimService] Failed to count victims: {:?}", e);
                AppError::InternalServerError
            })?;

        let victims_list = self
            .victim_read_repository
            .get_victims_paginated(
                allowed_cities.as_deref(),
                pagination.page_size,
                pagination.offset,
            )
            .await
            .map_err(|e| {
                error!("[VictimService] Failed to retrieve victims: {:?}", e);
                AppError::InternalServerError
            })?;

        info!(
            "[VictimService] Successfully retrieved {} victims (paged)",
            victims_list.len()
        );
        Ok(PaginatedResult {
            items: victims_list,
            page: pagination.page,
            page_size: pagination.page_size,
            total_items,
        })
    }

    pub async fn search_victims(
        &self,
        name: Option<String>,
        cpf: Option<String>,
        claims: &ClaimsToUserToken,
    ) -> Result<Vec<VictimWithDetails>, AppError> {
        info!("[VictimService] Starting victim search");

        let search = SearchCriteria::parse(name, cpf)
            .map_err(|e| AppError::BadRequest(format!("Error searching victims: {}", e)))?;

        let auth = AuthContext::load(&*self.user_repository, claims).await?;

        let victims = match search {
            SearchCriteria::ByName(name) => {
                self.victim_read_repository.get_victims_by_name(&name).await
            }
            SearchCriteria::ByCpf(cpf) => {
                self.victim_read_repository.get_victims_by_cpf(&cpf).await
            }
        };

        let victims = if let Some(allowed_cities) =
            auth.allowed_cities(&Policy::ReadVictims)
        {
            match victims {
                Ok(list) => {
                    let filtered: Vec<_> = list
                        .into_iter()
                        .filter(|v| allowed_cities.contains(&v.city_id))
                        .collect();
                    Ok(filtered)
                }
                Err(e) => Err(e),
            }
        } else {
            victims
        };

        match victims {
            Ok(victims_list) => {
                info!(
                    "[VictimService] Successfully retrieved {} victims from search",
                    victims_list.len()
                );
                Ok(victims_list)
            }
            Err(e) => {
                error!("[VictimService] Failed to search victims: {:?}", e);
                Err(AppError::InternalServerError)
            }
        }
    }

    pub async fn update_victim_by_id(
        &self,
        data: UpdateVictim,
        id: Uuid,
        claims: &ClaimsToUserToken,
    ) -> Result<VictimWithDetails, AppError> {
        info!("[VictimService] Starting victim update for id: {}", id);

        let auth = AuthContext::load(&*self.user_repository, claims).await?;
        let mut data = data;
        let city_id = resolve_city_id_from_addresses(&data.addresses, data.city_id)
            .map_err(|e| AppError::BadRequest(format!("Error updating victim: {}", e)))?;
        data.city_id = Some(city_id);

        let (has_special_needs, special_needs_type) =
            normalize_flag_from_list(&data.special_needs_type);
        data.has_special_needs = has_special_needs;
        data.special_needs_type = special_needs_type;

        let (has_psychiatric_issues, psychiatric_issues_type) =
            normalize_flag_from_list(&data.psychiatric_issues_type);
        data.has_psychiatric_issues = has_psychiatric_issues;
        data.psychiatric_issues_type = psychiatric_issues_type;
        data.has_children = data.children_count.is_some();

        if let Some(cpf) = data.cpf.as_ref() {
            let normalized = validate_cpf(cpf, "Error updating victim")?;
            data.cpf = Some(normalized);
        }

        auth.check_policy(&Policy::UpdateVictims, city_id)?;

        VictimValidator::validate_required_fields(&data.full_name, "Error updating victim")?;

        match self.victim_read_repository.get_victim_by_id(id).await {
            Ok(existing_victim) => {
                auth.check_policy(&Policy::UpdateVictims, existing_victim.city_id)?;
            }
            Err(RepositoryError::NotFound) => {
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

        match self
            .victim_write_repository
            .update_victim_by_id(data, id)
            .await
        {
            Ok(victim_with_address) => {
                let victim_with_address = victim_with_address.into_details();
                info!(
                    "[VictimService] Victim updated successfully with ID: {}",
                    victim_with_address.id
                );
                Ok(victim_with_address)
            }
            Err(RepositoryError::NotFound) => {
                error!("[VictimService] Victim with id {} not found for update", id);
                Err(AppError::NotFound(format!(
                    "Victim with id '{}' not found",
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
                                "fk_victims_city",
                                "Error updating victim: city_id not found",
                            ),
                            (
                                "fk_victim_addresses_city",
                                "Error updating victim: address city_id not found",
                            ),
                        ],
                    )
                {
                    return Err(app_err);
                }
                error!("[VictimService] Error updating victim in database: {:?}", e);
                Err(AppError::InternalServerError)
            }
        }
    }

    pub async fn delete_victim_by_id(
        &self,
        id: Uuid,
        claims: &ClaimsToUserToken,
    ) -> Result<VictimWithDetails, AppError> {
        info!(
            "[VictimService] Starting process to delete victim with id: {}",
            id
        );

        match self.victim_read_repository.get_victim_by_id(id).await {
            Ok(victim) => {
                let auth = AuthContext::load(&*self.user_repository, claims).await?;
                auth.check_policy(&Policy::DeleteVictims, victim.city_id)?;
            }
            Err(RepositoryError::NotFound) => {
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

        match self.victim_write_repository.delete_victim_by_id(id).await {
            Ok(deleted_victim) => {
                let deleted_victim = deleted_victim.into_details();
                info!("[VictimService] Victim with id {} deleted successfully", id);
                Ok(deleted_victim)
            }
            Err(RepositoryError::NotFound) => {
                info!(
                    "[VictimService] Victim with id {} not found for deletion",
                    id
                );
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
        claims: &ClaimsToUserToken,
    ) -> Result<VictimPhoneResponse, AppError> {
        info!("[VictimService] Adding phone to victim: {}", victim_id);

        let auth = AuthContext::load(&*self.user_repository, claims).await?;

        match self
            .victim_read_repository
            .get_victim_by_id(victim_id)
            .await
        {
            Ok(victim) => {
                auth.check_policy(&Policy::UpdateVictims, victim.city_id)?;
            }
            Err(RepositoryError::NotFound) => {
                return Err(AppError::NotFound(format!(
                    "Victim with id '{}' not found",
                    victim_id
                )));
            }
            Err(_) => return Err(AppError::InternalServerError),
        }

        match self
            .victim_write_repository
            .create_phone(victim_id, phone_data)
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
    ) -> Result<VictimPhoneResponse, AppError> {
        info!("[VictimService] Updating phone: {}", phone_id);

        let auth = AuthContext::load(&*self.user_repository, claims).await?;

        match self.victim_write_repository.get_phone_by_id(phone_id).await {
            Ok(phone) => {
                match self
                    .victim_read_repository
                    .get_victim_by_id(phone.victim_id)
                    .await
                {
                    Ok(victim) => {
                        auth.check_policy(&Policy::UpdateVictims, victim.city_id)?;
                    }
                    Err(e) => {
                        return match e {
                            RepositoryError::NotFound => Err(AppError::NotFound(
                                format!("Victim with id '{}' not found", phone.victim_id),
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
            .victim_write_repository
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
    ) -> Result<VictimPhoneResponse, AppError> {
        info!("[VictimService] Deleting phone: {}", phone_id);

        let auth = AuthContext::load(&*self.user_repository, claims).await?;

        match self.victim_write_repository.get_phone_by_id(phone_id).await {
            Ok(phone) => {
                match self
                    .victim_read_repository
                    .get_victim_by_id(phone.victim_id)
                    .await
                {
                    Ok(victim) => {
                        auth.check_policy(&Policy::UpdateVictims, victim.city_id)?;
                    }
                    Err(e) => {
                        return match e {
                            RepositoryError::NotFound => Err(AppError::NotFound(
                                format!("Victim with id '{}' not found", phone.victim_id),
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
            .victim_write_repository
            .delete_phone_by_id(phone_id)
            .await
        {
            Ok(phone) => Ok(phone.to_response()),
            Err(_) => Err(AppError::InternalServerError),
        }
    }

    pub async fn create_address(
        &self,
        victim_id: Uuid,
        address_data: AddressData,
        claims: &ClaimsToUserToken,
    ) -> Result<VictimAddressResponse, AppError> {
        info!("[VictimService] Adding address to victim: {}", victim_id);

        let auth = AuthContext::load(&*self.user_repository, claims).await?;

        match self
            .victim_read_repository
            .get_victim_by_id(victim_id)
            .await
        {
            Ok(victim) => {
                auth.check_policy(&Policy::UpdateVictims, victim.city_id)?;
            }
            Err(RepositoryError::NotFound) => {
                return Err(AppError::NotFound(format!(
                    "Victim with id '{}' not found",
                    victim_id
                )));
            }
            Err(_) => return Err(AppError::InternalServerError),
        }

        match self
            .victim_write_repository
            .create_address(victim_id, address_data)
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
    ) -> Result<VictimAddressResponse, AppError> {
        info!("[VictimService] Updating address: {}", address_id);

        let auth = AuthContext::load(&*self.user_repository, claims).await?;

        match self
            .victim_write_repository
            .get_address_by_id(address_id)
            .await
        {
            Ok(address) => {
                match self
                    .victim_read_repository
                    .get_victim_by_id(address.victim_id)
                    .await
                {
                    Ok(victim) => {
                        auth.check_policy(&Policy::UpdateVictims, victim.city_id)?;
                    }
                    Err(e) => {
                        return match e {
                            RepositoryError::NotFound => Err(AppError::NotFound(
                                format!("Victim with id '{}' not found", address.victim_id),
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
            .victim_write_repository
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
    ) -> Result<VictimAddressResponse, AppError> {
        info!("[VictimService] Deleting address: {}", address_id);

        let auth = AuthContext::load(&*self.user_repository, claims).await?;

        match self
            .victim_write_repository
            .get_address_by_id(address_id)
            .await
        {
            Ok(address) => {
                match self
                    .victim_read_repository
                    .get_victim_by_id(address.victim_id)
                    .await
                {
                    Ok(victim) => {
                        auth.check_policy(&Policy::UpdateVictims, victim.city_id)?;
                    }
                    Err(e) => {
                        return match e {
                            RepositoryError::NotFound => Err(AppError::NotFound(
                                format!("Victim with id '{}' not found", address.victim_id),
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
            .victim_write_repository
            .delete_address_by_id(address_id)
            .await
        {
            Ok(address) => Ok(address.to_response()),
            Err(_) => Err(AppError::InternalServerError),
        }
    }
}
