use log::{error, info};
use std::sync::Arc;
use uuid::Uuid;

use crate::core::commands::attendance_victims::{CreateAttendanceVictim, UpdateAttendanceVictim};
use crate::core::contracts::repository::attendance_members::AttendanceMemberRepository;
use crate::core::contracts::repository::attendance_victims::{
    AttendanceVictimReadRepository, AttendanceVictimWriteRepository,
};
use crate::core::contracts::repository::users::UserRepository;
use crate::core::contracts::repository::victims::VictimReadRepository;
use crate::core::contracts::repository::work_sessions::WorkSessionReadRepository;
use crate::core::entities::attendance_members::AddAttendanceMember;
use crate::core::entities::auth::ClaimsToUserToken;
use crate::core::entities::common::PaginatedResult;
use crate::utils::errors::AppError;
use crate::core::contracts::repository::error::RepositoryError;
use crate::core::read_models::attendance_members::AttendanceMemberWithDetails;
use crate::core::read_models::attendance_victims::AttendanceVictimWithAddress;
use crate::core::read_models::victims::VictimWithDetails;

use crate::services::auth_context::AuthContext;
use crate::services::error_mapping::{map_constraint, map_unique_constraint};
use crate::utils::pagination::Pagination;
use crate::core::value_objects::policies::Policy;

pub struct AttendanceVictimService {
    attendance_victim_read_repository: Arc<dyn AttendanceVictimReadRepository>,
    attendance_victim_write_repository: Arc<dyn AttendanceVictimWriteRepository>,
    victim_repository: Arc<dyn VictimReadRepository>,
    user_repository: Arc<dyn UserRepository>,
    work_session_repository: Arc<dyn WorkSessionReadRepository>,
    attendance_member_repository: Arc<dyn AttendanceMemberRepository>,
}

impl AttendanceVictimService {
    pub fn new(
        attendance_victim_read_repository: Arc<dyn AttendanceVictimReadRepository>,
        attendance_victim_write_repository: Arc<dyn AttendanceVictimWriteRepository>,
        victim_repository: Arc<dyn VictimReadRepository>,
        user_repository: Arc<dyn UserRepository>,
        work_session_repository: Arc<dyn WorkSessionReadRepository>,
        attendance_member_repository: Arc<dyn AttendanceMemberRepository>,
    ) -> Self {
        Self {
            attendance_victim_read_repository,
            attendance_victim_write_repository,
            victim_repository,
            user_repository,
            work_session_repository,
            attendance_member_repository,
        }
    }

    pub async fn create_attendance_victim(
        &self,
        attendance: CreateAttendanceVictim,
        claims: &ClaimsToUserToken,
    ) -> Result<AttendanceVictimWithAddress, AppError> {
        let user_id = Uuid::parse_str(&claims.id)
            .map_err(|_| AppError::Unauthorized("Invalid user id in token".to_string()))?;

        let victim = self
            .verify_victim_access(claims, attendance.victim_id)
            .await?;
        let auth = AuthContext::load(&*self.user_repository, claims).await?;
        auth.check_policy(&Policy::CreateAttendances, victim.city_id)?;

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
            .attendance_victim_write_repository
            .create_attendance_victim(attendance, members_for_tx)
            .await
        {
            Ok(attendance_with_address) => {
                let attendance_with_address = attendance_with_address.into_with_address();
                info!(
                    "[AttendanceVictimService] Attendance victim created: {}",
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
                                "fk_attendances_victim",
                                "Error adding attendance: victim_id not found",
                            ),
                            (
                                "fk_attendance_victims_offender",
                                "Error adding attendance: offender_id not found",
                            ),
                            (
                                "fk_attendance_victims_protective_measure",
                                "Error adding attendance: protective_measure_id not found",
                            ),
                            (
                                "fk_attendance_addresses_city",
                                "Error adding attendance: address city_id not found",
                            ),
                        ],
                    )
                {
                    return Err(app_err);
                }
                error!(
                    "[AttendanceVictimService] Failed to create attendance victim: {:?}",
                    e
                );
                Err(AppError::InternalServerError)
            }
        }
    }

    pub async fn get_attendance_victim_by_id(
        &self,
        id: Uuid,
        claims: &ClaimsToUserToken,
    ) -> Result<AttendanceVictimWithAddress, AppError> {
        match self
            .attendance_victim_read_repository
            .get_attendance_victim_by_id(id)
            .await
        {
            Ok(attendance_with_address) => {
                let victim = self
                    .victim_repository
                    .get_victim_by_id(attendance_with_address.victim_id)
                    .await
                    .map_err(|e| match e {
                        RepositoryError::NotFound => AppError::NotFound(format!(
                            "Victim with id '{}' not found",
                            attendance_with_address.victim_id
                        )),
                        _ => AppError::InternalServerError,
                    })?;
                let auth = AuthContext::load(&*self.user_repository, claims).await?;
                auth.check_policy(&Policy::ReadAttendances, victim.city_id)?;
                Ok(attendance_with_address)
            }
            Err(RepositoryError::NotFound) => Err(AppError::NotFound(format!(
                "Attendance victim '{}' not found",
                id
            ))),
            Err(_) => Err(AppError::InternalServerError),
        }
    }

    pub async fn get_all_attendance_victims(
        &self,
        pagination: Pagination,
        claims: &ClaimsToUserToken,
    ) -> Result<PaginatedResult<AttendanceVictimWithAddress>, AppError> {
        let auth = AuthContext::load(&*self.user_repository, claims).await?;
        let allowed_cities = auth.allowed_cities(&Policy::ReadAttendances);

        let total_items = self
            .attendance_victim_read_repository
            .count_attendance_victims(allowed_cities.as_deref())
            .await
            .map_err(|_| AppError::InternalServerError)?;

        let attendances_list = self
            .attendance_victim_read_repository
            .get_attendance_victims_paginated(
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

    pub async fn get_attendance_victims_by_victim(
        &self,
        victim_id: Uuid,
        claims: &ClaimsToUserToken,
    ) -> Result<Vec<AttendanceVictimWithAddress>, AppError> {
        let victim = self.verify_victim_access(claims, victim_id).await?;
        let auth = AuthContext::load(&*self.user_repository, claims).await?;
        auth.check_policy(&Policy::ReadAttendances, victim.city_id)?;

        match self
            .attendance_victim_read_repository
            .get_attendance_victims_by_victim(victim_id)
            .await
        {
            Ok(attendances_list) => Ok(attendances_list),
            Err(_) => Err(AppError::InternalServerError),
        }
    }

    pub async fn update_attendance_victim_by_id(
        &self,
        data: UpdateAttendanceVictim,
        id: Uuid,
        claims: &ClaimsToUserToken,
    ) -> Result<AttendanceVictimWithAddress, AppError> {
        let existing = self
            .attendance_victim_read_repository
            .get_attendance_victim_by_id(id)
            .await
            .map_err(|e| {
                if matches!(e, RepositoryError::NotFound) {
                    AppError::NotFound(format!("Attendance victim '{}' not found", id))
                } else {
                    AppError::InternalServerError
                }
            })?;

        let existing_victim = self
            .victim_repository
            .get_victim_by_id(existing.victim_id)
            .await
            .map_err(|e| match e {
                RepositoryError::NotFound => {
                    AppError::NotFound(format!("Victim with id '{}' not found", existing.victim_id))
                }
                _ => AppError::InternalServerError,
            })?;
        let auth = AuthContext::load(&*self.user_repository, claims).await?;
        auth.check_policy(&Policy::UpdateAttendances, existing_victim.city_id)?;

        if data.victim_id != existing.victim_id {
            let new_victim = self.verify_victim_access(claims, data.victim_id).await?;
            auth.check_policy(&Policy::UpdateAttendances, new_victim.city_id)?;
        }

        match self
            .attendance_victim_write_repository
            .update_attendance_victim_by_id(data, id)
            .await
        {
            Ok(attendance_with_address) => Ok(attendance_with_address.into_with_address()),
            Err(RepositoryError::NotFound) => Err(AppError::NotFound(format!(
                "Attendance victim '{}' not found",
                id
            ))),
            Err(e) => {
                if let RepositoryError::UniqueViolation { constraint }
                | RepositoryError::ForeignKeyViolation { constraint } = &e
                    && let Some(app_err) = map_constraint(
                        constraint.as_deref(),
                        &[
                            (
                                "fk_attendances_victim",
                                "Error updating attendance: victim_id not found",
                            ),
                            (
                                "fk_attendance_victims_offender",
                                "Error updating attendance: offender_id not found",
                            ),
                            (
                                "fk_attendance_victims_protective_measure",
                                "Error updating attendance: protective_measure_id not found",
                            ),
                            (
                                "fk_attendance_addresses_city",
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

    pub async fn delete_attendance_victim_by_id(
        &self,
        id: Uuid,
        claims: &ClaimsToUserToken,
    ) -> Result<AttendanceVictimWithAddress, AppError> {
        let attendance = self
            .attendance_victim_read_repository
            .get_attendance_victim_by_id(id)
            .await
            .map_err(|e| {
                if matches!(e, RepositoryError::NotFound) {
                    AppError::NotFound(format!("Attendance victim '{}' not found", id))
                } else {
                    AppError::InternalServerError
                }
            })?;

        let victim = self
            .victim_repository
            .get_victim_by_id(attendance.victim_id)
            .await
            .map_err(|e| match e {
                RepositoryError::NotFound => AppError::NotFound(format!(
                    "Victim with id '{}' not found",
                    attendance.victim_id
                )),
                _ => AppError::InternalServerError,
            })?;
        let auth = AuthContext::load(&*self.user_repository, claims).await?;
        auth.check_policy(&Policy::DeleteAttendances, victim.city_id)?;

        match self
            .attendance_victim_write_repository
            .delete_attendance_victim_by_id(id)
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
            "[AttendanceVictimService] Getting members for attendance: {}",
            attendance_id
        );

        let auth = AuthContext::load(&*self.user_repository, claims).await?;

        let attendance = self
            .attendance_victim_read_repository
            .get_attendance_victim_by_id(attendance_id)
            .await
            .map_err(|e| {
                if matches!(e, RepositoryError::NotFound) {
                    AppError::NotFound(format!("Attendance victim '{}' not found", attendance_id))
                } else {
                    AppError::InternalServerError
                }
            })?;

        let victim = self
            .verify_victim_access(claims, attendance.victim_id)
            .await?;
        auth.check_policy(&Policy::ReadAttendances, victim.city_id)?;

        let members = self
            .attendance_member_repository
            .get_victim_attendance_members(attendance_id)
            .await
            .map_err(|_| AppError::InternalServerError)?;

        info!("[AttendanceVictimService] Found {} members", members.len());
        Ok(members)
    }

    pub async fn add_attendance_member(
        &self,
        attendance_id: Uuid,
        data: AddAttendanceMember,
        claims: &ClaimsToUserToken,
    ) -> Result<String, AppError> {
        info!(
            "[AttendanceVictimService] Adding member to attendance: {}",
            attendance_id
        );

        let auth = AuthContext::load(&*self.user_repository, claims).await?;

        let attendance = self
            .attendance_victim_read_repository
            .get_attendance_victim_by_id(attendance_id)
            .await
            .map_err(|e| {
                if matches!(e, RepositoryError::NotFound) {
                    AppError::NotFound(format!("Attendance victim '{}' not found", attendance_id))
                } else {
                    AppError::InternalServerError
                }
            })?;

        let victim = self
            .verify_victim_access(claims, attendance.victim_id)
            .await?;
        auth.check_policy(&Policy::ManageAttendanceMembers, victim.city_id)?;

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

        if user_to_add.city_id != Some(victim.city_id) {
            return Err(AppError::BadRequest(
                "User must be from the same city as the victim".to_string(),
            ));
        }

        match self
            .attendance_member_repository
            .add_member_to_victim_attendance(attendance_id, data.user_id, None)
            .await
        {
            Ok(_) => {
                info!("[AttendanceVictimService] Member added successfully");
                Ok("Member added successfully".to_string())
            }
            Err(e) => {
                if let RepositoryError::UniqueViolation { constraint } = &e
                    && let Some(app_err) = map_unique_constraint(
                        constraint.as_deref(),
                        &[(
                            "attendance_victim_members_attendance_victim_id_user_id_key",
                            "Member already added to attendance",
                        )],
                    )
                {
                    return Err(app_err);
                }
                error!("[AttendanceVictimService] Failed to add member: {:?}", e);
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
            "[AttendanceVictimService] Removing member from attendance: {}",
            attendance_id
        );

        let auth = AuthContext::load(&*self.user_repository, claims).await?;

        let attendance = self
            .attendance_victim_read_repository
            .get_attendance_victim_by_id(attendance_id)
            .await
            .map_err(|e| {
                if matches!(e, RepositoryError::NotFound) {
                    AppError::NotFound(format!("Attendance victim '{}' not found", attendance_id))
                } else {
                    AppError::InternalServerError
                }
            })?;

        let victim = self
            .verify_victim_access(claims, attendance.victim_id)
            .await?;
        auth.check_policy(&Policy::ManageAttendanceMembers, victim.city_id)?;

        match self
            .attendance_member_repository
            .remove_member_from_victim_attendance(attendance_id, user_id)
            .await
        {
            Ok(_) => {
                info!("[AttendanceVictimService] Member removed successfully");
                Ok("Member removed successfully".to_string())
            }
            Err(e) => {
                error!("[AttendanceVictimService] Failed to remove member: {:?}", e);
                Err(AppError::InternalServerError)
            }
        }
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
