use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use sqlx::prelude::FromRow;


#[derive(Serialize, Deserialize)]
pub struct Login {
    pub email: String,
    pub password: String,
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
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}


#[derive(Serialize, Deserialize)]
pub struct LoginResponse {
    pub token: String,
    pub id: Uuid,
    pub full_name: String,
    pub email: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ClaimsToUserToken {
    pub id: String,
    pub exp: usize,
    pub rank: String,
    pub registration: String,
    pub full_name: String,
    pub profile: String,
    pub email: String,
}
