use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::core::read_models::work_sessions::WorkSessionWithMembers;

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
