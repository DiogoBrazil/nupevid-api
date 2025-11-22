use async_trait::async_trait;
use uuid::Uuid;
use crate::core::entities::attendances::{
    CreateAttendance,
    UpdateAttendance,
    Attendance,
    CreateAttendanceAddress,
    UpdateAttendanceAddress,
    AttendanceAddress
};

#[async_trait]
pub trait AttendanceRepository: Send + Sync {
    async fn create_attendance(&self, attendance: CreateAttendance) -> Result<Attendance, sqlx::Error>;
    async fn get_attendance_by_id(&self, id: Uuid) -> Result<Attendance, sqlx::Error>;
    async fn get_all_attendances(&self) -> Result<Vec<Attendance>, sqlx::Error>;
    async fn get_attendances_by_victim(&self, victim_id: Uuid) -> Result<Vec<Attendance>, sqlx::Error>;
    async fn update_attendance_by_id(&self, data: UpdateAttendance, id: Uuid) -> Result<Attendance, sqlx::Error>;
    async fn delete_attendance_by_id(&self, id: Uuid) -> Result<Attendance, sqlx::Error>;
}

#[async_trait]
pub trait AttendanceAddressRepository: Send + Sync {
    async fn create_attendance_address(&self, address: CreateAttendanceAddress) -> Result<AttendanceAddress, sqlx::Error>;
    async fn get_attendance_address_by_id(&self, id: Uuid) -> Result<AttendanceAddress, sqlx::Error>;
    async fn get_attendance_address_by_attendance_id(&self, attendance_id: Uuid) -> Result<Option<AttendanceAddress>, sqlx::Error>;
    async fn update_attendance_address_by_id(&self, data: UpdateAttendanceAddress, id: Uuid) -> Result<AttendanceAddress, sqlx::Error>;
    async fn delete_attendance_address_by_id(&self, id: Uuid) -> Result<AttendanceAddress, sqlx::Error>;
}
