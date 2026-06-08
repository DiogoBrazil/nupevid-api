use chrono::{DateTime, Utc};
use sqlx::prelude::FromRow;
use uuid::Uuid;

use crate::core::entities::cities::City;

#[derive(Debug, Clone, FromRow)]
pub struct CityRow {
    pub id: Uuid,
    pub name: String,
    pub state: String,
    pub battalion: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub is_deleted: bool,
}

impl From<CityRow> for City {
    fn from(row: CityRow) -> Self {
        City {
            id: row.id,
            name: row.name,
            state: row.state,
            battalion: row.battalion,
            created_at: row.created_at,
            updated_at: row.updated_at,
            is_deleted: row.is_deleted,
        }
    }
}
