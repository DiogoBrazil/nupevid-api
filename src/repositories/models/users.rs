use chrono::{DateTime, Utc};
use serde_json::Value as JsonValue;
use sqlx::prelude::FromRow;
use uuid::Uuid;

use crate::core::entities::users::UserRecord;
use crate::core::value_objects::profiles::Profile;
use crate::core::value_objects::ranks::Rank;

#[derive(Debug, Clone, FromRow)]
pub struct UserRecordRow {
    pub id: Uuid,
    pub rank: Rank,
    pub registration: String,
    pub full_name: String,
    pub profile: Profile,
    pub email: String,
    pub city_id: Option<Uuid>,
    pub permission_policies: JsonValue,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub is_deleted: bool,
}

impl From<UserRecordRow> for UserRecord {
    fn from(row: UserRecordRow) -> Self {
        UserRecord {
            id: row.id,
            rank: row.rank,
            registration: row.registration,
            full_name: row.full_name,
            profile: row.profile,
            email: row.email,
            city_id: row.city_id,
            permission_policies: row.permission_policies,
            created_at: row.created_at,
            updated_at: row.updated_at,
            is_deleted: row.is_deleted,
        }
    }
}
