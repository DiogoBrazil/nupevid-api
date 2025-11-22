use async_trait::async_trait;
use log::info;
use sqlx::PgPool;
use uuid::Uuid;

use crate::core::{
    contracts::repository::victims::{
        VictimRepository,
        VictimAddressRepository
    },
    entities::victims::{
        CreateVictim,
        UpdateVictim,
        Victim,
        CreateVictimAddress,
        UpdateVictimAddress,
        VictimAddress
    }
};
use crate::config::querys::victims::{
    VictimsQueries,
    VictimAddressesQueries
};

#[derive(Clone)]
pub struct PgVictimRepository {
    pool: PgPool,
}

impl PgVictimRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl VictimRepository for PgVictimRepository {
    async fn create_victim(&self, victim: CreateVictim) -> Result<Victim, sqlx::Error> {
        let id: Uuid = Uuid::new_v4();

        info!("[Repository] Executing SQL query to create victim: {} with ID: {}", victim.full_name, id);

        let victim_created: Victim = sqlx::query_as(VictimsQueries::CREATE_VICTIM)
            .bind(id)
            .bind(victim.full_name)
            .bind(victim.document_id)
            .bind(victim.birth_date)
            .bind(victim.phone)
            .bind(victim.city_id)
            .fetch_one(&self.pool)
            .await?;

        info!("[Repository] Victim successfully inserted into database with ID: {}", victim_created.id);

        Ok(victim_created)
    }

    async fn get_victim_by_id(&self, id: Uuid) -> Result<Victim, sqlx::Error> {
        info!("[Repository] Executing SQL query to get victim with id: {}", id);

        let victim: Victim = sqlx::query_as(VictimsQueries::GET_VICTIM_BY_ID)
            .bind(id)
            .fetch_one(&self.pool)
            .await?;

        info!("[Repository] Victim successfully found in the database with ID: {}", id);

        Ok(victim)
    }

    async fn get_all_victims(&self) -> Result<Vec<Victim>, sqlx::Error> {
        info!("[Repository] Executing SQL query to get all victims");

        let victims: Vec<Victim> = sqlx::query_as(VictimsQueries::GET_ALL_VICTIMS)
            .fetch_all(&self.pool)
            .await?;

        info!("[Repository] Found {} victims in database", victims.len());

        Ok(victims)
    }

    async fn get_victims_by_city(&self, city_id: Uuid) -> Result<Vec<Victim>, sqlx::Error> {
        info!("[Repository] Executing SQL query to get victims for city: {}", city_id);

        let victims: Vec<Victim> = sqlx::query_as(VictimsQueries::GET_VICTIMS_BY_CITY)
            .bind(city_id)
            .fetch_all(&self.pool)
            .await?;

        info!("[Repository] Found {} victims for city: {}", victims.len(), city_id);

        Ok(victims)
    }

    async fn update_victim_by_id(&self, data: UpdateVictim, id: Uuid) -> Result<Victim, sqlx::Error> {
        info!("[Repository] Executing SQL query to update victim with ID: {}", id);

        let victim_updated: Victim = sqlx::query_as(VictimsQueries::UPDATE_VICTIM_BY_ID)
            .bind(id)
            .bind(data.full_name)
            .bind(data.document_id)
            .bind(data.birth_date)
            .bind(data.phone)
            .bind(data.city_id)
            .fetch_one(&self.pool)
            .await?;

        info!("[Repository] Victim successfully updated in database with ID: {}", id);

        Ok(victim_updated)
    }

    async fn delete_victim_by_id(&self, id: Uuid) -> Result<Victim, sqlx::Error> {
        info!("[Repository] Executing SQL query to soft delete victim with id: {}", id);

        let deleted_victim: Victim = sqlx::query_as(VictimsQueries::DELETE_VICTIM_BY_ID)
            .bind(id)
            .fetch_one(&self.pool)
            .await?;

        info!("[Repository] Victim successfully soft deleted from database with ID: {}", id);

        Ok(deleted_victim)
    }
}

#[derive(Clone)]
pub struct PgVictimAddressRepository {
    pool: PgPool,
}

impl PgVictimAddressRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl VictimAddressRepository for PgVictimAddressRepository {
    async fn create_victim_address(&self, address: CreateVictimAddress) -> Result<VictimAddress, sqlx::Error> {
        let id: Uuid = Uuid::new_v4();

        info!("[Repository] Executing SQL query to create victim address for victim: {}", address.victim_id);

        let address_created: VictimAddress = sqlx::query_as(VictimAddressesQueries::CREATE_VICTIM_ADDRESS)
            .bind(id)
            .bind(address.victim_id)
            .bind(address.street)
            .bind(address.number)
            .bind(address.district)
            .bind(address.city_name)
            .bind(address.state)
            .bind(address.zip_code)
            .bind(address.complement)
            .fetch_one(&self.pool)
            .await?;

        info!("[Repository] Victim address successfully inserted into database with ID: {}", address_created.id);

        Ok(address_created)
    }

    async fn get_victim_address_by_id(&self, id: Uuid) -> Result<VictimAddress, sqlx::Error> {
        info!("[Repository] Executing SQL query to get victim address with id: {}", id);

        let address: VictimAddress = sqlx::query_as(VictimAddressesQueries::GET_VICTIM_ADDRESS_BY_ID)
            .bind(id)
            .fetch_one(&self.pool)
            .await?;

        info!("[Repository] Victim address successfully found in the database with ID: {}", id);

        Ok(address)
    }

    async fn get_victim_address_by_victim_id(&self, victim_id: Uuid) -> Result<Option<VictimAddress>, sqlx::Error> {
        info!("[Repository] Executing SQL query to get victim address for victim: {}", victim_id);

        let address: Option<VictimAddress> = sqlx::query_as(VictimAddressesQueries::GET_VICTIM_ADDRESS_BY_VICTIM_ID)
            .bind(victim_id)
            .fetch_optional(&self.pool)
            .await?;

        match &address {
            Some(_) => info!("[Repository] Victim address found for victim: {}", victim_id),
            None => info!("[Repository] No address found for victim: {}", victim_id),
        }

        Ok(address)
    }

    async fn update_victim_address_by_id(&self, data: UpdateVictimAddress, id: Uuid) -> Result<VictimAddress, sqlx::Error> {
        info!("[Repository] Executing SQL query to update victim address with ID: {}", id);

        let address_updated: VictimAddress = sqlx::query_as(VictimAddressesQueries::UPDATE_VICTIM_ADDRESS_BY_ID)
            .bind(id)
            .bind(data.street)
            .bind(data.number)
            .bind(data.district)
            .bind(data.city_name)
            .bind(data.state)
            .bind(data.zip_code)
            .bind(data.complement)
            .fetch_one(&self.pool)
            .await?;

        info!("[Repository] Victim address successfully updated in database with ID: {}", id);

        Ok(address_updated)
    }

    async fn delete_victim_address_by_id(&self, id: Uuid) -> Result<VictimAddress, sqlx::Error> {
        info!("[Repository] Executing SQL query to soft delete victim address with id: {}", id);

        let deleted_address: VictimAddress = sqlx::query_as(VictimAddressesQueries::DELETE_VICTIM_ADDRESS_BY_ID)
            .bind(id)
            .fetch_one(&self.pool)
            .await?;

        info!("[Repository] Victim address successfully soft deleted from database with ID: {}", id);

        Ok(deleted_address)
    }
}
