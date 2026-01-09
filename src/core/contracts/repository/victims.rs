use async_trait::async_trait;
use uuid::Uuid;

use crate::core::entities::victims::{
    AddressData, CreateVictim, PhoneData, UpdateVictim, VictimAddress, VictimPhone,
    VictimWithDetails,
};

#[async_trait]
pub trait VictimRepository: Send + Sync {
    async fn create_victim(&self, victim: CreateVictim) -> Result<VictimWithDetails, sqlx::Error>;
    async fn get_victim_by_id(&self, id: Uuid) -> Result<VictimWithDetails, sqlx::Error>;
    async fn get_all_victims(&self) -> Result<Vec<VictimWithDetails>, sqlx::Error>;
    async fn get_victims_by_city(&self, city_id: Uuid) -> Result<Vec<VictimWithDetails>, sqlx::Error>;
    async fn get_victims_paginated(
        &self,
        allowed_cities: Option<&[Uuid]>,
        limit: i64,
        offset: i64,
    ) -> Result<Vec<VictimWithDetails>, sqlx::Error>;
    async fn count_victims(&self, allowed_cities: Option<&[Uuid]>) -> Result<i64, sqlx::Error>;
    async fn get_victims_by_name(&self, name: &str) -> Result<Vec<VictimWithDetails>, sqlx::Error>;
    async fn get_victims_by_cpf(&self, cpf: &str) -> Result<Vec<VictimWithDetails>, sqlx::Error>;
    async fn update_victim_by_id(&self, data: UpdateVictim, id: Uuid,) -> Result<VictimWithDetails, sqlx::Error>;
    async fn delete_victim_by_id(&self, id: Uuid) -> Result<VictimWithDetails, sqlx::Error>;
    async fn create_phone(&self, victim_id: Uuid, phone_data: PhoneData) -> Result<VictimPhone, sqlx::Error>;
    async fn get_phone_by_id(&self, phone_id: Uuid) -> Result<VictimPhone, sqlx::Error>;
    async fn update_phone_by_id(&self, phone_id: Uuid, phone_data: PhoneData) -> Result<VictimPhone, sqlx::Error>;
    async fn delete_phone_by_id(&self, phone_id: Uuid) -> Result<VictimPhone, sqlx::Error>;
    async fn create_address(&self, victim_id: Uuid, address_data: AddressData) -> Result<VictimAddress, sqlx::Error>;
    async fn get_address_by_id(&self, address_id: Uuid) -> Result<VictimAddress, sqlx::Error>;
    async fn update_address_by_id(&self, address_id: Uuid, address_data: AddressData) -> Result<VictimAddress, sqlx::Error>;
    async fn delete_address_by_id(&self, address_id: Uuid) -> Result<VictimAddress, sqlx::Error>;
}
