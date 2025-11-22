use async_trait::async_trait;
use uuid::Uuid;

use crate::core::entities::victims::{CreateVictim, UpdateVictim, VictimWithAddress};

#[async_trait]
pub trait VictimRepository: Send + Sync {
    async fn create_victim(&self, victim: CreateVictim) -> Result<VictimWithAddress, sqlx::Error>;
    async fn get_victim_by_id(&self, id: Uuid) -> Result<VictimWithAddress, sqlx::Error>;
    async fn get_all_victims(&self) -> Result<Vec<VictimWithAddress>, sqlx::Error>;
    async fn get_victims_by_city(&self, city_id: Uuid) -> Result<Vec<VictimWithAddress>, sqlx::Error>;
    async fn update_victim_by_id(&self, data: UpdateVictim, id: Uuid,) -> Result<VictimWithAddress, sqlx::Error>;
    async fn delete_victim_by_id(&self, id: Uuid) -> Result<VictimWithAddress, sqlx::Error>;
}
