use async_trait::async_trait;
use log::info;
use sqlx::PgPool;
use uuid::Uuid;

use super::models::work_sessions::{
    WorkSessionMemberRow, WorkSessionMemberWithDetailsRow, WorkSessionMemberWithUserRowModel,
    WorkSessionRow,
};
use crate::core::commands::work_session_members::AddWorkSessionMember;
use crate::core::commands::work_sessions::CreateWorkSession;
use crate::core::contracts::repository::error::RepositoryError;
use crate::core::contracts::repository::work_sessions::{
    WorkSessionReadRepository, WorkSessionWriteRepository,
};
use crate::core::entities::work_session_members::{TeamMemberFunction, WorkSessionMember};
use crate::core::entities::work_sessions::WorkSession;
use crate::core::read_models::work_sessions::{
    WorkSessionMemberWithDetails, WorkSessionMemberWithUser, WorkSessionWithMemberDetails,
};
use crate::repositories::queries::work_sessions::{WorkSessionMembersQueries, WorkSessionsQueries};

use crate::repositories::error_mapper::map_sqlx_error;
fn map_work_session_error(err: sqlx::Error) -> RepositoryError {
    if let sqlx::Error::Database(ref db_err) = err {
        match db_err.constraint() {
            Some("unique_active_session_per_user") => {
                return RepositoryError::Conflict("User already has an active work session".into());
            }
            Some("work_session_members_work_session_id_user_id_key") => {
                return RepositoryError::DuplicateEntry("User already added to session".into());
            }
            _ => {}
        }
    }

    let base = map_sqlx_error(err);
    match base {
        RepositoryError::UniqueViolation { ref constraint } => match constraint.as_deref() {
            Some("unique_active_session_per_user") => {
                RepositoryError::Conflict("User already has an active work session".into())
            }
            Some("work_session_members_work_session_id_user_id_key") => {
                RepositoryError::DuplicateEntry("User already added to session".into())
            }
            _ => base,
        },
        _ => base,
    }
}

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
impl WorkSessionReadRepository for PgWorkSessionRepository {
    async fn get_active_session_by_user(
        &self,
        user_id: Uuid,
    ) -> Result<WorkSession, RepositoryError> {
        let session: WorkSessionRow =
            sqlx::query_as(WorkSessionsQueries::GET_ACTIVE_SESSION_BY_USER)
                .bind(user_id)
                .fetch_one(&self.pool)
                .await
                .map_err(map_work_session_error)?;

        Ok(session.into())
    }

    async fn get_user_active_session(&self, user_id: Uuid) -> Result<WorkSession, RepositoryError> {
        let session: WorkSessionRow = sqlx::query_as(WorkSessionsQueries::GET_USER_ACTIVE_SESSION)
            .bind(user_id)
            .fetch_one(&self.pool)
            .await
            .map_err(map_work_session_error)?;

        Ok(session.into())
    }

    async fn is_user_in_active_session(&self, user_id: Uuid) -> Result<bool, RepositoryError> {
        let result: (bool,) = sqlx::query_as(WorkSessionsQueries::CHECK_USER_IN_ACTIVE_SESSION)
            .bind(user_id)
            .fetch_one(&self.pool)
            .await
            .map_err(map_work_session_error)?;

        Ok(result.0)
    }

    async fn get_session_by_id(
        &self,
        session_id: Uuid,
    ) -> Result<WorkSessionWithMemberDetails, RepositoryError> {
        let row: WorkSessionRow = sqlx::query_as(WorkSessionsQueries::GET_SESSION_BY_ID)
            .bind(session_id)
            .fetch_one(&self.pool)
            .await
            .map_err(map_work_session_error)?;

        let session: WorkSession = row.into();
        let members = self.get_session_members_with_details(session_id).await?;

        Ok(WorkSessionWithMemberDetails::from_entity(session, members))
    }

    async fn get_session_by_id_base(
        &self,
        session_id: Uuid,
    ) -> Result<WorkSession, RepositoryError> {
        let session: WorkSessionRow = sqlx::query_as(WorkSessionsQueries::GET_SESSION_BY_ID)
            .bind(session_id)
            .fetch_one(&self.pool)
            .await
            .map_err(map_work_session_error)?;

        Ok(session.into())
    }

    async fn get_sessions_by_user(
        &self,
        user_id: Uuid,
    ) -> Result<Vec<WorkSessionWithMemberDetails>, RepositoryError> {
        let rows: Vec<WorkSessionRow> = sqlx::query_as(WorkSessionsQueries::GET_SESSIONS_BY_USER)
            .bind(user_id)
            .fetch_all(&self.pool)
            .await
            .map_err(map_work_session_error)?;

        let sessions: Vec<WorkSession> = rows.into_iter().map(Into::into).collect();
        let mut result = Vec::with_capacity(sessions.len());

        for session in sessions {
            let members = self.get_session_members_with_details(session.id).await?;
            result.push(WorkSessionWithMemberDetails::from_entity(session, members));
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
    ) -> Result<Vec<WorkSession>, RepositoryError> {
        let rows: Vec<WorkSessionRow> =
            sqlx::query_as(WorkSessionsQueries::LIST_SESSIONS_FILTERED_PAGED)
                .bind(user_id)
                .bind(start_date)
                .bind(end_date)
                .bind(city_id)
                .bind(limit)
                .bind(offset)
                .fetch_all(&self.pool)
                .await
                .map_err(map_work_session_error)?;

        Ok(rows.into_iter().map(Into::into).collect())
    }

    async fn count_sessions_filtered(
        &self,
        user_id: Option<Uuid>,
        start_date: Option<chrono::NaiveDate>,
        end_date: Option<chrono::NaiveDate>,
        city_id: Option<Uuid>,
    ) -> Result<i64, RepositoryError> {
        let count: i64 = sqlx::query_scalar(WorkSessionsQueries::COUNT_SESSIONS_FILTERED)
            .bind(user_id)
            .bind(start_date)
            .bind(end_date)
            .bind(city_id)
            .fetch_one(&self.pool)
            .await
            .map_err(map_work_session_error)?;

        Ok(count)
    }

    async fn get_session_members(
        &self,
        session_id: Uuid,
    ) -> Result<Vec<WorkSessionMember>, RepositoryError> {
        let rows: Vec<WorkSessionMemberRow> =
            sqlx::query_as(WorkSessionMembersQueries::GET_SESSION_MEMBERS)
                .bind(session_id)
                .fetch_all(&self.pool)
                .await
                .map_err(map_work_session_error)?;

        Ok(rows.into_iter().map(Into::into).collect())
    }

    async fn get_session_members_with_details(
        &self,
        session_id: Uuid,
    ) -> Result<Vec<WorkSessionMemberWithDetails>, RepositoryError> {
        let rows: Vec<WorkSessionMemberWithDetailsRow> =
            sqlx::query_as(WorkSessionMembersQueries::GET_SESSION_MEMBERS_WITH_DETAILS)
                .bind(session_id)
                .fetch_all(&self.pool)
                .await
                .map_err(map_work_session_error)?;

        Ok(rows.into_iter().map(Into::into).collect())
    }

    async fn get_session_members_with_user_details(
        &self,
        session_id: Uuid,
    ) -> Result<Vec<WorkSessionMemberWithUser>, RepositoryError> {
        let rows: Vec<WorkSessionMemberWithUserRowModel> =
            sqlx::query_as(WorkSessionMembersQueries::GET_SESSION_MEMBERS_WITH_USER_DETAILS)
                .bind(session_id)
                .fetch_all(&self.pool)
                .await
                .map_err(map_work_session_error)?;

        Ok(rows.into_iter().map(Into::into).collect())
    }
}

#[async_trait]
impl WorkSessionWriteRepository for PgWorkSessionRepository {
    async fn create_work_session(
        &self,
        data: CreateWorkSession,
        created_by_user_id: Uuid,
    ) -> Result<WorkSession, RepositoryError> {
        let session_id = Uuid::new_v4();

        info!(
            "[Repository] Creating work session for user: {} with ID: {}",
            created_by_user_id, session_id
        );

        let mut tx = self.pool.begin().await.map_err(map_work_session_error)?;

        let session: WorkSessionRow = sqlx::query_as(WorkSessionsQueries::CREATE_WORK_SESSION)
            .bind(session_id)
            .bind(created_by_user_id)
            .bind(&data.description)
            .fetch_one(&mut *tx)
            .await
            .map_err(map_work_session_error)?;

        for member in &data.members {
            let session_member_registration_id = Uuid::new_v4();
            let _: WorkSessionMemberRow =
                sqlx::query_as(WorkSessionMembersQueries::CREATE_WORK_SESSION_MEMBER)
                    .bind(session_member_registration_id)
                    .bind(session_id)
                    .bind(member.user_id)
                    .bind(&member.function)
                    .fetch_one(&mut *tx)
                    .await
                    .map_err(map_work_session_error)?;
        }

        tx.commit().await.map_err(map_work_session_error)?;

        Ok(session.into())
    }

    async fn create_auto_login_session(
        &self,
        session_id: Uuid,
        session_member_registration_id: Uuid,
        user_id: Uuid,
    ) -> Result<WorkSession, RepositoryError> {
        info!(
            "[Repository] Creating auto-login session {} for user {}",
            session_id, user_id
        );

        let mut tx = self.pool.begin().await.map_err(map_work_session_error)?;

        let session: WorkSessionRow = sqlx::query_as(WorkSessionsQueries::CREATE_WORK_SESSION)
            .bind(session_id)
            .bind(user_id)
            .bind(Some("Auto-created on login"))
            .fetch_one(&mut *tx)
            .await
            .map_err(map_work_session_error)?;

        let _: WorkSessionMemberRow =
            sqlx::query_as(WorkSessionMembersQueries::CREATE_WORK_SESSION_MEMBER)
                .bind(session_member_registration_id)
                .bind(session_id)
                .bind(user_id)
                .bind(Some(TeamMemberFunction::Commander))
                .fetch_one(&mut *tx)
                .await
                .map_err(map_work_session_error)?;

        tx.commit().await.map_err(map_work_session_error)?;

        Ok(session.into())
    }

    async fn end_session(&self, session_id: Uuid) -> Result<WorkSession, RepositoryError> {
        info!("[Repository] Ending work session: {}", session_id);

        let session: WorkSessionRow = sqlx::query_as(WorkSessionsQueries::END_WORK_SESSION)
            .bind(session_id)
            .fetch_one(&self.pool)
            .await
            .map_err(map_work_session_error)?;

        Ok(session.into())
    }

    async fn add_member_to_session(
        &self,
        session_member_registration_id: Uuid,
        session_id: Uuid,
        user_id: Uuid,
        function: Option<TeamMemberFunction>,
    ) -> Result<WorkSessionMember, RepositoryError> {
        let member: WorkSessionMemberRow =
            sqlx::query_as(WorkSessionMembersQueries::CREATE_WORK_SESSION_MEMBER)
                .bind(session_member_registration_id)
                .bind(session_id)
                .bind(user_id)
                .bind(function)
                .fetch_one(&self.pool)
                .await
                .map_err(map_work_session_error)?;

        Ok(member.into())
    }

    async fn remove_member_from_session(
        &self,
        session_id: Uuid,
        user_id: Uuid,
    ) -> Result<WorkSessionMember, RepositoryError> {
        let member: WorkSessionMemberRow = sqlx::query_as(WorkSessionMembersQueries::DELETE_MEMBER)
            .bind(session_id)
            .bind(user_id)
            .fetch_one(&self.pool)
            .await
            .map_err(map_work_session_error)?;

        Ok(member.into())
    }

    async fn update_member_function(
        &self,
        session_id: Uuid,
        user_id: Uuid,
        function: Option<TeamMemberFunction>,
    ) -> Result<WorkSessionMember, RepositoryError> {
        let member: WorkSessionMemberRow =
            sqlx::query_as(WorkSessionMembersQueries::UPDATE_MEMBER_FUNCTION)
                .bind(session_id)
                .bind(user_id)
                .bind(function)
                .fetch_one(&self.pool)
                .await
                .map_err(map_work_session_error)?;

        Ok(member.into())
    }

    async fn update_session_members(
        &self,
        session_id: Uuid,
        members: Vec<AddWorkSessionMember>,
    ) -> Result<Vec<WorkSessionMember>, RepositoryError> {
        let mut tx = self.pool.begin().await.map_err(map_work_session_error)?;

        let _: sqlx::postgres::PgQueryResult =
            sqlx::query(WorkSessionMembersQueries::DELETE_ALL_MEMBERS)
                .bind(session_id)
                .execute(&mut *tx)
                .await
                .map_err(map_work_session_error)?;

        let mut result = Vec::with_capacity(members.len());
        for member in members {
            let session_member_registration_id = Uuid::new_v4();
            let new_member: WorkSessionMemberRow =
                sqlx::query_as(WorkSessionMembersQueries::CREATE_WORK_SESSION_MEMBER)
                    .bind(session_member_registration_id)
                    .bind(session_id)
                    .bind(member.user_id)
                    .bind(member.function)
                    .fetch_one(&mut *tx)
                    .await
                    .map_err(map_work_session_error)?;

            result.push(new_member.into());
        }

        tx.commit().await.map_err(map_work_session_error)?;

        Ok(result)
    }

    async fn update_work_session_with_members(
        &self,
        session_id: Uuid,
        description: Option<String>,
        members: Vec<AddWorkSessionMember>,
    ) -> Result<WorkSession, RepositoryError> {
        let mut tx = self.pool.begin().await.map_err(map_work_session_error)?;

        if let Some(desc) = description {
            let _: WorkSessionRow =
                sqlx::query_as(WorkSessionsQueries::UPDATE_WORK_SESSION_DESCRIPTION)
                    .bind(session_id)
                    .bind(desc)
                    .fetch_one(&mut *tx)
                    .await
                    .map_err(map_work_session_error)?;
        }

        let _: sqlx::postgres::PgQueryResult =
            sqlx::query(WorkSessionMembersQueries::DELETE_ALL_MEMBERS)
                .bind(session_id)
                .execute(&mut *tx)
                .await
                .map_err(map_work_session_error)?;

        for member in members {
            let session_member_registration_id = Uuid::new_v4();
            let _: WorkSessionMemberRow =
                sqlx::query_as(WorkSessionMembersQueries::CREATE_WORK_SESSION_MEMBER)
                    .bind(session_member_registration_id)
                    .bind(session_id)
                    .bind(member.user_id)
                    .bind(member.function)
                    .fetch_one(&mut *tx)
                    .await
                    .map_err(map_work_session_error)?;
        }

        let session: WorkSessionRow = sqlx::query_as(WorkSessionsQueries::GET_SESSION_BY_ID)
            .bind(session_id)
            .fetch_one(&mut *tx)
            .await
            .map_err(map_work_session_error)?;

        tx.commit().await.map_err(map_work_session_error)?;

        Ok(session.into())
    }
}
