use async_trait::async_trait;
use log::info;
use sqlx::PgPool;
use uuid::Uuid;

use crate::core::commands::attendance_victims::{
    AttendanceAddressData, CreateAttendanceVictim, UpdateAttendanceVictim,
};
use crate::core::contracts::repository::attendance_victims::{
    AttendanceVictimReadRepository, AttendanceVictimWriteRepository,
};
use crate::core::contracts::repository::error::RepositoryError;
use crate::core::entities::attendance_victims::{
    AttendanceVictim, AttendanceVictimAddress, AttendanceVictimWriteResult,
};
use crate::core::read_models::attendance_victims::AttendanceVictimWithAddress;
use crate::repositories::queries::attendance_victims::{
    AttendanceVictimAddressesQueries, AttendanceVictimsQueries,
};

use super::models::attendance_victims::{AttendanceVictimAddressRow, AttendanceVictimRow};

use crate::repositories::error_mapper::map_sqlx_error;
fn map_attendance_victim_error(err: sqlx::Error) -> RepositoryError {
    let base = map_sqlx_error(err);
    match base {
        RepositoryError::ForeignKeyViolation { ref constraint } => match constraint.as_deref() {
            Some("fk_attendances_victim") => {
                RepositoryError::ReferencedEntityNotFound("Victim not found".into())
            }
            Some("fk_attendance_victims_offender") => {
                RepositoryError::ReferencedEntityNotFound("Offender not found".into())
            }
            Some("fk_attendance_victims_protective_measure") => {
                RepositoryError::ReferencedEntityNotFound("Protective measure not found".into())
            }
            Some("fk_attendance_addresses_city") | Some("fk_attendance_victim_addresses_city") => {
                RepositoryError::ReferencedEntityNotFound("Address city not found".into())
            }
            _ => base,
        },
        _ => base,
    }
}

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
    ) -> Result<Option<AttendanceVictimAddress>, RepositoryError> {
        let address: Option<AttendanceVictimAddress> =
            sqlx::query_as::<_, AttendanceVictimAddressRow>(
                AttendanceVictimAddressesQueries::GET_ATTENDANCE_VICTIM_ADDRESS_BY_ATTENDANCE_ID,
            )
            .bind(attendance_id)
            .fetch_optional(&self.pool)
            .await
            .map_err(map_attendance_victim_error)?
            .map(Into::into);
        Ok(address)
    }

    async fn create_address_internal(
        tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
        attendance_id: Uuid,
        address: &AttendanceAddressData,
    ) -> Result<AttendanceVictimAddress, RepositoryError> {
        let address_id = Uuid::new_v4();

        let created: AttendanceVictimAddress = sqlx::query_as::<_, AttendanceVictimAddressRow>(
            AttendanceVictimAddressesQueries::CREATE_ATTENDANCE_VICTIM_ADDRESS,
        )
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
        .map_err(map_attendance_victim_error)?
        .into();

        Ok(created)
    }

    async fn check_address_exists(
        tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
        attendance_id: Uuid,
    ) -> Result<bool, RepositoryError> {
        let result: (bool,) = sqlx::query_as(
            AttendanceVictimAddressesQueries::CHECK_ADDRESS_EXISTS_FOR_ATTENDANCE_VICTIM,
        )
        .bind(attendance_id)
        .fetch_one(&mut **tx)
        .await
        .map_err(map_attendance_victim_error)?;
        Ok(result.0)
    }

    async fn update_address_internal(
        tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
        attendance_id: Uuid,
        address: &AttendanceAddressData,
    ) -> Result<AttendanceVictimAddress, RepositoryError> {
        let updated: AttendanceVictimAddress = sqlx::query_as::<_, AttendanceVictimAddressRow>(
            AttendanceVictimAddressesQueries::UPDATE_ATTENDANCE_VICTIM_ADDRESS_BY_ATTENDANCE_ID,
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
        .map_err(map_attendance_victim_error)?
        .into();

        Ok(updated)
    }
}

#[async_trait]
impl AttendanceVictimReadRepository for PgAttendanceVictimRepository {
    async fn get_attendance_victim_by_id(
        &self,
        id: Uuid,
    ) -> Result<AttendanceVictimWithAddress, RepositoryError> {
        info!("[Repository] Fetching attendance victim with id: {}", id);

        let attendance: AttendanceVictim = sqlx::query_as::<_, AttendanceVictimRow>(
            AttendanceVictimsQueries::GET_ATTENDANCE_VICTIM_BY_ID,
        )
        .bind(id)
        .fetch_one(&self.pool)
        .await
        .map_err(map_attendance_victim_error)?
        .into();

        let address = self.get_address_by_attendance_id(id).await?;

        info!(
            "[Repository] Attendance victim {} found with address: {}",
            id,
            address.is_some()
        );

        Ok(AttendanceVictimWithAddress::from_entity(
            attendance, address,
        ))
    }

    async fn get_all_attendance_victims(
        &self,
    ) -> Result<Vec<AttendanceVictimWithAddress>, RepositoryError> {
        info!("[Repository] Fetching all attendance victims");

        let attendances: Vec<AttendanceVictim> = sqlx::query_as::<_, AttendanceVictimRow>(
            AttendanceVictimsQueries::GET_ALL_ATTENDANCE_VICTIMS,
        )
        .fetch_all(&self.pool)
        .await
        .map_err(map_attendance_victim_error)?
        .into_iter()
        .map(Into::into)
        .collect();

        let mut result = Vec::with_capacity(attendances.len());

        for attendance in attendances {
            let address = self.get_address_by_attendance_id(attendance.id).await?;
            result.push(AttendanceVictimWithAddress::from_entity(
                attendance, address,
            ));
        }

        info!("[Repository] Found {} attendance victims", result.len());

        Ok(result)
    }

    async fn get_attendance_victims_paginated(
        &self,
        allowed_cities: Option<&[Uuid]>,
        limit: i64,
        offset: i64,
    ) -> Result<Vec<AttendanceVictimWithAddress>, RepositoryError> {
        info!("[Repository] Fetching attendance victims paginated");

        let attendances: Vec<AttendanceVictim> = match allowed_cities {
            Some(cities) => sqlx::query_as::<_, AttendanceVictimRow>(
                AttendanceVictimsQueries::GET_ATTENDANCE_VICTIMS_PAGED_BY_CITIES,
            )
            .bind(cities)
            .bind(limit)
            .bind(offset)
            .fetch_all(&self.pool)
            .await
            .map_err(map_attendance_victim_error)?
            .into_iter()
            .map(Into::into)
            .collect(),
            None => sqlx::query_as::<_, AttendanceVictimRow>(
                AttendanceVictimsQueries::GET_ATTENDANCE_VICTIMS_PAGED,
            )
            .bind(limit)
            .bind(offset)
            .fetch_all(&self.pool)
            .await
            .map_err(map_attendance_victim_error)?
            .into_iter()
            .map(Into::into)
            .collect(),
        };

        let mut result = Vec::with_capacity(attendances.len());

        for attendance in attendances {
            let address = self.get_address_by_attendance_id(attendance.id).await?;
            result.push(AttendanceVictimWithAddress::from_entity(
                attendance, address,
            ));
        }

        Ok(result)
    }

    async fn count_attendance_victims(
        &self,
        allowed_cities: Option<&[Uuid]>,
    ) -> Result<i64, RepositoryError> {
        let total: i64 = match allowed_cities {
            Some(cities) => {
                sqlx::query_scalar(AttendanceVictimsQueries::COUNT_ATTENDANCE_VICTIMS_BY_CITIES)
                    .bind(cities)
                    .fetch_one(&self.pool)
                    .await
                    .map_err(map_attendance_victim_error)?
            }
            None => sqlx::query_scalar(AttendanceVictimsQueries::COUNT_ATTENDANCE_VICTIMS)
                .fetch_one(&self.pool)
                .await
                .map_err(map_attendance_victim_error)?,
        };
        Ok(total)
    }

    async fn get_attendance_victims_by_victim(
        &self,
        victim_id: Uuid,
        protective_measure_id: Option<Uuid>,
    ) -> Result<Vec<AttendanceVictimWithAddress>, RepositoryError> {
        info!(
            "[Repository] Fetching attendance victims for victim: {}",
            victim_id
        );

        let attendances: Vec<AttendanceVictim> = match protective_measure_id {
            Some(measure_id) => {
                sqlx::query_as::<_, AttendanceVictimRow>(
                    AttendanceVictimsQueries::GET_ATTENDANCE_VICTIMS_BY_VICTIM_AND_MEASURE,
                )
                .bind(victim_id)
                .bind(measure_id)
                .fetch_all(&self.pool)
                .await
            }
            None => {
                sqlx::query_as::<_, AttendanceVictimRow>(
                    AttendanceVictimsQueries::GET_ATTENDANCE_VICTIMS_BY_VICTIM,
                )
                .bind(victim_id)
                .fetch_all(&self.pool)
                .await
            }
        }
        .map_err(map_attendance_victim_error)?
        .into_iter()
        .map(Into::into)
        .collect();

        let mut result = Vec::with_capacity(attendances.len());

        for attendance in attendances {
            let address = self.get_address_by_attendance_id(attendance.id).await?;
            result.push(AttendanceVictimWithAddress::from_entity(
                attendance, address,
            ));
        }

        info!(
            "[Repository] Found {} attendance victims for victim: {}",
            result.len(),
            victim_id
        );

        Ok(result)
    }
}

#[async_trait]
impl AttendanceVictimWriteRepository for PgAttendanceVictimRepository {
    async fn create_attendance_victim(
        &self,
        attendance: CreateAttendanceVictim,
        victim_id: Uuid,
        offender_id: Option<Uuid>,
        session_members: Vec<(Uuid, Option<Uuid>)>,
    ) -> Result<AttendanceVictimWriteResult, RepositoryError> {
        let attendance_id = Uuid::new_v4();

        info!(
            "[Repository] Starting transaction to create attendance victim for victim: {} with ID: {}",
            victim_id, attendance_id
        );

        let mut tx = self
            .pool
            .begin()
            .await
            .map_err(map_attendance_victim_error)?;

        let attendance_created: AttendanceVictim = sqlx::query_as::<_, AttendanceVictimRow>(
            AttendanceVictimsQueries::CREATE_ATTENDANCE_VICTIM,
        )
        .bind(attendance_id)
        .bind(victim_id)
        .bind(attendance.was_victim_present)
        .bind(attendance.attendance_date)
        .bind(attendance.attendance_time)
        .bind(&attendance.description)
        .bind(attendance.latitude)
        .bind(attendance.longitude)
        .bind(offender_id)
        .bind(attendance.protective_measure_id)
        .bind(attendance.is_remote)
        .bind(&attendance.risk_level)
        .bind(&attendance.offender_freedom_status)
        .bind(&attendance.offender_has_firearm_access)
        .bind(attendance.needs_legal_assistance)
        .bind(attendance.needs_psychological_support)
        .bind(attendance.was_instructed_about_protective_measure_procedures)
        .bind(attendance.offender_violated_protective_measure)
        .fetch_one(&mut *tx)
        .await
        .map_err(map_attendance_victim_error)?
        .into();

        info!("[Repository] Attendance victim inserted, now creating address if provided");

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
                "INSERT INTO attendance_victim_members (id, attendance_victim_id, user_id, work_session_id, created_at)
                 VALUES ($1, $2, $3, $4, NOW())"
            )
            .bind(Uuid::new_v4())
            .bind(attendance_id)
            .bind(user_id)
            .bind(work_session_id)
            .execute(&mut *tx)
            .await
                .map_err(map_attendance_victim_error)?;
        }

        tx.commit().await.map_err(map_attendance_victim_error)?;

        info!(
            "[Repository] Transaction committed. Attendance victim {} created successfully",
            attendance_id
        );

        Ok(AttendanceVictimWriteResult {
            attendance: attendance_created,
            address,
        })
    }

    async fn update_attendance_victim_by_id(
        &self,
        data: UpdateAttendanceVictim,
        id: Uuid,
        victim_id: Uuid,
        offender_id: Option<Uuid>,
    ) -> Result<AttendanceVictimWriteResult, RepositoryError> {
        info!(
            "[Repository] Starting transaction to update attendance victim: {}",
            id
        );

        let mut tx = self
            .pool
            .begin()
            .await
            .map_err(map_attendance_victim_error)?;

        let attendance_updated: AttendanceVictim = sqlx::query_as::<_, AttendanceVictimRow>(
            AttendanceVictimsQueries::UPDATE_ATTENDANCE_VICTIM_BY_ID,
        )
        .bind(id)
        .bind(victim_id)
        .bind(data.was_victim_present)
        .bind(data.attendance_date)
        .bind(data.attendance_time)
        .bind(&data.description)
        .bind(data.latitude)
        .bind(data.longitude)
        .bind(offender_id)
        .bind(data.protective_measure_id)
        .bind(data.is_remote)
        .bind(&data.risk_level)
        .bind(&data.offender_freedom_status)
        .bind(&data.offender_has_firearm_access)
        .bind(data.needs_legal_assistance)
        .bind(data.needs_psychological_support)
        .bind(data.was_instructed_about_protective_measure_procedures)
        .bind(data.offender_violated_protective_measure)
        .fetch_one(&mut *tx)
        .await
        .map_err(map_attendance_victim_error)?
        .into();

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

        tx.commit().await.map_err(map_attendance_victim_error)?;

        info!(
            "[Repository] Transaction committed. Attendance victim {} updated",
            id
        );

        let final_address = if address.is_some() {
            address
        } else {
            self.get_address_by_attendance_id(id).await?
        };

        Ok(AttendanceVictimWriteResult {
            attendance: attendance_updated,
            address: final_address,
        })
    }

    async fn delete_attendance_victim_by_id(
        &self,
        id: Uuid,
    ) -> Result<AttendanceVictimWriteResult, RepositoryError> {
        info!(
            "[Repository] Starting transaction to soft delete attendance victim: {}",
            id
        );

        let address = self.get_address_by_attendance_id(id).await?;

        let mut tx = self
            .pool
            .begin()
            .await
            .map_err(map_attendance_victim_error)?;

        let _: Option<AttendanceVictimAddress> = sqlx::query_as::<_, AttendanceVictimAddressRow>(
            AttendanceVictimAddressesQueries::DELETE_ATTENDANCE_VICTIM_ADDRESS_BY_ATTENDANCE_ID,
        )
        .bind(id)
        .fetch_optional(&mut *tx)
        .await
        .map_err(map_attendance_victim_error)?
        .map(Into::into);

        let deleted_attendance: AttendanceVictim = sqlx::query_as::<_, AttendanceVictimRow>(
            AttendanceVictimsQueries::DELETE_ATTENDANCE_VICTIM_BY_ID,
        )
        .bind(id)
        .fetch_one(&mut *tx)
        .await
        .map_err(map_attendance_victim_error)?
        .into();

        tx.commit().await.map_err(map_attendance_victim_error)?;

        info!(
            "[Repository] Transaction committed. Attendance victim {} soft deleted",
            id
        );

        Ok(AttendanceVictimWriteResult {
            attendance: deleted_attendance,
            address,
        })
    }
}
