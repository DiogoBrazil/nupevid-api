use async_trait::async_trait;
use log::info;
use sqlx::PgPool;
use uuid::Uuid;

use crate::config::querys::victims::{VictimAddressesQueries, VictimsQueries};
use crate::core::contracts::repository::victims::VictimRepository;
use crate::core::entities::victims::{
    AddressData, CreateVictim, UpdateVictim, Victim, VictimAddress, VictimWithAddress,
};

#[derive(Clone)]
pub struct PgVictimRepository {
    pool: PgPool,
}

impl PgVictimRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    async fn get_address_by_victim_id(
        &self,
        victim_id: Uuid,
    ) -> Result<Option<VictimAddress>, sqlx::Error> {
        let address: Option<VictimAddress> =
            sqlx::query_as(VictimAddressesQueries::GET_VICTIM_ADDRESS_BY_VICTIM_ID)
                .bind(victim_id)
                .fetch_optional(&self.pool)
                .await?;
        Ok(address)
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
            .bind(&address.city_name)
            .bind(&address.state)
            .bind(&address.zip_code)
            .bind(&address.complement)
            .fetch_one(&mut **tx)
            .await?;

        Ok(created)
    }

    async fn check_address_exists(
        tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
        victim_id: Uuid,
    ) -> Result<bool, sqlx::Error> {
        let result: (bool,) = sqlx::query_as(VictimAddressesQueries::CHECK_ADDRESS_EXISTS_FOR_VICTIM)
            .bind(victim_id)
            .fetch_one(&mut **tx)
            .await?;
        Ok(result.0)
    }

    async fn update_address_internal(
        tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
        victim_id: Uuid,
        address: &AddressData,
    ) -> Result<VictimAddress, sqlx::Error> {
        let updated: VictimAddress =
            sqlx::query_as(VictimAddressesQueries::UPDATE_VICTIM_ADDRESS_BY_VICTIM_ID)
                .bind(victim_id)
                .bind(&address.street)
                .bind(&address.number)
                .bind(&address.district)
                .bind(&address.city_name)
                .bind(&address.state)
                .bind(&address.zip_code)
                .bind(&address.complement)
                .fetch_one(&mut **tx)
                .await?;

        Ok(updated)
    }
}

#[async_trait]
impl VictimRepository for PgVictimRepository {
    async fn create_victim(&self, victim: CreateVictim) -> Result<VictimWithAddress, sqlx::Error> {
        let victim_id = Uuid::new_v4();

        info!(
            "[Repository] Starting transaction to create victim: {} with ID: {}",
            victim.full_name, victim_id
        );

        let mut tx = self.pool.begin().await?;

        let victim_created: Victim = sqlx::query_as(VictimsQueries::CREATE_VICTIM)
            .bind(victim_id)
            .bind(&victim.full_name)
            .bind(&victim.document_id)
            .bind(&victim.birth_date)
            .bind(&victim.phone)
            .bind(&victim.city_id)
            .fetch_one(&mut *tx)
            .await?;

        info!(
            "[Repository] Victim inserted, now creating address if provided"
        );

        let address = if let Some(addr_data) = &victim.address {
            let created_addr = Self::create_address_internal(&mut tx, victim_id, addr_data).await?;
            info!(
                "[Repository] Address created with ID: {}",
                created_addr.id
            );
            Some(created_addr)
        } else {
            None
        };

        tx.commit().await?;

        info!(
            "[Repository] Transaction committed. Victim {} created successfully",
            victim_id
        );

        Ok(victim_created.with_address(address))
    }

    async fn get_victim_by_id(&self, id: Uuid) -> Result<VictimWithAddress, sqlx::Error> {
        info!("[Repository] Fetching victim with id: {}", id);

        let victim: Victim = sqlx::query_as(VictimsQueries::GET_VICTIM_BY_ID)
            .bind(id)
            .fetch_one(&self.pool)
            .await?;

        let address = self.get_address_by_victim_id(id).await?;

        info!("[Repository] Victim {} found with address: {}", id, address.is_some());

        Ok(victim.with_address(address))
    }

    async fn get_all_victims(&self) -> Result<Vec<VictimWithAddress>, sqlx::Error> {
        info!("[Repository] Fetching all victims");

        let victims: Vec<Victim> = sqlx::query_as(VictimsQueries::GET_ALL_VICTIMS)
            .fetch_all(&self.pool)
            .await?;

        let mut result = Vec::with_capacity(victims.len());

        for victim in victims {
            let address = self.get_address_by_victim_id(victim.id).await?;
            result.push(victim.with_address(address));
        }

        info!("[Repository] Found {} victims", result.len());

        Ok(result)
    }

    async fn get_victims_by_city(&self, city_id: Uuid) -> Result<Vec<VictimWithAddress>, sqlx::Error> {
        info!("[Repository] Fetching victims for city: {}", city_id);

        let victims: Vec<Victim> = sqlx::query_as(VictimsQueries::GET_VICTIMS_BY_CITY)
            .bind(city_id)
            .fetch_all(&self.pool)
            .await?;

        let mut result = Vec::with_capacity(victims.len());

        for victim in victims {
            let address = self.get_address_by_victim_id(victim.id).await?;
            result.push(victim.with_address(address));
        }

        info!(
            "[Repository] Found {} victims for city: {}",
            result.len(),
            city_id
        );

        Ok(result)
    }

    async fn update_victim_by_id(
        &self,
        data: UpdateVictim,
        id: Uuid,
    ) -> Result<VictimWithAddress, sqlx::Error> {
        info!("[Repository] Starting transaction to update victim: {}", id);

        let mut tx = self.pool.begin().await?;

        let victim_updated: Victim = sqlx::query_as(VictimsQueries::UPDATE_VICTIM_BY_ID)
            .bind(id)
            .bind(&data.full_name)
            .bind(&data.document_id)
            .bind(&data.birth_date)
            .bind(&data.phone)
            .bind(&data.city_id)
            .fetch_one(&mut *tx)
            .await?;

        let address = if let Some(addr_data) = &data.address {
            let has_address = Self::check_address_exists(&mut tx, id).await?;

            let addr = if has_address {
                info!("[Repository] Updating existing address for victim: {}", id);
                Self::update_address_internal(&mut tx, id, addr_data).await?
            } else {
                info!("[Repository] Creating new address for victim: {}", id);
                Self::create_address_internal(&mut tx, id, addr_data).await?
            };

            Some(addr)
        } else {
            None
        };

        tx.commit().await?;

        info!("[Repository] Transaction committed. Victim {} updated", id);

        let final_address = if address.is_some() {
            address
        } else {
            self.get_address_by_victim_id(id).await?
        };

        Ok(victim_updated.with_address(final_address))
    }

    async fn delete_victim_by_id(&self, id: Uuid) -> Result<VictimWithAddress, sqlx::Error> {
        info!(
            "[Repository] Starting transaction to soft delete victim: {}",
            id
        );

        let address = self.get_address_by_victim_id(id).await?;

        let mut tx = self.pool.begin().await?;

        let _: Option<VictimAddress> =
            sqlx::query_as(VictimAddressesQueries::DELETE_VICTIM_ADDRESS_BY_VICTIM_ID)
                .bind(id)
                .fetch_optional(&mut *tx)
                .await?;

        let deleted_victim: Victim = sqlx::query_as(VictimsQueries::DELETE_VICTIM_BY_ID)
            .bind(id)
            .fetch_one(&mut *tx)
            .await?;

        tx.commit().await?;

        info!(
            "[Repository] Transaction committed. Victim {} soft deleted",
            id
        );

        Ok(deleted_victim.with_address(address))
    }
}
