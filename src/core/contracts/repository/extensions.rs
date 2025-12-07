use async_trait::async_trait;
use uuid::Uuid;
use crate::core::entities::protective_measures::{
    CreateExtension,
    UpdateExtension,
    ProtectiveMeasureExtension
};

#[async_trait]
pub trait ExtensionRepository: Send + Sync {
    async fn create_extension(&self, protective_measure_id: Uuid, extension: CreateExtension) -> Result<ProtectiveMeasureExtension, sqlx::Error>;
    async fn get_extension_by_id(&self, id: Uuid) -> Result<ProtectiveMeasureExtension, sqlx::Error>;
    async fn get_extensions_by_measure(&self, protective_measure_id: Uuid) -> Result<Vec<ProtectiveMeasureExtension>, sqlx::Error>;
    async fn get_all_extensions(&self) -> Result<Vec<ProtectiveMeasureExtension>, sqlx::Error>;
    async fn update_extension_by_id(&self, data: UpdateExtension, id: Uuid) -> Result<ProtectiveMeasureExtension, sqlx::Error>;
    async fn delete_extension_by_id(&self, id: Uuid) -> Result<ProtectiveMeasureExtension, sqlx::Error>;
}
