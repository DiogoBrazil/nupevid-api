use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

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
