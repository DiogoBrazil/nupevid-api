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
use crate::utils::{errors::AppError, responses::ApiResponse};

pub struct AttendanceService {
    attendance_repository: web::Data<PgAttendanceRepository>,
    victim_repository: web::Data<PgVictimRepository>,
}

impl AttendanceService {
    pub fn new(
        attendance_repository: web::Data<PgAttendanceRepository>,
        victim_repository: web::Data<PgVictimRepository>,
    ) -> Self {
        Self {
            attendance_repository,
            victim_repository,
        }
    }

    pub async fn create_attendance(
        &self,
        attendance: CreateAttendance,
        req: HttpRequest,
    ) -> Result<HttpResponse, AppError> {
        let claims = self.get_claims(&req)?;
        let victim = self.verify_victim_access(&claims, attendance.victim_id).await?;
        self.validate_city_access(&claims, &victim.city_id)?;

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
                self.validate_city_access(&claims, &victim.city_id)?;
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

        let attendances = if claims.profile == "ROOT" {
            self.attendance_repository.get_all_attendances().await
        } else {
            match self.attendance_repository.get_all_attendances().await {
                Ok(all) => {
                    let city_id = self.get_user_city_id(&claims)?;
                    let mut filtered = Vec::new();
                    for attendance in all {
                        if let Ok(victim) = self
                            .victim_repository
                            .get_victim_by_id(attendance.victim_id)
                            .await
                        {
                            if victim.city_id == city_id {
                                filtered.push(attendance);
                            }
                        }
                    }
                    Ok(filtered)
                }
                Err(e) => Err(e),
            }
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
        self.validate_city_access(&claims, &victim.city_id)?;

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
        self.validate_city_access(&claims, &existing_victim.city_id)?;

        if data.victim_id != existing.victim_id {
            let new_victim = self.verify_victim_access(&claims, data.victim_id).await?;
            self.validate_city_access(&claims, &new_victim.city_id)?;
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
        self.validate_city_access(&claims, &victim.city_id)?;

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

    fn get_user_city_id(&self, claims: &ClaimsToUserToken) -> Result<Uuid, AppError> {
        claims
            .city_id
            .as_ref()
            .and_then(|id| Uuid::parse_str(id).ok())
            .ok_or_else(|| AppError::Forbidden("User must be associated with a city".to_string()))
    }

    fn validate_city_access(
        &self,
        claims: &ClaimsToUserToken,
        city_id: &Uuid,
    ) -> Result<(), AppError> {
        if claims.profile == "ROOT" {
            return Ok(());
        }
        let user_city_id = self.get_user_city_id(claims)?;
        if &user_city_id != city_id {
            return Err(AppError::Forbidden(
                "Access denied to this city's data".to_string(),
            ));
        }
        Ok(())
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
