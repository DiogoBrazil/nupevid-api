use log::{error, info};
use std::sync::Arc;
use uuid::Uuid;

use crate::core::commands::attendance_offenders::{
    CreateAttendanceOffender, UpdateAttendanceOffender,
};
use crate::core::contracts::repository::attendance_members::AttendanceMemberRepository;
use crate::core::contracts::repository::attendance_offenders::{
    AttendanceOffenderReadRepository, AttendanceOffenderWriteRepository,
};
use crate::core::contracts::repository::offenders::OffenderReadRepository;
use crate::core::contracts::repository::protective_measures::ProtectiveMeasureReadRepository;
use crate::core::contracts::repository::users::UserRepository;
use crate::core::contracts::repository::victims::VictimReadRepository;
use crate::core::contracts::repository::work_sessions::WorkSessionReadRepository;
use crate::core::entities::attendance_members::AddAttendanceMember;
use crate::core::entities::auth::ClaimsToUserToken;
use crate::core::entities::common::PaginatedResult;
use crate::utils::errors::AppError;
use crate::core::contracts::repository::error::RepositoryError;
use crate::core::read_models::attendance_members::AttendanceMemberWithDetails;
use crate::core::read_models::attendance_offenders::AttendanceOffenderWithAddress;
use crate::core::read_models::offenders::OffenderWithDetails;
use crate::core::read_models::victims::VictimWithDetails;

use crate::services::auth_context::AuthContext;
use crate::services::error_mapping::{map_constraint, map_unique_constraint};
use crate::utils::pagination::Pagination;
use crate::validators::common::{
    POLICY_CREATE_ATTENDANCES, POLICY_DELETE_ATTENDANCES, POLICY_MANAGE_ATTENDANCE_MEMBERS,
    POLICY_READ_ATTENDANCES, POLICY_UPDATE_ATTENDANCES,
};

pub struct AttendanceOffenderService {
    attendance_offender_read_repository: Arc<dyn AttendanceOffenderReadRepository>,
    attendance_offender_write_repository: Arc<dyn AttendanceOffenderWriteRepository>,
    offender_repository: Arc<dyn OffenderReadRepository>,
    victim_repository: Arc<dyn VictimReadRepository>,
    protective_measure_repository: Arc<dyn ProtectiveMeasureReadRepository>,
    user_repository: Arc<dyn UserRepository>,
    work_session_repository: Arc<dyn WorkSessionReadRepository>,
    attendance_member_repository: Arc<dyn AttendanceMemberRepository>,
}

pub struct AttendanceOffenderServiceDeps {
    pub attendance_offender_read_repository: Arc<dyn AttendanceOffenderReadRepository>,
    pub attendance_offender_write_repository: Arc<dyn AttendanceOffenderWriteRepository>,
    pub offender_repository: Arc<dyn OffenderReadRepository>,
    pub victim_repository: Arc<dyn VictimReadRepository>,
    pub protective_measure_repository: Arc<dyn ProtectiveMeasureReadRepository>,
    pub user_repository: Arc<dyn UserRepository>,
    pub work_session_repository: Arc<dyn WorkSessionReadRepository>,
    pub attendance_member_repository: Arc<dyn AttendanceMemberRepository>,
}

impl AttendanceOffenderService {
    pub fn new(deps: AttendanceOffenderServiceDeps) -> Self {
        Self {
            attendance_offender_read_repository: deps.attendance_offender_read_repository,
            attendance_offender_write_repository: deps.attendance_offender_write_repository,
            offender_repository: deps.offender_repository,
            victim_repository: deps.victim_repository,
            protective_measure_repository: deps.protective_measure_repository,
            user_repository: deps.user_repository,
            work_session_repository: deps.work_session_repository,
            attendance_member_repository: deps.attendance_member_repository,
        }
    }

    pub async fn create_attendance_offender(
        &self,
        attendance: CreateAttendanceOffender,
        claims: &ClaimsToUserToken,
    ) -> Result<AttendanceOffenderWithAddress, AppError> {
        let user_id = Uuid::parse_str(&claims.id)
            .map_err(|_| AppError::Unauthorized("Invalid user id in token".to_string()))?;

        let offender = self
            .verify_offender_access(claims, attendance.offender_id)
            .await?;

        let _victim = self
            .verify_victim_access(claims, attendance.victim_id)
            .await?;

        if let Some(pm_id) = attendance.protective_measure_id {
            match self
                .protective_measure_repository
                .get_protective_measure_by_id(pm_id)
                .await
            {
                Ok(_) => {}
                Err(RepositoryError::NotFound) => {
                    return Err(AppError::NotFound(format!(
                        "Protective measure with id '{}' not found",
                        pm_id
                    )));
                }
                Err(e) => {
                    error!(
                        "[AttendanceOffenderService] Error checking protective measure: {:?}",
                        e
                    );
                    return Err(AppError::InternalServerError);
                }
            }
        }

        let auth = AuthContext::load(&*self.user_repository, claims).await?;
        auth.check_policy(POLICY_CREATE_ATTENDANCES, offender.city_id)?;

        let active_session = self.work_session_repository
            .get_active_session_by_user(user_id)
            .await
            .map_err(|_| AppError::BadRequest("No active work session found. You must have an active work session to create an attendance.".to_string()))?;

        let session_members = self
            .work_session_repository
            .get_session_members(active_session.id)
            .await
            .map_err(|_| AppError::InternalServerError)?;

        let members_for_tx: Vec<(Uuid, Option<Uuid>)> = session_members
            .iter()
            .map(|m| (m.user_id, Some(active_session.id)))
            .collect();

        match self
            .attendance_offender_write_repository
            .create_attendance_offender(attendance, members_for_tx)
            .await
        {
            Ok(attendance_with_address) => {
                let attendance_with_address = attendance_with_address.into_with_address();
                info!(
                    "[AttendanceOffenderService] Attendance offender created: {}",
                    attendance_with_address.id
                );
                Ok(attendance_with_address)
            }
            Err(e) => {
                if let RepositoryError::UniqueViolation { constraint }
                | RepositoryError::ForeignKeyViolation { constraint } = &e
                    && let Some(app_err) = map_constraint(
                        constraint.as_deref(),
                        &[
                            (
                                "fk_attendance_offenders_offender",
                                "Error adding attendance: offender_id not found",
                            ),
                            (
                                "fk_attendance_offenders_victim",
                                "Error adding attendance: victim_id not found",
                            ),
                            (
                                "fk_attendance_offenders_protective_measure",
                                "Error adding attendance: protective_measure_id not found",
                            ),
                            (
                                "fk_attendance_offender_addresses_city",
                                "Error adding attendance: address city_id not found",
                            ),
                        ],
                    )
                {
                    return Err(app_err);
                }
                error!(
                    "[AttendanceOffenderService] Failed to create attendance offender: {:?}",
                    e
                );
                Err(AppError::InternalServerError)
            }
        }
    }

    pub async fn get_attendance_offender_by_id(
        &self,
        id: Uuid,
        claims: &ClaimsToUserToken,
    ) -> Result<AttendanceOffenderWithAddress, AppError> {
        match self
            .attendance_offender_read_repository
            .get_attendance_offender_by_id(id)
            .await
        {
            Ok(attendance_with_address) => {
                let offender = self
                    .offender_repository
                    .get_offender_by_id(attendance_with_address.offender_id)
                    .await
                    .map_err(|e| match e {
                        RepositoryError::NotFound => AppError::NotFound(format!(
                            "Offender with id '{}' not found",
                            attendance_with_address.offender_id
                        )),
                        _ => AppError::InternalServerError,
                    })?;

                let auth = AuthContext::load(&*self.user_repository, claims).await?;
                auth.check_policy(POLICY_READ_ATTENDANCES, offender.city_id)?;

                Ok(attendance_with_address)
            }
            Err(RepositoryError::NotFound) => Err(AppError::NotFound(format!(
                "Attendance offender '{}' not found",
                id
            ))),
            Err(_) => Err(AppError::InternalServerError),
        }
    }

    pub async fn get_all_attendance_offenders(
        &self,
        pagination: Pagination,
        claims: &ClaimsToUserToken,
    ) -> Result<PaginatedResult<AttendanceOffenderWithAddress>, AppError> {
        let auth = AuthContext::load(&*self.user_repository, claims).await?;
        let allowed_cities = auth.allowed_cities(POLICY_READ_ATTENDANCES);

        let total_items = self
            .attendance_offender_read_repository
            .count_attendance_offenders(allowed_cities.as_deref())
            .await
            .map_err(|_| AppError::InternalServerError)?;

        let attendances_list = self
            .attendance_offender_read_repository
            .get_attendance_offenders_paginated(
                allowed_cities.as_deref(),
                pagination.page_size,
                pagination.offset,
            )
            .await
            .map_err(|_| AppError::InternalServerError)?;

        Ok(PaginatedResult {
            items: attendances_list,
            page: pagination.page,
            page_size: pagination.page_size,
            total_items,
        })
    }

    pub async fn get_attendance_offenders_by_offender(
        &self,
        offender_id: Uuid,
        claims: &ClaimsToUserToken,
    ) -> Result<Vec<AttendanceOffenderWithAddress>, AppError> {
        let offender = self.verify_offender_access(claims, offender_id).await?;

        let auth = AuthContext::load(&*self.user_repository, claims).await?;
        auth.check_policy(POLICY_READ_ATTENDANCES, offender.city_id)?;

        match self
            .attendance_offender_read_repository
            .get_attendance_offenders_by_offender(offender_id)
            .await
        {
            Ok(attendances_list) => Ok(attendances_list),
            Err(_) => Err(AppError::InternalServerError),
        }
    }

    pub async fn get_attendance_offenders_by_victim(
        &self,
        victim_id: Uuid,
        claims: &ClaimsToUserToken,
    ) -> Result<Vec<AttendanceOffenderWithAddress>, AppError> {
        let victim = self.verify_victim_access(claims, victim_id).await?;

        let auth = AuthContext::load(&*self.user_repository, claims).await?;
        auth.check_policy(POLICY_READ_ATTENDANCES, victim.city_id)?;

        match self
            .attendance_offender_read_repository
            .get_attendance_offenders_by_victim(victim_id)
            .await
        {
            Ok(attendances_list) => Ok(attendances_list),
            Err(_) => Err(AppError::InternalServerError),
        }
    }

    pub async fn update_attendance_offender_by_id(
        &self,
        data: UpdateAttendanceOffender,
        id: Uuid,
        claims: &ClaimsToUserToken,
    ) -> Result<AttendanceOffenderWithAddress, AppError> {
        let existing = self
            .attendance_offender_read_repository
            .get_attendance_offender_by_id(id)
            .await
            .map_err(|e| {
                if matches!(e, RepositoryError::NotFound) {
                    AppError::NotFound(format!("Attendance offender '{}' not found", id))
                } else {
                    AppError::InternalServerError
                }
            })?;

        let existing_offender = self
            .offender_repository
            .get_offender_by_id(existing.offender_id)
            .await
            .map_err(|e| match e {
                RepositoryError::NotFound => AppError::NotFound(format!(
                    "Offender with id '{}' not found",
                    existing.offender_id
                )),
                _ => AppError::InternalServerError,
            })?;

        let auth = AuthContext::load(&*self.user_repository, claims).await?;
        auth.check_policy(POLICY_UPDATE_ATTENDANCES, existing_offender.city_id)?;

        if data.offender_id != existing.offender_id {
            let new_offender = self
                .verify_offender_access(claims, data.offender_id)
                .await?;
            auth.check_policy(POLICY_UPDATE_ATTENDANCES, new_offender.city_id)?;
        }

        if data.victim_id != existing.victim_id {
            let victim = self.verify_victim_access(claims, data.victim_id).await?;
            auth.check_policy(POLICY_UPDATE_ATTENDANCES, victim.city_id)?;
        }

        if data.protective_measure_id != existing.protective_measure_id
            && let Some(pm_id) = data.protective_measure_id
        {
            match self
                .protective_measure_repository
                .get_protective_measure_by_id(pm_id)
                .await
            {
                Ok(_) => {}
                Err(RepositoryError::NotFound) => {
                    return Err(AppError::NotFound(format!(
                        "Protective measure with id '{}' not found",
                        pm_id
                    )));
                }
                Err(e) => {
                    error!(
                        "[AttendanceOffenderService] Error checking protective measure: {:?}",
                        e
                    );
                    return Err(AppError::InternalServerError);
                }
            }
        }

        match self
            .attendance_offender_write_repository
            .update_attendance_offender_by_id(data, id)
            .await
        {
            Ok(attendance_with_address) => Ok(attendance_with_address.into_with_address()),
            Err(RepositoryError::NotFound) => Err(AppError::NotFound(format!(
                "Attendance offender '{}' not found",
                id
            ))),
            Err(e) => {
                if let RepositoryError::UniqueViolation { constraint }
                | RepositoryError::ForeignKeyViolation { constraint } = &e
                    && let Some(app_err) = map_constraint(
                        constraint.as_deref(),
                        &[
                            (
                                "fk_attendance_offenders_offender",
                                "Error updating attendance: offender_id not found",
                            ),
                            (
                                "fk_attendance_offenders_victim",
                                "Error updating attendance: victim_id not found",
                            ),
                            (
                                "fk_attendance_offenders_protective_measure",
                                "Error updating attendance: protective_measure_id not found",
                            ),
                            (
                                "fk_attendance_offender_addresses_city",
                                "Error updating attendance: address city_id not found",
                            ),
                        ],
                    )
                {
                    return Err(app_err);
                }
                Err(AppError::InternalServerError)
            }
        }
    }

    pub async fn delete_attendance_offender_by_id(
        &self,
        id: Uuid,
        claims: &ClaimsToUserToken,
    ) -> Result<AttendanceOffenderWithAddress, AppError> {
        let attendance = self
            .attendance_offender_read_repository
            .get_attendance_offender_by_id(id)
            .await
            .map_err(|e| {
                if matches!(e, RepositoryError::NotFound) {
                    AppError::NotFound(format!("Attendance offender '{}' not found", id))
                } else {
                    AppError::InternalServerError
                }
            })?;

        let offender = self
            .offender_repository
            .get_offender_by_id(attendance.offender_id)
            .await
            .map_err(|e| match e {
                RepositoryError::NotFound => AppError::NotFound(format!(
                    "Offender with id '{}' not found",
                    attendance.offender_id
                )),
                _ => AppError::InternalServerError,
            })?;

        let auth = AuthContext::load(&*self.user_repository, claims).await?;
        auth.check_policy(POLICY_DELETE_ATTENDANCES, offender.city_id)?;

        match self
            .attendance_offender_write_repository
            .delete_attendance_offender_by_id(id)
            .await
        {
            Ok(deleted) => Ok(deleted.into_with_address()),
            Err(_) => Err(AppError::InternalServerError),
        }
    }

    pub async fn get_attendance_members(
        &self,
        attendance_id: Uuid,
        claims: &ClaimsToUserToken,
    ) -> Result<Vec<AttendanceMemberWithDetails>, AppError> {
        info!(
            "[AttendanceOffenderService] Getting members for attendance: {}",
            attendance_id
        );

        let auth = AuthContext::load(&*self.user_repository, claims).await?;

        let attendance = self
            .attendance_offender_read_repository
            .get_attendance_offender_by_id(attendance_id)
            .await
            .map_err(|e| {
                if matches!(e, RepositoryError::NotFound) {
                    AppError::NotFound(format!("Attendance offender '{}' not found", attendance_id))
                } else {
                    AppError::InternalServerError
                }
            })?;

        let offender = self
            .verify_offender_access(claims, attendance.offender_id)
            .await?;
        auth.check_policy(POLICY_READ_ATTENDANCES, offender.city_id)?;

        let members = self
            .attendance_member_repository
            .get_offender_attendance_members(attendance_id)
            .await
            .map_err(|_| AppError::InternalServerError)?;

        info!(
            "[AttendanceOffenderService] Found {} members",
            members.len()
        );
        Ok(members)
    }

    pub async fn add_attendance_member(
        &self,
        attendance_id: Uuid,
        data: AddAttendanceMember,
        claims: &ClaimsToUserToken,
    ) -> Result<String, AppError> {
        info!(
            "[AttendanceOffenderService] Adding member to attendance: {}",
            attendance_id
        );

        let auth = AuthContext::load(&*self.user_repository, claims).await?;

        let attendance = self
            .attendance_offender_read_repository
            .get_attendance_offender_by_id(attendance_id)
            .await
            .map_err(|e| {
                if matches!(e, RepositoryError::NotFound) {
                    AppError::NotFound(format!("Attendance offender '{}' not found", attendance_id))
                } else {
                    AppError::InternalServerError
                }
            })?;

        let offender = self
            .verify_offender_access(claims, attendance.offender_id)
            .await?;
        auth.check_policy(POLICY_MANAGE_ATTENDANCE_MEMBERS, offender.city_id)?;

        let user_to_add = self
            .user_repository
            .get_user_by_id(data.user_id)
            .await
            .map_err(|e| {
                if matches!(e, RepositoryError::NotFound) {
                    AppError::NotFound(format!("User '{}' not found", data.user_id))
                } else {
                    AppError::InternalServerError
                }
            })?;

        if user_to_add.city_id != Some(offender.city_id) {
            return Err(AppError::BadRequest(
                "User must be from the same city as the offender".to_string(),
            ));
        }

        match self
            .attendance_member_repository
            .add_member_to_offender_attendance(attendance_id, data.user_id, None)
            .await
        {
            Ok(_) => {
                info!("[AttendanceOffenderService] Member added successfully");
                Ok("Member added successfully".to_string())
            }
            Err(e) => {
                if let RepositoryError::UniqueViolation { constraint } = &e
                    && let Some(app_err) = map_unique_constraint(
                        constraint.as_deref(),
                        &[(
                            "attendance_offender_members_attendance_offender_id_user_id_key",
                            "Member already added to attendance",
                        )],
                    )
                {
                    return Err(app_err);
                }
                error!("[AttendanceOffenderService] Failed to add member: {:?}", e);
                Err(AppError::InternalServerError)
            }
        }
    }

    pub async fn remove_attendance_member(
        &self,
        attendance_id: Uuid,
        user_id: Uuid,
        claims: &ClaimsToUserToken,
    ) -> Result<String, AppError> {
        info!(
            "[AttendanceOffenderService] Removing member from attendance: {}",
            attendance_id
        );

        let auth = AuthContext::load(&*self.user_repository, claims).await?;

        let attendance = self
            .attendance_offender_read_repository
            .get_attendance_offender_by_id(attendance_id)
            .await
            .map_err(|e| {
                if matches!(e, RepositoryError::NotFound) {
                    AppError::NotFound(format!("Attendance offender '{}' not found", attendance_id))
                } else {
                    AppError::InternalServerError
                }
            })?;

        let offender = self
            .verify_offender_access(claims, attendance.offender_id)
            .await?;
        auth.check_policy(POLICY_MANAGE_ATTENDANCE_MEMBERS, offender.city_id)?;

        match self
            .attendance_member_repository
            .remove_member_from_offender_attendance(attendance_id, user_id)
            .await
        {
            Ok(_) => {
                info!("[AttendanceOffenderService] Member removed successfully");
                Ok("Member removed successfully".to_string())
            }
            Err(e) => {
                error!(
                    "[AttendanceOffenderService] Failed to remove member: {:?}",
                    e
                );
                Err(AppError::InternalServerError)
            }
        }
    }

    async fn verify_offender_access(
        &self,
        _claims: &ClaimsToUserToken,
        offender_id: Uuid,
    ) -> Result<OffenderWithDetails, AppError> {
        self.offender_repository
            .get_offender_by_id(offender_id)
            .await
            .map_err(|e| {
                if matches!(e, RepositoryError::NotFound) {
                    AppError::NotFound(format!("Offender '{}' not found", offender_id))
                } else {
                    AppError::InternalServerError
                }
            })
    }

    async fn verify_victim_access(
        &self,
        _claims: &ClaimsToUserToken,
        victim_id: Uuid,
    ) -> Result<VictimWithDetails, AppError> {
        self.victim_repository
            .get_victim_by_id(victim_id)
            .await
            .map_err(|e| {
                if matches!(e, RepositoryError::NotFound) {
                    AppError::NotFound(format!("Victim '{}' not found", victim_id))
                } else {
                    AppError::InternalServerError
                }
            })
    }
}
