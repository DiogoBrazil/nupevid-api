use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use sqlx::prelude::FromRow;

use super::work_sessions::WorkSessionWithMembers;

#[derive(Serialize, Deserialize)]
pub struct Login {
    pub email: String,
    pub password: String,
    #[serde(default = "default_auto_create_session")]
    pub auto_create_session: bool,
}

fn default_auto_create_session() -> bool {
    true
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct CompleteUserData {
    pub id: Uuid,
    pub rank: String,
    pub registration: String,
    pub full_name: String,
    pub profile: String,
    pub email: String,
    pub password: String,
    pub city_id: Option<Uuid>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub is_deleted: bool,
}


#[derive(Serialize, Deserialize)]
pub struct LoginResponse {
    pub token: String,
    pub id: Uuid,
    pub full_name: String,
    pub email: String,
    pub rank: String,
    pub registration: String,
    pub profile: String,
    pub work_session: Option<WorkSessionWithMembers>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClaimsToUserToken {
    pub id: String,
    pub exp: usize,
    pub rank: String,
    pub registration: String,
    pub full_name: String,
    pub profile: String,
    pub email: String,
    pub city_id: Option<String>,
}
