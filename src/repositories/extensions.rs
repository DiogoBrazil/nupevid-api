use async_trait::async_trait;
use log::info;
use sqlx::{PgPool, Postgres, Transaction};
use uuid::Uuid;

use crate::config::querys::extensions::ExtensionsQueries;
use crate::core::{
    contracts::repository::extensions::ExtensionRepository,
    entities::protective_measures::{CreateExtension, ProtectiveMeasureExtension, UpdateExtension},
};

#[derive(Clone)]
pub struct PgExtensionRepository {
    pool: PgPool,
}

impl PgExtensionRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn create_extension_with_tx(
        &self,
        tx: &mut Transaction<'_, Postgres>,
        protective_measure_id: Uuid,
        extension: &CreateExtension,
    ) -> Result<ProtectiveMeasureExtension, sqlx::Error> {
        let id = Uuid::new_v4();

        let extension_created: ProtectiveMeasureExtension =
            sqlx::query_as(ExtensionsQueries::CREATE_EXTENSION)
                .bind(id)
                .bind(protective_measure_id)
                .bind(extension.extension_number)
                .bind(extension.extension_date)
                .bind(extension.new_valid_until)
                .bind(&extension.notes)
                .fetch_one(&mut **tx)
                .await?;

        Ok(extension_created)
    }

    pub async fn update_extension_by_id_with_tx(
        &self,
        tx: &mut Transaction<'_, Postgres>,
        data: &UpdateExtension,
        id: Uuid,
    ) -> Result<ProtectiveMeasureExtension, sqlx::Error> {
        let extension_updated: ProtectiveMeasureExtension =
            sqlx::query_as(ExtensionsQueries::UPDATE_EXTENSION_BY_ID)
                .bind(id)
                .bind(data.extension_number)
                .bind(data.extension_date)
                .bind(data.new_valid_until)
                .bind(&data.notes)
                .fetch_one(&mut **tx)
                .await?;

        Ok(extension_updated)
    }
}

#[async_trait]
impl ExtensionRepository for PgExtensionRepository {
    async fn create_extension(
        &self,
        protective_measure_id: Uuid,
        extension: CreateExtension,
    ) -> Result<ProtectiveMeasureExtension, sqlx::Error> {
        let id = Uuid::new_v4();

        info!(
            "[Repository] Executing SQL query to create extension {} for protective measure: {}",
            extension.extension_number, protective_measure_id
        );

        let extension_created: ProtectiveMeasureExtension =
            sqlx::query_as(ExtensionsQueries::CREATE_EXTENSION)
                .bind(id)
                .bind(protective_measure_id)
                .bind(extension.extension_number)
                .bind(extension.extension_date)
                .bind(extension.new_valid_until)
                .bind(extension.notes)
                .fetch_one(&self.pool)
                .await?;

        info!(
            "[Repository] Extension successfully inserted into database with ID: {}",
            extension_created.id
        );

        Ok(extension_created)
    }

    async fn get_extension_by_id(
        &self,
        id: Uuid,
    ) -> Result<ProtectiveMeasureExtension, sqlx::Error> {
        info!(
            "[Repository] Executing SQL query to get extension with id: {}",
            id
        );

        let extension: ProtectiveMeasureExtension =
            sqlx::query_as(ExtensionsQueries::GET_EXTENSION_BY_ID)
                .bind(id)
                .fetch_one(&self.pool)
                .await?;

        info!(
            "[Repository] Extension successfully found in the database with ID: {}",
            id
        );

        Ok(extension)
    }

    async fn get_extensions_by_measure(
        &self,
        protective_measure_id: Uuid,
    ) -> Result<Vec<ProtectiveMeasureExtension>, sqlx::Error> {
        info!(
            "[Repository] Executing SQL query to get extensions for protective measure: {}",
            protective_measure_id
        );

        let extensions: Vec<ProtectiveMeasureExtension> =
            sqlx::query_as(ExtensionsQueries::GET_EXTENSIONS_BY_MEASURE)
                .bind(protective_measure_id)
                .fetch_all(&self.pool)
                .await?;

        info!(
            "[Repository] Found {} extensions for protective measure: {}",
            extensions.len(),
            protective_measure_id
        );

        Ok(extensions)
    }

    async fn get_all_extensions(&self) -> Result<Vec<ProtectiveMeasureExtension>, sqlx::Error> {
        info!("[Repository] Executing SQL query to get all extensions");

        let extensions: Vec<ProtectiveMeasureExtension> =
            sqlx::query_as(ExtensionsQueries::GET_ALL_EXTENSIONS)
                .fetch_all(&self.pool)
                .await?;

        info!(
            "[Repository] Found {} extensions in database",
            extensions.len()
        );

        Ok(extensions)
    }

    async fn update_extension_by_id(
        &self,
        data: UpdateExtension,
        id: Uuid,
    ) -> Result<ProtectiveMeasureExtension, sqlx::Error> {
        info!(
            "[Repository] Executing SQL query to update extension with ID: {}",
            id
        );

        let extension_updated: ProtectiveMeasureExtension =
            sqlx::query_as(ExtensionsQueries::UPDATE_EXTENSION_BY_ID)
                .bind(id)
                .bind(data.extension_number)
                .bind(data.extension_date)
                .bind(data.new_valid_until)
                .bind(data.notes)
                .fetch_one(&self.pool)
                .await?;

        info!(
            "[Repository] Extension successfully updated in database with ID: {}",
            id
        );

        Ok(extension_updated)
    }

    async fn delete_extension_by_id(
        &self,
        id: Uuid,
    ) -> Result<ProtectiveMeasureExtension, sqlx::Error> {
        info!(
            "[Repository] Executing SQL query to soft delete extension with id: {}",
            id
        );

        let deleted_extension: ProtectiveMeasureExtension =
            sqlx::query_as(ExtensionsQueries::DELETE_EXTENSION_BY_ID)
                .bind(id)
                .fetch_one(&self.pool)
                .await?;

        info!(
            "[Repository] Extension successfully soft deleted from database with ID: {}",
            id
        );

        Ok(deleted_extension)
    }
}
