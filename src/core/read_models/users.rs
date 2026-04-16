use serde::{Deserialize, Serialize};
use serde_json::Value as JsonValue;
use uuid::Uuid;

use crate::core::entities::users::UserRecord;
use crate::core::value_objects::profiles::Profile;
use crate::core::value_objects::ranks::Rank;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserSummary {
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

impl From<UserRecord> for UserSummary {
    fn from(user: UserRecord) -> Self {
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
