use async_trait::async_trait;
use uuid::Uuid;

use super::error::RepositoryError;
use crate::core::commands::attendance_victims::{CreateAttendanceVictim, UpdateAttendanceVictim};
use crate::core::entities::attendance_victims::AttendanceVictimWriteResult;
use crate::core::read_models::attendance_victims::AttendanceVictimWithAddress;

#[async_trait]
pub trait AttendanceVictimReadRepository: Send + Sync {
    async fn get_attendance_victim_by_id(
        &self,
        id: Uuid,
    ) -> Result<AttendanceVictimWithAddress, RepositoryError>;
    async fn get_all_attendance_victims(
        &self,
    ) -> Result<Vec<AttendanceVictimWithAddress>, RepositoryError>;
    async fn get_attendance_victims_paginated(
        &self,
        allowed_cities: Option<&[Uuid]>,
        limit: i64,
        offset: i64,
    ) -> Result<Vec<AttendanceVictimWithAddress>, RepositoryError>;
    async fn count_attendance_victims(
        &self,
        allowed_cities: Option<&[Uuid]>,
    ) -> Result<i64, RepositoryError>;
    async fn get_attendance_victims_by_victim(
        &self,
        victim_id: Uuid,
    ) -> Result<Vec<AttendanceVictimWithAddress>, RepositoryError>;
}

#[async_trait]
pub trait AttendanceVictimWriteRepository: Send + Sync {
    async fn create_attendance_victim(
        &self,
        attendance: CreateAttendanceVictim,
        session_members: Vec<(Uuid, Option<Uuid>)>,
    ) -> Result<AttendanceVictimWriteResult, RepositoryError>;
    async fn update_attendance_victim_by_id(
        &self,
        data: UpdateAttendanceVictim,
        id: Uuid,
    ) -> Result<AttendanceVictimWriteResult, RepositoryError>;
    async fn delete_attendance_victim_by_id(
        &self,
        id: Uuid,
    ) -> Result<AttendanceVictimWriteResult, RepositoryError>;
}
