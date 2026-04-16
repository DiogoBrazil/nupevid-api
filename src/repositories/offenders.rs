use async_trait::async_trait;
use log::info;
use sqlx::PgPool;
use std::collections::HashMap;
use uuid::Uuid;

use crate::repositories::queries::offenders::{
    OffenderAddressesQueries, OffenderPhonesQueries, OffendersQueries,
};
use crate::core::commands::offenders::{CreateOffender, UpdateOffender};
use crate::core::contracts::repository::offenders::{
    OffenderReadRepository, OffenderWriteRepository,
};
use crate::core::entities::common::{AddressData, PhoneData};
use crate::core::entities::offenders::{
    Offender, OffenderAddress, OffenderPhone, OffenderWriteResult,
};
use crate::core::contracts::repository::error::RepositoryError;
use crate::core::read_models::offenders::OffenderWithDetails;

use super::models::offenders::{OffenderRow, OffenderPhoneRow, OffenderAddressRow};

use crate::repositories::error_mapper::map_sqlx_error;
fn map_offender_error(err: sqlx::Error) -> RepositoryError {
    let base = map_sqlx_error(err);
    match base {
        RepositoryError::ForeignKeyViolation { ref constraint } => {
            match constraint.as_deref() {
                Some("fk_offenders_city") => {
                    RepositoryError::ReferencedEntityNotFound("City not found".into())
                }
                Some("fk_offender_addresses_city") => {
                    RepositoryError::ReferencedEntityNotFound("Address city not found".into())
                }
                _ => base,
            }
        }
        _ => base,
    }
}

#[derive(Clone)]
pub struct PgOffenderRepository {
    pool: PgPool,
}

impl PgOffenderRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    async fn create_phone_internal(
        tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
        offender_id: Uuid,
        phone_data: &PhoneData,
    ) -> Result<OffenderPhone, RepositoryError> {
        let phone_id = Uuid::new_v4();

        let created: OffenderPhone = sqlx::query_as::<_, OffenderPhoneRow>(OffenderPhonesQueries::CREATE_OFFENDER_PHONE)
            .bind(phone_id)
            .bind(offender_id)
            .bind(&phone_data.phone)
            .bind(&phone_data.phone_type)
            .fetch_one(&mut **tx)
            .await
            .map_err(map_offender_error)?
            .into();

        Ok(created)
    }

    async fn delete_phones_by_offender_id(
        tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
        offender_id: Uuid,
    ) -> Result<(), RepositoryError> {
        let _: Vec<OffenderPhoneRow> =
            sqlx::query_as::<_, OffenderPhoneRow>(OffenderPhonesQueries::DELETE_OFFENDER_PHONES_BY_OFFENDER_ID)
                .bind(offender_id)
                .fetch_all(&mut **tx)
                .await
                .map_err(map_offender_error)?;
        Ok(())
    }

    async fn create_address_internal(
        tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
        offender_id: Uuid,
        address: &AddressData,
    ) -> Result<OffenderAddress, RepositoryError> {
        let address_id = Uuid::new_v4();

        let created: OffenderAddress =
            sqlx::query_as::<_, OffenderAddressRow>(OffenderAddressesQueries::CREATE_OFFENDER_ADDRESS)
                .bind(address_id)
                .bind(offender_id)
                .bind(&address.street)
                .bind(&address.number)
                .bind(&address.district)
                .bind(address.city_id)
                .bind(&address.zip_code)
                .bind(&address.complement)
                .bind(&address.address_type)
                .fetch_one(&mut **tx)
                .await
                .map_err(map_offender_error)?
                .into();

        Ok(created)
    }

    async fn delete_addresses_by_offender_id(
        tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
        offender_id: Uuid,
    ) -> Result<(), RepositoryError> {
        let _: Vec<OffenderAddressRow> =
            sqlx::query_as::<_, OffenderAddressRow>(OffenderAddressesQueries::DELETE_OFFENDER_ADDRESSES_BY_OFFENDER_ID)
                .bind(offender_id)
                .fetch_all(&mut **tx)
                .await
                .map_err(map_offender_error)?;
        Ok(())
    }

    async fn get_phones_by_offender_id(
        &self,
        offender_id: Uuid,
    ) -> Result<Vec<OffenderPhone>, RepositoryError> {
        let phones: Vec<OffenderPhone> =
            sqlx::query_as::<_, OffenderPhoneRow>(OffenderPhonesQueries::GET_OFFENDER_PHONES_BY_OFFENDER_ID)
                .bind(offender_id)
                .fetch_all(&self.pool)
                .await
                .map_err(map_offender_error)?
                .into_iter().map(Into::into).collect();
        Ok(phones)
    }

    async fn get_addresses_by_offender_id(
        &self,
        offender_id: Uuid,
    ) -> Result<Vec<OffenderAddress>, RepositoryError> {
        let addresses: Vec<OffenderAddress> =
            sqlx::query_as::<_, OffenderAddressRow>(OffenderAddressesQueries::GET_OFFENDER_ADDRESSES_BY_OFFENDER_ID)
                .bind(offender_id)
                .fetch_all(&self.pool)
                .await
                .map_err(map_offender_error)?
                .into_iter().map(Into::into).collect();
        Ok(addresses)
    }

    async fn get_phones_by_offender_ids(
        &self,
        offender_ids: &[Uuid],
    ) -> Result<HashMap<Uuid, Vec<OffenderPhone>>, RepositoryError> {
        if offender_ids.is_empty() {
            return Ok(HashMap::new());
        }

        let phones: Vec<OffenderPhone> = sqlx::query_as::<_, OffenderPhoneRow>(
            OffenderPhonesQueries::GET_OFFENDER_PHONES_BY_OFFENDER_IDS,
        )
        .bind(offender_ids)
        .fetch_all(&self.pool)
        .await
        .map_err(map_offender_error)?
        .into_iter()
        .map(Into::into)
        .collect();

        let mut grouped = HashMap::new();
        for phone in phones {
            grouped
                .entry(phone.offender_id)
                .or_insert_with(Vec::new)
                .push(phone);
        }

        Ok(grouped)
    }

    async fn get_addresses_by_offender_ids(
        &self,
        offender_ids: &[Uuid],
    ) -> Result<HashMap<Uuid, Vec<OffenderAddress>>, RepositoryError> {
        if offender_ids.is_empty() {
            return Ok(HashMap::new());
        }

        let addresses: Vec<OffenderAddress> = sqlx::query_as::<_, OffenderAddressRow>(
            OffenderAddressesQueries::GET_OFFENDER_ADDRESSES_BY_OFFENDER_IDS,
        )
        .bind(offender_ids)
        .fetch_all(&self.pool)
        .await
        .map_err(map_offender_error)?
        .into_iter()
        .map(Into::into)
        .collect();

        let mut grouped = HashMap::new();
        for address in addresses {
            grouped
                .entry(address.offender_id)
                .or_insert_with(Vec::new)
                .push(address);
        }

        Ok(grouped)
    }

    async fn assemble_offenders_with_details(
        &self,
        offenders: Vec<Offender>,
    ) -> Result<Vec<OffenderWithDetails>, RepositoryError> {
        if offenders.is_empty() {
            return Ok(Vec::new());
        }

        let offender_ids: Vec<Uuid> = offenders.iter().map(|offender| offender.id).collect();
        let mut phones_by_offender = self.get_phones_by_offender_ids(&offender_ids).await?;
        let mut addresses_by_offender =
            self.get_addresses_by_offender_ids(&offender_ids).await?;

        Ok(offenders
            .into_iter()
            .map(|offender| {
                let phones = phones_by_offender.remove(&offender.id).unwrap_or_default();
                let addresses = addresses_by_offender
                    .remove(&offender.id)
                    .unwrap_or_default();
                OffenderWithDetails::from_entity(offender, phones, addresses)
            })
            .collect())
    }
}

#[async_trait]
impl OffenderReadRepository for PgOffenderRepository {
    async fn get_offender_by_id(
        &self,
        id: Uuid,
    ) -> Result<OffenderWithDetails, RepositoryError> {
        info!("[Repository] Fetching offender with id: {}", id);

        let offender: Offender = sqlx::query_as::<_, OffenderRow>(OffendersQueries::GET_OFFENDER_BY_ID)
            .bind(id)
            .fetch_one(&self.pool)
            .await
            .map_err(map_offender_error)?
            .into();

        let phones = self.get_phones_by_offender_id(id).await?;
        let addresses = self.get_addresses_by_offender_id(id).await?;

        info!(
            "[Repository] Offender {} found with {} phone(s) and {} address(es)",
            id,
            phones.len(),
            addresses.len(),
        );

        Ok(OffenderWithDetails::from_entity(offender, phones, addresses))
    }

    async fn get_all_offenders(
        &self,
    ) -> Result<Vec<OffenderWithDetails>, RepositoryError> {
        info!("[Repository] Fetching all offenders");

        let offenders: Vec<Offender> = sqlx::query_as::<_, OffenderRow>(OffendersQueries::GET_ALL_OFFENDERS)
            .fetch_all(&self.pool)
            .await
            .map_err(map_offender_error)?
            .into_iter().map(Into::into).collect();
        let result = self.assemble_offenders_with_details(offenders).await?;

        info!("[Repository] Found {} offenders", result.len());

        Ok(result)
    }

    async fn get_offenders_by_city(
        &self,
        city_id: Uuid,
    ) -> Result<Vec<OffenderWithDetails>, RepositoryError> {
        info!("[Repository] Fetching offenders for city: {}", city_id);

        let offenders: Vec<Offender> = sqlx::query_as::<_, OffenderRow>(OffendersQueries::GET_OFFENDERS_BY_CITY)
            .bind(city_id)
            .fetch_all(&self.pool)
            .await
            .map_err(map_offender_error)?
            .into_iter().map(Into::into).collect();
        let result = self.assemble_offenders_with_details(offenders).await?;

        info!(
            "[Repository] Found {} offenders for city: {}",
            result.len(),
            city_id
        );

        Ok(result)
    }

    async fn get_offenders_by_name(
        &self,
        name: &str,
    ) -> Result<Vec<OffenderWithDetails>, RepositoryError> {
        let pattern = format!("%{}%", name);
        info!(
            "[Repository] Fetching offenders by name pattern: {}",
            pattern
        );

        let offenders: Vec<Offender> = sqlx::query_as::<_, OffenderRow>(OffendersQueries::GET_OFFENDERS_BY_NAME)
            .bind(pattern)
            .fetch_all(&self.pool)
            .await
            .map_err(map_offender_error)?
            .into_iter().map(Into::into).collect();
        let result = self.assemble_offenders_with_details(offenders).await?;

        info!("[Repository] Found {} offenders by name", result.len());
        Ok(result)
    }

    async fn get_offenders_by_cpf(
        &self,
        cpf: &str,
    ) -> Result<Vec<OffenderWithDetails>, RepositoryError> {
        info!("[Repository] Fetching offenders by cpf");

        let offenders: Vec<Offender> = sqlx::query_as::<_, OffenderRow>(OffendersQueries::GET_OFFENDERS_BY_CPF)
            .bind(cpf)
            .fetch_all(&self.pool)
            .await
            .map_err(map_offender_error)?
            .into_iter().map(Into::into).collect();
        let result = self.assemble_offenders_with_details(offenders).await?;

        info!("[Repository] Found {} offenders by cpf", result.len());
        Ok(result)
    }

    async fn get_offenders_by_victim_id(
        &self,
        victim_id: Uuid,
    ) -> Result<Vec<OffenderWithDetails>, RepositoryError> {
        info!("[Repository] Fetching offenders for victim: {}", victim_id);

        let offenders: Vec<Offender> = sqlx::query_as::<_, OffenderRow>(OffendersQueries::GET_OFFENDERS_BY_VICTIM_ID)
            .bind(victim_id)
            .fetch_all(&self.pool)
            .await
            .map_err(map_offender_error)?
            .into_iter().map(Into::into).collect();
        let result = self.assemble_offenders_with_details(offenders).await?;

        info!(
            "[Repository] Found {} offenders for victim: {}",
            result.len(),
            victim_id
        );

        Ok(result)
    }

    async fn get_offenders_paginated(
        &self,
        allowed_cities: Option<&[Uuid]>,
        limit: i64,
        offset: i64,
    ) -> Result<Vec<OffenderWithDetails>, RepositoryError> {
        info!("[Repository] Fetching offenders paginated");

        let offenders: Vec<Offender> = match allowed_cities {
            Some(cities) => sqlx::query_as::<_, OffenderRow>(OffendersQueries::GET_OFFENDERS_PAGED_BY_CITIES)
                .bind(cities)
                .bind(limit)
                .bind(offset)
                .fetch_all(&self.pool)
                .await
                .map_err(map_offender_error)?
                .into_iter().map(Into::into).collect(),
            None => sqlx::query_as::<_, OffenderRow>(OffendersQueries::GET_OFFENDERS_PAGED)
                .bind(limit)
                .bind(offset)
                .fetch_all(&self.pool)
                .await
                .map_err(map_offender_error)?
                .into_iter().map(Into::into).collect(),
        };
        let result = self.assemble_offenders_with_details(offenders).await?;

        info!("[Repository] Found {} offenders (paged)", result.len());
        Ok(result)
    }

    async fn count_offenders(
        &self,
        allowed_cities: Option<&[Uuid]>,
    ) -> Result<i64, RepositoryError> {
        let total: i64 = match allowed_cities {
            Some(cities) => sqlx::query_scalar(OffendersQueries::COUNT_OFFENDERS_BY_CITIES)
                .bind(cities)
                .fetch_one(&self.pool)
                .await
                .map_err(map_offender_error)?,
            None => sqlx::query_scalar(OffendersQueries::COUNT_OFFENDERS)
                .fetch_one(&self.pool)
                .await
                .map_err(map_offender_error)?,
        };
        Ok(total)
    }
}

#[async_trait]
impl OffenderWriteRepository for PgOffenderRepository {
    async fn create_offender(
        &self,
        offender: CreateOffender,
    ) -> Result<OffenderWriteResult, RepositoryError> {
        let offender_id = Uuid::new_v4();

        info!(
            "[Repository] Starting transaction to create offender: {} with ID: {}",
            offender.full_name, offender_id
        );

        let mut tx = self
            .pool
            .begin()
            .await
            .map_err(map_offender_error)?;

        let offender_created: Offender = sqlx::query_as::<_, OffenderRow>(OffendersQueries::CREATE_OFFENDER)
            .bind(offender_id)
            .bind(&offender.full_name)
            .bind(&offender.cpf)
            .bind(offender.birth_date)
            .bind(offender.city_id)
            .bind(offender.imprisoned)
            .bind(&offender.occupation)
            .bind(offender.is_public_security_agent)
            .bind(&offender.security_force)
            .bind(offender.uses_alcohol)
            .bind(offender.uses_drugs)
            .bind(offender.has_psychiatric_issues)
            .bind(&offender.psychiatric_issues_type)
            .bind(&offender.education_level)
            .bind(&offender.observation)
            .fetch_one(&mut *tx)
            .await
            .map_err(map_offender_error)?
            .into();

        info!("[Repository] Offender inserted, now creating phones if provided");

        let mut created_phones = Vec::new();
        if let Some(phones) = &offender.phones {
            for phone_data in phones {
                let phone = Self::create_phone_internal(&mut tx, offender_id, phone_data).await?;
                created_phones.push(phone);
            }
            info!("[Repository] Created {} phone(s)", created_phones.len());
        }

        info!("[Repository] Now creating addresses if provided");

        let mut created_addresses = Vec::new();
        if let Some(addresses) = &offender.addresses {
            for addr_data in addresses {
                let created_addr =
                    Self::create_address_internal(&mut tx, offender_id, addr_data).await?;
                created_addresses.push(created_addr);
            }
            info!(
                "[Repository] Created {} address(es)",
                created_addresses.len()
            );
        }

        tx.commit()
            .await
            .map_err(map_offender_error)?;

        info!(
            "[Repository] Transaction committed. Offender {} created successfully",
            offender_id
        );

        Ok(OffenderWriteResult {
            offender: offender_created,
            phones: created_phones,
            addresses: created_addresses,
        })
    }

    async fn update_offender_by_id(
        &self,
        data: UpdateOffender,
        id: Uuid,
    ) -> Result<OffenderWriteResult, RepositoryError> {
        info!(
            "[Repository] Starting transaction to update offender: {}",
            id
        );

        let mut tx = self
            .pool
            .begin()
            .await
            .map_err(map_offender_error)?;

        let offender_updated: Offender = sqlx::query_as::<_, OffenderRow>(OffendersQueries::UPDATE_OFFENDER_BY_ID)
            .bind(id)
            .bind(&data.full_name)
            .bind(&data.cpf)
            .bind(data.birth_date)
            .bind(data.city_id)
            .bind(data.imprisoned)
            .bind(&data.occupation)
            .bind(data.is_public_security_agent)
            .bind(&data.security_force)
            .bind(data.uses_alcohol)
            .bind(data.uses_drugs)
            .bind(data.has_psychiatric_issues)
            .bind(&data.psychiatric_issues_type)
            .bind(&data.education_level)
            .bind(&data.observation)
            .fetch_one(&mut *tx)
            .await
            .map_err(map_offender_error)?
            .into();

        Self::delete_phones_by_offender_id(&mut tx, id).await?;
        let mut updated_phones = Vec::new();
        if let Some(phones) = &data.phones {
            for phone_data in phones {
                let phone = Self::create_phone_internal(&mut tx, id, phone_data).await?;
                updated_phones.push(phone);
            }
            info!(
                "[Repository] Updated {} phone(s) for offender: {}",
                phones.len(),
                id
            );
        }

        Self::delete_addresses_by_offender_id(&mut tx, id).await?;
        let mut updated_addresses = Vec::new();
        if let Some(addresses) = &data.addresses {
            for addr_data in addresses {
                let created_addr = Self::create_address_internal(&mut tx, id, addr_data).await?;
                updated_addresses.push(created_addr);
            }
            info!(
                "[Repository] Updated {} address(es) for offender: {}",
                addresses.len(),
                id
            );
        }

        tx.commit()
            .await
            .map_err(map_offender_error)?;

        info!(
            "[Repository] Transaction committed. Offender {} updated",
            id
        );

        Ok(OffenderWriteResult {
            offender: offender_updated,
            phones: updated_phones,
            addresses: updated_addresses,
        })
    }

    async fn delete_offender_by_id(
        &self,
        id: Uuid,
    ) -> Result<OffenderWriteResult, RepositoryError> {
        info!(
            "[Repository] Starting transaction to soft delete offender: {}",
            id
        );

        let phones = self.get_phones_by_offender_id(id).await?;
        let addresses = self.get_addresses_by_offender_id(id).await?;

        let mut tx = self
            .pool
            .begin()
            .await
            .map_err(map_offender_error)?;

        Self::delete_phones_by_offender_id(&mut tx, id).await?;
        Self::delete_addresses_by_offender_id(&mut tx, id).await?;
        let deleted_offender: Offender = sqlx::query_as::<_, OffenderRow>(OffendersQueries::DELETE_OFFENDER_BY_ID)
            .bind(id)
            .fetch_one(&mut *tx)
            .await
            .map_err(map_offender_error)?
            .into();

        tx.commit()
            .await
            .map_err(map_offender_error)?;

        info!(
            "[Repository] Transaction committed. Offender {} soft deleted",
            id
        );

        Ok(OffenderWriteResult {
            offender: deleted_offender,
            phones,
            addresses,
        })
    }

    async fn create_phone(
        &self,
        offender_id: Uuid,
        phone_data: PhoneData,
    ) -> Result<OffenderPhone, RepositoryError> {
        info!("[Repository] Creating phone for offender: {}", offender_id);
        let phone_id = Uuid::new_v4();

        let phone: OffenderPhone = sqlx::query_as::<_, OffenderPhoneRow>(OffenderPhonesQueries::CREATE_OFFENDER_PHONE)
            .bind(phone_id)
            .bind(offender_id)
            .bind(&phone_data.phone)
            .bind(&phone_data.phone_type)
            .fetch_one(&self.pool)
            .await
            .map_err(map_offender_error)?
            .into();

        info!("[Repository] Phone {} created successfully", phone_id);
        Ok(phone)
    }

    async fn get_phone_by_id(
        &self,
        phone_id: Uuid,
    ) -> Result<OffenderPhone, RepositoryError> {
        info!("[Repository] Fetching phone with id: {}", phone_id);

        let phone: OffenderPhone = sqlx::query_as::<_, OffenderPhoneRow>(OffenderPhonesQueries::GET_OFFENDER_PHONE_BY_ID)
            .bind(phone_id)
            .fetch_one(&self.pool)
            .await
            .map_err(map_offender_error)?
            .into();

        info!("[Repository] Phone {} found", phone_id);
        Ok(phone)
    }

    async fn update_phone_by_id(
        &self,
        phone_id: Uuid,
        phone_data: PhoneData,
    ) -> Result<OffenderPhone, RepositoryError> {
        info!("[Repository] Updating phone: {}", phone_id);

        let phone: OffenderPhone =
            sqlx::query_as::<_, OffenderPhoneRow>(OffenderPhonesQueries::UPDATE_OFFENDER_PHONE_BY_ID)
                .bind(phone_id)
                .bind(&phone_data.phone)
                .bind(&phone_data.phone_type)
                .fetch_one(&self.pool)
                .await
                .map_err(map_offender_error)?
                .into();

        info!("[Repository] Phone {} updated successfully", phone_id);
        Ok(phone)
    }

    async fn delete_phone_by_id(
        &self,
        phone_id: Uuid,
    ) -> Result<OffenderPhone, RepositoryError> {
        info!("[Repository] Soft deleting phone: {}", phone_id);

        let phone: OffenderPhone =
            sqlx::query_as::<_, OffenderPhoneRow>(OffenderPhonesQueries::DELETE_OFFENDER_PHONE_BY_ID)
                .bind(phone_id)
                .fetch_one(&self.pool)
                .await
                .map_err(map_offender_error)?
                .into();

        info!("[Repository] Phone {} soft deleted successfully", phone_id);
        Ok(phone)
    }

    async fn create_address(
        &self,
        offender_id: Uuid,
        address_data: AddressData,
    ) -> Result<OffenderAddress, RepositoryError> {
        info!(
            "[Repository] Creating address for offender: {}",
            offender_id
        );
        let address_id = Uuid::new_v4();

        let address: OffenderAddress =
            sqlx::query_as::<_, OffenderAddressRow>(OffenderAddressesQueries::CREATE_OFFENDER_ADDRESS)
                .bind(address_id)
                .bind(offender_id)
                .bind(&address_data.street)
                .bind(&address_data.number)
                .bind(&address_data.district)
                .bind(address_data.city_id)
                .bind(&address_data.zip_code)
                .bind(&address_data.complement)
                .bind(&address_data.address_type)
                .fetch_one(&self.pool)
                .await
                .map_err(map_offender_error)?
                .into();

        info!("[Repository] Address {} created successfully", address_id);
        Ok(address)
    }

    async fn get_address_by_id(
        &self,
        address_id: Uuid,
    ) -> Result<OffenderAddress, RepositoryError> {
        info!("[Repository] Fetching address with id: {}", address_id);

        let address: OffenderAddress =
            sqlx::query_as::<_, OffenderAddressRow>(OffenderAddressesQueries::GET_OFFENDER_ADDRESS_BY_ID)
                .bind(address_id)
                .fetch_one(&self.pool)
                .await
                .map_err(map_offender_error)?
                .into();

        info!("[Repository] Address {} found", address_id);
        Ok(address)
    }

    async fn update_address_by_id(
        &self,
        address_id: Uuid,
        address_data: AddressData,
    ) -> Result<OffenderAddress, RepositoryError> {
        info!("[Repository] Updating address: {}", address_id);

        let address: OffenderAddress =
            sqlx::query_as::<_, OffenderAddressRow>(OffenderAddressesQueries::UPDATE_OFFENDER_ADDRESS_BY_ID)
                .bind(address_id)
                .bind(&address_data.street)
                .bind(&address_data.number)
                .bind(&address_data.district)
                .bind(address_data.city_id)
                .bind(&address_data.zip_code)
                .bind(&address_data.complement)
                .bind(&address_data.address_type)
                .fetch_one(&self.pool)
                .await
                .map_err(map_offender_error)?
                .into();

        info!("[Repository] Address {} updated successfully", address_id);
        Ok(address)
    }

    async fn delete_address_by_id(
        &self,
        address_id: Uuid,
    ) -> Result<OffenderAddress, RepositoryError> {
        info!("[Repository] Soft deleting address: {}", address_id);

        let address: OffenderAddress =
            sqlx::query_as::<_, OffenderAddressRow>(OffenderAddressesQueries::DELETE_OFFENDER_ADDRESS_BY_ID)
                .bind(address_id)
                .fetch_one(&self.pool)
                .await
                .map_err(map_offender_error)?
                .into();

        info!(
            "[Repository] Address {} soft deleted successfully",
            address_id
        );
        Ok(address)
    }
}
