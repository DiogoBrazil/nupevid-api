use async_trait::async_trait;
use uuid::Uuid;
use crate::core::entities::protective_measures::{
    CreateProtectiveMeasure,
    UpdateProtectiveMeasure,
    ProtectiveMeasure
};

#[async_trait]
pub trait ProtectiveMeasureRepository: Send + Sync {
    async fn create_protective_measure(&self, measure: CreateProtectiveMeasure) -> Result<ProtectiveMeasure, sqlx::Error>;
    async fn get_protective_measure_by_id(&self, id: Uuid) -> Result<ProtectiveMeasure, sqlx::Error>;
    async fn get_all_protective_measures(&self) -> Result<Vec<ProtectiveMeasure>, sqlx::Error>;
    async fn get_protective_measures_paginated(
        &self,
        allowed_cities: Option<&[Uuid]>,
        limit: i64,
        offset: i64,
    ) -> Result<Vec<ProtectiveMeasure>, sqlx::Error>;
    async fn count_protective_measures(&self, allowed_cities: Option<&[Uuid]>) -> Result<i64, sqlx::Error>;
    async fn get_protective_measures_by_victim(&self, victim_id: Uuid) -> Result<Vec<ProtectiveMeasure>, sqlx::Error>;
    async fn check_active_measure_exists_for_victim(&self, victim_id: Uuid, exclude_measure_id: Uuid) -> Result<bool, sqlx::Error>;
    async fn update_protective_measure_by_id(&self, data: UpdateProtectiveMeasure, id: Uuid) -> Result<ProtectiveMeasure, sqlx::Error>;
    async fn delete_protective_measure_by_id(&self, id: Uuid) -> Result<ProtectiveMeasure, sqlx::Error>;
}
