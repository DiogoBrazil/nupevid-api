use actix_web::{web, HttpRequest, HttpResponse};
use log::{error, info};
use uuid::Uuid;

use crate::core::contracts::repository::attendance_victims::AttendanceVictimRepository;
use crate::core::contracts::repository::victims::VictimRepository;
use crate::core::contracts::repository::users::UserRepository;
use crate::core::contracts::repository::work_sessions::WorkSessionRepository;
use crate::core::contracts::repository::attendance_members::AttendanceMemberRepository;
use crate::core::entities::attendance_victims::{CreateAttendanceVictim, UpdateAttendanceVictim};
use crate::core::entities::attendance_members::AddAttendanceMember;
use crate::core::entities::auth::ClaimsToUserToken;
use crate::core::entities::victims::VictimWithDetails;
use crate::repositories::attendance_victims::PgAttendanceVictimRepository;
use crate::repositories::victims::PgVictimRepository;
use crate::repositories::users::PgUserRepository;
use crate::repositories::work_sessions::PgWorkSessionRepository;
use crate::repositories::attendance_members::PgAttendanceMemberRepository;

use crate::utils::{
    errors::AppError,
    responses::{ApiResponse, PaginatedResponse},
    authorization::{check_policy, get_allowed_cities_for_policy},
    service_helpers::{extract_claims, get_user_policies_with_defaults},
    db_error_mapper::{map_constraint, map_unique_constraint},
    pagination::{PaginationParams, normalize_pagination}
};
use crate::validators::common::{
    POLICY_CREATE_ATTENDANCES, POLICY_READ_ATTENDANCES, POLICY_UPDATE_ATTENDANCES, POLICY_DELETE_ATTENDANCES,
    POLICY_MANAGE_ATTENDANCE_MEMBERS
};

pub struct AttendanceVictimService {
    attendance_victim_repository: web::Data<PgAttendanceVictimRepository>,
    victim_repository: web::Data<PgVictimRepository>,
    user_repository: web::Data<PgUserRepository>,
    work_session_repository: web::Data<PgWorkSessionRepository>,
    attendance_member_repository: web::Data<PgAttendanceMemberRepository>,
}

impl AttendanceVictimService {
    pub fn new(
        attendance_victim_repository: web::Data<PgAttendanceVictimRepository>,
        victim_repository: web::Data<PgVictimRepository>,
        user_repository: web::Data<PgUserRepository>,
        work_session_repository: web::Data<PgWorkSessionRepository>,
        attendance_member_repository: web::Data<PgAttendanceMemberRepository>,
    ) -> Self {
        Self {
            attendance_victim_repository,
            victim_repository,
            user_repository,
            work_session_repository,
            attendance_member_repository,
        }
    }

    pub async fn create_attendance_victim(
        &self,
        attendance: CreateAttendanceVictim,
        req: HttpRequest,
    ) -> Result<HttpResponse, AppError> {
        let claims = extract_claims(&req)?;
        let user_id = Uuid::parse_str(&claims.id)
            .map_err(|_| AppError::Unauthorized("Invalid user id in token".to_string()))?;

        let victim = self.verify_victim_access(&claims, attendance.victim_id).await?;
        let policies = get_user_policies_with_defaults(&**self.user_repository, &claims).await?;
        check_policy(&claims, POLICY_CREATE_ATTENDANCES, victim.city_id, &policies)?;

        let active_session = self.work_session_repository
            .get_active_session_by_user(user_id)
            .await
            .map_err(|_| AppError::BadRequest("No active work session found. You must have an active work session to create an attendance.".to_string()))?;

        let session_members = self.work_session_repository
            .get_session_members(active_session.id)
            .await
            .map_err(|_| AppError::InternalServerError)?;

        match self.attendance_victim_repository.create_attendance_victim(attendance).await {
            Ok(attendance_with_address) => {
                info!(
                    "[AttendanceVictimService] Attendance victim created: {}",
                    attendance_with_address.id
                );

                for member in session_members {
                    match self.attendance_member_repository
                        .add_member_to_victim_attendance(attendance_with_address.id, member.user_id, Some(active_session.id))
                        .await
                    {
                        Ok(_) => {
                            info!(
                                "[AttendanceVictimService] Member {} added to attendance {}",
                                member.user_id, attendance_with_address.id
                            );
                        }
                        Err(e) => {
                            error!(
                                "[AttendanceVictimService] Failed to add member {} to attendance: {:?}",
                                member.user_id, e
                            );
                        }
                    }
                }

                Ok(ApiResponse::created(attendance_with_address).into_response())
            }
            Err(e) => {
                if let sqlx::Error::Database(db_err) = &e {
                    if let Some(app_err) = map_constraint(db_err.constraint(), &[
                        ("fk_attendances_victim", "Error adding attendance: victim_id not found"),
                        ("fk_attendance_victims_offender", "Error adding attendance: offender_id not found"),
                        ("fk_attendance_victims_protective_measure", "Error adding attendance: protective_measure_id not found"),
                        ("fk_attendance_addresses_city", "Error adding attendance: address city_id not found"),
                    ]) {
                        return Err(app_err);
                    }
                }
                error!("[AttendanceVictimService] Failed to create attendance victim: {:?}", e);
                Err(AppError::InternalServerError)
            }
        }
    }

    pub async fn get_attendance_victim_by_id(
        &self,
        id: Uuid,
        req: HttpRequest,
    ) -> Result<HttpResponse, AppError> {
        let claims = extract_claims(&req)?;

        match self.attendance_victim_repository.get_attendance_victim_by_id(id).await {
            Ok(attendance_with_address) => {
                let victim = self
                    .victim_repository
                    .get_victim_by_id(attendance_with_address.victim_id)
                    .await
                    .map_err(|e| match e {
                        sqlx::Error::RowNotFound => AppError::NotFound(
                            format!("Victim with id '{}' not found", attendance_with_address.victim_id)
                        ),
                        _ => AppError::InternalServerError,
                    })?;
                let policies = get_user_policies_with_defaults(&**self.user_repository, &claims).await?;
                check_policy(&claims, POLICY_READ_ATTENDANCES, victim.city_id, &policies)?;
                Ok(ApiResponse::success(attendance_with_address).into_response())
            }
            Err(sqlx::Error::RowNotFound) => {
                Err(AppError::NotFound(format!("Attendance victim '{}' not found", id)))
            }
            Err(_) => Err(AppError::InternalServerError),
        }
    }

    pub async fn get_all_attendance_victims(
        &self,
        params: PaginationParams,
        req: HttpRequest,
    ) -> Result<HttpResponse, AppError> {
        let claims = extract_claims(&req)?;

        let policies = get_user_policies_with_defaults(&**self.user_repository, &claims).await?;
        let pagination = normalize_pagination(&params);
        let allowed_cities = get_allowed_cities_for_policy(&claims, POLICY_READ_ATTENDANCES, &policies);

        let total_items = self.attendance_victim_repository
            .count_attendance_victims(allowed_cities.as_deref())
            .await
            .map_err(|_| AppError::InternalServerError)?;

        let attendances_list = self.attendance_victim_repository
            .get_attendance_victims_paginated(
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
        ).into_response())
    }

    pub async fn get_attendance_victims_by_victim(
        &self,
        victim_id: Uuid,
        req: HttpRequest,
    ) -> Result<HttpResponse, AppError> {
        let claims = extract_claims(&req)?;
        let victim = self.verify_victim_access(&claims, victim_id).await?;
        let policies = get_user_policies_with_defaults(&**self.user_repository, &claims).await?;
        check_policy(&claims, POLICY_READ_ATTENDANCES, victim.city_id, &policies)?;

        match self
            .attendance_victim_repository
            .get_attendance_victims_by_victim(victim_id)
            .await
        {
            Ok(attendances_list) => Ok(ApiResponse::success(attendances_list).into_response()),
            Err(_) => Err(AppError::InternalServerError),
        }
    }

    pub async fn update_attendance_victim_by_id(
        &self,
        data: UpdateAttendanceVictim,
        id: Uuid,
        req: HttpRequest,
    ) -> Result<HttpResponse, AppError> {
        let claims = extract_claims(&req)?;

        let existing = self
            .attendance_victim_repository
            .get_attendance_victim_by_id(id)
            .await
            .map_err(|e| {
                if matches!(e, sqlx::Error::RowNotFound) {
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
                sqlx::Error::RowNotFound => AppError::NotFound(
                    format!("Victim with id '{}' not found", existing.victim_id)
                ),
                _ => AppError::InternalServerError,
            })?;
        let policies = get_user_policies_with_defaults(&**self.user_repository, &claims).await?;
        check_policy(&claims, POLICY_UPDATE_ATTENDANCES, existing_victim.city_id, &policies)?;

        if data.victim_id != existing.victim_id {
            let new_victim = self.verify_victim_access(&claims, data.victim_id).await?;
            check_policy(&claims, POLICY_UPDATE_ATTENDANCES, new_victim.city_id, &policies)?;
        }

        match self
            .attendance_victim_repository
            .update_attendance_victim_by_id(data, id)
            .await
        {
            Ok(attendance_with_address) => {
                Ok(ApiResponse::success(attendance_with_address).into_response())
            }
            Err(sqlx::Error::RowNotFound) => {
                Err(AppError::NotFound(format!("Attendance victim '{}' not found", id)))
            }
            Err(e) => {
                if let sqlx::Error::Database(db_err) = &e {
                    if let Some(app_err) = map_constraint(db_err.constraint(), &[
                        ("fk_attendances_victim", "Error updating attendance: victim_id not found"),
                        ("fk_attendance_victims_offender", "Error updating attendance: offender_id not found"),
                        ("fk_attendance_victims_protective_measure", "Error updating attendance: protective_measure_id not found"),
                        ("fk_attendance_addresses_city", "Error updating attendance: address city_id not found"),
                    ]) {
                        return Err(app_err);
                    }
                }
                Err(AppError::InternalServerError)
            }
        }
    }

    pub async fn delete_attendance_victim_by_id(
        &self,
        id: Uuid,
        req: HttpRequest,
    ) -> Result<HttpResponse, AppError> {
        let claims = extract_claims(&req)?;

        let attendance = self
            .attendance_victim_repository
            .get_attendance_victim_by_id(id)
            .await
            .map_err(|e| {
                if matches!(e, sqlx::Error::RowNotFound) {
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
                sqlx::Error::RowNotFound => AppError::NotFound(
                    format!("Victim with id '{}' not found", attendance.victim_id)
                ),
                _ => AppError::InternalServerError,
            })?;
        let policies = get_user_policies_with_defaults(&**self.user_repository, &claims).await?;
        check_policy(&claims, POLICY_DELETE_ATTENDANCES, victim.city_id, &policies)?;

        match self.attendance_victim_repository.delete_attendance_victim_by_id(id).await {
            Ok(deleted) => Ok(ApiResponse::success(deleted).into_response()),
            Err(_) => Err(AppError::InternalServerError),
        }
    }

    pub async fn get_attendance_members(
        &self,
        attendance_id: Uuid,
        req: HttpRequest,
    ) -> Result<HttpResponse, AppError> {
        info!("[AttendanceVictimService] Getting members for attendance: {}", attendance_id);

        let claims = extract_claims(&req)?;
        let policies = get_user_policies_with_defaults(&**self.user_repository, &claims).await?;

        let attendance = self.attendance_victim_repository
            .get_attendance_victim_by_id(attendance_id)
            .await
            .map_err(|e| {
                if matches!(e, sqlx::Error::RowNotFound) {
                    AppError::NotFound(format!("Attendance victim '{}' not found", attendance_id))
                } else {
                    AppError::InternalServerError
                }
            })?;

        let victim = self.verify_victim_access(&claims, attendance.victim_id).await?;
        check_policy(&claims, POLICY_READ_ATTENDANCES, victim.city_id, &policies)?;

        let members = self.attendance_member_repository
            .get_victim_attendance_members(attendance_id)
            .await
            .map_err(|_| AppError::InternalServerError)?;

        info!("[AttendanceVictimService] Found {} members", members.len());
        Ok(ApiResponse::success(members).into_response())
    }

    pub async fn add_attendance_member(
        &self,
        attendance_id: Uuid,
        data: AddAttendanceMember,
        req: HttpRequest,
    ) -> Result<HttpResponse, AppError> {
        info!("[AttendanceVictimService] Adding member to attendance: {}", attendance_id);

        let claims = extract_claims(&req)?;
        let policies = get_user_policies_with_defaults(&**self.user_repository, &claims).await?;

        let attendance = self.attendance_victim_repository
            .get_attendance_victim_by_id(attendance_id)
            .await
            .map_err(|e| {
                if matches!(e, sqlx::Error::RowNotFound) {
                    AppError::NotFound(format!("Attendance victim '{}' not found", attendance_id))
                } else {
                    AppError::InternalServerError
                }
            })?;

        let victim = self.verify_victim_access(&claims, attendance.victim_id).await?;
        check_policy(&claims, POLICY_MANAGE_ATTENDANCE_MEMBERS, victim.city_id, &policies)?;

        let user_to_add = self.user_repository
            .get_user_by_id(data.user_id)
            .await
            .map_err(|e| {
                if matches!(e, sqlx::Error::RowNotFound) {
                    AppError::NotFound(format!("User '{}' not found", data.user_id))
                } else {
                    AppError::InternalServerError
                }
            })?;

        if user_to_add.city_id != Some(victim.city_id) {
            return Err(AppError::BadRequest("User must be from the same city as the victim".to_string()));
        }

        match self.attendance_member_repository
            .add_member_to_victim_attendance(attendance_id, data.user_id, None)
            .await
        {
            Ok(_) => {
                info!("[AttendanceVictimService] Member added successfully");
                Ok(ApiResponse::success("Member added successfully").into_response())
            }
            Err(e) => {
                if let sqlx::Error::Database(db_err) = &e {
                    if db_err.is_unique_violation() {
                        if let Some(app_err) = map_unique_constraint(db_err.constraint(), &[
                            ("attendance_victim_members_attendance_victim_id_user_id_key", "Member already added to attendance"),
                        ]) {
                            return Err(app_err);
                        }
                    }
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
        req: HttpRequest,
    ) -> Result<HttpResponse, AppError> {
        info!("[AttendanceVictimService] Removing member from attendance: {}", attendance_id);

        let claims = extract_claims(&req)?;
        let policies = get_user_policies_with_defaults(&**self.user_repository, &claims).await?;

        let attendance = self.attendance_victim_repository
            .get_attendance_victim_by_id(attendance_id)
            .await
            .map_err(|e| {
                if matches!(e, sqlx::Error::RowNotFound) {
                    AppError::NotFound(format!("Attendance victim '{}' not found", attendance_id))
                } else {
                    AppError::InternalServerError
                }
            })?;

        let victim = self.verify_victim_access(&claims, attendance.victim_id).await?;
        check_policy(&claims, POLICY_MANAGE_ATTENDANCE_MEMBERS, victim.city_id, &policies)?;

        match self.attendance_member_repository
            .remove_member_from_victim_attendance(attendance_id, user_id)
            .await
        {
            Ok(_) => {
                info!("[AttendanceVictimService] Member removed successfully");
                Ok(ApiResponse::success("Member removed successfully").into_response())
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
                if matches!(e, sqlx::Error::RowNotFound) {
                    AppError::NotFound(format!("Victim '{}' not found", victim_id))
                } else {
                    AppError::InternalServerError
                }
            })
    }
}
