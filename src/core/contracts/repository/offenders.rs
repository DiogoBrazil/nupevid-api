use async_trait::async_trait;
use uuid::Uuid;

use super::error::RepositoryError;
use crate::core::commands::offenders::{CreateOffender, UpdateOffender};
use crate::core::entities::common::{AddressData, PhoneData};
use crate::core::entities::offenders::{OffenderAddress, OffenderPhone, OffenderWriteResult};
use crate::core::read_models::offenders::OffenderWithDetails;

#[async_trait]
pub trait OffenderReadRepository: Send + Sync {
    async fn get_offender_by_id(&self, id: Uuid) -> Result<OffenderWithDetails, RepositoryError>;
    async fn get_all_offenders(&self) -> Result<Vec<OffenderWithDetails>, RepositoryError>;
    async fn get_offenders_paginated(
        &self,
        allowed_cities: Option<&[Uuid]>,
        limit: i64,
        offset: i64,
    ) -> Result<Vec<OffenderWithDetails>, RepositoryError>;
    async fn count_offenders(
        &self,
        allowed_cities: Option<&[Uuid]>,
    ) -> Result<i64, RepositoryError>;
    async fn get_offenders_by_city(
        &self,
        city_id: Uuid,
    ) -> Result<Vec<OffenderWithDetails>, RepositoryError>;
    async fn get_offenders_by_name(
        &self,
        name: &str,
    ) -> Result<Vec<OffenderWithDetails>, RepositoryError>;
    async fn get_offenders_by_cpf(
        &self,
        cpf: &str,
    ) -> Result<Vec<OffenderWithDetails>, RepositoryError>;
    async fn get_offenders_by_victim_id(
        &self,
        victim_id: Uuid,
    ) -> Result<Vec<OffenderWithDetails>, RepositoryError>;
}

#[async_trait]
pub trait OffenderWriteRepository: Send + Sync {
    async fn create_offender(
        &self,
        offender: CreateOffender,
    ) -> Result<OffenderWriteResult, RepositoryError>;
    async fn update_offender_by_id(
        &self,
        data: UpdateOffender,
        id: Uuid,
    ) -> Result<OffenderWriteResult, RepositoryError>;
    async fn delete_offender_by_id(&self, id: Uuid)
    -> Result<OffenderWriteResult, RepositoryError>;
    async fn create_phone(
        &self,
        offender_id: Uuid,
        phone_data: PhoneData,
    ) -> Result<OffenderPhone, RepositoryError>;
    async fn get_phone_by_id(&self, phone_id: Uuid) -> Result<OffenderPhone, RepositoryError>;
    async fn update_phone_by_id(
        &self,
        phone_id: Uuid,
        phone_data: PhoneData,
    ) -> Result<OffenderPhone, RepositoryError>;
    async fn delete_phone_by_id(&self, phone_id: Uuid) -> Result<OffenderPhone, RepositoryError>;
    async fn create_address(
        &self,
        offender_id: Uuid,
        address_data: AddressData,
    ) -> Result<OffenderAddress, RepositoryError>;
    async fn get_address_by_id(&self, address_id: Uuid)
    -> Result<OffenderAddress, RepositoryError>;
    async fn update_address_by_id(
        &self,
        address_id: Uuid,
        address_data: AddressData,
    ) -> Result<OffenderAddress, RepositoryError>;
    async fn delete_address_by_id(
        &self,
        address_id: Uuid,
    ) -> Result<OffenderAddress, RepositoryError>;
}
