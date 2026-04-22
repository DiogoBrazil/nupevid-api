use chrono::{DateTime, Utc};
use sqlx::prelude::FromRow;
use uuid::Uuid;

use crate::core::entities::auth::CompleteUserData;
use crate::core::value_objects::profiles::Profile;
use crate::core::value_objects::ranks::Rank;

#[derive(Debug, Clone, FromRow)]
pub struct CompleteUserDataRow {
    pub id: Uuid,
    pub rank: Rank,
    pub registration: String,
    pub full_name: String,
    pub profile: Profile,
    pub email: String,
    pub password: String,
    pub city_id: Option<Uuid>,
    pub is_temporary_password: bool,
    pub temporary_password_expires_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub is_deleted: bool,
}

impl From<CompleteUserDataRow> for CompleteUserData {
    fn from(row: CompleteUserDataRow) -> Self {
        CompleteUserData {
            id: row.id,
            rank: row.rank,
            registration: row.registration,
            full_name: row.full_name,
            profile: row.profile,
            email: row.email,
            password: row.password,
            city_id: row.city_id,
            is_temporary_password: row.is_temporary_password,
            temporary_password_expires_at: row.temporary_password_expires_at,
            created_at: row.created_at,
            updated_at: row.updated_at,
            is_deleted: row.is_deleted,
        }
    }
}
