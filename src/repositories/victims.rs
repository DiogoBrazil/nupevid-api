use async_trait::async_trait;
use log::info;
use sqlx::PgPool;
use uuid::Uuid;

use crate::config::querys::victims::{VictimAddressesQueries, VictimPhonesQueries, VictimsQueries};
use crate::core::contracts::repository::victims::VictimRepository;
use crate::core::entities::victims::{
    AddressData, CreateVictim, PhoneData, UpdateVictim, Victim, VictimAddress, VictimPhone,
    VictimWithDetails,
};

#[derive(Clone)]
pub struct PgVictimRepository {
    pool: PgPool,
}

impl PgVictimRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    async fn create_phone_internal(
        tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
        victim_id: Uuid,
        phone_data: &PhoneData,
    ) -> Result<VictimPhone, sqlx::Error> {
        let phone_id = Uuid::new_v4();

        let created: VictimPhone = sqlx::query_as(VictimPhonesQueries::CREATE_VICTIM_PHONE)
            .bind(phone_id)
            .bind(victim_id)
            .bind(&phone_data.phone)
            .bind(&phone_data.phone_type)
            .fetch_one(&mut **tx)
            .await?;

        Ok(created)
    }

    async fn delete_phones_by_victim_id(
        tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
        victim_id: Uuid,
    ) -> Result<(), sqlx::Error> {
        let _: Vec<VictimPhone> =
            sqlx::query_as(VictimPhonesQueries::DELETE_VICTIM_PHONES_BY_VICTIM_ID)
                .bind(victim_id)
                .fetch_all(&mut **tx)
                .await?;
        Ok(())
    }

    async fn create_address_internal(
        tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
        victim_id: Uuid,
        address: &AddressData,
    ) -> Result<VictimAddress, sqlx::Error> {
        let address_id = Uuid::new_v4();

        let created: VictimAddress = sqlx::query_as(VictimAddressesQueries::CREATE_VICTIM_ADDRESS)
            .bind(address_id)
            .bind(victim_id)
            .bind(&address.street)
            .bind(&address.number)
            .bind(&address.district)
            .bind(&address.city_id)
            .bind(&address.zip_code)
            .bind(&address.complement)
            .bind(&address.address_type)
            .fetch_one(&mut **tx)
            .await?;

        Ok(created)
    }

    async fn delete_addresses_by_victim_id(
        tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
        victim_id: Uuid,
    ) -> Result<(), sqlx::Error> {
        let _: Vec<VictimAddress> =
            sqlx::query_as(VictimAddressesQueries::DELETE_VICTIM_ADDRESSES_BY_VICTIM_ID)
                .bind(victim_id)
                .fetch_all(&mut **tx)
                .await?;
        Ok(())
    }

    async fn get_phones_by_victim_id(
        &self,
        victim_id: Uuid,
    ) -> Result<Vec<VictimPhone>, sqlx::Error> {
        let phones: Vec<VictimPhone> =
            sqlx::query_as(VictimPhonesQueries::GET_VICTIM_PHONES_BY_VICTIM_ID)
                .bind(victim_id)
                .fetch_all(&self.pool)
                .await?;
        Ok(phones)
    }

    async fn get_addresses_by_victim_id(
        &self,
        victim_id: Uuid,
    ) -> Result<Vec<VictimAddress>, sqlx::Error> {
        let addresses: Vec<VictimAddress> =
            sqlx::query_as(VictimAddressesQueries::GET_VICTIM_ADDRESSES_BY_VICTIM_ID)
                .bind(victim_id)
                .fetch_all(&self.pool)
                .await?;
        Ok(addresses)
    }
}

#[async_trait]
impl VictimRepository for PgVictimRepository {
    async fn create_victim(&self, victim: CreateVictim) -> Result<VictimWithDetails, sqlx::Error> {
        let victim_id = Uuid::new_v4();

        info!(
            "[Repository] Starting transaction to create victim: {} with ID: {}",
            victim.full_name, victim_id
        );

        let mut tx = self.pool.begin().await?;

        let victim_created: Victim = sqlx::query_as(VictimsQueries::CREATE_VICTIM)
            .bind(victim_id)
            .bind(&victim.full_name)
            .bind(&victim.cpf)
            .bind(&victim.birth_date)
            .bind(victim.city_id)
            .bind(&victim.education_level)
            .bind(&victim.occupation)
            .bind(&victim.has_children)
            .bind(&victim.children_count)
            .bind(&victim.has_special_needs)
            .bind(&victim.special_needs_type)
            .bind(&victim.uses_alcohol)
            .bind(&victim.uses_drugs)
            .bind(&victim.has_psychiatric_issues)
            .bind(&victim.psychiatric_issues_type)
            .fetch_one(&mut *tx)
            .await?;

        info!("[Repository] Victim inserted, now creating phones if provided");

        let mut created_phones = Vec::new();
        if let Some(phones) = &victim.phones {
            for phone_data in phones {
                let phone = Self::create_phone_internal(&mut tx, victim_id, phone_data).await?;
                created_phones.push(phone);
            }
            info!("[Repository] Created {} phone(s)", created_phones.len());
        }

        info!("[Repository] Now creating addresses if provided");

        let mut created_addresses = Vec::new();
        if let Some(addresses) = &victim.addresses {
            for addr_data in addresses {
                let created_addr = Self::create_address_internal(&mut tx, victim_id, addr_data).await?;
                created_addresses.push(created_addr);
            }
            info!("[Repository] Created {} address(es)", created_addresses.len());
        }

        tx.commit().await?;

        info!(
            "[Repository] Transaction committed. Victim {} created successfully",
            victim_id
        );

        Ok(victim_created.with_details(created_phones, created_addresses))
    }

    async fn get_victim_by_id(&self, id: Uuid) -> Result<VictimWithDetails, sqlx::Error> {
        info!("[Repository] Fetching victim with id: {}", id);

        let victim: Victim = sqlx::query_as(VictimsQueries::GET_VICTIM_BY_ID)
            .bind(id)
            .fetch_one(&self.pool)
            .await?;

        let phones = self.get_phones_by_victim_id(id).await?;
        let addresses = self.get_addresses_by_victim_id(id).await?;

        info!("[Repository] Victim {} found with {} phone(s) and {} address(es)",
              id, phones.len(), addresses.len());

        Ok(victim.with_details(phones, addresses))
    }

    async fn get_all_victims(&self) -> Result<Vec<VictimWithDetails>, sqlx::Error> {
        info!("[Repository] Fetching all victims");

        let victims: Vec<Victim> = sqlx::query_as(VictimsQueries::GET_ALL_VICTIMS)
            .fetch_all(&self.pool)
            .await?;

        let mut result = Vec::with_capacity(victims.len());

        for victim in victims {
            let phones = self.get_phones_by_victim_id(victim.id).await?;
            let addresses = self.get_addresses_by_victim_id(victim.id).await?;
            result.push(victim.with_details(phones, addresses));
        }

        info!("[Repository] Found {} victims", result.len());

        Ok(result)
    }

    async fn get_victims_by_city(&self, city_id: Uuid) -> Result<Vec<VictimWithDetails>, sqlx::Error> {
        info!("[Repository] Fetching victims for city: {}", city_id);

        let victims: Vec<Victim> = sqlx::query_as(VictimsQueries::GET_VICTIMS_BY_CITY)
            .bind(city_id)
            .fetch_all(&self.pool)
            .await?;

        let mut result = Vec::with_capacity(victims.len());

        for victim in victims {
            let phones = self.get_phones_by_victim_id(victim.id).await?;
            let addresses = self.get_addresses_by_victim_id(victim.id).await?;
            result.push(victim.with_details(phones, addresses));
        }

        info!(
            "[Repository] Found {} victims for city: {}",
            result.len(),
            city_id
        );

        Ok(result)
    }

    async fn get_victims_paginated(
        &self,
        allowed_cities: Option<&[Uuid]>,
        limit: i64,
        offset: i64,
    ) -> Result<Vec<VictimWithDetails>, sqlx::Error> {
        info!("[Repository] Fetching victims paginated");

        let victims: Vec<Victim> = match allowed_cities {
            Some(cities) => sqlx::query_as(VictimsQueries::GET_VICTIMS_PAGED_BY_CITIES)
                .bind(cities)
                .bind(limit)
                .bind(offset)
                .fetch_all(&self.pool)
                .await?,
            None => sqlx::query_as(VictimsQueries::GET_VICTIMS_PAGED)
                .bind(limit)
                .bind(offset)
                .fetch_all(&self.pool)
                .await?,
        };

        let mut result = Vec::with_capacity(victims.len());

        for victim in victims {
            let phones = self.get_phones_by_victim_id(victim.id).await?;
            let addresses = self.get_addresses_by_victim_id(victim.id).await?;
            result.push(victim.with_details(phones, addresses));
        }

        info!("[Repository] Found {} victims (paged)", result.len());
        Ok(result)
    }

    async fn count_victims(&self, allowed_cities: Option<&[Uuid]>) -> Result<i64, sqlx::Error> {
        let total: i64 = match allowed_cities {
            Some(cities) => sqlx::query_scalar(VictimsQueries::COUNT_VICTIMS_BY_CITIES)
                .bind(cities)
                .fetch_one(&self.pool)
                .await?,
            None => sqlx::query_scalar(VictimsQueries::COUNT_VICTIMS)
                .fetch_one(&self.pool)
                .await?,
        };
        Ok(total)
    }

    async fn get_victims_by_name(&self, name: &str) -> Result<Vec<VictimWithDetails>, sqlx::Error> {
        let pattern = format!("%{}%", name);
        info!("[Repository] Fetching victims by name pattern: {}", pattern);

        let victims: Vec<Victim> = sqlx::query_as(VictimsQueries::GET_VICTIMS_BY_NAME)
            .bind(pattern)
            .fetch_all(&self.pool)
            .await?;

        let mut result = Vec::with_capacity(victims.len());

        for victim in victims {
            let phones = self.get_phones_by_victim_id(victim.id).await?;
            let addresses = self.get_addresses_by_victim_id(victim.id).await?;
            result.push(victim.with_details(phones, addresses));
        }

        info!("[Repository] Found {} victims by name", result.len());
        Ok(result)
    }

    async fn get_victims_by_cpf(&self, cpf: &str) -> Result<Vec<VictimWithDetails>, sqlx::Error> {
        info!("[Repository] Fetching victims by cpf");

        let victims: Vec<Victim> = sqlx::query_as(VictimsQueries::GET_VICTIMS_BY_CPF)
            .bind(cpf)
            .fetch_all(&self.pool)
            .await?;

        let mut result = Vec::with_capacity(victims.len());

        for victim in victims {
            let phones = self.get_phones_by_victim_id(victim.id).await?;
            let addresses = self.get_addresses_by_victim_id(victim.id).await?;
            result.push(victim.with_details(phones, addresses));
        }

        info!("[Repository] Found {} victims by cpf", result.len());
        Ok(result)
    }

    async fn update_victim_by_id(
        &self,
        data: UpdateVictim,
        id: Uuid,
    ) -> Result<VictimWithDetails, sqlx::Error> {
        info!("[Repository] Starting transaction to update victim: {}", id);

        let mut tx = self.pool.begin().await?;

        let victim_updated: Victim = sqlx::query_as(VictimsQueries::UPDATE_VICTIM_BY_ID)
            .bind(id)
            .bind(&data.full_name)
            .bind(&data.cpf)
            .bind(&data.birth_date)
            .bind(data.city_id)
            .bind(&data.education_level)
            .bind(&data.occupation)
            .bind(&data.has_children)
            .bind(&data.children_count)
            .bind(&data.has_special_needs)
            .bind(&data.special_needs_type)
            .bind(&data.uses_alcohol)
            .bind(&data.uses_drugs)
            .bind(&data.has_psychiatric_issues)
            .bind(&data.psychiatric_issues_type)
            .fetch_one(&mut *tx)
            .await?;

        Self::delete_phones_by_victim_id(&mut tx, id).await?;
        let mut updated_phones = Vec::new();
        if let Some(phones) = &data.phones {
            for phone_data in phones {
                let phone = Self::create_phone_internal(&mut tx, id, phone_data).await?;
                updated_phones.push(phone);
            }
            info!("[Repository] Updated {} phone(s) for victim: {}", phones.len(), id);
        }

        Self::delete_addresses_by_victim_id(&mut tx, id).await?;
        let mut updated_addresses = Vec::new();
        if let Some(addresses) = &data.addresses {
            for addr_data in addresses {
                let created_addr = Self::create_address_internal(&mut tx, id, addr_data).await?;
                updated_addresses.push(created_addr);
            }
            info!("[Repository] Updated {} address(es) for victim: {}", addresses.len(), id);
        }

        tx.commit().await?;

        info!("[Repository] Transaction committed. Victim {} updated", id);

        Ok(victim_updated.with_details(updated_phones, updated_addresses))
    }

    async fn delete_victim_by_id(&self, id: Uuid) -> Result<VictimWithDetails, sqlx::Error> {
        info!(
            "[Repository] Starting transaction to soft delete victim: {}",
            id
        );

        let phones = self.get_phones_by_victim_id(id).await?;
        let addresses = self.get_addresses_by_victim_id(id).await?;

        let mut tx = self.pool.begin().await?;

        Self::delete_phones_by_victim_id(&mut tx, id).await?;

        Self::delete_addresses_by_victim_id(&mut tx, id).await?;

        let deleted_victim: Victim = sqlx::query_as(VictimsQueries::DELETE_VICTIM_BY_ID)
            .bind(id)
            .fetch_one(&mut *tx)
            .await?;

        tx.commit().await?;

        info!(
            "[Repository] Transaction committed. Victim {} soft deleted",
            id
        );

        Ok(deleted_victim.with_details(phones, addresses))
    }

    async fn create_phone(&self, victim_id: Uuid, phone_data: PhoneData) -> Result<VictimPhone, sqlx::Error> {
        info!("[Repository] Creating phone for victim: {}", victim_id);
        let phone_id = Uuid::new_v4();

        let phone: VictimPhone = sqlx::query_as(VictimPhonesQueries::CREATE_VICTIM_PHONE)
            .bind(phone_id)
            .bind(victim_id)
            .bind(&phone_data.phone)
            .bind(&phone_data.phone_type)
            .fetch_one(&self.pool)
            .await?;

        info!("[Repository] Phone {} created successfully", phone_id);
        Ok(phone)
    }

    async fn get_phone_by_id(&self, phone_id: Uuid) -> Result<VictimPhone, sqlx::Error> {
        info!("[Repository] Fetching phone with id: {}", phone_id);

        let phone: VictimPhone = sqlx::query_as(VictimPhonesQueries::GET_VICTIM_PHONE_BY_ID)
            .bind(phone_id)
            .fetch_one(&self.pool)
            .await?;

        info!("[Repository] Phone {} found", phone_id);
        Ok(phone)
    }

    async fn update_phone_by_id(&self, phone_id: Uuid, phone_data: PhoneData) -> Result<VictimPhone, sqlx::Error> {
        info!("[Repository] Updating phone: {}", phone_id);

        let phone: VictimPhone = sqlx::query_as(VictimPhonesQueries::UPDATE_VICTIM_PHONE_BY_ID)
            .bind(phone_id)
            .bind(&phone_data.phone)
            .bind(&phone_data.phone_type)
            .fetch_one(&self.pool)
            .await?;

        info!("[Repository] Phone {} updated successfully", phone_id);
        Ok(phone)
    }

    async fn delete_phone_by_id(&self, phone_id: Uuid) -> Result<VictimPhone, sqlx::Error> {
        info!("[Repository] Soft deleting phone: {}", phone_id);

        let phone: VictimPhone = sqlx::query_as(VictimPhonesQueries::DELETE_VICTIM_PHONE_BY_ID)
            .bind(phone_id)
            .fetch_one(&self.pool)
            .await?;

        info!("[Repository] Phone {} soft deleted successfully", phone_id);
        Ok(phone)
    }

    async fn create_address(&self, victim_id: Uuid, address_data: AddressData) -> Result<VictimAddress, sqlx::Error> {
        info!("[Repository] Creating address for victim: {}", victim_id);
        let address_id = Uuid::new_v4();

        let address: VictimAddress = sqlx::query_as(VictimAddressesQueries::CREATE_VICTIM_ADDRESS)
            .bind(address_id)
            .bind(victim_id)
            .bind(&address_data.street)
            .bind(&address_data.number)
            .bind(&address_data.district)
            .bind(&address_data.city_id)
            .bind(&address_data.zip_code)
            .bind(&address_data.complement)
            .bind(&address_data.address_type)
            .fetch_one(&self.pool)
            .await?;

        info!("[Repository] Address {} created successfully", address_id);
        Ok(address)
    }

    async fn get_address_by_id(&self, address_id: Uuid) -> Result<VictimAddress, sqlx::Error> {
        info!("[Repository] Fetching address with id: {}", address_id);

        let address: VictimAddress = sqlx::query_as(VictimAddressesQueries::GET_VICTIM_ADDRESS_BY_ID)
            .bind(address_id)
            .fetch_one(&self.pool)
            .await?;

        info!("[Repository] Address {} found", address_id);
        Ok(address)
    }

    async fn update_address_by_id(&self, address_id: Uuid, address_data: AddressData) -> Result<VictimAddress, sqlx::Error> {
        info!("[Repository] Updating address: {}", address_id);

        let address: VictimAddress = sqlx::query_as(VictimAddressesQueries::UPDATE_VICTIM_ADDRESS_BY_ID)
            .bind(address_id)
            .bind(&address_data.street)
            .bind(&address_data.number)
            .bind(&address_data.district)
            .bind(&address_data.city_id)
            .bind(&address_data.zip_code)
            .bind(&address_data.complement)
            .bind(&address_data.address_type)
            .fetch_one(&self.pool)
            .await?;

        info!("[Repository] Address {} updated successfully", address_id);
        Ok(address)
    }

    async fn delete_address_by_id(&self, address_id: Uuid) -> Result<VictimAddress, sqlx::Error> {
        info!("[Repository] Soft deleting address: {}", address_id);

        let address: VictimAddress = sqlx::query_as(VictimAddressesQueries::DELETE_VICTIM_ADDRESS_BY_ID)
            .bind(address_id)
            .fetch_one(&self.pool)
            .await?;

        info!("[Repository] Address {} soft deleted successfully", address_id);
        Ok(address)
    }
}
