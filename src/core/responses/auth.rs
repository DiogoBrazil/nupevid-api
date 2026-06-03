use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::core::read_models::work_sessions::WorkSessionWithMemberDetails;
use crate::core::value_objects::profiles::Profile;
use crate::core::value_objects::ranks::Rank;

#[derive(Serialize, Deserialize)]
pub struct LoginResponse {
    pub access_token: String,
    pub refresh_token: String,
    pub token_type: String,
    pub expires_in: i64,
    pub id: Uuid,
    pub full_name: String,
    pub email: String,
    pub rank: Rank,
    pub registration: String,
    pub profile: Profile,
    pub work_session: Option<WorkSessionWithMemberDetails>,
}

#[derive(Serialize, Deserialize)]
pub struct RefreshResponse {
    pub access_token: String,
    pub refresh_token: String,
    pub token_type: String,
    pub expires_in: i64,
}

#[derive(Serialize, Deserialize)]
pub struct LogoutResponse {
    pub message: String,
}
