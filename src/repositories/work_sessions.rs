use async_trait::async_trait;
use log::info;
use sqlx::PgPool;
use uuid::Uuid;

use crate::config::querys::work_sessions::{WorkSessionMembersQueries, WorkSessionsQueries};
use crate::core::contracts::repository::work_sessions::WorkSessionRepository;
use crate::core::entities::work_sessions::{
    CreateWorkSession, WorkSession, WorkSessionWithMembers, WorkSessionMemberWithDetails,
};
use crate::core::entities::work_session_members::{
    WorkSessionMember, TeamMemberFunction, AddWorkSessionMember,
};

#[derive(Clone)]
pub struct PgWorkSessionRepository {
    pool: PgPool,
}

impl PgWorkSessionRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl WorkSessionRepository for PgWorkSessionRepository {
    async fn create_work_session(
        &self,
        data: CreateWorkSession,
        created_by_user_id: Uuid,
    ) -> Result<WorkSessionWithMembers, sqlx::Error> {
        let session_id = Uuid::new_v4();

        info!(
            "[Repository] Creating work session for user: {} with ID: {}",
            created_by_user_id, session_id
        );

        let mut tx = self.pool.begin().await?;

        let session: WorkSession = sqlx::query_as(WorkSessionsQueries::CREATE_WORK_SESSION)
            .bind(session_id)
            .bind(created_by_user_id)
            .bind(&data.description)
            .fetch_one(&mut *tx)
            .await?;

        let creator_session_member_registration_id = Uuid::new_v4();
        let _: WorkSessionMember = sqlx::query_as(WorkSessionMembersQueries::CREATE_WORK_SESSION_MEMBER)
            .bind(creator_session_member_registration_id)
            .bind(session_id)
            .bind(created_by_user_id)
            .bind(Some(TeamMemberFunction::Commander))
            .fetch_one(&mut *tx)
            .await?;

        for member in &data.members {
            let session_member_registration_id = Uuid::new_v4();
            let _: WorkSessionMember = sqlx::query_as(WorkSessionMembersQueries::CREATE_WORK_SESSION_MEMBER)
                .bind(session_member_registration_id)
                .bind(session_id)
                .bind(member.user_id)
                .bind(&member.function)
                .fetch_one(&mut *tx)
                .await?;
        }

        tx.commit().await?;

        let members = self.get_session_members_with_details(session_id).await?;

        Ok(session.with_members(members))
    }

    async fn create_auto_login_session(
        &self,
        session_id: Uuid,
        session_member_registration_id: Uuid,
        user_id: Uuid,
    ) -> Result<WorkSessionWithMembers, sqlx::Error> {
        info!(
            "[Repository] Creating auto-login session {} for user {}",
            session_id, user_id
        );

        let mut tx = self.pool.begin().await?;

        let session: WorkSession = sqlx::query_as(WorkSessionsQueries::CREATE_WORK_SESSION)
            .bind(session_id)
            .bind(user_id)
            .bind(Some("Auto-created on login"))
            .fetch_one(&mut *tx)
            .await?;

        let _: WorkSessionMember = sqlx::query_as(WorkSessionMembersQueries::CREATE_WORK_SESSION_MEMBER)
            .bind(session_member_registration_id)
            .bind(session_id)
            .bind(user_id)
            .bind(Some(TeamMemberFunction::Commander))
            .fetch_one(&mut *tx)
            .await?;

        tx.commit().await?;

        let members = self.get_session_members_with_details(session_id).await?;

        Ok(session.with_members(members))
    }

    async fn get_active_session_by_user(
        &self,
        user_id: Uuid,
    ) -> Result<WorkSession, sqlx::Error> {
        let session: WorkSession = sqlx::query_as(WorkSessionsQueries::GET_ACTIVE_SESSION_BY_USER)
            .bind(user_id)
            .fetch_one(&self.pool)
            .await?;

        Ok(session)
    }

    async fn get_user_active_session(
        &self,
        user_id: Uuid,
    ) -> Result<WorkSession, sqlx::Error> {
        let session: WorkSession = sqlx::query_as(WorkSessionsQueries::GET_USER_ACTIVE_SESSION)
            .bind(user_id)
            .fetch_one(&self.pool)
            .await?;

        Ok(session)
    }

    async fn is_user_in_active_session(
        &self,
        user_id: Uuid,
    ) -> Result<bool, sqlx::Error> {
        let result: (bool,) = sqlx::query_as(WorkSessionsQueries::CHECK_USER_IN_ACTIVE_SESSION)
            .bind(user_id)
            .fetch_one(&self.pool)
            .await?;

        Ok(result.0)
    }

    async fn end_session(
        &self,
        session_id: Uuid,
    ) -> Result<WorkSession, sqlx::Error> {
        info!("[Repository] Ending work session: {}", session_id);

        let session: WorkSession = sqlx::query_as(WorkSessionsQueries::END_WORK_SESSION)
            .bind(session_id)
            .fetch_one(&self.pool)
            .await?;

        Ok(session)
    }

    async fn get_session_by_id(
        &self,
        session_id: Uuid,
    ) -> Result<WorkSessionWithMembers, sqlx::Error> {
        let session: WorkSession = sqlx::query_as(WorkSessionsQueries::GET_SESSION_BY_ID)
            .bind(session_id)
            .fetch_one(&self.pool)
            .await?;

        let members = self.get_session_members_with_details(session_id).await?;

        Ok(session.with_members(members))
    }

    async fn get_sessions_by_user(
        &self,
        user_id: Uuid,
    ) -> Result<Vec<WorkSessionWithMembers>, sqlx::Error> {
        let sessions: Vec<WorkSession> = sqlx::query_as(WorkSessionsQueries::GET_SESSIONS_BY_USER)
            .bind(user_id)
            .fetch_all(&self.pool)
            .await?;

        let mut result = Vec::with_capacity(sessions.len());

        for session in sessions {
            let members = self.get_session_members_with_details(session.id).await?;
            result.push(session.with_members(members));
        }

        Ok(result)
    }

    async fn list_sessions_filtered(
        &self,
        user_id: Option<Uuid>,
        start_date: Option<chrono::NaiveDate>,
        end_date: Option<chrono::NaiveDate>,
        city_id: Option<Uuid>,
        limit: i64,
        offset: i64,
    ) -> Result<Vec<WorkSession>, sqlx::Error> {
        let sessions: Vec<WorkSession> = sqlx::query_as(WorkSessionsQueries::LIST_SESSIONS_FILTERED_PAGED)
            .bind(user_id)
            .bind(start_date)
            .bind(end_date)
            .bind(city_id)
            .bind(limit)
            .bind(offset)
            .fetch_all(&self.pool)
            .await?;

        Ok(sessions)
    }

    async fn count_sessions_filtered(
        &self,
        user_id: Option<Uuid>,
        start_date: Option<chrono::NaiveDate>,
        end_date: Option<chrono::NaiveDate>,
        city_id: Option<Uuid>,
    ) -> Result<i64, sqlx::Error> {
        let count: i64 = sqlx::query_scalar(WorkSessionsQueries::COUNT_SESSIONS_FILTERED)
            .bind(user_id)
            .bind(start_date)
            .bind(end_date)
            .bind(city_id)
            .fetch_one(&self.pool)
            .await?;

        Ok(count)
    }

    async fn get_session_members(
        &self,
        session_id: Uuid,
    ) -> Result<Vec<WorkSessionMember>, sqlx::Error> {
        let members: Vec<WorkSessionMember> = sqlx::query_as(WorkSessionMembersQueries::GET_SESSION_MEMBERS)
            .bind(session_id)
            .fetch_all(&self.pool)
            .await?;

        Ok(members)
    }

    async fn get_session_members_with_details(
        &self,
        session_id: Uuid,
    ) -> Result<Vec<WorkSessionMemberWithDetails>, sqlx::Error> {
        let members: Vec<WorkSessionMemberWithDetails> =
            sqlx::query_as(WorkSessionMembersQueries::GET_SESSION_MEMBERS_WITH_DETAILS)
                .bind(session_id)
                .fetch_all(&self.pool)
                .await?;

        Ok(members)
    }

    async fn add_member_to_session(
        &self,
        session_member_registration_id: Uuid,
        session_id: Uuid,
        user_id: Uuid,
        function: Option<TeamMemberFunction>,
    ) -> Result<WorkSessionMember, sqlx::Error> {
        let member: WorkSessionMember = sqlx::query_as(WorkSessionMembersQueries::CREATE_WORK_SESSION_MEMBER)
            .bind(session_member_registration_id)
            .bind(session_id)
            .bind(user_id)
            .bind(function)
            .fetch_one(&self.pool)
            .await?;

        Ok(member)
    }

    async fn remove_member_from_session(
        &self,
        session_id: Uuid,
        user_id: Uuid,
    ) -> Result<WorkSessionMember, sqlx::Error> {
        let member: WorkSessionMember = sqlx::query_as(WorkSessionMembersQueries::DELETE_MEMBER)
            .bind(session_id)
            .bind(user_id)
            .fetch_one(&self.pool)
            .await?;

        Ok(member)
    }

    async fn update_member_function(
        &self,
        session_id: Uuid,
        user_id: Uuid,
        function: Option<TeamMemberFunction>,
    ) -> Result<WorkSessionMember, sqlx::Error> {
        let member: WorkSessionMember = sqlx::query_as(WorkSessionMembersQueries::UPDATE_MEMBER_FUNCTION)
            .bind(session_id)
            .bind(user_id)
            .bind(function)
            .fetch_one(&self.pool)
            .await?;

        Ok(member)
    }

    async fn update_session_members(
        &self,
        session_id: Uuid,
        members: Vec<AddWorkSessionMember>,
    ) -> Result<Vec<WorkSessionMember>, sqlx::Error> {
        let mut tx = self.pool.begin().await?;

        let _: sqlx::postgres::PgQueryResult = sqlx::query(WorkSessionMembersQueries::DELETE_ALL_MEMBERS)
            .bind(session_id)
            .execute(&mut *tx)
            .await?;

        let mut result = Vec::with_capacity(members.len());
        for member in members {
            let session_member_registration_id = Uuid::new_v4();
            let new_member: WorkSessionMember = sqlx::query_as(WorkSessionMembersQueries::CREATE_WORK_SESSION_MEMBER)
                .bind(session_member_registration_id)
                .bind(session_id)
                .bind(member.user_id)
                .bind(member.function)
                .fetch_one(&mut *tx)
                .await?;

            result.push(new_member);
        }

        tx.commit().await?;

        Ok(result)
    }
}
