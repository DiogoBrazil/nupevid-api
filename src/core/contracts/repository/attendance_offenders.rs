use async_trait::async_trait;
use uuid::Uuid;

use crate::core::entities::attendance_offenders::{
    AttendanceOffenderWithAddress, CreateAttendanceOffender, UpdateAttendanceOffender,
};

#[async_trait]
pub trait AttendanceOffenderRepository: Send + Sync {
    async fn create_attendance_offender(
        &self,
        attendance: CreateAttendanceOffender,
    ) -> Result<AttendanceOffenderWithAddress, sqlx::Error>;
    async fn get_attendance_offender_by_id(
        &self,
        id: Uuid,
    ) -> Result<AttendanceOffenderWithAddress, sqlx::Error>;
    async fn get_all_attendance_offenders(
        &self,
    ) -> Result<Vec<AttendanceOffenderWithAddress>, sqlx::Error>;
    async fn get_attendance_offenders_paginated(
        &self,
        allowed_cities: Option<&[Uuid]>,
        limit: i64,
        offset: i64,
    ) -> Result<Vec<AttendanceOffenderWithAddress>, sqlx::Error>;
    async fn count_attendance_offenders(
        &self,
        allowed_cities: Option<&[Uuid]>,
    ) -> Result<i64, sqlx::Error>;
    async fn get_attendance_offenders_by_offender(
        &self,
        offender_id: Uuid,
    ) -> Result<Vec<AttendanceOffenderWithAddress>, sqlx::Error>;
    async fn get_attendance_offenders_by_victim(
        &self,
        victim_id: Uuid,
    ) -> Result<Vec<AttendanceOffenderWithAddress>, sqlx::Error>;
    async fn update_attendance_offender_by_id(
        &self,
        data: UpdateAttendanceOffender,
        id: Uuid,
    ) -> Result<AttendanceOffenderWithAddress, sqlx::Error>;
    async fn delete_attendance_offender_by_id(
        &self,
        id: Uuid,
    ) -> Result<AttendanceOffenderWithAddress, sqlx::Error>;
}
