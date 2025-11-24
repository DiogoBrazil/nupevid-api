use async_trait::async_trait;
use log::info;
use sqlx::PgPool;
use uuid::Uuid;

use crate::core::{
    contracts::repository::protective_measures::ProtectiveMeasureRepository,
    entities::protective_measures::{
        CreateProtectiveMeasure,
        UpdateProtectiveMeasure,
        ProtectiveMeasure
    }
};
use crate::config::querys::protective_measures::ProtectiveMeasuresQueries;

#[derive(Clone)]
pub struct PgProtectiveMeasureRepository {
    pool: PgPool,
}

impl PgProtectiveMeasureRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl ProtectiveMeasureRepository for PgProtectiveMeasureRepository {
    async fn create_protective_measure(&self, measure: CreateProtectiveMeasure) -> Result<ProtectiveMeasure, sqlx::Error> {
        let id: Uuid = Uuid::new_v4();

        info!("[Repository] Executing SQL query to create protective measure for victim: {}", measure.victim_id);

        let measure_created: ProtectiveMeasure = sqlx::query_as(ProtectiveMeasuresQueries::CREATE_PROTECTIVE_MEASURE)
            .bind(id)
            .bind(measure.process_number)
            .bind(measure.issued_at)
            .bind(measure.valid_until)
            .bind(measure.judicial_authority)
            .bind(measure.court_district_id)
            .bind(measure.is_active)
            .bind(measure.victim_id)
            .fetch_one(&self.pool)
            .await?;

        info!("[Repository] Protective measure successfully inserted into database with ID: {}", measure_created.id);

        Ok(measure_created)
    }

    async fn get_protective_measure_by_id(&self, id: Uuid) -> Result<ProtectiveMeasure, sqlx::Error> {
        info!("[Repository] Executing SQL query to get protective measure with id: {}", id);

        let measure: ProtectiveMeasure = sqlx::query_as(ProtectiveMeasuresQueries::GET_PROTECTIVE_MEASURE_BY_ID)
            .bind(id)
            .fetch_one(&self.pool)
            .await?;

        info!("[Repository] Protective measure successfully found in the database with ID: {}", id);

        Ok(measure)
    }

    async fn get_all_protective_measures(&self) -> Result<Vec<ProtectiveMeasure>, sqlx::Error> {
        info!("[Repository] Executing SQL query to get all protective measures");

        let measures: Vec<ProtectiveMeasure> = sqlx::query_as(ProtectiveMeasuresQueries::GET_ALL_PROTECTIVE_MEASURES)
            .fetch_all(&self.pool)
            .await?;

        info!("[Repository] Found {} protective measures in database", measures.len());

        Ok(measures)
    }

    async fn get_protective_measures_by_victim(&self, victim_id: Uuid) -> Result<Vec<ProtectiveMeasure>, sqlx::Error> {
        info!("[Repository] Executing SQL query to get protective measures for victim: {}", victim_id);

        let measures: Vec<ProtectiveMeasure> = sqlx::query_as(ProtectiveMeasuresQueries::GET_PROTECTIVE_MEASURES_BY_VICTIM)
            .bind(victim_id)
            .fetch_all(&self.pool)
            .await?;

        info!("[Repository] Found {} protective measures for victim: {}", measures.len(), victim_id);

        Ok(measures)
    }

    async fn check_active_measure_exists_for_victim(&self, victim_id: Uuid, exclude_measure_id: Uuid) -> Result<bool, sqlx::Error> {
        info!("[Repository] Checking if active measure exists for victim: {} excluding measure: {}", victim_id, exclude_measure_id);

        let result: bool = sqlx::query_scalar(ProtectiveMeasuresQueries::CHECK_ACTIVE_MEASURE_EXISTS_FOR_VICTIM)
            .bind(victim_id)
            .bind(exclude_measure_id)
            .fetch_one(&self.pool)
            .await?;

        info!("[Repository] Active measure exists for victim {}: {}", victim_id, result);

        Ok(result)
    }

    async fn update_protective_measure_by_id(&self, data: UpdateProtectiveMeasure, id: Uuid) -> Result<ProtectiveMeasure, sqlx::Error> {
        info!("[Repository] Executing SQL query to update protective measure with ID: {}", id);

        let measure_updated: ProtectiveMeasure = sqlx::query_as(ProtectiveMeasuresQueries::UPDATE_PROTECTIVE_MEASURE_BY_ID)
            .bind(id)
            .bind(data.process_number)
            .bind(data.issued_at)
            .bind(data.valid_until)
            .bind(data.judicial_authority)
            .bind(data.court_district_id)
            .bind(data.is_active)
            .bind(data.victim_id)
            .fetch_one(&self.pool)
            .await?;

        info!("[Repository] Protective measure successfully updated in database with ID: {}", id);

        Ok(measure_updated)
    }

    async fn delete_protective_measure_by_id(&self, id: Uuid) -> Result<ProtectiveMeasure, sqlx::Error> {
        info!("[Repository] Executing SQL query to soft delete protective measure with id: {}", id);

        let deleted_measure: ProtectiveMeasure = sqlx::query_as(ProtectiveMeasuresQueries::DELETE_PROTECTIVE_MEASURE_BY_ID)
            .bind(id)
            .fetch_one(&self.pool)
            .await?;

        info!("[Repository] Protective measure successfully soft deleted from database with ID: {}", id);

        Ok(deleted_measure)
    }
}
