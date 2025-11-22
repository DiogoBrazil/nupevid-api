use async_trait::async_trait;
use uuid::Uuid;
use crate::core::entities::victims::{
    CreateVictim,
    UpdateVictim,
    Victim,
    CreateVictimAddress,
    UpdateVictimAddress,
    VictimAddress
};

#[async_trait]
pub trait VictimRepository: Send + Sync {
    async fn create_victim(&self, victim: CreateVictim) -> Result<Victim, sqlx::Error>;
    async fn get_victim_by_id(&self, id: Uuid) -> Result<Victim, sqlx::Error>;
    async fn get_all_victims(&self) -> Result<Vec<Victim>, sqlx::Error>;
    async fn get_victims_by_city(&self, city_id: Uuid) -> Result<Vec<Victim>, sqlx::Error>;
    async fn update_victim_by_id(&self, data: UpdateVictim, id: Uuid) -> Result<Victim, sqlx::Error>;
    async fn delete_victim_by_id(&self, id: Uuid) -> Result<Victim, sqlx::Error>;
}

#[async_trait]
pub trait VictimAddressRepository: Send + Sync {
    async fn create_victim_address(&self, address: CreateVictimAddress) -> Result<VictimAddress, sqlx::Error>;
    async fn get_victim_address_by_id(&self, id: Uuid) -> Result<VictimAddress, sqlx::Error>;
    async fn get_victim_address_by_victim_id(&self, victim_id: Uuid) -> Result<Option<VictimAddress>, sqlx::Error>;
    async fn update_victim_address_by_id(&self, data: UpdateVictimAddress, id: Uuid) -> Result<VictimAddress, sqlx::Error>;
    async fn delete_victim_address_by_id(&self, id: Uuid) -> Result<VictimAddress, sqlx::Error>;
}
