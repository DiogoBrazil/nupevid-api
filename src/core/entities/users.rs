use serde::{Deserialize, Serialize};
use serde_json::Value as JsonValue;
use sqlx::prelude::FromRow;
use uuid::Uuid;
use chrono::{DateTime, Utc};
use std::collections::HashMap;

pub type PermissionPolicies = HashMap<String, Vec<Uuid>>;

#[derive(Serialize, Deserialize, Debug)]
pub struct CreateUser {
    pub rank: String,
    pub registration: String,
    pub full_name: String,
    pub profile: String,
    pub email: String,
    pub password: String,
    pub city_id: Option<Uuid>,
    #[serde(default)]
    pub permission_policies: Option<PermissionPolicies>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct UpdateUser {
    pub rank: String,
    pub registration: String,
    pub full_name: String,
    pub profile: String,
    pub email: String,
    pub city_id: Option<Uuid>,
    #[serde(default)]
    pub permission_policies: Option<PermissionPolicies>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UpdateUserPassword {
    pub current_password: Option<String>,
    pub new_password: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ResetUserPasswordResponse {
    pub temporary_password: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
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
