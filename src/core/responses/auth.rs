use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::core::read_models::work_sessions::WorkSessionWithMembers;
use crate::core::value_objects::profiles::Profile;
use crate::core::value_objects::ranks::Rank;

#[derive(Serialize, Deserialize)]
pub struct LoginResponse {
    pub token: String,
    pub id: Uuid,
    pub full_name: String,
    pub email: String,
    pub rank: Rank,
    pub registration: String,
    pub profile: Profile,
    pub work_session: Option<WorkSessionWithMembers>,
}
