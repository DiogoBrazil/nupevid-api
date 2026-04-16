use async_trait::async_trait;
use log::info;
use sqlx::PgPool;
use uuid::Uuid;

use crate::repositories::queries::attendance_offenders::{
    AttendanceOffenderAddressesQueries, AttendanceOffendersQueries,
};
use crate::core::commands::attendance_offenders::{
    CreateAttendanceOffender, UpdateAttendanceOffender,
};
use crate::core::commands::attendance_victims::AttendanceAddressData;
use crate::core::contracts::repository::attendance_offenders::{
    AttendanceOffenderReadRepository, AttendanceOffenderWriteRepository,
};
use crate::core::contracts::repository::error::RepositoryError;
use crate::core::entities::attendance_offenders::{
    AttendanceOffender, AttendanceOffenderAddress, AttendanceOffenderWriteResult,
};
use crate::core::read_models::attendance_offenders::AttendanceOffenderWithAddress;

use super::models::attendance_offenders::{AttendanceOffenderAddressRow, AttendanceOffenderRow};

use crate::repositories::error_mapper::map_sqlx_error;
fn map_attendance_offender_error(err: sqlx::Error) -> RepositoryError {
    let base = map_sqlx_error(err);
    match base {
        RepositoryError::ForeignKeyViolation { ref constraint } => {
            match constraint.as_deref() {
                Some("fk_attendance_offenders_offender") => {
                    RepositoryError::ReferencedEntityNotFound("Offender not found".into())
                }
                Some("fk_attendance_offenders_victim") => {
                    RepositoryError::ReferencedEntityNotFound("Victim not found".into())
                }
                Some("fk_attendance_offenders_protective_measure") => {
                    RepositoryError::ReferencedEntityNotFound("Protective measure not found".into())
                }
                Some("fk_attendance_offender_addresses_city") => {
                    RepositoryError::ReferencedEntityNotFound("Address city not found".into())
                }
                _ => base,
            }
        }
        _ => base,
    }
}

#[derive(Clone)]
pub struct PgAttendanceOffenderRepository {
    pool: PgPool,
}

impl PgAttendanceOffenderRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    async fn get_address_by_attendance_id(
        &self,
        attendance_id: Uuid,
    ) -> Result<Option<AttendanceOffenderAddress>, RepositoryError> {
        let address: Option<AttendanceOffenderAddress> = sqlx::query_as::<_, AttendanceOffenderAddressRow>(
            AttendanceOffenderAddressesQueries::GET_ATTENDANCE_OFFENDER_ADDRESS_BY_ATTENDANCE_ID,
        )
        .bind(attendance_id)
        .fetch_optional(&self.pool)
        .await
        .map_err(map_attendance_offender_error)?
        .map(Into::into);
        Ok(address)
    }

    async fn create_address_internal(
        tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
        attendance_id: Uuid,
        address: &AttendanceAddressData,
    ) -> Result<AttendanceOffenderAddress, RepositoryError> {
        let address_id = Uuid::new_v4();

        let created: AttendanceOffenderAddress =
            sqlx::query_as::<_, AttendanceOffenderAddressRow>(AttendanceOffenderAddressesQueries::CREATE_ATTENDANCE_OFFENDER_ADDRESS)
                .bind(address_id)
                .bind(attendance_id)
                .bind(&address.street)
                .bind(&address.number)
                .bind(&address.district)
                .bind(address.city_id)
                .bind(&address.zip_code)
                .bind(&address.complement)
                .fetch_one(&mut **tx)
                .await
                .map_err(map_attendance_offender_error)?
                .into();

        Ok(created)
    }

    async fn check_address_exists(
        tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
        attendance_id: Uuid,
    ) -> Result<bool, RepositoryError> {
        let result: (bool,) = sqlx::query_as(
            AttendanceOffenderAddressesQueries::CHECK_ADDRESS_EXISTS_FOR_ATTENDANCE_OFFENDER,
        )
        .bind(attendance_id)
        .fetch_one(&mut **tx)
        .await
        .map_err(map_attendance_offender_error)?;
        Ok(result.0)
    }

    async fn update_address_internal(
        tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
        attendance_id: Uuid,
        address: &AttendanceAddressData,
    ) -> Result<AttendanceOffenderAddress, RepositoryError> {
        let updated: AttendanceOffenderAddress = sqlx::query_as::<_, AttendanceOffenderAddressRow>(
            AttendanceOffenderAddressesQueries::UPDATE_ATTENDANCE_OFFENDER_ADDRESS_BY_ATTENDANCE_ID,
        )
        .bind(attendance_id)
        .bind(&address.street)
        .bind(&address.number)
        .bind(&address.district)
        .bind(address.city_id)
        .bind(&address.zip_code)
        .bind(&address.complement)
        .fetch_one(&mut **tx)
        .await
        .map_err(map_attendance_offender_error)?
        .into();

        Ok(updated)
    }
}

#[async_trait]
impl AttendanceOffenderReadRepository for PgAttendanceOffenderRepository {
    async fn get_attendance_offender_by_id(
        &self,
        id: Uuid,
    ) -> Result<AttendanceOffenderWithAddress, RepositoryError> {
        info!("[Repository] Fetching attendance offender with id: {}", id);

        let attendance: AttendanceOffender =
            sqlx::query_as::<_, AttendanceOffenderRow>(AttendanceOffendersQueries::GET_ATTENDANCE_OFFENDER_BY_ID)
                .bind(id)
                .fetch_one(&self.pool)
                .await
                .map_err(map_attendance_offender_error)?
                .into();

        let address = self.get_address_by_attendance_id(id).await?;

        info!(
            "[Repository] Attendance offender {} found with address: {}",
            id,
            address.is_some()
        );

        Ok(AttendanceOffenderWithAddress::from_entity(attendance, address))
    }

    async fn get_all_attendance_offenders(
        &self,
    ) -> Result<Vec<AttendanceOffenderWithAddress>, RepositoryError> {
        info!("[Repository] Fetching all attendance offenders");

        let attendances: Vec<AttendanceOffender> =
            sqlx::query_as::<_, AttendanceOffenderRow>(AttendanceOffendersQueries::GET_ALL_ATTENDANCE_OFFENDERS)
                .fetch_all(&self.pool)
                .await
                .map_err(map_attendance_offender_error)?
                .into_iter()
                .map(Into::into)
                .collect();

        let mut result = Vec::with_capacity(attendances.len());

        for attendance in attendances {
            let address = self.get_address_by_attendance_id(attendance.id).await?;
            result.push(AttendanceOffenderWithAddress::from_entity(attendance, address));
        }

        info!("[Repository] Found {} attendance offenders", result.len());

        Ok(result)
    }

    async fn get_attendance_offenders_paginated(
        &self,
        allowed_cities: Option<&[Uuid]>,
        limit: i64,
        offset: i64,
    ) -> Result<Vec<AttendanceOffenderWithAddress>, RepositoryError> {
        info!("[Repository] Fetching attendance offenders paginated");

        let attendances: Vec<AttendanceOffender> = match allowed_cities {
            Some(cities) => {
                sqlx::query_as::<_, AttendanceOffenderRow>(AttendanceOffendersQueries::GET_ATTENDANCE_OFFENDERS_PAGED_BY_CITIES)
                    .bind(cities)
                    .bind(limit)
                    .bind(offset)
                    .fetch_all(&self.pool)
                    .await
                    .map_err(map_attendance_offender_error)?
                    .into_iter()
                    .map(Into::into)
                    .collect()
            }
            None => sqlx::query_as::<_, AttendanceOffenderRow>(AttendanceOffendersQueries::GET_ATTENDANCE_OFFENDERS_PAGED)
                .bind(limit)
                .bind(offset)
                .fetch_all(&self.pool)
                .await
                .map_err(map_attendance_offender_error)?
                .into_iter()
                .map(Into::into)
                .collect(),
        };

        let mut result = Vec::with_capacity(attendances.len());

        for attendance in attendances {
            let address = self.get_address_by_attendance_id(attendance.id).await?;
            result.push(AttendanceOffenderWithAddress::from_entity(attendance, address));
        }

        Ok(result)
    }

    async fn count_attendance_offenders(
        &self,
        allowed_cities: Option<&[Uuid]>,
    ) -> Result<i64, RepositoryError> {
        let total: i64 = match allowed_cities {
            Some(cities) => {
                sqlx::query_scalar(AttendanceOffendersQueries::COUNT_ATTENDANCE_OFFENDERS_BY_CITIES)
                    .bind(cities)
                    .fetch_one(&self.pool)
                    .await
                    .map_err(map_attendance_offender_error)?
            }
            None => sqlx::query_scalar(AttendanceOffendersQueries::COUNT_ATTENDANCE_OFFENDERS)
                .fetch_one(&self.pool)
                .await
                .map_err(map_attendance_offender_error)?,
        };
        Ok(total)
    }

    async fn get_attendance_offenders_by_offender(
        &self,
        offender_id: Uuid,
        protective_measure_id: Option<Uuid>,
    ) -> Result<Vec<AttendanceOffenderWithAddress>, RepositoryError> {
        info!(
            "[Repository] Fetching attendance offenders for offender: {}",
            offender_id
        );

        let attendances: Vec<AttendanceOffender> = match protective_measure_id {
            Some(measure_id) => {
                sqlx::query_as::<_, AttendanceOffenderRow>(AttendanceOffendersQueries::GET_ATTENDANCE_OFFENDERS_BY_OFFENDER_AND_MEASURE)
                    .bind(offender_id)
                    .bind(measure_id)
                    .fetch_all(&self.pool)
                    .await
            }
            None => {
                sqlx::query_as::<_, AttendanceOffenderRow>(AttendanceOffendersQueries::GET_ATTENDANCE_OFFENDERS_BY_OFFENDER)
                    .bind(offender_id)
                    .fetch_all(&self.pool)
                    .await
            }
        }
        .map_err(map_attendance_offender_error)?
        .into_iter()
        .map(Into::into)
        .collect();

        let mut result = Vec::with_capacity(attendances.len());

        for attendance in attendances {
            let address = self.get_address_by_attendance_id(attendance.id).await?;
            result.push(AttendanceOffenderWithAddress::from_entity(attendance, address));
        }

        info!(
            "[Repository] Found {} attendance offenders for offender: {}",
            result.len(),
            offender_id
        );

        Ok(result)
    }

    async fn get_attendance_offenders_by_victim(
        &self,
        victim_id: Uuid,
        protective_measure_id: Option<Uuid>,
    ) -> Result<Vec<AttendanceOffenderWithAddress>, RepositoryError> {
        info!(
            "[Repository] Fetching attendance offenders for victim: {}",
            victim_id
        );

        let attendances: Vec<AttendanceOffender> = match protective_measure_id {
            Some(measure_id) => {
                sqlx::query_as::<_, AttendanceOffenderRow>(AttendanceOffendersQueries::GET_ATTENDANCE_OFFENDERS_BY_VICTIM_AND_MEASURE)
                    .bind(victim_id)
                    .bind(measure_id)
                    .fetch_all(&self.pool)
                    .await
            }
            None => {
                sqlx::query_as::<_, AttendanceOffenderRow>(AttendanceOffendersQueries::GET_ATTENDANCE_OFFENDERS_BY_VICTIM)
                    .bind(victim_id)
                    .fetch_all(&self.pool)
                    .await
            }
        }
        .map_err(map_attendance_offender_error)?
        .into_iter()
        .map(Into::into)
        .collect();

        let mut result = Vec::with_capacity(attendances.len());

        for attendance in attendances {
            let address = self.get_address_by_attendance_id(attendance.id).await?;
            result.push(AttendanceOffenderWithAddress::from_entity(attendance, address));
        }

        info!(
            "[Repository] Found {} attendance offenders for victim: {}",
            result.len(),
            victim_id
        );

        Ok(result)
    }
}

#[async_trait]
impl AttendanceOffenderWriteRepository for PgAttendanceOffenderRepository {
    async fn create_attendance_offender(
        &self,
        attendance: CreateAttendanceOffender,
        session_members: Vec<(Uuid, Option<Uuid>)>,
    ) -> Result<AttendanceOffenderWriteResult, RepositoryError> {
        let attendance_id = Uuid::new_v4();

        info!(
            "[Repository] Starting transaction to create attendance offender for offender: {} with ID: {}",
            attendance.offender_id, attendance_id
        );

        let mut tx = self
            .pool
            .begin()
            .await
            .map_err(map_attendance_offender_error)?;

        let attendance_created: AttendanceOffender =
            sqlx::query_as::<_, AttendanceOffenderRow>(AttendanceOffendersQueries::CREATE_ATTENDANCE_OFFENDER)
                .bind(attendance_id)
                .bind(attendance.offender_id)
                .bind(attendance.victim_id)
                .bind(attendance.protective_measure_id)
                .bind(attendance.was_offender_present)
                .bind(attendance.attendance_date)
                .bind(attendance.attendance_time)
                .bind(attendance.is_remote)
                .bind(attendance.assaults_children)
                .bind(&attendance.violence_aggravator)
                .bind(&attendance.violence_aggravator_other)
                .bind(&attendance.description)
                .fetch_one(&mut *tx)
                .await
                .map_err(map_attendance_offender_error)?
                .into();

        info!("[Repository] Attendance offender inserted, now creating address if provided");

        let address = if let Some(addr_data) = &attendance.address {
            let created_addr =
                Self::create_address_internal(&mut tx, attendance_id, addr_data).await?;
            info!("[Repository] Address created with ID: {}", created_addr.id);
            Some(created_addr)
        } else {
            None
        };

        for (user_id, work_session_id) in &session_members {
            sqlx::query(
                "INSERT INTO attendance_offender_members (id, attendance_offender_id, user_id, work_session_id, created_at)
                 VALUES ($1, $2, $3, $4, NOW())"
            )
            .bind(Uuid::new_v4())
            .bind(attendance_id)
            .bind(user_id)
            .bind(work_session_id)
            .execute(&mut *tx)
            .await
                .map_err(map_attendance_offender_error)?;
        }

        tx.commit()
            .await
            .map_err(map_attendance_offender_error)?;

        info!(
            "[Repository] Transaction committed. Attendance offender {} created successfully",
            attendance_id
        );

        Ok(AttendanceOffenderWriteResult {
            attendance: attendance_created,
            address,
        })
    }

    async fn update_attendance_offender_by_id(
        &self,
        data: UpdateAttendanceOffender,
        id: Uuid,
    ) -> Result<AttendanceOffenderWriteResult, RepositoryError> {
        info!(
            "[Repository] Starting transaction to update attendance offender: {}",
            id
        );

        let mut tx = self
            .pool
            .begin()
            .await
            .map_err(map_attendance_offender_error)?;

        let attendance_updated: AttendanceOffender =
            sqlx::query_as::<_, AttendanceOffenderRow>(AttendanceOffendersQueries::UPDATE_ATTENDANCE_OFFENDER_BY_ID)
                .bind(id)
                .bind(data.offender_id)
                .bind(data.victim_id)
                .bind(data.protective_measure_id)
                .bind(data.was_offender_present)
                .bind(data.attendance_date)
                .bind(data.attendance_time)
                .bind(data.is_remote)
                .bind(data.assaults_children)
                .bind(&data.violence_aggravator)
                .bind(&data.violence_aggravator_other)
                .bind(&data.description)
                .fetch_one(&mut *tx)
                .await
                .map_err(map_attendance_offender_error)?
                .into();

        let address = if let Some(addr_data) = &data.address {
            let has_address = Self::check_address_exists(&mut tx, id).await?;

            let addr = if has_address {
                info!(
                    "[Repository] Updating existing address for attendance offender: {}",
                    id
                );
                Self::update_address_internal(&mut tx, id, addr_data).await?
            } else {
                info!(
                    "[Repository] Creating new address for attendance offender: {}",
                    id
                );
                Self::create_address_internal(&mut tx, id, addr_data).await?
            };

            Some(addr)
        } else {
            None
        };

        tx.commit()
            .await
            .map_err(map_attendance_offender_error)?;

        info!(
            "[Repository] Transaction committed. Attendance offender {} updated",
            id
        );

        let final_address = if address.is_some() {
            address
        } else {
            self.get_address_by_attendance_id(id).await?
        };

        Ok(AttendanceOffenderWriteResult {
            attendance: attendance_updated,
            address: final_address,
        })
    }

    async fn delete_attendance_offender_by_id(
        &self,
        id: Uuid,
    ) -> Result<AttendanceOffenderWriteResult, RepositoryError> {
        info!(
            "[Repository] Starting transaction to soft delete attendance offender: {}",
            id
        );

        let address = self.get_address_by_attendance_id(id).await?;

        let mut tx = self
            .pool
            .begin()
            .await
            .map_err(map_attendance_offender_error)?;

        let _: Option<AttendanceOffenderAddress> = sqlx::query_as::<_, AttendanceOffenderAddressRow>(
            AttendanceOffenderAddressesQueries::DELETE_ATTENDANCE_OFFENDER_ADDRESS_BY_ATTENDANCE_ID,
        )
        .bind(id)
        .fetch_optional(&mut *tx)
        .await
        .map_err(map_attendance_offender_error)?
        .map(Into::into);

        let deleted_attendance: AttendanceOffender =
            sqlx::query_as::<_, AttendanceOffenderRow>(AttendanceOffendersQueries::DELETE_ATTENDANCE_OFFENDER_BY_ID)
                .bind(id)
                .fetch_one(&mut *tx)
                .await
                .map_err(map_attendance_offender_error)?
                .into();

        tx.commit()
            .await
            .map_err(map_attendance_offender_error)?;

        info!(
            "[Repository] Transaction committed. Attendance offender {} soft deleted",
            id
        );

        Ok(AttendanceOffenderWriteResult {
            attendance: deleted_attendance,
            address,
        })
    }
}
