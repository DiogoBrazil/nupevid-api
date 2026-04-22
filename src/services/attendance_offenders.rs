use actix_web::{HttpRequest, HttpResponse, web};
use log::{error, info};
use uuid::Uuid;

use crate::core::contracts::repository::attendance_members::AttendanceMemberRepository;
use crate::core::contracts::repository::attendance_offenders::AttendanceOffenderRepository;
use crate::core::contracts::repository::offenders::OffenderRepository;
use crate::core::contracts::repository::protective_measures::ProtectiveMeasureRepository;
use crate::core::contracts::repository::users::UserRepository;
use crate::core::contracts::repository::victims::VictimRepository;
use crate::core::contracts::repository::work_sessions::WorkSessionRepository;
use crate::core::entities::attendance_members::AddAttendanceMember;
use crate::core::entities::attendance_offenders::{
    CreateAttendanceOffender, UpdateAttendanceOffender,
};
use crate::core::entities::auth::ClaimsToUserToken;
use crate::core::entities::offenders::OffenderWithDetails;
use crate::core::entities::victims::VictimWithDetails;
use crate::repositories::attendance_members::PgAttendanceMemberRepository;
use crate::repositories::attendance_offenders::PgAttendanceOffenderRepository;
use crate::repositories::offenders::PgOffenderRepository;
use crate::repositories::protective_measures::PgProtectiveMeasureRepository;
use crate::repositories::users::PgUserRepository;
use crate::repositories::victims::PgVictimRepository;
use crate::repositories::work_sessions::PgWorkSessionRepository;

use crate::utils::{
    authorization::{check_policy, get_allowed_cities_for_policy},
    db_error_mapper::{map_constraint, map_unique_constraint},
    errors::AppError,
    pagination::{PaginationParams, normalize_pagination},
    responses::{ApiResponse, PaginatedResponse},
    service_helpers::{extract_claims, get_user_policies_with_defaults},
};
use crate::validators::common::{
    POLICY_CREATE_ATTENDANCES, POLICY_DELETE_ATTENDANCES, POLICY_MANAGE_ATTENDANCE_MEMBERS,
    POLICY_READ_ATTENDANCES, POLICY_UPDATE_ATTENDANCES,
};

pub struct AttendanceOffenderService {
    attendance_offender_repository: web::Data<PgAttendanceOffenderRepository>,
    offender_repository: web::Data<PgOffenderRepository>,
    victim_repository: web::Data<PgVictimRepository>,
    protective_measure_repository: web::Data<PgProtectiveMeasureRepository>,
    user_repository: web::Data<PgUserRepository>,
    work_session_repository: web::Data<PgWorkSessionRepository>,
    attendance_member_repository: web::Data<PgAttendanceMemberRepository>,
}

impl AttendanceOffenderService {
    pub fn new(
        attendance_offender_repository: web::Data<PgAttendanceOffenderRepository>,
        offender_repository: web::Data<PgOffenderRepository>,
        victim_repository: web::Data<PgVictimRepository>,
        protective_measure_repository: web::Data<PgProtectiveMeasureRepository>,
        user_repository: web::Data<PgUserRepository>,
        work_session_repository: web::Data<PgWorkSessionRepository>,
        attendance_member_repository: web::Data<PgAttendanceMemberRepository>,
    ) -> Self {
        Self {
            attendance_offender_repository,
            offender_repository,
            victim_repository,
            protective_measure_repository,
            user_repository,
            work_session_repository,
            attendance_member_repository,
        }
    }

    pub async fn create_attendance_offender(
        &self,
        attendance: CreateAttendanceOffender,
        req: HttpRequest,
    ) -> Result<HttpResponse, AppError> {
        let claims = extract_claims(&req)?;
        let user_id = Uuid::parse_str(&claims.id)
            .map_err(|_| AppError::Unauthorized("Invalid user id in token".to_string()))?;

        let offender = self
            .verify_offender_access(&claims, attendance.offender_id)
            .await?;

        let _victim = self
            .verify_victim_access(&claims, attendance.victim_id)
            .await?;

        if let Some(pm_id) = attendance.protective_measure_id {
            match self
                .protective_measure_repository
                .get_protective_measure_by_id(pm_id)
                .await
            {
                Ok(_) => {}
                Err(sqlx::Error::RowNotFound) => {
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

        let policies = get_user_policies_with_defaults(&**self.user_repository, &claims).await?;
        check_policy(
            &claims,
            POLICY_CREATE_ATTENDANCES,
            offender.city_id,
            &policies,
        )?;

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
            .attendance_offender_repository
            .create_attendance_offender(attendance, members_for_tx)
            .await
        {
            Ok(attendance_with_address) => {
                info!(
                    "[AttendanceOffenderService] Attendance offender created: {}",
                    attendance_with_address.id
                );
                Ok(ApiResponse::created(attendance_with_address).into_response())
            }
            Err(e) => {
                if let sqlx::Error::Database(db_err) = &e
                    && let Some(app_err) = map_constraint(
                        db_err.constraint(),
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
        req: HttpRequest,
    ) -> Result<HttpResponse, AppError> {
        let claims = extract_claims(&req)?;

        match self
            .attendance_offender_repository
            .get_attendance_offender_by_id(id)
            .await
        {
            Ok(attendance_with_address) => {
                let offender = self
                    .offender_repository
                    .get_offender_by_id(attendance_with_address.offender_id)
                    .await
                    .map_err(|e| match e {
                        sqlx::Error::RowNotFound => AppError::NotFound(format!(
                            "Offender with id '{}' not found",
                            attendance_with_address.offender_id
                        )),
                        _ => AppError::InternalServerError,
                    })?;

                let policies =
                    get_user_policies_with_defaults(&**self.user_repository, &claims).await?;
                check_policy(
                    &claims,
                    POLICY_READ_ATTENDANCES,
                    offender.city_id,
                    &policies,
                )?;

                Ok(ApiResponse::success(attendance_with_address).into_response())
            }
            Err(sqlx::Error::RowNotFound) => Err(AppError::NotFound(format!(
                "Attendance offender '{}' not found",
                id
            ))),
            Err(_) => Err(AppError::InternalServerError),
        }
    }

    pub async fn get_all_attendance_offenders(
        &self,
        params: PaginationParams,
        req: HttpRequest,
    ) -> Result<HttpResponse, AppError> {
        let claims = extract_claims(&req)?;

        let policies = get_user_policies_with_defaults(&**self.user_repository, &claims).await?;
        let pagination = normalize_pagination(&params);
        let allowed_cities =
            get_allowed_cities_for_policy(&claims, POLICY_READ_ATTENDANCES, &policies);

        let total_items = self
            .attendance_offender_repository
            .count_attendance_offenders(allowed_cities.as_deref())
            .await
            .map_err(|_| AppError::InternalServerError)?;

        let attendances_list = self
            .attendance_offender_repository
            .get_attendance_offenders_paginated(
                allowed_cities.as_deref(),
                pagination.page_size,
                pagination.offset,
            )
            .await
            .map_err(|_| AppError::InternalServerError)?;

        Ok(PaginatedResponse::success(
            attendances_list,
            pagination.page,
            pagination.page_size,
            total_items,
        )
        .into_response())
    }

    pub async fn get_attendance_offenders_by_offender(
        &self,
        offender_id: Uuid,
        req: HttpRequest,
    ) -> Result<HttpResponse, AppError> {
        let claims = extract_claims(&req)?;
        let offender = self.verify_offender_access(&claims, offender_id).await?;

        let policies = get_user_policies_with_defaults(&**self.user_repository, &claims).await?;
        check_policy(
            &claims,
            POLICY_READ_ATTENDANCES,
            offender.city_id,
            &policies,
        )?;

        match self
            .attendance_offender_repository
            .get_attendance_offenders_by_offender(offender_id)
            .await
        {
            Ok(attendances_list) => Ok(ApiResponse::success(attendances_list).into_response()),
            Err(_) => Err(AppError::InternalServerError),
        }
    }

    pub async fn get_attendance_offenders_by_victim(
        &self,
        victim_id: Uuid,
        req: HttpRequest,
    ) -> Result<HttpResponse, AppError> {
        let claims = extract_claims(&req)?;
        let victim = self.verify_victim_access(&claims, victim_id).await?;

        let policies = get_user_policies_with_defaults(&**self.user_repository, &claims).await?;
        check_policy(&claims, POLICY_READ_ATTENDANCES, victim.city_id, &policies)?;

        match self
            .attendance_offender_repository
            .get_attendance_offenders_by_victim(victim_id)
            .await
        {
            Ok(attendances_list) => Ok(ApiResponse::success(attendances_list).into_response()),
            Err(_) => Err(AppError::InternalServerError),
        }
    }

    pub async fn update_attendance_offender_by_id(
        &self,
        data: UpdateAttendanceOffender,
        id: Uuid,
        req: HttpRequest,
    ) -> Result<HttpResponse, AppError> {
        let claims = extract_claims(&req)?;

        let existing = self
            .attendance_offender_repository
            .get_attendance_offender_by_id(id)
            .await
            .map_err(|e| {
                if matches!(e, sqlx::Error::RowNotFound) {
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
                sqlx::Error::RowNotFound => AppError::NotFound(format!(
                    "Offender with id '{}' not found",
                    existing.offender_id
                )),
                _ => AppError::InternalServerError,
            })?;

        let policies = get_user_policies_with_defaults(&**self.user_repository, &claims).await?;
        check_policy(
            &claims,
            POLICY_UPDATE_ATTENDANCES,
            existing_offender.city_id,
            &policies,
        )?;

        if data.offender_id != existing.offender_id {
            let new_offender = self
                .verify_offender_access(&claims, data.offender_id)
                .await?;
            check_policy(
                &claims,
                POLICY_UPDATE_ATTENDANCES,
                new_offender.city_id,
                &policies,
            )?;
        }

        if data.victim_id != existing.victim_id {
            let victim = self.verify_victim_access(&claims, data.victim_id).await?;
            check_policy(
                &claims,
                POLICY_UPDATE_ATTENDANCES,
                victim.city_id,
                &policies,
            )?;
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
                Err(sqlx::Error::RowNotFound) => {
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
            .attendance_offender_repository
            .update_attendance_offender_by_id(data, id)
            .await
        {
            Ok(attendance_with_address) => {
                Ok(ApiResponse::success(attendance_with_address).into_response())
            }
            Err(sqlx::Error::RowNotFound) => Err(AppError::NotFound(format!(
                "Attendance offender '{}' not found",
                id
            ))),
            Err(e) => {
                if let sqlx::Error::Database(db_err) = &e
                    && let Some(app_err) = map_constraint(
                        db_err.constraint(),
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
        req: HttpRequest,
    ) -> Result<HttpResponse, AppError> {
        let claims = extract_claims(&req)?;

        let attendance = self
            .attendance_offender_repository
            .get_attendance_offender_by_id(id)
            .await
            .map_err(|e| {
                if matches!(e, sqlx::Error::RowNotFound) {
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
                sqlx::Error::RowNotFound => AppError::NotFound(format!(
                    "Offender with id '{}' not found",
                    attendance.offender_id
                )),
                _ => AppError::InternalServerError,
            })?;

        let policies = get_user_policies_with_defaults(&**self.user_repository, &claims).await?;
        check_policy(
            &claims,
            POLICY_DELETE_ATTENDANCES,
            offender.city_id,
            &policies,
        )?;

        match self
            .attendance_offender_repository
            .delete_attendance_offender_by_id(id)
            .await
        {
            Ok(deleted) => Ok(ApiResponse::success(deleted).into_response()),
            Err(_) => Err(AppError::InternalServerError),
        }
    }

    pub async fn get_attendance_members(
        &self,
        attendance_id: Uuid,
        req: HttpRequest,
    ) -> Result<HttpResponse, AppError> {
        info!(
            "[AttendanceOffenderService] Getting members for attendance: {}",
            attendance_id
        );

        let claims = extract_claims(&req)?;
        let policies = get_user_policies_with_defaults(&**self.user_repository, &claims).await?;

        let attendance = self
            .attendance_offender_repository
            .get_attendance_offender_by_id(attendance_id)
            .await
            .map_err(|e| {
                if matches!(e, sqlx::Error::RowNotFound) {
                    AppError::NotFound(format!("Attendance offender '{}' not found", attendance_id))
                } else {
                    AppError::InternalServerError
                }
            })?;

        let offender = self
            .verify_offender_access(&claims, attendance.offender_id)
            .await?;
        check_policy(
            &claims,
            POLICY_READ_ATTENDANCES,
            offender.city_id,
            &policies,
        )?;

        let members = self
            .attendance_member_repository
            .get_offender_attendance_members(attendance_id)
            .await
            .map_err(|_| AppError::InternalServerError)?;

        info!(
            "[AttendanceOffenderService] Found {} members",
            members.len()
        );
        Ok(ApiResponse::success(members).into_response())
    }

    pub async fn add_attendance_member(
        &self,
        attendance_id: Uuid,
        data: AddAttendanceMember,
        req: HttpRequest,
    ) -> Result<HttpResponse, AppError> {
        info!(
            "[AttendanceOffenderService] Adding member to attendance: {}",
            attendance_id
        );

        let claims = extract_claims(&req)?;
        let policies = get_user_policies_with_defaults(&**self.user_repository, &claims).await?;

        let attendance = self
            .attendance_offender_repository
            .get_attendance_offender_by_id(attendance_id)
            .await
            .map_err(|e| {
                if matches!(e, sqlx::Error::RowNotFound) {
                    AppError::NotFound(format!("Attendance offender '{}' not found", attendance_id))
                } else {
                    AppError::InternalServerError
                }
            })?;

        let offender = self
            .verify_offender_access(&claims, attendance.offender_id)
            .await?;
        check_policy(
            &claims,
            POLICY_MANAGE_ATTENDANCE_MEMBERS,
            offender.city_id,
            &policies,
        )?;

        let user_to_add = self
            .user_repository
            .get_user_by_id(data.user_id)
            .await
            .map_err(|e| {
                if matches!(e, sqlx::Error::RowNotFound) {
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
                Ok(ApiResponse::success("Member added successfully").into_response())
            }
            Err(e) => {
                if let sqlx::Error::Database(db_err) = &e
                    && db_err.is_unique_violation()
                    && let Some(app_err) = map_unique_constraint(
                        db_err.constraint(),
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
        req: HttpRequest,
    ) -> Result<HttpResponse, AppError> {
        info!(
            "[AttendanceOffenderService] Removing member from attendance: {}",
            attendance_id
        );

        let claims = extract_claims(&req)?;
        let policies = get_user_policies_with_defaults(&**self.user_repository, &claims).await?;

        let attendance = self
            .attendance_offender_repository
            .get_attendance_offender_by_id(attendance_id)
            .await
            .map_err(|e| {
                if matches!(e, sqlx::Error::RowNotFound) {
                    AppError::NotFound(format!("Attendance offender '{}' not found", attendance_id))
                } else {
                    AppError::InternalServerError
                }
            })?;

        let offender = self
            .verify_offender_access(&claims, attendance.offender_id)
            .await?;
        check_policy(
            &claims,
            POLICY_MANAGE_ATTENDANCE_MEMBERS,
            offender.city_id,
            &policies,
        )?;

        match self
            .attendance_member_repository
            .remove_member_from_offender_attendance(attendance_id, user_id)
            .await
        {
            Ok(_) => {
                info!("[AttendanceOffenderService] Member removed successfully");
                Ok(ApiResponse::success("Member removed successfully").into_response())
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
                if matches!(e, sqlx::Error::RowNotFound) {
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
                if matches!(e, sqlx::Error::RowNotFound) {
                    AppError::NotFound(format!("Victim '{}' not found", victim_id))
                } else {
                    AppError::InternalServerError
                }
            })
    }
}
