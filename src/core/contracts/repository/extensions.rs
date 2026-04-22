use async_trait::async_trait;
use uuid::Uuid;

use super::error::RepositoryError;
use crate::core::commands::protective_measures::{CreateExtension, UpdateExtension};
use crate::core::entities::protective_measures::ProtectiveMeasureExtension;

#[async_trait]
pub trait ExtensionRepository: Send + Sync {
    async fn create_extension(
        &self,
        protective_measure_id: Uuid,
        extension: CreateExtension,
    ) -> Result<ProtectiveMeasureExtension, RepositoryError>;
    async fn get_extension_by_id(
        &self,
        id: Uuid,
    ) -> Result<ProtectiveMeasureExtension, RepositoryError>;
    async fn get_extensions_by_measure(
        &self,
        protective_measure_id: Uuid,
    ) -> Result<Vec<ProtectiveMeasureExtension>, RepositoryError>;
    async fn get_all_extensions(&self) -> Result<Vec<ProtectiveMeasureExtension>, RepositoryError>;
    async fn update_extension_by_id(
        &self,
        data: UpdateExtension,
        id: Uuid,
    ) -> Result<ProtectiveMeasureExtension, RepositoryError>;
    async fn delete_extension_by_id(
        &self,
        id: Uuid,
    ) -> Result<ProtectiveMeasureExtension, RepositoryError>;
}
