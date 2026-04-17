use async_trait::async_trait;
use chrono::NaiveDate;
use uuid::Uuid;

use super::error::RepositoryError;
use crate::core::commands::work_session_members::AddWorkSessionMember;
use crate::core::commands::work_sessions::CreateWorkSession;
use crate::core::entities::work_session_members::{TeamMemberFunction, WorkSessionMember};
use crate::core::entities::work_sessions::WorkSession;
use crate::core::read_models::work_sessions::{
    WorkSessionMemberWithDetails, WorkSessionMemberWithUser, WorkSessionWithMemberDetails,
};

#[async_trait]
pub trait WorkSessionReadRepository: Send + Sync {
    async fn get_active_session_by_user(
        &self,
        user_id: Uuid,
    ) -> Result<WorkSession, RepositoryError>;

    async fn get_user_active_session(&self, user_id: Uuid) -> Result<WorkSession, RepositoryError>;

    async fn is_user_in_active_session(&self, user_id: Uuid) -> Result<bool, RepositoryError>;

    async fn get_session_by_id(
        &self,
        session_id: Uuid,
    ) -> Result<WorkSessionWithMemberDetails, RepositoryError>;

    async fn get_session_by_id_base(
        &self,
        session_id: Uuid,
    ) -> Result<WorkSession, RepositoryError>;

    async fn get_sessions_by_user(
        &self,
        user_id: Uuid,
    ) -> Result<Vec<WorkSessionWithMemberDetails>, RepositoryError>;

    async fn list_sessions_filtered(
        &self,
        user_id: Option<Uuid>,
        start_date: Option<NaiveDate>,
        end_date: Option<NaiveDate>,
        city_id: Option<Uuid>,
        limit: i64,
        offset: i64,
    ) -> Result<Vec<WorkSession>, RepositoryError>;
    async fn count_sessions_filtered(
        &self,
        user_id: Option<Uuid>,
        start_date: Option<NaiveDate>,
        end_date: Option<NaiveDate>,
        city_id: Option<Uuid>,
    ) -> Result<i64, RepositoryError>;

    async fn get_session_members(
        &self,
        session_id: Uuid,
    ) -> Result<Vec<WorkSessionMember>, RepositoryError>;

    async fn get_session_members_with_details(
        &self,
        session_id: Uuid,
    ) -> Result<Vec<WorkSessionMemberWithDetails>, RepositoryError>;

    async fn get_session_members_with_user_details(
        &self,
        session_id: Uuid,
    ) -> Result<Vec<WorkSessionMemberWithUser>, RepositoryError>;
}

#[async_trait]
pub trait WorkSessionWriteRepository: Send + Sync {
    async fn create_work_session(
        &self,
        data: CreateWorkSession,
        created_by_user_id: Uuid,
    ) -> Result<WorkSession, RepositoryError>;

    async fn create_auto_login_session(
        &self,
        session_id: Uuid,
        session_member_registration_id: Uuid,
        user_id: Uuid,
    ) -> Result<WorkSession, RepositoryError>;

    async fn end_session(&self, session_id: Uuid) -> Result<WorkSession, RepositoryError>;

    async fn add_member_to_session(
        &self,
        session_member_registration_id: Uuid,
        session_id: Uuid,
        user_id: Uuid,
        function: Option<TeamMemberFunction>,
    ) -> Result<WorkSessionMember, RepositoryError>;

    async fn remove_member_from_session(
        &self,
        session_id: Uuid,
        user_id: Uuid,
    ) -> Result<WorkSessionMember, RepositoryError>;

    async fn update_member_function(
        &self,
        session_id: Uuid,
        user_id: Uuid,
        function: Option<TeamMemberFunction>,
    ) -> Result<WorkSessionMember, RepositoryError>;

    async fn update_session_members(
        &self,
        session_id: Uuid,
        members: Vec<AddWorkSessionMember>,
    ) -> Result<Vec<WorkSessionMember>, RepositoryError>;

    async fn update_work_session_with_members(
        &self,
        session_id: Uuid,
        description: Option<String>,
        members: Vec<AddWorkSessionMember>,
    ) -> Result<WorkSession, RepositoryError>;
}
