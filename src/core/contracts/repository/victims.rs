use async_trait::async_trait;
use uuid::Uuid;

use super::error::RepositoryError;
use crate::core::commands::victims::{CreateVictim, UpdateVictim};
use crate::core::entities::common::{AddressData, PhoneData};
use crate::core::entities::victims::{VictimAddress, VictimPhone, VictimWriteResult};
use crate::core::read_models::victims::VictimWithDetails;

#[async_trait]
pub trait VictimReadRepository: Send + Sync {
    async fn get_victim_by_id(
        &self,
        id: Uuid,
    ) -> Result<VictimWithDetails, RepositoryError>;
    async fn get_all_victims(
        &self,
    ) -> Result<Vec<VictimWithDetails>, RepositoryError>;
    async fn get_victims_by_city(
        &self,
        city_id: Uuid,
    ) -> Result<Vec<VictimWithDetails>, RepositoryError>;
    async fn get_victims_paginated(
        &self,
        allowed_cities: Option<&[Uuid]>,
        limit: i64,
        offset: i64,
    ) -> Result<Vec<VictimWithDetails>, RepositoryError>;
    async fn count_victims(
        &self,
        allowed_cities: Option<&[Uuid]>,
    ) -> Result<i64, RepositoryError>;
    async fn get_victims_by_name(
        &self,
        name: &str,
    ) -> Result<Vec<VictimWithDetails>, RepositoryError>;
    async fn get_victims_by_cpf(
        &self,
        cpf: &str,
    ) -> Result<Vec<VictimWithDetails>, RepositoryError>;
}

#[async_trait]
pub trait VictimWriteRepository: Send + Sync {
    async fn create_victim(
        &self,
        victim: CreateVictim,
    ) -> Result<VictimWriteResult, RepositoryError>;
    async fn update_victim_by_id(
        &self,
        data: UpdateVictim,
        id: Uuid,
    ) -> Result<VictimWriteResult, RepositoryError>;
    async fn delete_victim_by_id(
        &self,
        id: Uuid,
    ) -> Result<VictimWriteResult, RepositoryError>;
    async fn create_phone(
        &self,
        victim_id: Uuid,
        phone_data: PhoneData,
    ) -> Result<VictimPhone, RepositoryError>;
    async fn get_phone_by_id(
        &self,
        phone_id: Uuid,
    ) -> Result<VictimPhone, RepositoryError>;
    async fn update_phone_by_id(
        &self,
        phone_id: Uuid,
        phone_data: PhoneData,
    ) -> Result<VictimPhone, RepositoryError>;
    async fn delete_phone_by_id(
        &self,
        phone_id: Uuid,
    ) -> Result<VictimPhone, RepositoryError>;
    async fn create_address(
        &self,
        victim_id: Uuid,
        address_data: AddressData,
    ) -> Result<VictimAddress, RepositoryError>;
    async fn get_address_by_id(
        &self,
        address_id: Uuid,
    ) -> Result<VictimAddress, RepositoryError>;
    async fn update_address_by_id(
        &self,
        address_id: Uuid,
        address_data: AddressData,
    ) -> Result<VictimAddress, RepositoryError>;
    async fn delete_address_by_id(
        &self,
        address_id: Uuid,
    ) -> Result<VictimAddress, RepositoryError>;
}
