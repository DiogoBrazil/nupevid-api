use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::core::value_objects::policies::PermissionPolicies;
use crate::core::value_objects::profiles::Profile;
use crate::core::value_objects::ranks::Rank;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct User {
    pub id: Uuid,
    pub rank: Rank,
    pub registration: String,
    pub full_name: String,
    pub profile: Profile,
    pub email: String,
    pub city_id: Option<Uuid>,
    pub permission_policies: PermissionPolicies,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub is_deleted: bool,
}
