use async_trait::async_trait;
use log::info;
use sqlx::PgPool;
use uuid::Uuid;

use crate::core::{
    contracts::repository::attendances::{
        AttendanceRepository,
        AttendanceAddressRepository
    },
    entities::attendances::{
        CreateAttendance,
        UpdateAttendance,
        Attendance,
        CreateAttendanceAddress,
        UpdateAttendanceAddress,
        AttendanceAddress
    }
};
use crate::config::querys::attendances::{
    AttendancesQueries,
    AttendanceAddressesQueries
};

#[derive(Clone)]
pub struct PgAttendanceRepository {
    pool: PgPool,
}

impl PgAttendanceRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl AttendanceRepository for PgAttendanceRepository {
    async fn create_attendance(&self, attendance: CreateAttendance) -> Result<Attendance, sqlx::Error> {
        let id: Uuid = Uuid::new_v4();
        info!("[Repository] Creating attendance for victim: {}", attendance.victim_id);

        let attendance_created: Attendance = sqlx::query_as(AttendancesQueries::CREATE_ATTENDANCE)
            .bind(id)
            .bind(attendance.victim_id)
            .bind(attendance.was_victim_present)
            .bind(attendance.attendance_date)
            .bind(attendance.attendance_time)
            .bind(attendance.description)
            .bind(attendance.latitude)
            .bind(attendance.longitude)
            .fetch_one(&self.pool)
            .await?;

        info!("[Repository] Attendance created with ID: {}", attendance_created.id);
        Ok(attendance_created)
    }

    async fn get_attendance_by_id(&self, id: Uuid) -> Result<Attendance, sqlx::Error> {
        info!("[Repository] Getting attendance with id: {}", id);
        let attendance: Attendance = sqlx::query_as(AttendancesQueries::GET_ATTENDANCE_BY_ID)
            .bind(id)
            .fetch_one(&self.pool)
            .await?;
        info!("[Repository] Attendance found with ID: {}", id);
        Ok(attendance)
    }

    async fn get_all_attendances(&self) -> Result<Vec<Attendance>, sqlx::Error> {
        info!("[Repository] Getting all attendances");
        let attendances: Vec<Attendance> = sqlx::query_as(AttendancesQueries::GET_ALL_ATTENDANCES)
            .fetch_all(&self.pool)
            .await?;
        info!("[Repository] Found {} attendances", attendances.len());
        Ok(attendances)
    }

    async fn get_attendances_by_victim(&self, victim_id: Uuid) -> Result<Vec<Attendance>, sqlx::Error> {
        info!("[Repository] Getting attendances for victim: {}", victim_id);
        let attendances: Vec<Attendance> = sqlx::query_as(AttendancesQueries::GET_ATTENDANCES_BY_VICTIM)
            .bind(victim_id)
            .fetch_all(&self.pool)
            .await?;
        info!("[Repository] Found {} attendances for victim", attendances.len());
        Ok(attendances)
    }

    async fn update_attendance_by_id(&self, data: UpdateAttendance, id: Uuid) -> Result<Attendance, sqlx::Error> {
        info!("[Repository] Updating attendance with ID: {}", id);
        let attendance_updated: Attendance = sqlx::query_as(AttendancesQueries::UPDATE_ATTENDANCE_BY_ID)
            .bind(id)
            .bind(data.victim_id)
            .bind(data.was_victim_present)
            .bind(data.attendance_date)
            .bind(data.attendance_time)
            .bind(data.description)
            .bind(data.latitude)
            .bind(data.longitude)
            .fetch_one(&self.pool)
            .await?;
        info!("[Repository] Attendance updated with ID: {}", id);
        Ok(attendance_updated)
    }

    async fn delete_attendance_by_id(&self, id: Uuid) -> Result<Attendance, sqlx::Error> {
        info!("[Repository] Soft deleting attendance with id: {}", id);
        let deleted_attendance: Attendance = sqlx::query_as(AttendancesQueries::DELETE_ATTENDANCE_BY_ID)
            .bind(id)
            .fetch_one(&self.pool)
            .await?;
        info!("[Repository] Attendance soft deleted with ID: {}", id);
        Ok(deleted_attendance)
    }
}

#[derive(Clone)]
pub struct PgAttendanceAddressRepository {
    pool: PgPool,
}

impl PgAttendanceAddressRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl AttendanceAddressRepository for PgAttendanceAddressRepository {
    async fn create_attendance_address(&self, address: CreateAttendanceAddress) -> Result<AttendanceAddress, sqlx::Error> {
        let id: Uuid = Uuid::new_v4();
        info!("[Repository] Creating attendance address for attendance: {}", address.attendance_id);

        let address_created: AttendanceAddress = sqlx::query_as(AttendanceAddressesQueries::CREATE_ATTENDANCE_ADDRESS)
            .bind(id)
            .bind(address.attendance_id)
            .bind(address.street)
            .bind(address.number)
            .bind(address.district)
            .bind(address.city_name)
            .bind(address.state)
            .bind(address.zip_code)
            .bind(address.complement)
            .fetch_one(&self.pool)
            .await?;

        info!("[Repository] Attendance address created with ID: {}", address_created.id);
        Ok(address_created)
    }

    async fn get_attendance_address_by_id(&self, id: Uuid) -> Result<AttendanceAddress, sqlx::Error> {
        info!("[Repository] Getting attendance address with id: {}", id);
        let address: AttendanceAddress = sqlx::query_as(AttendanceAddressesQueries::GET_ATTENDANCE_ADDRESS_BY_ID)
            .bind(id)
            .fetch_one(&self.pool)
            .await?;
        info!("[Repository] Attendance address found with ID: {}", id);
        Ok(address)
    }

    async fn get_attendance_address_by_attendance_id(&self, attendance_id: Uuid) -> Result<Option<AttendanceAddress>, sqlx::Error> {
        info!("[Repository] Getting attendance address for attendance: {}", attendance_id);
        let address: Option<AttendanceAddress> = sqlx::query_as(AttendanceAddressesQueries::GET_ATTENDANCE_ADDRESS_BY_ATTENDANCE_ID)
            .bind(attendance_id)
            .fetch_optional(&self.pool)
            .await?;
        match &address {
            Some(_) => info!("[Repository] Attendance address found for attendance: {}", attendance_id),
            None => info!("[Repository] No address found for attendance: {}", attendance_id),
        }
        Ok(address)
    }

    async fn update_attendance_address_by_id(&self, data: UpdateAttendanceAddress, id: Uuid) -> Result<AttendanceAddress, sqlx::Error> {
        info!("[Repository] Updating attendance address with ID: {}", id);
        let address_updated: AttendanceAddress = sqlx::query_as(AttendanceAddressesQueries::UPDATE_ATTENDANCE_ADDRESS_BY_ID)
            .bind(id)
            .bind(data.street)
            .bind(data.number)
            .bind(data.district)
            .bind(data.city_name)
            .bind(data.state)
            .bind(data.zip_code)
            .bind(data.complement)
            .fetch_one(&self.pool)
            .await?;
        info!("[Repository] Attendance address updated with ID: {}", id);
        Ok(address_updated)
    }

    async fn delete_attendance_address_by_id(&self, id: Uuid) -> Result<AttendanceAddress, sqlx::Error> {
        info!("[Repository] Soft deleting attendance address with id: {}", id);
        let deleted_address: AttendanceAddress = sqlx::query_as(AttendanceAddressesQueries::DELETE_ATTENDANCE_ADDRESS_BY_ID)
            .bind(id)
            .fetch_one(&self.pool)
            .await?;
        info!("[Repository] Attendance address soft deleted with ID: {}", id);
        Ok(deleted_address)
    }
}
