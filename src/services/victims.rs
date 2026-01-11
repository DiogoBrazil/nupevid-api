use actix_web::{HttpRequest, HttpResponse, web};
use log::{error, info};
use uuid::Uuid;

use crate::core::contracts::repository::victims::VictimRepository;

use crate::core::entities::victims::{
    AddressData, AddressType, CreateVictim, PhoneData, UpdateVictim,
};
use crate::repositories::users::PgUserRepository;
use crate::repositories::victims::PgVictimRepository;

use crate::utils::{
    authorization::{check_policy, get_allowed_cities_for_policy},
    db_error_mapper::map_constraint,
    errors::AppError,
    pagination::{PaginationParams, normalize_pagination},
    responses::{ApiResponse, PaginatedResponse},
    service_helpers::{extract_claims, get_user_policies_with_defaults},
};
use crate::validators::{
    common::{
        POLICY_CREATE_VICTIMS, POLICY_DELETE_VICTIMS, POLICY_READ_VICTIMS, POLICY_UPDATE_VICTIMS,
    },
    cpf_validator::validate_cpf,
    victim_validator::VictimValidator,
};

pub struct VictimService {
    victim_repository: web::Data<PgVictimRepository>,
    user_repository: web::Data<PgUserRepository>,
}

enum VictimSearchCriteria {
    Name(String),
    Cpf(String),
}

impl VictimService {
    pub fn new(
        victim_repository: web::Data<PgVictimRepository>,
        user_repository: web::Data<PgUserRepository>,
    ) -> Self {
        Self {
            victim_repository,
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

    fn normalize_flag_from_list(values: &Option<Vec<String>>) -> (bool, Option<Vec<String>>) {
        match values {
            Some(list) if !list.is_empty() => (true, Some(list.clone())),
            _ => (false, None),
        }
    }

    fn parse_search_criteria(
        name: Option<String>,
        cpf: Option<String>,
        error_context: &str,
    ) -> Result<VictimSearchCriteria, AppError> {
        match (name, cpf) {
            (Some(_name), Some(_)) => Err(AppError::BadRequest(format!(
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
                Ok(VictimSearchCriteria::Name(trimmed.to_string()))
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
                Ok(VictimSearchCriteria::Cpf(normalized))
            }
        }
    }

    pub async fn create_victim(
        &self,
        victim: CreateVictim,
        req: HttpRequest,
    ) -> Result<HttpResponse, AppError> {
        let mut victim = victim;
        let city_id =
            Self::resolve_city_id(&victim.addresses, victim.city_id, "Error adding victim")?;
        victim.city_id = Some(city_id);

        let (has_special_needs, special_needs_type) =
            Self::normalize_flag_from_list(&victim.special_needs_type);
        victim.has_special_needs = has_special_needs;
        victim.special_needs_type = special_needs_type;

        let (has_psychiatric_issues, psychiatric_issues_type) =
            Self::normalize_flag_from_list(&victim.psychiatric_issues_type);
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

        let claims = extract_claims(&req)?;
        let policies = get_user_policies_with_defaults(&**self.user_repository, &claims).await?;

        check_policy(&claims, POLICY_CREATE_VICTIMS, city_id, &policies)?;

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
                    if db_err.is_unique_violation()
                        && db_err.constraint() == Some("idx_victims_cpf_unique")
                    {
                        error!("[VictimService] Attempt to create victim with duplicate CPF");
                        return Err(AppError::Conflict(
                            "A victim with this CPF already exists.".to_string(),
                        ));
                    }
                    if let Some(app_err) = map_constraint(
                        db_err.constraint(),
                        &[
                            ("fk_victims_city", "Error adding victim: city_id not found"),
                            (
                                "fk_victim_addresses_city",
                                "Error adding victim: address city_id not found",
                            ),
                        ],
                    ) {
                        return Err(app_err);
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
                let policies =
                    get_user_policies_with_defaults(&**self.user_repository, &claims).await?;
                check_policy(
                    &claims,
                    POLICY_READ_VICTIMS,
                    victim_with_address.city_id,
                    &policies,
                )?;

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

    pub async fn get_all_victims(
        &self,
        params: PaginationParams,
        req: HttpRequest,
    ) -> Result<HttpResponse, AppError> {
        info!("[VictimService] Starting process to get victims");

        let claims = extract_claims(&req)?;
        let policies = get_user_policies_with_defaults(&**self.user_repository, &claims).await?;
        let pagination = normalize_pagination(&params);
        let allowed_cities = get_allowed_cities_for_policy(&claims, POLICY_READ_VICTIMS, &policies);

        let total_items = self
            .victim_repository
            .count_victims(allowed_cities.as_deref())
            .await
            .map_err(|e| {
                error!("[VictimService] Failed to count victims: {:?}", e);
                AppError::InternalServerError
            })?;

        let victims_list = self
            .victim_repository
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
        Ok(PaginatedResponse::success(
            victims_list,
            pagination.page,
            pagination.page_size,
            total_items,
        )
        .into_response())
    }

    pub async fn search_victims(
        &self,
        name: Option<String>,
        cpf: Option<String>,
        req: HttpRequest,
    ) -> Result<HttpResponse, AppError> {
        info!("[VictimService] Starting victim search");

        let search = Self::parse_search_criteria(name, cpf, "Error searching victims")?;

        let claims = extract_claims(&req)?;
        let policies = get_user_policies_with_defaults(&**self.user_repository, &claims).await?;

        let victims = match search {
            VictimSearchCriteria::Name(name) => {
                self.victim_repository.get_victims_by_name(&name).await
            }
            VictimSearchCriteria::Cpf(cpf) => self.victim_repository.get_victims_by_cpf(&cpf).await,
        };

        let victims = if let Some(allowed_cities) =
            get_allowed_cities_for_policy(&claims, POLICY_READ_VICTIMS, &policies)
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
                Ok(ApiResponse::success(victims_list).into_response())
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
        req: HttpRequest,
    ) -> Result<HttpResponse, AppError> {
        info!("[VictimService] Starting victim update for id: {}", id);

        let claims = extract_claims(&req)?;
        let policies = get_user_policies_with_defaults(&**self.user_repository, &claims).await?;
        let mut data = data;
        let city_id =
            Self::resolve_city_id(&data.addresses, data.city_id, "Error updating victim")?;
        data.city_id = Some(city_id);

        let (has_special_needs, special_needs_type) =
            Self::normalize_flag_from_list(&data.special_needs_type);
        data.has_special_needs = has_special_needs;
        data.special_needs_type = special_needs_type;

        let (has_psychiatric_issues, psychiatric_issues_type) =
            Self::normalize_flag_from_list(&data.psychiatric_issues_type);
        data.has_psychiatric_issues = has_psychiatric_issues;
        data.psychiatric_issues_type = psychiatric_issues_type;
        data.has_children = data.children_count.is_some();

        if let Some(cpf) = data.cpf.as_ref() {
            let normalized = validate_cpf(cpf, "Error updating victim")?;
            data.cpf = Some(normalized);
        }

        check_policy(&claims, POLICY_UPDATE_VICTIMS, city_id, &policies)?;

        VictimValidator::validate_required_fields(&data.full_name, "Error updating victim")?;

        match self.victim_repository.get_victim_by_id(id).await {
            Ok(existing_victim) => {
                check_policy(
                    &claims,
                    POLICY_UPDATE_VICTIMS,
                    existing_victim.city_id,
                    &policies,
                )?;
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
                if let sqlx::Error::Database(db_err) = &e
                    && let Some(app_err) = map_constraint(
                        db_err.constraint(),
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
        req: HttpRequest,
    ) -> Result<HttpResponse, AppError> {
        info!(
            "[VictimService] Starting process to delete victim with id: {}",
            id
        );

        let claims = extract_claims(&req)?;

        match self.victim_repository.get_victim_by_id(id).await {
            Ok(victim) => {
                let policies =
                    get_user_policies_with_defaults(&**self.user_repository, &claims).await?;
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

        match self
            .victim_repository
            .create_phone(victim_id, phone_data)
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
        info!("[VictimService] Updating phone: {}", phone_id);

        let claims = extract_claims(&req)?;
        let policies = get_user_policies_with_defaults(&**self.user_repository, &claims).await?;

        match self.victim_repository.get_phone_by_id(phone_id).await {
            Ok(phone) => {
                match self
                    .victim_repository
                    .get_victim_by_id(phone.victim_id)
                    .await
                {
                    Ok(victim) => {
                        check_policy(&claims, POLICY_UPDATE_VICTIMS, victim.city_id, &policies)?;
                    }
                    Err(e) => {
                        return match e {
                            sqlx::Error::RowNotFound => Err(AppError::NotFound(format!(
                                "Victim with id '{}' not found",
                                phone.victim_id
                            ))),
                            _ => Err(AppError::InternalServerError),
                        };
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
            .victim_repository
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
        info!("[VictimService] Deleting phone: {}", phone_id);

        let claims = extract_claims(&req)?;
        let policies = get_user_policies_with_defaults(&**self.user_repository, &claims).await?;

        match self.victim_repository.get_phone_by_id(phone_id).await {
            Ok(phone) => {
                match self
                    .victim_repository
                    .get_victim_by_id(phone.victim_id)
                    .await
                {
                    Ok(victim) => {
                        check_policy(&claims, POLICY_UPDATE_VICTIMS, victim.city_id, &policies)?;
                    }
                    Err(e) => {
                        return match e {
                            sqlx::Error::RowNotFound => Err(AppError::NotFound(format!(
                                "Victim with id '{}' not found",
                                phone.victim_id
                            ))),
                            _ => Err(AppError::InternalServerError),
                        };
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

        match self
            .victim_repository
            .create_address(victim_id, address_data)
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
        info!("[VictimService] Updating address: {}", address_id);

        let claims = extract_claims(&req)?;
        let policies = get_user_policies_with_defaults(&**self.user_repository, &claims).await?;

        match self.victim_repository.get_address_by_id(address_id).await {
            Ok(address) => {
                match self
                    .victim_repository
                    .get_victim_by_id(address.victim_id)
                    .await
                {
                    Ok(victim) => {
                        check_policy(&claims, POLICY_UPDATE_VICTIMS, victim.city_id, &policies)?;
                    }
                    Err(e) => {
                        return match e {
                            sqlx::Error::RowNotFound => Err(AppError::NotFound(format!(
                                "Victim with id '{}' not found",
                                address.victim_id
                            ))),
                            _ => Err(AppError::InternalServerError),
                        };
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
            .victim_repository
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
        info!("[VictimService] Deleting address: {}", address_id);

        let claims = extract_claims(&req)?;
        let policies = get_user_policies_with_defaults(&**self.user_repository, &claims).await?;

        match self.victim_repository.get_address_by_id(address_id).await {
            Ok(address) => {
                match self
                    .victim_repository
                    .get_victim_by_id(address.victim_id)
                    .await
                {
                    Ok(victim) => {
                        check_policy(&claims, POLICY_UPDATE_VICTIMS, victim.city_id, &policies)?;
                    }
                    Err(e) => {
                        return match e {
                            sqlx::Error::RowNotFound => Err(AppError::NotFound(format!(
                                "Victim with id '{}' not found",
                                address.victim_id
                            ))),
                            _ => Err(AppError::InternalServerError),
                        };
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
            .victim_repository
            .delete_address_by_id(address_id)
            .await
        {
            Ok(address) => Ok(ApiResponse::success(address).into_response()),
            Err(_) => Err(AppError::InternalServerError),
        }
    }
}
