use serde::{Deserialize, Serialize};
use uuid::Uuid;

pub use crate::core::value_objects::policies::PermissionPolicies;
use crate::core::value_objects::profiles::Profile;
use crate::core::value_objects::ranks::Rank;

#[derive(Serialize, Deserialize, Debug)]
pub struct CreateUser {
    pub rank: Rank,
    pub registration: String,
    pub full_name: String,
    pub profile: Profile,
    pub email: String,
    pub password: String,
    pub city_id: Option<Uuid>,
    #[serde(default)]
    pub permission_policies: Option<PermissionPolicies>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct UpdateUser {
    pub rank: Rank,
    pub registration: String,
    pub full_name: String,
    pub profile: Profile,
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
