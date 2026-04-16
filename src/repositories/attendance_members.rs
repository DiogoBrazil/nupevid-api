use async_trait::async_trait;
use log::info;
use sqlx::PgPool;
use uuid::Uuid;

use crate::core::contracts::repository::attendance_members::AttendanceMemberRepository;
use crate::core::contracts::repository::error::RepositoryError;
use crate::core::entities::attendance_members::{AttendanceOffenderMember, AttendanceVictimMember};
use crate::core::read_models::attendance_members::AttendanceMemberWithDetails;
use crate::repositories::queries::attendance_members::{
    AttendanceOffenderMembersQueries, AttendanceVictimMembersQueries,
};

use super::models::attendance_members::{
    AttendanceMemberWithDetailsRow, AttendanceOffenderMemberRow, AttendanceVictimMemberRow,
};

use crate::repositories::error_mapper::map_sqlx_error;
fn map_attendance_member_error(err: sqlx::Error) -> RepositoryError {
    let base = map_sqlx_error(err);
    match base {
        RepositoryError::UniqueViolation { ref constraint } => match constraint.as_deref() {
            Some("attendance_victim_members_attendance_victim_id_user_id_key")
            | Some("attendance_offender_members_attendance_offender_id_user_id_key") => {
                RepositoryError::DuplicateEntry("Member already added to attendance".into())
            }
            _ => base,
        },
        _ => base,
    }
}

#[derive(Clone)]
pub struct PgAttendanceMemberRepository {
    pool: PgPool,
}

impl PgAttendanceMemberRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl AttendanceMemberRepository for PgAttendanceMemberRepository {
    async fn add_member_to_victim_attendance(
        &self,
        attendance_id: Uuid,
        user_id: Uuid,
        work_session_id: Option<Uuid>,
    ) -> Result<AttendanceVictimMember, RepositoryError> {
        let member_id = Uuid::new_v4();

        info!(
            "[Repository] Adding member {} to victim attendance {}",
            user_id, attendance_id
        );

        let member: AttendanceVictimMember = sqlx::query_as::<_, AttendanceVictimMemberRow>(
            AttendanceVictimMembersQueries::ADD_MEMBER,
        )
        .bind(member_id)
        .bind(attendance_id)
        .bind(user_id)
        .bind(work_session_id)
        .fetch_one(&self.pool)
        .await
        .map_err(map_attendance_member_error)?
        .into();

        Ok(member)
    }

    async fn get_victim_attendance_members(
        &self,
        attendance_id: Uuid,
    ) -> Result<Vec<AttendanceMemberWithDetails>, RepositoryError> {
        let rows: Vec<AttendanceMemberWithDetailsRow> =
            sqlx::query_as(AttendanceVictimMembersQueries::GET_MEMBERS_WITH_DETAILS)
                .bind(attendance_id)
                .fetch_all(&self.pool)
                .await
                .map_err(map_attendance_member_error)?;

        Ok(rows.into_iter().map(Into::into).collect())
    }

    async fn remove_member_from_victim_attendance(
        &self,
        attendance_id: Uuid,
        user_id: Uuid,
    ) -> Result<AttendanceVictimMember, RepositoryError> {
        let member: AttendanceVictimMember = sqlx::query_as::<_, AttendanceVictimMemberRow>(
            AttendanceVictimMembersQueries::REMOVE_MEMBER,
        )
        .bind(attendance_id)
        .bind(user_id)
        .fetch_one(&self.pool)
        .await
        .map_err(map_attendance_member_error)?
        .into();

        Ok(member)
    }

    async fn add_member_to_offender_attendance(
        &self,
        attendance_id: Uuid,
        user_id: Uuid,
        work_session_id: Option<Uuid>,
    ) -> Result<AttendanceOffenderMember, RepositoryError> {
        let member_id = Uuid::new_v4();

        info!(
            "[Repository] Adding member {} to offender attendance {}",
            user_id, attendance_id
        );

        let member: AttendanceOffenderMember = sqlx::query_as::<_, AttendanceOffenderMemberRow>(
            AttendanceOffenderMembersQueries::ADD_MEMBER,
        )
        .bind(member_id)
        .bind(attendance_id)
        .bind(user_id)
        .bind(work_session_id)
        .fetch_one(&self.pool)
        .await
        .map_err(map_attendance_member_error)?
        .into();

        Ok(member)
    }

    async fn get_offender_attendance_members(
        &self,
        attendance_id: Uuid,
    ) -> Result<Vec<AttendanceMemberWithDetails>, RepositoryError> {
        let rows: Vec<AttendanceMemberWithDetailsRow> =
            sqlx::query_as(AttendanceOffenderMembersQueries::GET_MEMBERS_WITH_DETAILS)
                .bind(attendance_id)
                .fetch_all(&self.pool)
                .await
                .map_err(map_attendance_member_error)?;

        Ok(rows.into_iter().map(Into::into).collect())
    }

    async fn remove_member_from_offender_attendance(
        &self,
        attendance_id: Uuid,
        user_id: Uuid,
    ) -> Result<AttendanceOffenderMember, RepositoryError> {
        let member: AttendanceOffenderMember = sqlx::query_as::<_, AttendanceOffenderMemberRow>(
            AttendanceOffenderMembersQueries::REMOVE_MEMBER,
        )
        .bind(attendance_id)
        .bind(user_id)
        .fetch_one(&self.pool)
        .await
        .map_err(map_attendance_member_error)?
        .into();

        Ok(member)
    }
}
