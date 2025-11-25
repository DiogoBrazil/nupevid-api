use actix_web::{web, HttpMessage, HttpRequest, HttpResponse};
use log::{error, info};
use uuid::Uuid;

use crate::core::contracts::repository::attendances::AttendanceRepository;
use crate::core::contracts::repository::victims::VictimRepository;
use crate::core::entities::attendances::{CreateAttendance, UpdateAttendance};
use crate::core::entities::auth::ClaimsToUserToken;
use crate::core::entities::victims::VictimWithAddress;
use crate::repositories::attendances::PgAttendanceRepository;
use crate::repositories::victims::PgVictimRepository;
use crate::repositories::users::PgUserRepository;
use crate::core::contracts::repository::users::UserRepository;
use crate::utils::{
    errors::AppError,
    responses::ApiResponse,
    validations::{
        PROFILE_ROOT, POLICY_CREATE_ATTENDANCES, POLICY_READ_ATTENDANCES, POLICY_UPDATE_ATTENDANCES, POLICY_DELETE_ATTENDANCES
    }
};
use crate::utils::authorization::{check_policy, get_allowed_cities_for_policy};

pub struct AttendanceService {
    attendance_repository: web::Data<PgAttendanceRepository>,
    victim_repository: web::Data<PgVictimRepository>,
    user_repository: web::Data<PgUserRepository>,
}

impl AttendanceService {
    pub fn new(
        attendance_repository: web::Data<PgAttendanceRepository>,
        victim_repository: web::Data<PgVictimRepository>,
        user_repository: web::Data<PgUserRepository>,
    ) -> Self {
        Self {
            attendance_repository,
            victim_repository,
            user_repository,
        }
    }

    pub async fn create_attendance(
        &self,
        attendance: CreateAttendance,
        req: HttpRequest,
    ) -> Result<HttpResponse, AppError> {
        let claims = self.get_claims(&req)?;
        let victim = self.verify_victim_access(&claims, attendance.victim_id).await?;
        let policies = self.get_user_policies(&claims).await?;
        check_policy(&claims, POLICY_CREATE_ATTENDANCES, victim.city_id, &policies)?;

        match self.attendance_repository.create_attendance(attendance).await {
            Ok(attendance_with_address) => {
                info!(
                    "[AttendanceService] Attendance created: {}",
                    attendance_with_address.id
                );
                Ok(ApiResponse::created(attendance_with_address).into_response())
            }
            Err(e) => {
                error!("[AttendanceService] Failed to create attendance: {:?}", e);
                Err(AppError::InternalServerError)
            }
        }
    }

    pub async fn get_attendance_by_id(
        &self,
        id: Uuid,
        req: HttpRequest,
    ) -> Result<HttpResponse, AppError> {
        let claims = self.get_claims(&req)?;

        match self.attendance_repository.get_attendance_by_id(id).await {
            Ok(attendance_with_address) => {
                let victim = self
                    .victim_repository
                    .get_victim_by_id(attendance_with_address.victim_id)
                    .await
                    .map_err(|_| AppError::InternalServerError)?;
                let policies = self.get_user_policies(&claims).await?;
                check_policy(&claims, POLICY_READ_ATTENDANCES, victim.city_id, &policies)?;
                Ok(ApiResponse::success(attendance_with_address).into_response())
            }
            Err(sqlx::Error::RowNotFound) => {
                Err(AppError::NotFound(format!("Attendance '{}' not found", id)))
            }
            Err(_) => Err(AppError::InternalServerError),
        }
    }

    pub async fn get_all_attendances(&self, req: HttpRequest) -> Result<HttpResponse, AppError> {
        let claims = self.get_claims(&req)?;

        let policies = self.get_user_policies(&claims).await?;
        let attendances = if let Some(allowed_cities) = get_allowed_cities_for_policy(&claims, POLICY_READ_ATTENDANCES, &policies) {
            match self.attendance_repository.get_all_attendances().await {
                Ok(all) => {
                    let mut filtered = Vec::new();
                    for attendance in all {
                        if let Ok(victim) = self.victim_repository.get_victim_by_id(attendance.victim_id).await {
                            if allowed_cities.contains(&victim.city_id) {
                                filtered.push(attendance);
                            }
                        }
                    }
                    Ok(filtered)
                }
                Err(e) => Err(e),
            }
        } else {
            self.attendance_repository.get_all_attendances().await
        };

        match attendances {
            Ok(attendances_list) => Ok(ApiResponse::success(attendances_list).into_response()),
            Err(_) => Err(AppError::InternalServerError),
        }
    }

    pub async fn get_attendances_by_victim(
        &self,
        victim_id: Uuid,
        req: HttpRequest,
    ) -> Result<HttpResponse, AppError> {
        let claims = self.get_claims(&req)?;
        let victim = self.verify_victim_access(&claims, victim_id).await?;
        let policies = self.get_user_policies(&claims).await?;
        check_policy(&claims, POLICY_READ_ATTENDANCES, victim.city_id, &policies)?;

        match self
            .attendance_repository
            .get_attendances_by_victim(victim_id)
            .await
        {
            Ok(attendances_list) => Ok(ApiResponse::success(attendances_list).into_response()),
            Err(_) => Err(AppError::InternalServerError),
        }
    }

    pub async fn update_attendance_by_id(
        &self,
        data: UpdateAttendance,
        id: Uuid,
        req: HttpRequest,
    ) -> Result<HttpResponse, AppError> {
        let claims = self.get_claims(&req)?;

        let existing = self
            .attendance_repository
            .get_attendance_by_id(id)
            .await
            .map_err(|e| {
                if matches!(e, sqlx::Error::RowNotFound) {
                    AppError::NotFound(format!("Attendance '{}' not found", id))
                } else {
                    AppError::InternalServerError
                }
            })?;

        let existing_victim = self
            .victim_repository
            .get_victim_by_id(existing.victim_id)
            .await
            .map_err(|_| AppError::InternalServerError)?;
        let policies = self.get_user_policies(&claims).await?;
        check_policy(&claims, POLICY_UPDATE_ATTENDANCES, existing_victim.city_id, &policies)?;

        if data.victim_id != existing.victim_id {
            let new_victim = self.verify_victim_access(&claims, data.victim_id).await?;
            check_policy(&claims, POLICY_UPDATE_ATTENDANCES, new_victim.city_id, &policies)?;
        }

        match self
            .attendance_repository
            .update_attendance_by_id(data, id)
            .await
        {
            Ok(attendance_with_address) => {
                Ok(ApiResponse::success(attendance_with_address).into_response())
            }
            Err(sqlx::Error::RowNotFound) => {
                Err(AppError::NotFound(format!("Attendance '{}' not found", id)))
            }
            Err(_) => Err(AppError::InternalServerError),
        }
    }

    pub async fn delete_attendance_by_id(
        &self,
        id: Uuid,
        req: HttpRequest,
    ) -> Result<HttpResponse, AppError> {
        let claims = self.get_claims(&req)?;

        let attendance = self
            .attendance_repository
            .get_attendance_by_id(id)
            .await
            .map_err(|e| {
                if matches!(e, sqlx::Error::RowNotFound) {
                    AppError::NotFound(format!("Attendance '{}' not found", id))
                } else {
                    AppError::InternalServerError
                }
            })?;

        let victim = self
            .victim_repository
            .get_victim_by_id(attendance.victim_id)
            .await
            .map_err(|_| AppError::InternalServerError)?;
        let policies = self.get_user_policies(&claims).await?;
        check_policy(&claims, POLICY_DELETE_ATTENDANCES, victim.city_id, &policies)?;

        match self.attendance_repository.delete_attendance_by_id(id).await {
            Ok(deleted) => Ok(ApiResponse::success(deleted).into_response()),
            Err(_) => Err(AppError::InternalServerError),
        }
    }

    // Helper methods
    fn get_claims(&self, req: &HttpRequest) -> Result<ClaimsToUserToken, AppError> {
        req.extensions()
            .get::<ClaimsToUserToken>()
            .cloned()
            .ok_or_else(|| AppError::Unauthorized("Unauthorized".to_string()))
    }

    async fn get_user_policies(&self, claims: &ClaimsToUserToken) -> Result<serde_json::Value, AppError> {
        if claims.profile == PROFILE_ROOT {
            return Ok(serde_json::json!({}));
        }
        if let Some(city_id_str) = &claims.city_id {
            if let Ok(_cid) = Uuid::parse_str(city_id_str) {
                if let Ok(uid) = Uuid::parse_str(&claims.id) {
                    match self.user_repository.get_user_policies_json_by_id(uid).await {
                        Ok(p) => return Ok(p),
                        Err(sqlx::Error::RowNotFound) => {}
                        Err(_) => return Err(AppError::InternalServerError),
                    }
                }
                let defaults = crate::utils::validations::generate_default_policies(&claims.profile, Uuid::parse_str(city_id_str).ok());
                return Ok(serde_json::json!(defaults));
            }
        }
        Ok(serde_json::json!({}))
    }

    async fn verify_victim_access(
        &self,
        _claims: &ClaimsToUserToken,
        victim_id: Uuid,
    ) -> Result<VictimWithAddress, AppError> {
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
