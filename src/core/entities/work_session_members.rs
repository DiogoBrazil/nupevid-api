use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value as JsonValue;
use sqlx::Type;
use sqlx::prelude::FromRow;
use uuid::Uuid;

use crate::core::entities::users::UserComplement;

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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkSessionMemberWithUser {
    pub id: Uuid,
    pub function: Option<TeamMemberFunction>,
    pub user: UserComplement,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct WorkSessionMemberWithUserRow {
    pub id: Uuid,
    pub function: Option<TeamMemberFunction>,
    pub user_id: Uuid,
    pub user_rank: String,
    pub user_registration: String,
    pub user_full_name: String,
    pub user_profile: String,
    pub user_email: String,
    pub user_city_id: Option<Uuid>,
    pub user_permission_policies: JsonValue,
    pub user_is_deleted: bool,
}

impl WorkSessionMemberWithUserRow {
    pub fn into_member(self) -> WorkSessionMemberWithUser {
        WorkSessionMemberWithUser {
            id: self.id,
            function: self.function,
            user: UserComplement {
                id: self.user_id,
                rank: self.user_rank,
                registration: self.user_registration,
                full_name: self.user_full_name,
                profile: self.user_profile,
                email: self.user_email,
                city_id: self.user_city_id,
                permission_policies: self.user_permission_policies,
                is_deleted: self.user_is_deleted,
            },
        }
    }
}
