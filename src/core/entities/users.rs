use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value as JsonValue;
use uuid::Uuid;

use crate::core::read_models::users::UserComplement;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserDataCreatedWithoutPassword {
    pub id: Uuid,
    pub rank: String,
    pub registration: String,
    pub full_name: String,
    pub profile: String,
    pub email: String,
    pub city_id: Option<Uuid>,
    pub permission_policies: JsonValue,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub is_deleted: bool,
}

impl From<UserDataCreatedWithoutPassword> for UserComplement {
    fn from(user: UserDataCreatedWithoutPassword) -> Self {
        Self {
            id: user.id,
            rank: user.rank,
            registration: user.registration,
            full_name: user.full_name,
            profile: user.profile,
            email: user.email,
            city_id: user.city_id,
            permission_policies: user.permission_policies,
            is_deleted: user.is_deleted,
        }
    }
}
