use async_trait::async_trait;
use uuid::Uuid;

use crate::core::entities::offenders::{
    AddressData, CreateOffender, OffenderAddress, OffenderPhone, OffenderWithDetails, PhoneData,
    UpdateOffender,
};

#[async_trait]
pub trait OffenderRepository: Send + Sync {
    async fn create_offender(
        &self,
        offender: CreateOffender,
    ) -> Result<OffenderWithDetails, sqlx::Error>;
    async fn get_offender_by_id(&self, id: Uuid) -> Result<OffenderWithDetails, sqlx::Error>;
    async fn get_all_offenders(&self) -> Result<Vec<OffenderWithDetails>, sqlx::Error>;
    async fn get_offenders_paginated(
        &self,
        allowed_cities: Option<&[Uuid]>,
        limit: i64,
        offset: i64,
    ) -> Result<Vec<OffenderWithDetails>, sqlx::Error>;
    async fn count_offenders(&self, allowed_cities: Option<&[Uuid]>) -> Result<i64, sqlx::Error>;
    async fn get_offenders_by_city(
        &self,
        city_id: Uuid,
    ) -> Result<Vec<OffenderWithDetails>, sqlx::Error>;
    async fn get_offenders_by_name(
        &self,
        name: &str,
    ) -> Result<Vec<OffenderWithDetails>, sqlx::Error>;
    async fn get_offenders_by_cpf(
        &self,
        cpf: &str,
    ) -> Result<Vec<OffenderWithDetails>, sqlx::Error>;
    async fn get_offenders_by_victim_id(
        &self,
        victim_id: Uuid,
    ) -> Result<Vec<OffenderWithDetails>, sqlx::Error>;
    async fn update_offender_by_id(
        &self,
        data: UpdateOffender,
        id: Uuid,
    ) -> Result<OffenderWithDetails, sqlx::Error>;
    async fn delete_offender_by_id(&self, id: Uuid) -> Result<OffenderWithDetails, sqlx::Error>;
    async fn create_phone(
        &self,
        offender_id: Uuid,
        phone_data: PhoneData,
    ) -> Result<OffenderPhone, sqlx::Error>;
    async fn get_phone_by_id(&self, phone_id: Uuid) -> Result<OffenderPhone, sqlx::Error>;
    async fn update_phone_by_id(
        &self,
        phone_id: Uuid,
        phone_data: PhoneData,
    ) -> Result<OffenderPhone, sqlx::Error>;
    async fn delete_phone_by_id(&self, phone_id: Uuid) -> Result<OffenderPhone, sqlx::Error>;
    async fn create_address(
        &self,
        offender_id: Uuid,
        address_data: AddressData,
    ) -> Result<OffenderAddress, sqlx::Error>;
    async fn get_address_by_id(&self, address_id: Uuid) -> Result<OffenderAddress, sqlx::Error>;
    async fn update_address_by_id(
        &self,
        address_id: Uuid,
        address_data: AddressData,
    ) -> Result<OffenderAddress, sqlx::Error>;
    async fn delete_address_by_id(&self, address_id: Uuid) -> Result<OffenderAddress, sqlx::Error>;
}
