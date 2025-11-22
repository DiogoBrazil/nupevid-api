use actix_web::{web, HttpResponse, HttpRequest, HttpMessage};
use log::{error, info};
use uuid::Uuid;

use crate::core::entities::attendances::{
    CreateAttendance,
    UpdateAttendance,
    CreateAttendanceAddress,
    UpdateAttendanceAddress
};
use crate::core::entities::auth::ClaimsToUserToken;
use crate::core::contracts::repository::attendances::{
    AttendanceRepository,
    AttendanceAddressRepository
};
use crate::core::contracts::repository::victims::VictimRepository;
use crate::repositories::attendances::{
    PgAttendanceRepository,
    PgAttendanceAddressRepository
};
use crate::repositories::victims::PgVictimRepository;
use crate::utils::{errors::AppError, responses::ApiResponse};

pub struct AttendanceService {
    attendance_repository: web::Data<PgAttendanceRepository>,
    attendance_address_repository: web::Data<PgAttendanceAddressRepository>,
    victim_repository: web::Data<PgVictimRepository>,
}

impl AttendanceService {
    pub fn new(
        attendance_repository: web::Data<PgAttendanceRepository>,
        attendance_address_repository: web::Data<PgAttendanceAddressRepository>,
        victim_repository: web::Data<PgVictimRepository>,
    ) -> Self {
        Self { attendance_repository, attendance_address_repository, victim_repository }
    }

    pub async fn create_attendance(&self, attendance: CreateAttendance, req: HttpRequest) -> Result<HttpResponse, AppError> {
        let claims = self.get_claims(&req)?;
        let victim = self.verify_victim_access(&claims, attendance.victim_id).await?;
        self.validate_city_access(&claims, &victim.city_id)?;

        match self.attendance_repository.create_attendance(attendance).await {
            Ok(attendance) => {
                info!("[AttendanceService] Attendance created: {}", attendance.id);
                Ok(ApiResponse::created(attendance).into_response())
            }
            Err(e) => {
                error!("[AttendanceService] Failed to create attendance: {:?}", e);
                Err(AppError::InternalServerError)
            }
        }
    }

    pub async fn get_attendance_by_id(&self, id: Uuid, req: HttpRequest) -> Result<HttpResponse, AppError> {
        let claims = self.get_claims(&req)?;
        match self.attendance_repository.get_attendance_by_id(id).await {
            Ok(attendance) => {
                let victim = self.victim_repository.get_victim_by_id(attendance.victim_id).await
                    .map_err(|_| AppError::InternalServerError)?;
                self.validate_city_access(&claims, &victim.city_id)?;
                Ok(ApiResponse::success(attendance).into_response())
            }
            Err(sqlx::Error::RowNotFound) => Err(AppError::NotFound(format!("Attendance '{}' not found", id))),
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
                        if let Ok(victim) = self.victim_repository.get_victim_by_id(attendance.victim_id).await {
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
            Ok(attendances) => Ok(ApiResponse::success(attendances).into_response()),
            Err(_) => Err(AppError::InternalServerError),
        }
    }

    pub async fn get_attendances_by_victim(&self, victim_id: Uuid, req: HttpRequest) -> Result<HttpResponse, AppError> {
        let claims = self.get_claims(&req)?;
        let victim = self.verify_victim_access(&claims, victim_id).await?;
        self.validate_city_access(&claims, &victim.city_id)?;

        match self.attendance_repository.get_attendances_by_victim(victim_id).await {
            Ok(attendances) => Ok(ApiResponse::success(attendances).into_response()),
            Err(_) => Err(AppError::InternalServerError),
        }
    }

    pub async fn update_attendance_by_id(&self, data: UpdateAttendance, id: Uuid, req: HttpRequest) -> Result<HttpResponse, AppError> {
        let claims = self.get_claims(&req)?;
        let existing = self.attendance_repository.get_attendance_by_id(id).await
            .map_err(|e| if matches!(e, sqlx::Error::RowNotFound) { AppError::NotFound(format!("Attendance '{}' not found", id)) } else { AppError::InternalServerError })?;

        let existing_victim = self.victim_repository.get_victim_by_id(existing.victim_id).await
            .map_err(|_| AppError::InternalServerError)?;
        self.validate_city_access(&claims, &existing_victim.city_id)?;

        if data.victim_id != existing.victim_id {
            let new_victim = self.verify_victim_access(&claims, data.victim_id).await?;
            self.validate_city_access(&claims, &new_victim.city_id)?;
        }

        match self.attendance_repository.update_attendance_by_id(data, id).await {
            Ok(attendance) => Ok(ApiResponse::success(attendance).into_response()),
            Err(sqlx::Error::RowNotFound) => Err(AppError::NotFound(format!("Attendance '{}' not found", id))),
            Err(_) => Err(AppError::InternalServerError),
        }
    }

    pub async fn delete_attendance_by_id(&self, id: Uuid, req: HttpRequest) -> Result<HttpResponse, AppError> {
        let claims = self.get_claims(&req)?;
        let attendance = self.attendance_repository.get_attendance_by_id(id).await
            .map_err(|e| if matches!(e, sqlx::Error::RowNotFound) { AppError::NotFound(format!("Attendance '{}' not found", id)) } else { AppError::InternalServerError })?;

        let victim = self.victim_repository.get_victim_by_id(attendance.victim_id).await
            .map_err(|_| AppError::InternalServerError)?;
        self.validate_city_access(&claims, &victim.city_id)?;

        match self.attendance_repository.delete_attendance_by_id(id).await {
            Ok(deleted) => Ok(ApiResponse::success(deleted).into_response()),
            Err(_) => Err(AppError::InternalServerError),
        }
    }

    pub async fn create_attendance_address(&self, address: CreateAttendanceAddress, req: HttpRequest) -> Result<HttpResponse, AppError> {
        let claims = self.get_claims(&req)?;
        let attendance = self.attendance_repository.get_attendance_by_id(address.attendance_id).await
            .map_err(|e| if matches!(e, sqlx::Error::RowNotFound) { AppError::NotFound(format!("Attendance '{}' not found", address.attendance_id)) } else { AppError::InternalServerError })?;

        let victim = self.verify_victim_access(&claims, attendance.victim_id).await?;
        self.validate_city_access(&claims, &victim.city_id)?;

        match self.attendance_address_repository.create_attendance_address(address).await {
            Ok(address) => Ok(ApiResponse::created(address).into_response()),
            Err(_) => Err(AppError::InternalServerError),
        }
    }

    pub async fn get_attendance_address(&self, attendance_id: Uuid, req: HttpRequest) -> Result<HttpResponse, AppError> {
        let claims = self.get_claims(&req)?;
        let attendance = self.attendance_repository.get_attendance_by_id(attendance_id).await
            .map_err(|e| if matches!(e, sqlx::Error::RowNotFound) { AppError::NotFound(format!("Attendance '{}' not found", attendance_id)) } else { AppError::InternalServerError })?;

        let victim = self.verify_victim_access(&claims, attendance.victim_id).await?;
        self.validate_city_access(&claims, &victim.city_id)?;

        match self.attendance_address_repository.get_attendance_address_by_attendance_id(attendance_id).await {
            Ok(Some(address)) => Ok(ApiResponse::success(address).into_response()),
            Ok(None) => Err(AppError::NotFound(format!("No address found for attendance '{}'", attendance_id))),
            Err(_) => Err(AppError::InternalServerError),
        }
    }

    pub async fn update_attendance_address(&self, data: UpdateAttendanceAddress, address_id: Uuid, req: HttpRequest) -> Result<HttpResponse, AppError> {
        let claims = self.get_claims(&req)?;
        let address = self.attendance_address_repository.get_attendance_address_by_id(address_id).await
            .map_err(|e| if matches!(e, sqlx::Error::RowNotFound) { AppError::NotFound(format!("Address '{}' not found", address_id)) } else { AppError::InternalServerError })?;

        let attendance = self.attendance_repository.get_attendance_by_id(address.attendance_id).await
            .map_err(|_| AppError::InternalServerError)?;
        let victim = self.victim_repository.get_victim_by_id(attendance.victim_id).await
            .map_err(|_| AppError::InternalServerError)?;
        self.validate_city_access(&claims, &victim.city_id)?;

        match self.attendance_address_repository.update_attendance_address_by_id(data, address_id).await {
            Ok(updated) => Ok(ApiResponse::success(updated).into_response()),
            Err(_) => Err(AppError::InternalServerError),
        }
    }

    // Helper methods
    fn get_claims(&self, req: &HttpRequest) -> Result<ClaimsToUserToken, AppError> {
        req.extensions().get::<ClaimsToUserToken>().cloned()
            .ok_or_else(|| AppError::Unauthorized("Unauthorized".to_string()))
    }

    fn get_user_city_id(&self, claims: &ClaimsToUserToken) -> Result<Uuid, AppError> {
        claims.city_id.as_ref().and_then(|id| Uuid::parse_str(id).ok())
            .ok_or_else(|| AppError::Forbidden("User must be associated with a city".to_string()))
    }

    fn validate_city_access(&self, claims: &ClaimsToUserToken, city_id: &Uuid) -> Result<(), AppError> {
        if claims.profile == "ROOT" { return Ok(()); }
        let user_city_id = self.get_user_city_id(claims)?;
        if &user_city_id != city_id {
            return Err(AppError::Forbidden("Access denied to this city's data".to_string()));
        }
        Ok(())
    }

    async fn verify_victim_access(&self, _claims: &ClaimsToUserToken, victim_id: Uuid) -> Result<crate::core::entities::victims::Victim, AppError> {
        self.victim_repository.get_victim_by_id(victim_id).await
            .map_err(|e| if matches!(e, sqlx::Error::RowNotFound) { AppError::NotFound(format!("Victim '{}' not found", victim_id)) } else { AppError::InternalServerError })
    }
}
