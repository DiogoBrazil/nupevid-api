use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompleteUserData {
    pub id: Uuid,
    pub rank: String,
    pub registration: String,
    pub full_name: String,
    pub profile: String,
    pub email: String,
    pub password: String,
    pub city_id: Option<Uuid>,
    pub is_temporary_password: bool,
    pub temporary_password_expires_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub is_deleted: bool,
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
