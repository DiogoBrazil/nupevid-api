use actix_web::{web, HttpRequest, HttpResponse};
use log::{error, info};
use uuid::Uuid;

use crate::core::contracts::repository::attendance_offenders::AttendanceOffenderRepository;
use crate::core::contracts::repository::offenders::OffenderRepository;
use crate::core::contracts::repository::protective_measures::ProtectiveMeasureRepository;
use crate::core::contracts::repository::victims::VictimRepository;
use crate::core::entities::attendance_offenders::{CreateAttendanceOffender, UpdateAttendanceOffender};
use crate::core::entities::auth::ClaimsToUserToken;
use crate::core::entities::offenders::OffenderWithDetails;
use crate::core::entities::victims::VictimWithDetails;
use crate::repositories::attendance_offenders::PgAttendanceOffenderRepository;
use crate::repositories::offenders::PgOffenderRepository;
use crate::repositories::protective_measures::PgProtectiveMeasureRepository;
use crate::repositories::victims::PgVictimRepository;
use crate::repositories::users::PgUserRepository;

use crate::utils::{
    errors::AppError,
    responses::ApiResponse,
    authorization::{check_policy, get_allowed_cities_for_policy},
    service_helpers::{extract_claims, get_user_policies_with_defaults}
};
use crate::validators::common::{
    POLICY_CREATE_ATTENDANCES, POLICY_READ_ATTENDANCES, POLICY_UPDATE_ATTENDANCES, POLICY_DELETE_ATTENDANCES
};

pub struct AttendanceOffenderService {
    attendance_offender_repository: web::Data<PgAttendanceOffenderRepository>,
    offender_repository: web::Data<PgOffenderRepository>,
    victim_repository: web::Data<PgVictimRepository>,
    protective_measure_repository: web::Data<PgProtectiveMeasureRepository>,
    user_repository: web::Data<PgUserRepository>,
}

impl AttendanceOffenderService {
    pub fn new(
        attendance_offender_repository: web::Data<PgAttendanceOffenderRepository>,
        offender_repository: web::Data<PgOffenderRepository>,
        victim_repository: web::Data<PgVictimRepository>,
        protective_measure_repository: web::Data<PgProtectiveMeasureRepository>,
        user_repository: web::Data<PgUserRepository>,
    ) -> Self {
        Self {
            attendance_offender_repository,
            offender_repository,
            victim_repository,
            protective_measure_repository,
            user_repository,
        }
    }

    pub async fn create_attendance_offender(
        &self,
        attendance: CreateAttendanceOffender,
        req: HttpRequest,
    ) -> Result<HttpResponse, AppError> {
        let claims = extract_claims(&req)?;

        let offender = self.verify_offender_access(&claims, attendance.offender_id).await?;

        let _victim = self.verify_victim_access(&claims, attendance.victim_id).await?;

        if let Some(pm_id) = attendance.protective_measure_id {
            match self.protective_measure_repository.get_protective_measure_by_id(pm_id).await {
                Ok(_) => {},
                Err(sqlx::Error::RowNotFound) => {
                    return Err(AppError::NotFound(format!(
                        "Protective measure with id '{}' not found",
                        pm_id
                    )));
                }
                Err(e) => {
                    error!("[AttendanceOffenderService] Error checking protective measure: {:?}", e);
                    return Err(AppError::InternalServerError);
                }
            }
        }

        let policies = get_user_policies_with_defaults(&**self.user_repository, &claims).await?;
        check_policy(&claims, POLICY_CREATE_ATTENDANCES, offender.city_id, &policies)?;

        match self.attendance_offender_repository.create_attendance_offender(attendance).await {
            Ok(attendance_with_address) => {
                info!(
                    "[AttendanceOffenderService] Attendance offender created: {}",
                    attendance_with_address.id
                );
                Ok(ApiResponse::created(attendance_with_address).into_response())
            }
            Err(e) => {
                error!("[AttendanceOffenderService] Failed to create attendance offender: {:?}", e);
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

        match self.attendance_offender_repository.get_attendance_offender_by_id(id).await {
            Ok(attendance_with_address) => {
                let offender = self
                    .offender_repository
                    .get_offender_by_id(attendance_with_address.offender_id)
                    .await
                    .map_err(|_| AppError::InternalServerError)?;

                let policies = get_user_policies_with_defaults(&**self.user_repository, &claims).await?;
                check_policy(&claims, POLICY_READ_ATTENDANCES, offender.city_id, &policies)?;

                Ok(ApiResponse::success(attendance_with_address).into_response())
            }
            Err(sqlx::Error::RowNotFound) => {
                Err(AppError::NotFound(format!("Attendance offender '{}' not found", id)))
            }
            Err(_) => Err(AppError::InternalServerError),
        }
    }

    pub async fn get_all_attendance_offenders(&self, req: HttpRequest) -> Result<HttpResponse, AppError> {
        let claims = extract_claims(&req)?;

        let policies = get_user_policies_with_defaults(&**self.user_repository, &claims).await?;
        let attendances = if let Some(allowed_cities) = get_allowed_cities_for_policy(&claims, POLICY_READ_ATTENDANCES, &policies) {
            match self.attendance_offender_repository.get_all_attendance_offenders().await {
                Ok(all) => {
                    let mut filtered = Vec::new();
                    for attendance in all {
                        if let Ok(offender) = self.offender_repository.get_offender_by_id(attendance.offender_id).await {
                            if allowed_cities.contains(&offender.city_id) {
                                filtered.push(attendance);
                            }
                        }
                    }
                    Ok(filtered)
                }
                Err(e) => Err(e),
            }
        } else {
            self.attendance_offender_repository.get_all_attendance_offenders().await
        };

        match attendances {
            Ok(attendances_list) => Ok(ApiResponse::success(attendances_list).into_response()),
            Err(_) => Err(AppError::InternalServerError),
        }
    }

    pub async fn get_attendance_offenders_by_offender(
        &self,
        offender_id: Uuid,
        req: HttpRequest,
    ) -> Result<HttpResponse, AppError> {
        let claims = extract_claims(&req)?;
        let offender = self.verify_offender_access(&claims, offender_id).await?;

        let policies = get_user_policies_with_defaults(&**self.user_repository, &claims).await?;
        check_policy(&claims, POLICY_READ_ATTENDANCES, offender.city_id, &policies)?;

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
            .map_err(|_| AppError::InternalServerError)?;

        let policies = get_user_policies_with_defaults(&**self.user_repository, &claims).await?;
        check_policy(&claims, POLICY_UPDATE_ATTENDANCES, existing_offender.city_id, &policies)?;

        if data.offender_id != existing.offender_id {
            let new_offender = self.verify_offender_access(&claims, data.offender_id).await?;
            check_policy(&claims, POLICY_UPDATE_ATTENDANCES, new_offender.city_id, &policies)?;
        }

        if data.victim_id != existing.victim_id {
            let _victim = self.verify_victim_access(&claims, data.victim_id).await?;
        }

        if data.protective_measure_id != existing.protective_measure_id {
            if let Some(pm_id) = data.protective_measure_id {
                match self.protective_measure_repository.get_protective_measure_by_id(pm_id).await {
                    Ok(_) => {},
                    Err(sqlx::Error::RowNotFound) => {
                        return Err(AppError::NotFound(format!(
                            "Protective measure with id '{}' not found",
                            pm_id
                        )));
                    }
                    Err(e) => {
                        error!("[AttendanceOffenderService] Error checking protective measure: {:?}", e);
                        return Err(AppError::InternalServerError);
                    }
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
            Err(sqlx::Error::RowNotFound) => {
                Err(AppError::NotFound(format!("Attendance offender '{}' not found", id)))
            }
            Err(_) => Err(AppError::InternalServerError),
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
            .map_err(|_| AppError::InternalServerError)?;

        let policies = get_user_policies_with_defaults(&**self.user_repository, &claims).await?;
        check_policy(&claims, POLICY_DELETE_ATTENDANCES, offender.city_id, &policies)?;

        match self.attendance_offender_repository.delete_attendance_offender_by_id(id).await {
            Ok(deleted) => Ok(ApiResponse::success(deleted).into_response()),
            Err(_) => Err(AppError::InternalServerError),
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
