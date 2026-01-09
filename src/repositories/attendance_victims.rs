use async_trait::async_trait;
use log::info;
use sqlx::PgPool;
use uuid::Uuid;

use crate::config::querys::attendance_victims::{AttendanceVictimAddressesQueries, AttendanceVictimsQueries};
use crate::core::contracts::repository::attendance_victims::AttendanceVictimRepository;
use crate::core::entities::attendance_victims::{
    AttendanceVictim, AttendanceVictimAddress, AttendanceAddressData, AttendanceVictimWithAddress, CreateAttendanceVictim,
    UpdateAttendanceVictim,
};

#[derive(Clone)]
pub struct PgAttendanceVictimRepository {
    pool: PgPool,
}

impl PgAttendanceVictimRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    async fn get_address_by_attendance_id(
        &self,
        attendance_id: Uuid,
    ) -> Result<Option<AttendanceVictimAddress>, sqlx::Error> {
        let address: Option<AttendanceVictimAddress> =
            sqlx::query_as(AttendanceVictimAddressesQueries::GET_ATTENDANCE_VICTIM_ADDRESS_BY_ATTENDANCE_ID)
                .bind(attendance_id)
                .fetch_optional(&self.pool)
                .await?;
        Ok(address)
    }

    async fn create_address_internal(
        tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
        attendance_id: Uuid,
        address: &AttendanceAddressData,
    ) -> Result<AttendanceVictimAddress, sqlx::Error> {
        let address_id = Uuid::new_v4();

        let created: AttendanceVictimAddress =
            sqlx::query_as(AttendanceVictimAddressesQueries::CREATE_ATTENDANCE_VICTIM_ADDRESS)
                .bind(address_id)
                .bind(attendance_id)
                .bind(&address.street)
                .bind(&address.number)
                .bind(&address.district)
                .bind(&address.city_id)
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
            sqlx::query_as(AttendanceVictimAddressesQueries::CHECK_ADDRESS_EXISTS_FOR_ATTENDANCE_VICTIM)
                .bind(attendance_id)
                .fetch_one(&mut **tx)
                .await?;
        Ok(result.0)
    }

    async fn update_address_internal(
        tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
        attendance_id: Uuid,
        address: &AttendanceAddressData,
    ) -> Result<AttendanceVictimAddress, sqlx::Error> {
        let updated: AttendanceVictimAddress =
            sqlx::query_as(AttendanceVictimAddressesQueries::UPDATE_ATTENDANCE_VICTIM_ADDRESS_BY_ATTENDANCE_ID)
                .bind(attendance_id)
                .bind(&address.street)
                .bind(&address.number)
                .bind(&address.district)
                .bind(&address.city_id)
                .bind(&address.zip_code)
                .bind(&address.complement)
                .fetch_one(&mut **tx)
                .await?;

        Ok(updated)
    }
}

#[async_trait]
impl AttendanceVictimRepository for PgAttendanceVictimRepository {
    async fn create_attendance_victim(
        &self,
        attendance: CreateAttendanceVictim,
    ) -> Result<AttendanceVictimWithAddress, sqlx::Error> {
        let attendance_id = Uuid::new_v4();

        info!(
            "[Repository] Starting transaction to create attendance victim for victim: {} with ID: {}",
            attendance.victim_id, attendance_id
        );

        let mut tx = self.pool.begin().await?;

        let attendance_created: AttendanceVictim = sqlx::query_as(AttendanceVictimsQueries::CREATE_ATTENDANCE_VICTIM)
            .bind(attendance_id)
            .bind(&attendance.victim_id)
            .bind(&attendance.was_victim_present)
            .bind(&attendance.attendance_date)
            .bind(&attendance.attendance_time)
            .bind(&attendance.description)
            .bind(&attendance.latitude)
            .bind(&attendance.longitude)
            .bind(&attendance.offender_id)
            .bind(&attendance.protective_measure_id)
            .bind(&attendance.is_remote)
            .bind(&attendance.risk_level)
            .bind(&attendance.offender_freedom_status)
            .bind(&attendance.offender_has_firearm_access)
            .bind(&attendance.needs_legal_assistance)
            .bind(&attendance.needs_psychological_support)
            .bind(&attendance.was_instructed_about_protective_measure_procedures)
            .bind(&attendance.offender_violated_protective_measure)
            .fetch_one(&mut *tx)
            .await?;

        info!("[Repository] Attendance victim inserted, now creating address if provided");

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
            "[Repository] Transaction committed. Attendance victim {} created successfully",
            attendance_id
        );

        Ok(attendance_created.with_address(address))
    }

    async fn get_attendance_victim_by_id(&self, id: Uuid) -> Result<AttendanceVictimWithAddress, sqlx::Error> {
        info!("[Repository] Fetching attendance victim with id: {}", id);

        let attendance: AttendanceVictim = sqlx::query_as(AttendanceVictimsQueries::GET_ATTENDANCE_VICTIM_BY_ID)
            .bind(id)
            .fetch_one(&self.pool)
            .await?;

        let address = self.get_address_by_attendance_id(id).await?;

        info!(
            "[Repository] Attendance victim {} found with address: {}",
            id,
            address.is_some()
        );

        Ok(attendance.with_address(address))
    }

    async fn get_all_attendance_victims(&self) -> Result<Vec<AttendanceVictimWithAddress>, sqlx::Error> {
        info!("[Repository] Fetching all attendance victims");

        let attendances: Vec<AttendanceVictim> = sqlx::query_as(AttendanceVictimsQueries::GET_ALL_ATTENDANCE_VICTIMS)
            .fetch_all(&self.pool)
            .await?;

        let mut result = Vec::with_capacity(attendances.len());

        for attendance in attendances {
            let address = self.get_address_by_attendance_id(attendance.id).await?;
            result.push(attendance.with_address(address));
        }

        info!("[Repository] Found {} attendance victims", result.len());

        Ok(result)
    }

    async fn get_attendance_victims_paginated(
        &self,
        allowed_cities: Option<&[Uuid]>,
        limit: i64,
        offset: i64,
    ) -> Result<Vec<AttendanceVictimWithAddress>, sqlx::Error> {
        info!("[Repository] Fetching attendance victims paginated");

        let attendances: Vec<AttendanceVictim> = match allowed_cities {
            Some(cities) => sqlx::query_as(AttendanceVictimsQueries::GET_ATTENDANCE_VICTIMS_PAGED_BY_CITIES)
                .bind(cities)
                .bind(limit)
                .bind(offset)
                .fetch_all(&self.pool)
                .await?,
            None => sqlx::query_as(AttendanceVictimsQueries::GET_ATTENDANCE_VICTIMS_PAGED)
                .bind(limit)
                .bind(offset)
                .fetch_all(&self.pool)
                .await?,
        };

        let mut result = Vec::with_capacity(attendances.len());

        for attendance in attendances {
            let address = self.get_address_by_attendance_id(attendance.id).await?;
            result.push(attendance.with_address(address));
        }

        Ok(result)
    }

    async fn count_attendance_victims(&self, allowed_cities: Option<&[Uuid]>) -> Result<i64, sqlx::Error> {
        let total: i64 = match allowed_cities {
            Some(cities) => sqlx::query_scalar(AttendanceVictimsQueries::COUNT_ATTENDANCE_VICTIMS_BY_CITIES)
                .bind(cities)
                .fetch_one(&self.pool)
                .await?,
            None => sqlx::query_scalar(AttendanceVictimsQueries::COUNT_ATTENDANCE_VICTIMS)
                .fetch_one(&self.pool)
                .await?,
        };
        Ok(total)
    }

    async fn get_attendance_victims_by_victim(
        &self,
        victim_id: Uuid,
    ) -> Result<Vec<AttendanceVictimWithAddress>, sqlx::Error> {
        info!("[Repository] Fetching attendance victims for victim: {}", victim_id);

        let attendances: Vec<AttendanceVictim> =
            sqlx::query_as(AttendanceVictimsQueries::GET_ATTENDANCE_VICTIMS_BY_VICTIM)
                .bind(victim_id)
                .fetch_all(&self.pool)
                .await?;

        let mut result = Vec::with_capacity(attendances.len());

        for attendance in attendances {
            let address = self.get_address_by_attendance_id(attendance.id).await?;
            result.push(attendance.with_address(address));
        }

        info!(
            "[Repository] Found {} attendance victims for victim: {}",
            result.len(),
            victim_id
        );

        Ok(result)
    }

    async fn update_attendance_victim_by_id(
        &self,
        data: UpdateAttendanceVictim,
        id: Uuid,
    ) -> Result<AttendanceVictimWithAddress, sqlx::Error> {
        info!(
            "[Repository] Starting transaction to update attendance victim: {}",
            id
        );

        let mut tx = self.pool.begin().await?;

        let attendance_updated: AttendanceVictim =
            sqlx::query_as(AttendanceVictimsQueries::UPDATE_ATTENDANCE_VICTIM_BY_ID)
                .bind(id)
                .bind(&data.victim_id)
                .bind(&data.was_victim_present)
                .bind(&data.attendance_date)
                .bind(&data.attendance_time)
                .bind(&data.description)
                .bind(&data.latitude)
                .bind(&data.longitude)
                .bind(&data.offender_id)
                .bind(&data.protective_measure_id)
                .bind(&data.is_remote)
                .bind(&data.risk_level)
                .bind(&data.offender_freedom_status)
                .bind(&data.offender_has_firearm_access)
                .bind(&data.needs_legal_assistance)
                .bind(&data.needs_psychological_support)
                .bind(&data.was_instructed_about_protective_measure_procedures)
                .bind(&data.offender_violated_protective_measure)
                .fetch_one(&mut *tx)
                .await?;

        let address = if let Some(addr_data) = &data.address {
            let has_address = Self::check_address_exists(&mut tx, id).await?;

            let addr = if has_address {
                info!(
                    "[Repository] Updating existing address for attendance victim: {}",
                    id
                );
                Self::update_address_internal(&mut tx, id, addr_data).await?
            } else {
                info!(
                    "[Repository] Creating new address for attendance victim: {}",
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
            "[Repository] Transaction committed. Attendance victim {} updated",
            id
        );

        let final_address = if address.is_some() {
            address
        } else {
            self.get_address_by_attendance_id(id).await?
        };

        Ok(attendance_updated.with_address(final_address))
    }

    async fn delete_attendance_victim_by_id(&self, id: Uuid) -> Result<AttendanceVictimWithAddress, sqlx::Error> {
        info!(
            "[Repository] Starting transaction to soft delete attendance victim: {}",
            id
        );

        let address = self.get_address_by_attendance_id(id).await?;

        let mut tx = self.pool.begin().await?;

        let _: Option<AttendanceVictimAddress> =
            sqlx::query_as(AttendanceVictimAddressesQueries::DELETE_ATTENDANCE_VICTIM_ADDRESS_BY_ATTENDANCE_ID)
                .bind(id)
                .fetch_optional(&mut *tx)
                .await?;

        let deleted_attendance: AttendanceVictim =
            sqlx::query_as(AttendanceVictimsQueries::DELETE_ATTENDANCE_VICTIM_BY_ID)
                .bind(id)
                .fetch_one(&mut *tx)
                .await?;

        tx.commit().await?;

        info!(
            "[Repository] Transaction committed. Attendance victim {} soft deleted",
            id
        );

        Ok(deleted_attendance.with_address(address))
    }
}
