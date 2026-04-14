use serde::{Deserialize, Serialize};
use serde_json::Value as JsonValue;
use uuid::Uuid;

use crate::core::value_objects::profiles::Profile;
use crate::core::value_objects::ranks::Rank;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserComplement {
    pub id: Uuid,
    pub rank: Rank,
    pub registration: String,
    pub full_name: String,
    pub profile: Profile,
    pub email: String,
    pub city_id: Option<Uuid>,
    pub permission_policies: JsonValue,
    pub is_deleted: bool,
}
