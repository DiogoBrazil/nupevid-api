use async_trait::async_trait;
use log::info;
use sqlx::PgPool;
use uuid::Uuid;

use super::models::protective_measures::ProtectiveMeasureRow;
use crate::core::{
    commands::protective_measures::{CreateProtectiveMeasure, UpdateProtectiveMeasure},
    contracts::repository::{
        error::RepositoryError,
        protective_measures::{ProtectiveMeasureReadRepository, ProtectiveMeasureWriteRepository},
    },
    entities::protective_measures::ProtectiveMeasure,
};
use crate::repositories::queries::protective_measures::ProtectiveMeasuresQueries;

use crate::repositories::error_mapper::map_sqlx_error;
fn map_protective_measure_error(err: sqlx::Error) -> RepositoryError {
    let base = map_sqlx_error(err);
    match base {
        RepositoryError::UniqueViolation { ref constraint } => match constraint.as_deref() {
            Some("idx_protective_measures_unique_active_per_victim_offender")
            | Some("idx_protective_measures_one_active_per_victim") => RepositoryError::Conflict(
                "Victim and offender already have an active protective measure".into(),
            ),
            _ => base,
        },
        RepositoryError::ForeignKeyViolation { ref constraint } => match constraint.as_deref() {
            Some("fk_protective_measures_court_district") => {
                RepositoryError::ReferencedEntityNotFound("Court district not found".into())
            }
            Some("fk_protective_measures_victim") => {
                RepositoryError::ReferencedEntityNotFound("Victim not found".into())
            }
            Some("fk_protective_measures_offender") => {
                RepositoryError::ReferencedEntityNotFound("Offender not found".into())
            }
            _ => base,
        },
        _ => base,
    }
}

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
impl ProtectiveMeasureReadRepository for PgProtectiveMeasureRepository {
    async fn get_protective_measure_by_id(
        &self,
        id: Uuid,
    ) -> Result<ProtectiveMeasure, RepositoryError> {
        info!(
            "[Repository] Executing SQL query to get protective measure with id: {}",
            id
        );

        let measure: ProtectiveMeasure = sqlx::query_as::<_, ProtectiveMeasureRow>(
            ProtectiveMeasuresQueries::GET_PROTECTIVE_MEASURE_BY_ID,
        )
        .bind(id)
        .fetch_one(&self.pool)
        .await
        .map_err(map_protective_measure_error)?
        .into();

        info!(
            "[Repository] Protective measure successfully found in the database with ID: {}",
            id
        );

        Ok(measure)
    }

    async fn get_all_protective_measures(&self) -> Result<Vec<ProtectiveMeasure>, RepositoryError> {
        info!("[Repository] Executing SQL query to get all protective measures");

        let measures: Vec<ProtectiveMeasure> = sqlx::query_as::<_, ProtectiveMeasureRow>(
            ProtectiveMeasuresQueries::GET_ALL_PROTECTIVE_MEASURES,
        )
        .fetch_all(&self.pool)
        .await
        .map_err(map_protective_measure_error)?
        .into_iter()
        .map(Into::into)
        .collect();

        info!(
            "[Repository] Found {} protective measures in database",
            measures.len()
        );

        Ok(measures)
    }

    async fn get_protective_measures_paginated(
        &self,
        allowed_cities: Option<&[Uuid]>,
        victim_id: Option<Uuid>,
        offender_id: Option<Uuid>,
        limit: i64,
        offset: i64,
    ) -> Result<Vec<ProtectiveMeasure>, RepositoryError> {
        info!("[Repository] Executing SQL query to get paginated protective measures");

        let rows: Vec<ProtectiveMeasureRow> = match allowed_cities {
            Some(city_ids) => sqlx::query_as::<_, ProtectiveMeasureRow>(
                ProtectiveMeasuresQueries::GET_PROTECTIVE_MEASURES_PAGED_BY_CITIES,
            )
            .bind(city_ids)
            .bind(victim_id)
            .bind(offender_id)
            .bind(limit)
            .bind(offset)
            .fetch_all(&self.pool)
            .await
            .map_err(map_protective_measure_error)?,
            None => sqlx::query_as::<_, ProtectiveMeasureRow>(
                ProtectiveMeasuresQueries::GET_PROTECTIVE_MEASURES_PAGED,
            )
            .bind(victim_id)
            .bind(offender_id)
            .bind(limit)
            .bind(offset)
            .fetch_all(&self.pool)
            .await
            .map_err(map_protective_measure_error)?,
        };
        let measures: Vec<ProtectiveMeasure> = rows.into_iter().map(Into::into).collect();

        info!(
            "[Repository] Found {} protective measures in database",
            measures.len()
        );

        Ok(measures)
    }

    async fn count_protective_measures(
        &self,
        allowed_cities: Option<&[Uuid]>,
        victim_id: Option<Uuid>,
        offender_id: Option<Uuid>,
    ) -> Result<i64, RepositoryError> {
        let count: i64 = match allowed_cities {
            Some(city_ids) => {
                sqlx::query_scalar(ProtectiveMeasuresQueries::COUNT_PROTECTIVE_MEASURES_BY_CITIES)
                    .bind(city_ids)
                    .bind(victim_id)
                    .bind(offender_id)
                    .fetch_one(&self.pool)
                    .await
                    .map_err(map_protective_measure_error)?
            }
            None => sqlx::query_scalar(ProtectiveMeasuresQueries::COUNT_PROTECTIVE_MEASURES)
                .bind(victim_id)
                .bind(offender_id)
                .fetch_one(&self.pool)
                .await
                .map_err(map_protective_measure_error)?,
        };

        Ok(count)
    }

    async fn get_protective_measures_by_victim(
        &self,
        victim_id: Uuid,
    ) -> Result<Vec<ProtectiveMeasure>, RepositoryError> {
        info!(
            "[Repository] Executing SQL query to get protective measures for victim: {}",
            victim_id
        );

        let measures: Vec<ProtectiveMeasure> = sqlx::query_as::<_, ProtectiveMeasureRow>(
            ProtectiveMeasuresQueries::GET_PROTECTIVE_MEASURES_BY_VICTIM,
        )
        .bind(victim_id)
        .fetch_all(&self.pool)
        .await
        .map_err(map_protective_measure_error)?
        .into_iter()
        .map(Into::into)
        .collect();

        info!(
            "[Repository] Found {} protective measures for victim: {}",
            measures.len(),
            victim_id
        );

        Ok(measures)
    }

    async fn check_active_measure_exists_for_victim(
        &self,
        victim_id: Uuid,
        offender_id: Uuid,
        exclude_measure_id: Option<Uuid>,
    ) -> Result<bool, RepositoryError> {
        info!(
            "[Repository] Checking if active measure exists for victim: {} offender: {} excluding measure: {:?}",
            victim_id, offender_id, exclude_measure_id
        );

        let result: bool =
            sqlx::query_scalar(ProtectiveMeasuresQueries::CHECK_ACTIVE_MEASURE_EXISTS_FOR_VICTIM)
                .bind(victim_id)
                .bind(offender_id)
                .bind(exclude_measure_id)
                .fetch_one(&self.pool)
                .await
                .map_err(map_protective_measure_error)?;

        info!(
            "[Repository] Active measure exists for victim {} offender {}: {}",
            victim_id, offender_id, result
        );

        Ok(result)
    }
}

#[async_trait]
impl ProtectiveMeasureWriteRepository for PgProtectiveMeasureRepository {
    async fn create_protective_measure(
        &self,
        measure: CreateProtectiveMeasure,
    ) -> Result<ProtectiveMeasure, RepositoryError> {
        let id: Uuid = Uuid::new_v4();

        info!(
            "[Repository] Executing SQL query to create protective measure for victim: {}",
            measure.victim_id
        );

        let measure_created: ProtectiveMeasure = sqlx::query_as::<_, ProtectiveMeasureRow>(
            ProtectiveMeasuresQueries::CREATE_PROTECTIVE_MEASURE,
        )
        .bind(id)
        .bind(measure.process_number)
        .bind(measure.sei_process_number)
        .bind(measure.occurrence_report_number)
        .bind(measure.issued_at)
        .bind(measure.judicial_authority)
        .bind(measure.court_district_id)
        .bind(measure.distance_meters)
        .bind(measure.status)
        .bind(measure.violence_types)
        .bind(measure.relationship_to_victim)
        .bind(measure.assaults_children)
        .bind(measure.was_drunk_during_assault)
        .bind(measure.victim_id)
        .bind(measure.offender_id)
        .fetch_one(&self.pool)
        .await
        .map_err(map_protective_measure_error)?
        .into();

        info!(
            "[Repository] Protective measure successfully inserted into database with ID: {}",
            measure_created.id
        );

        Ok(measure_created)
    }

    async fn update_protective_measure_by_id(
        &self,
        data: UpdateProtectiveMeasure,
        id: Uuid,
    ) -> Result<ProtectiveMeasure, RepositoryError> {
        info!(
            "[Repository] Executing SQL query to update protective measure with ID: {}",
            id
        );

        let measure_updated: ProtectiveMeasure = sqlx::query_as::<_, ProtectiveMeasureRow>(
            ProtectiveMeasuresQueries::UPDATE_PROTECTIVE_MEASURE_BY_ID,
        )
        .bind(id)
        .bind(data.process_number)
        .bind(data.sei_process_number)
        .bind(data.occurrence_report_number)
        .bind(data.issued_at)
        .bind(data.judicial_authority)
        .bind(data.court_district_id)
        .bind(data.distance_meters)
        .bind(data.status)
        .bind(data.violence_types)
        .bind(data.relationship_to_victim)
        .bind(data.assaults_children)
        .bind(data.was_drunk_during_assault)
        .bind(data.victim_id)
        .bind(data.offender_id)
        .fetch_one(&self.pool)
        .await
        .map_err(map_protective_measure_error)?
        .into();

        info!(
            "[Repository] Protective measure successfully updated in database with ID: {}",
            id
        );

        Ok(measure_updated)
    }

    async fn delete_protective_measure_by_id(
        &self,
        id: Uuid,
    ) -> Result<ProtectiveMeasure, RepositoryError> {
        info!(
            "[Repository] Executing SQL query to soft delete protective measure with id: {}",
            id
        );

        let deleted_measure: ProtectiveMeasure = sqlx::query_as::<_, ProtectiveMeasureRow>(
            ProtectiveMeasuresQueries::DELETE_PROTECTIVE_MEASURE_BY_ID,
        )
        .bind(id)
        .fetch_one(&self.pool)
        .await
        .map_err(map_protective_measure_error)?
        .into();

        info!(
            "[Repository] Protective measure successfully soft deleted from database with ID: {}",
            id
        );

        Ok(deleted_measure)
    }
}
