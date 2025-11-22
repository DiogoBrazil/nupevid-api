use async_trait::async_trait;
use uuid::Uuid;

use crate::core::entities::attendances::{AttendanceWithAddress, CreateAttendance, UpdateAttendance};

#[async_trait]
pub trait AttendanceRepository: Send + Sync {
    async fn create_attendance(&self, attendance: CreateAttendance,) -> Result<AttendanceWithAddress, sqlx::Error>;
    async fn get_attendance_by_id(&self, id: Uuid) -> Result<AttendanceWithAddress, sqlx::Error>;
    async fn get_all_attendances(&self) -> Result<Vec<AttendanceWithAddress>, sqlx::Error>;
    async fn get_attendances_by_victim(&self, victim_id: Uuid,) -> Result<Vec<AttendanceWithAddress>, sqlx::Error>;
    async fn update_attendance_by_id(&self, data: UpdateAttendance, id: Uuid,) -> Result<AttendanceWithAddress, sqlx::Error>;
    async fn delete_attendance_by_id(&self, id: Uuid) -> Result<AttendanceWithAddress, sqlx::Error>;
}
