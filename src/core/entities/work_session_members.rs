use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::prelude::FromRow;
use sqlx::Type;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, Type, PartialEq)]
#[sqlx(type_name = "team_member_function", rename_all = "PascalCase")]
pub enum TeamMemberFunction {
    Commander,
    Driver,
    Patroller,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct WorkSessionMember {
    pub id: Uuid,
    pub work_session_id: Uuid,
    pub user_id: Uuid,
    pub function: Option<TeamMemberFunction>,
    pub created_at: DateTime<Utc>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct AddWorkSessionMember {
    pub user_id: Uuid,
    pub function: Option<TeamMemberFunction>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct UpdateMemberFunction {
    pub function: Option<TeamMemberFunction>,
}
