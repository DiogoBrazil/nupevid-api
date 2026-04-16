use async_trait::async_trait;
use uuid::Uuid;

use super::error::RepositoryError;
use crate::core::commands::attendance_offenders::{
    CreateAttendanceOffender, UpdateAttendanceOffender,
};
use crate::core::entities::attendance_offenders::AttendanceOffenderWriteResult;
use crate::core::read_models::attendance_offenders::AttendanceOffenderWithAddress;

#[async_trait]
pub trait AttendanceOffenderReadRepository: Send + Sync {
    async fn get_attendance_offender_by_id(
        &self,
        id: Uuid,
    ) -> Result<AttendanceOffenderWithAddress, RepositoryError>;
    async fn get_all_attendance_offenders(
        &self,
    ) -> Result<Vec<AttendanceOffenderWithAddress>, RepositoryError>;
    async fn get_attendance_offenders_paginated(
        &self,
        allowed_cities: Option<&[Uuid]>,
        limit: i64,
        offset: i64,
    ) -> Result<Vec<AttendanceOffenderWithAddress>, RepositoryError>;
    async fn count_attendance_offenders(
        &self,
        allowed_cities: Option<&[Uuid]>,
    ) -> Result<i64, RepositoryError>;
    async fn get_attendance_offenders_by_offender(
        &self,
        offender_id: Uuid,
        protective_measure_id: Option<Uuid>,
    ) -> Result<Vec<AttendanceOffenderWithAddress>, RepositoryError>;
    async fn get_attendance_offenders_by_victim(
        &self,
        victim_id: Uuid,
        protective_measure_id: Option<Uuid>,
    ) -> Result<Vec<AttendanceOffenderWithAddress>, RepositoryError>;
}

#[async_trait]
pub trait AttendanceOffenderWriteRepository: Send + Sync {
    async fn create_attendance_offender(
        &self,
        attendance: CreateAttendanceOffender,
        session_members: Vec<(Uuid, Option<Uuid>)>,
    ) -> Result<AttendanceOffenderWriteResult, RepositoryError>;
    async fn update_attendance_offender_by_id(
        &self,
        data: UpdateAttendanceOffender,
        id: Uuid,
    ) -> Result<AttendanceOffenderWriteResult, RepositoryError>;
    async fn delete_attendance_offender_by_id(
        &self,
        id: Uuid,
    ) -> Result<AttendanceOffenderWriteResult, RepositoryError>;
}
