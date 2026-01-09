use async_trait::async_trait;
use chrono::NaiveDate;
use uuid::Uuid;

use crate::core::entities::work_sessions::{
    CreateWorkSession, WorkSession, WorkSessionWithMembers, WorkSessionMemberWithDetails,
};
use crate::core::entities::work_session_members::{
    WorkSessionMember, TeamMemberFunction, AddWorkSessionMember,
};

#[async_trait]
pub trait WorkSessionRepository: Send + Sync {
    async fn create_work_session(
        &self,
        data: CreateWorkSession,
        created_by_user_id: Uuid,
    ) -> Result<WorkSessionWithMembers, sqlx::Error>;

    async fn create_auto_login_session(
        &self,
        session_id: Uuid,
        session_member_registration_id: Uuid,
        user_id: Uuid,
    ) -> Result<WorkSessionWithMembers, sqlx::Error>;

    async fn get_active_session_by_user(
        &self,
        user_id: Uuid,
    ) -> Result<WorkSession, sqlx::Error>;

    async fn get_user_active_session(
        &self,
        user_id: Uuid,
    ) -> Result<WorkSession, sqlx::Error>;

    async fn is_user_in_active_session(
        &self,
        user_id: Uuid,
    ) -> Result<bool, sqlx::Error>;

    async fn end_session(
        &self,
        session_id: Uuid,
    ) -> Result<WorkSession, sqlx::Error>;

    async fn get_session_by_id(
        &self,
        session_id: Uuid,
    ) -> Result<WorkSessionWithMembers, sqlx::Error>;

    async fn get_sessions_by_user(
        &self,
        user_id: Uuid,
    ) -> Result<Vec<WorkSessionWithMembers>, sqlx::Error>;

    async fn list_sessions_filtered(
        &self,
        user_id: Option<Uuid>,
        start_date: Option<NaiveDate>,
        end_date: Option<NaiveDate>,
        city_id: Option<Uuid>,
        limit: i64,
        offset: i64,
    ) -> Result<Vec<WorkSession>, sqlx::Error>;
    async fn count_sessions_filtered(
        &self,
        user_id: Option<Uuid>,
        start_date: Option<NaiveDate>,
        end_date: Option<NaiveDate>,
        city_id: Option<Uuid>,
    ) -> Result<i64, sqlx::Error>;

    async fn get_session_members(
        &self,
        session_id: Uuid,
    ) -> Result<Vec<WorkSessionMember>, sqlx::Error>;

    async fn get_session_members_with_details(
        &self,
        session_id: Uuid,
    ) -> Result<Vec<WorkSessionMemberWithDetails>, sqlx::Error>;

    async fn add_member_to_session(
        &self,
        session_member_registration_id: Uuid,
        session_id: Uuid,
        user_id: Uuid,
        function: Option<TeamMemberFunction>,
    ) -> Result<WorkSessionMember, sqlx::Error>;

    async fn remove_member_from_session(
        &self,
        session_id: Uuid,
        user_id: Uuid,
    ) -> Result<WorkSessionMember, sqlx::Error>;

    async fn update_member_function(
        &self,
        session_id: Uuid,
        user_id: Uuid,
        function: Option<TeamMemberFunction>,
    ) -> Result<WorkSessionMember, sqlx::Error>;

    async fn update_session_members(
        &self,
        session_id: Uuid,
        members: Vec<AddWorkSessionMember>,
    ) -> Result<Vec<WorkSessionMember>, sqlx::Error>;
}
