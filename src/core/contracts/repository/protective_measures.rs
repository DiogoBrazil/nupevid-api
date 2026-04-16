use async_trait::async_trait;
use uuid::Uuid;

use super::error::RepositoryError;
use crate::core::commands::protective_measures::{
    CreateExtension, CreateProtectiveMeasure, UpdateExtension, UpdateProtectiveMeasure,
};
use crate::core::entities::protective_measures::ProtectiveMeasure;

#[async_trait]
pub trait ProtectiveMeasureReadRepository: Send + Sync {
    async fn get_protective_measure_by_id(
        &self,
        id: Uuid,
    ) -> Result<ProtectiveMeasure, RepositoryError>;
    async fn get_all_protective_measures(&self) -> Result<Vec<ProtectiveMeasure>, RepositoryError>;
    async fn get_protective_measures_paginated(
        &self,
        allowed_cities: Option<&[Uuid]>,
        limit: i64,
        offset: i64,
    ) -> Result<Vec<ProtectiveMeasure>, RepositoryError>;
    async fn count_protective_measures(
        &self,
        allowed_cities: Option<&[Uuid]>,
    ) -> Result<i64, RepositoryError>;
    async fn get_protective_measures_by_victim(
        &self,
        victim_id: Uuid,
    ) -> Result<Vec<ProtectiveMeasure>, RepositoryError>;
    async fn check_active_measure_exists_for_victim(
        &self,
        victim_id: Uuid,
        exclude_measure_id: Uuid,
    ) -> Result<bool, RepositoryError>;
}

#[async_trait]
pub trait ProtectiveMeasureWriteRepository: Send + Sync {
    async fn create_protective_measure(
        &self,
        measure: CreateProtectiveMeasure,
    ) -> Result<ProtectiveMeasure, RepositoryError>;
    async fn create_protective_measure_with_extensions(
        &self,
        measure: &CreateProtectiveMeasure,
        extensions: &[CreateExtension],
    ) -> Result<ProtectiveMeasure, RepositoryError>;
    async fn update_protective_measure_by_id(
        &self,
        data: UpdateProtectiveMeasure,
        id: Uuid,
    ) -> Result<ProtectiveMeasure, RepositoryError>;
    async fn update_protective_measure_with_extensions(
        &self,
        data: &UpdateProtectiveMeasure,
        id: Uuid,
        extensions_to_create: &[CreateExtension],
        extensions_to_update: &[(Uuid, UpdateExtension)],
    ) -> Result<ProtectiveMeasure, RepositoryError>;
    async fn delete_protective_measure_by_id(
        &self,
        id: Uuid,
    ) -> Result<ProtectiveMeasure, RepositoryError>;
}
