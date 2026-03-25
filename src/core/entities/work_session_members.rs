use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::Type;
use uuid::Uuid;

use crate::core::read_models::users::UserComplement;

#[derive(Debug, Clone, Serialize, Deserialize, Type, PartialEq)]
#[sqlx(type_name = "team_member_function", rename_all = "PascalCase")]
pub enum TeamMemberFunction {
    Commander,
    Driver,
    Patroller,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkSessionMember {
    pub id: Uuid,
    pub work_session_id: Uuid,
    pub user_id: Uuid,
    pub function: Option<TeamMemberFunction>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkSessionMemberWithUser {
    pub id: Uuid,
    pub function: Option<TeamMemberFunction>,
    pub user: UserComplement,
}
