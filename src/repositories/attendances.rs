use async_trait::async_trait;
use log::info;
use sqlx::PgPool;
use uuid::Uuid;

use crate::config::querys::attendances::{AttendanceAddressesQueries, AttendancesQueries};
use crate::core::contracts::repository::attendances::AttendanceRepository;
use crate::core::entities::attendances::{
    Attendance, AttendanceAddress, AttendanceAddressData, AttendanceWithAddress, CreateAttendance,
    UpdateAttendance,
};

#[derive(Clone)]
pub struct PgAttendanceRepository {
    pool: PgPool,
}

impl PgAttendanceRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    async fn get_address_by_attendance_id(
        &self,
        attendance_id: Uuid,
    ) -> Result<Option<AttendanceAddress>, sqlx::Error> {
        let address: Option<AttendanceAddress> =
            sqlx::query_as(AttendanceAddressesQueries::GET_ATTENDANCE_ADDRESS_BY_ATTENDANCE_ID)
                .bind(attendance_id)
                .fetch_optional(&self.pool)
                .await?;
        Ok(address)
    }

    async fn create_address_internal(
        tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
        attendance_id: Uuid,
        address: &AttendanceAddressData,
    ) -> Result<AttendanceAddress, sqlx::Error> {
        let address_id = Uuid::new_v4();

        let created: AttendanceAddress =
            sqlx::query_as(AttendanceAddressesQueries::CREATE_ATTENDANCE_ADDRESS)
                .bind(address_id)
                .bind(attendance_id)
                .bind(&address.street)
                .bind(&address.number)
                .bind(&address.district)
                .bind(&address.city_name)
                .bind(&address.state)
                .bind(&address.zip_code)
                .bind(&address.complement)
                .fetch_one(&mut **tx)
                .await?;

        Ok(created)
    }

    async fn check_address_exists(
        tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
        attendance_id: Uuid,
    ) -> Result<bool, sqlx::Error> {
        let result: (bool,) =
            sqlx::query_as(AttendanceAddressesQueries::CHECK_ADDRESS_EXISTS_FOR_ATTENDANCE)
                .bind(attendance_id)
                .fetch_one(&mut **tx)
                .await?;
        Ok(result.0)
    }

    async fn update_address_internal(
        tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
        attendance_id: Uuid,
        address: &AttendanceAddressData,
    ) -> Result<AttendanceAddress, sqlx::Error> {
        let updated: AttendanceAddress =
            sqlx::query_as(AttendanceAddressesQueries::UPDATE_ATTENDANCE_ADDRESS_BY_ATTENDANCE_ID)
                .bind(attendance_id)
                .bind(&address.street)
                .bind(&address.number)
                .bind(&address.district)
                .bind(&address.city_name)
                .bind(&address.state)
                .bind(&address.zip_code)
                .bind(&address.complement)
                .fetch_one(&mut **tx)
                .await?;

        Ok(updated)
    }
}

#[async_trait]
impl AttendanceRepository for PgAttendanceRepository {
    async fn create_attendance(
        &self,
        attendance: CreateAttendance,
    ) -> Result<AttendanceWithAddress, sqlx::Error> {
        let attendance_id = Uuid::new_v4();

        info!(
            "[Repository] Starting transaction to create attendance for victim: {} with ID: {}",
            attendance.victim_id, attendance_id
        );

        let mut tx = self.pool.begin().await?;

        let attendance_created: Attendance = sqlx::query_as(AttendancesQueries::CREATE_ATTENDANCE)
            .bind(attendance_id)
            .bind(&attendance.victim_id)
            .bind(&attendance.was_victim_present)
            .bind(&attendance.attendance_date)
            .bind(&attendance.attendance_time)
            .bind(&attendance.description)
            .bind(&attendance.latitude)
            .bind(&attendance.longitude)
            .fetch_one(&mut *tx)
            .await?;

        info!("[Repository] Attendance inserted, now creating address if provided");

        let address = if let Some(addr_data) = &attendance.address {
            let created_addr =
                Self::create_address_internal(&mut tx, attendance_id, addr_data).await?;
            info!("[Repository] Address created with ID: {}", created_addr.id);
            Some(created_addr)
        } else {
            None
        };

        tx.commit().await?;

        info!(
            "[Repository] Transaction committed. Attendance {} created successfully",
            attendance_id
        );

        Ok(attendance_created.with_address(address))
    }

    async fn get_attendance_by_id(&self, id: Uuid) -> Result<AttendanceWithAddress, sqlx::Error> {
        info!("[Repository] Fetching attendance with id: {}", id);

        let attendance: Attendance = sqlx::query_as(AttendancesQueries::GET_ATTENDANCE_BY_ID)
            .bind(id)
            .fetch_one(&self.pool)
            .await?;

        let address = self.get_address_by_attendance_id(id).await?;

        info!(
            "[Repository] Attendance {} found with address: {}",
            id,
            address.is_some()
        );

        Ok(attendance.with_address(address))
    }

    async fn get_all_attendances(&self) -> Result<Vec<AttendanceWithAddress>, sqlx::Error> {
        info!("[Repository] Fetching all attendances");

        let attendances: Vec<Attendance> = sqlx::query_as(AttendancesQueries::GET_ALL_ATTENDANCES)
            .fetch_all(&self.pool)
            .await?;

        let mut result = Vec::with_capacity(attendances.len());

        for attendance in attendances {
            let address = self.get_address_by_attendance_id(attendance.id).await?;
            result.push(attendance.with_address(address));
        }

        info!("[Repository] Found {} attendances", result.len());

        Ok(result)
    }

    async fn get_attendances_by_victim(
        &self,
        victim_id: Uuid,
    ) -> Result<Vec<AttendanceWithAddress>, sqlx::Error> {
        info!("[Repository] Fetching attendances for victim: {}", victim_id);

        let attendances: Vec<Attendance> =
            sqlx::query_as(AttendancesQueries::GET_ATTENDANCES_BY_VICTIM)
                .bind(victim_id)
                .fetch_all(&self.pool)
                .await?;

        let mut result = Vec::with_capacity(attendances.len());

        for attendance in attendances {
            let address = self.get_address_by_attendance_id(attendance.id).await?;
            result.push(attendance.with_address(address));
        }

        info!(
            "[Repository] Found {} attendances for victim: {}",
            result.len(),
            victim_id
        );

        Ok(result)
    }

    async fn update_attendance_by_id(
        &self,
        data: UpdateAttendance,
        id: Uuid,
    ) -> Result<AttendanceWithAddress, sqlx::Error> {
        info!(
            "[Repository] Starting transaction to update attendance: {}",
            id
        );

        let mut tx = self.pool.begin().await?;

        let attendance_updated: Attendance =
            sqlx::query_as(AttendancesQueries::UPDATE_ATTENDANCE_BY_ID)
                .bind(id)
                .bind(&data.victim_id)
                .bind(&data.was_victim_present)
                .bind(&data.attendance_date)
                .bind(&data.attendance_time)
                .bind(&data.description)
                .bind(&data.latitude)
                .bind(&data.longitude)
                .fetch_one(&mut *tx)
                .await?;

        let address = if let Some(addr_data) = &data.address {
            let has_address = Self::check_address_exists(&mut tx, id).await?;

            let addr = if has_address {
                info!(
                    "[Repository] Updating existing address for attendance: {}",
                    id
                );
                Self::update_address_internal(&mut tx, id, addr_data).await?
            } else {
                info!(
                    "[Repository] Creating new address for attendance: {}",
                    id
                );
                Self::create_address_internal(&mut tx, id, addr_data).await?
            };

            Some(addr)
        } else {
            None
        };

        tx.commit().await?;

        info!(
            "[Repository] Transaction committed. Attendance {} updated",
            id
        );

        let final_address = if address.is_some() {
            address
        } else {
            self.get_address_by_attendance_id(id).await?
        };

        Ok(attendance_updated.with_address(final_address))
    }

    async fn delete_attendance_by_id(&self, id: Uuid) -> Result<AttendanceWithAddress, sqlx::Error> {
        info!(
            "[Repository] Starting transaction to soft delete attendance: {}",
            id
        );

        let address = self.get_address_by_attendance_id(id).await?;

        let mut tx = self.pool.begin().await?;

        let _: Option<AttendanceAddress> =
            sqlx::query_as(AttendanceAddressesQueries::DELETE_ATTENDANCE_ADDRESS_BY_ATTENDANCE_ID)
                .bind(id)
                .fetch_optional(&mut *tx)
                .await?;

        let deleted_attendance: Attendance =
            sqlx::query_as(AttendancesQueries::DELETE_ATTENDANCE_BY_ID)
                .bind(id)
                .fetch_one(&mut *tx)
                .await?;

        tx.commit().await?;

        info!(
            "[Repository] Transaction committed. Attendance {} soft deleted",
            id
        );

        Ok(deleted_attendance.with_address(address))
    }
}
