use chrono::{DateTime, Utc};
use serde_json::Value as JsonValue;
use sqlx::prelude::FromRow;
use uuid::Uuid;

use crate::core::entities::work_session_members::{TeamMemberFunction, WorkSessionMember};
use crate::core::entities::work_sessions::WorkSession;
use crate::core::read_models::users::UserSummary;
use crate::core::read_models::work_sessions::WorkSessionMemberWithDetails;
use crate::core::read_models::work_sessions::WorkSessionMemberWithUser;
use crate::core::value_objects::profiles::Profile;
use crate::core::value_objects::ranks::Rank;

#[derive(Debug, Clone, FromRow)]
pub struct WorkSessionRow {
    pub id: Uuid,
    pub created_by_user_id: Uuid,
    pub started_at: DateTime<Utc>,
    pub ended_at: Option<DateTime<Utc>>,
    pub is_active: bool,
    pub description: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl From<WorkSessionRow> for WorkSession {
    fn from(row: WorkSessionRow) -> Self {
        WorkSession {
            id: row.id,
            created_by_user_id: row.created_by_user_id,
            started_at: row.started_at,
            ended_at: row.ended_at,
            is_active: row.is_active,
            description: row.description,
            created_at: row.created_at,
            updated_at: row.updated_at,
        }
    }
}

#[derive(Debug, Clone, FromRow)]
pub struct WorkSessionMemberRow {
    pub id: Uuid,
    pub work_session_id: Uuid,
    pub user_id: Uuid,
    pub function: Option<TeamMemberFunction>,
    pub created_at: DateTime<Utc>,
}

impl From<WorkSessionMemberRow> for WorkSessionMember {
    fn from(row: WorkSessionMemberRow) -> Self {
        WorkSessionMember {
            id: row.id,
            work_session_id: row.work_session_id,
            user_id: row.user_id,
            function: row.function,
            created_at: row.created_at,
        }
    }
}

#[derive(Debug, Clone, FromRow)]
pub struct WorkSessionMemberWithUserRowModel {
    pub id: Uuid,
    pub function: Option<TeamMemberFunction>,
    pub user_id: Uuid,
    pub user_rank: Rank,
    pub user_registration: String,
    pub user_full_name: String,
    pub user_profile: Profile,
    pub user_email: String,
    pub user_city_id: Option<Uuid>,
    pub user_permission_policies: JsonValue,
    pub user_is_deleted: bool,
}

#[derive(Debug, Clone, FromRow)]
pub struct WorkSessionMemberWithDetailsRow {
    pub id: Uuid,
    pub user_id: Uuid,
    pub user_name: String,
    pub function: Option<TeamMemberFunction>,
    pub created_at: DateTime<Utc>,
}

impl From<WorkSessionMemberWithDetailsRow> for WorkSessionMemberWithDetails {
    fn from(row: WorkSessionMemberWithDetailsRow) -> Self {
        WorkSessionMemberWithDetails {
            id: row.id,
            user_id: row.user_id,
            user_name: row.user_name,
            function: row.function,
            created_at: row.created_at,
        }
    }
}

impl From<WorkSessionMemberWithUserRowModel> for WorkSessionMemberWithUser {
    fn from(row: WorkSessionMemberWithUserRowModel) -> Self {
        WorkSessionMemberWithUser {
            id: row.id,
            function: row.function,
            user: UserSummary {
                id: row.user_id,
                rank: row.user_rank,
                registration: row.user_registration,
                full_name: row.user_full_name,
                profile: row.user_profile,
                email: row.user_email,
                city_id: row.user_city_id,
                permission_policies: row.user_permission_policies,
                is_deleted: row.user_is_deleted,
            },
        }
    }
}
