use async_trait::async_trait;
use uuid::Uuid;

use crate::core::entities::attendance_victims::{AttendanceVictimWithAddress, CreateAttendanceVictim, UpdateAttendanceVictim};

#[async_trait]
pub trait AttendanceVictimRepository: Send + Sync {
    async fn create_attendance_victim(&self, attendance: CreateAttendanceVictim,) -> Result<AttendanceVictimWithAddress, sqlx::Error>;
    async fn get_attendance_victim_by_id(&self, id: Uuid) -> Result<AttendanceVictimWithAddress, sqlx::Error>;
    async fn get_all_attendance_victims(&self) -> Result<Vec<AttendanceVictimWithAddress>, sqlx::Error>;
    async fn get_attendance_victims_by_victim(&self, victim_id: Uuid,) -> Result<Vec<AttendanceVictimWithAddress>, sqlx::Error>;
    async fn update_attendance_victim_by_id(&self, data: UpdateAttendanceVictim, id: Uuid,) -> Result<AttendanceVictimWithAddress, sqlx::Error>;
    async fn delete_attendance_victim_by_id(&self, id: Uuid) -> Result<AttendanceVictimWithAddress, sqlx::Error>;
}
