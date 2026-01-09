use actix_web::{web, HttpRequest, HttpResponse};
use log::{error, info};
use uuid::Uuid;

use crate::core::contracts::repository::extensions::ExtensionRepository;
use crate::core::contracts::repository::protective_measures::ProtectiveMeasureRepository;
use crate::core::contracts::repository::victims::VictimRepository;
use crate::core::entities::protective_measures::{CreateExtension, UpdateExtension};
use crate::repositories::extensions::PgExtensionRepository;
use crate::repositories::protective_measures::PgProtectiveMeasureRepository;
use crate::repositories::victims::PgVictimRepository;
use crate::repositories::users::PgUserRepository;
use crate::utils::{
    errors::AppError,
    responses::ApiResponse,
    authorization::check_policy,
    service_helpers::{extract_claims, get_user_policies_with_defaults},
    db_error_mapper::map_constraint
};
use crate::validators::common::{
    POLICY_CREATE_PROTECTIVE_MEASURES, POLICY_READ_PROTECTIVE_MEASURES,
    POLICY_UPDATE_PROTECTIVE_MEASURES, POLICY_DELETE_PROTECTIVE_MEASURES
};

pub struct ExtensionService {
    extension_repository: web::Data<PgExtensionRepository>,
    protective_measure_repository: web::Data<PgProtectiveMeasureRepository>,
    victim_repository: web::Data<PgVictimRepository>,
    user_repository: web::Data<PgUserRepository>,
}

impl ExtensionService {
    pub fn new(
        extension_repository: web::Data<PgExtensionRepository>,
        protective_measure_repository: web::Data<PgProtectiveMeasureRepository>,
        victim_repository: web::Data<PgVictimRepository>,
        user_repository: web::Data<PgUserRepository>,
    ) -> Self {
        Self {
            extension_repository,
            protective_measure_repository,
            victim_repository,
            user_repository,
        }
    }

    pub async fn create_extension(
        &self,
        protective_measure_id: Uuid,
        data: CreateExtension,
        req: HttpRequest,
    ) -> Result<HttpResponse, AppError> {
        info!("[Service] Creating extension {} for protective measure: {}", data.extension_number, protective_measure_id);

        let claims = extract_claims(&req)?;

        let protective_measure = match self.protective_measure_repository.get_protective_measure_by_id(protective_measure_id).await {
            Ok(pm) => {
                info!("[Service] Protective measure found");
                pm
            }
            Err(sqlx::Error::RowNotFound) => {
                error!("[Service] Protective measure not found: {}", protective_measure_id);
                return Err(AppError::NotFound("Protective measure not found".to_string()));
            }
            Err(e) => {
                error!("[Service] Database error checking protective measure: {:?}", e);
                return Err(AppError::InternalServerError);
            }
        };

        let victim = match self.victim_repository.get_victim_by_id(protective_measure.victim_id).await {
            Ok(v) => v,
            Err(sqlx::Error::RowNotFound) => {
                error!("[Service] Victim not found: {}", protective_measure.victim_id);
                return Err(AppError::NotFound(format!("Victim with id '{}' not found", protective_measure.victim_id)));
            }
            Err(e) => {
                error!("[Service] Error checking victim: {:?}", e);
                return Err(AppError::InternalServerError);
            }
        };

        let policies = get_user_policies_with_defaults(self.user_repository.as_ref(), &claims).await?;
        check_policy(&claims, POLICY_CREATE_PROTECTIVE_MEASURES, victim.city_id, &policies)?;

        match self.extension_repository.create_extension(protective_measure_id, data).await {
            Ok(extension) => {
                info!("[Service] Extension created successfully with ID: {}", extension.id);
                Ok(ApiResponse::created(extension).into_response())
            }
            Err(e) => {
                if let sqlx::Error::Database(db_err) = &e {
                    if let Some(app_err) = map_constraint(db_err.constraint(), &[
                        ("fk_extensions_protective_measure", "Error adding extension: protective_measure_id not found"),
                    ]) {
                        return Err(app_err);
                    }
                }
                error!("[Service] Error creating extension: {:?}", e);
                Err(AppError::InternalServerError)
            }
        }
    }

    pub async fn get_extension_by_id(&self, id: Uuid, req: HttpRequest) -> Result<HttpResponse, AppError> {
        info!("[Service] Getting extension with ID: {}", id);

        let claims = extract_claims(&req)?;

        let extension = match self.extension_repository.get_extension_by_id(id).await {
            Ok(ext) => ext,
            Err(sqlx::Error::RowNotFound) => {
                error!("[Service] Extension not found with ID: {}", id);
                return Err(AppError::NotFound("Extension not found".to_string()));
            }
            Err(e) => {
                error!("[Service] Database error: {:?}", e);
                return Err(AppError::InternalServerError);
            }
        };

        let protective_measure = self.protective_measure_repository.get_protective_measure_by_id(extension.protective_measure_id).await
            .map_err(|e| {
                error!("[Service] Error fetching protective measure: {:?}", e);
                AppError::InternalServerError
            })?;

        let victim = self.victim_repository.get_victim_by_id(protective_measure.victim_id).await
            .map_err(|e| {
                error!("[Service] Error fetching victim: {:?}", e);
                AppError::InternalServerError
            })?;

        let policies = get_user_policies_with_defaults(&**self.user_repository, &claims).await?;
        check_policy(&claims, POLICY_READ_PROTECTIVE_MEASURES, victim.city_id, &policies)?;

        info!("[Service] Extension found with ID: {}", id);
        Ok(ApiResponse::success(extension).into_response())
    }

    pub async fn get_extensions_by_measure(
        &self,
        protective_measure_id: Uuid,
        req: HttpRequest,
    ) -> Result<HttpResponse, AppError> {
        info!("[Service] Getting extensions for protective measure: {}", protective_measure_id);

        let claims = extract_claims(&req)?;

        let protective_measure = match self.protective_measure_repository.get_protective_measure_by_id(protective_measure_id).await {
            Ok(pm) => pm,
            Err(sqlx::Error::RowNotFound) => {
                error!("[Service] Protective measure not found: {}", protective_measure_id);
                return Err(AppError::NotFound("Protective measure not found".to_string()));
            }
            Err(e) => {
                error!("[Service] Database error: {:?}", e);
                return Err(AppError::InternalServerError);
            }
        };

        let victim = self.victim_repository.get_victim_by_id(protective_measure.victim_id).await
            .map_err(|e| {
                error!("[Service] Error fetching victim: {:?}", e);
                AppError::InternalServerError
            })?;

        let policies = get_user_policies_with_defaults(&**self.user_repository, &claims).await?;
        check_policy(&claims, POLICY_READ_PROTECTIVE_MEASURES, victim.city_id, &policies)?;

        match self.extension_repository.get_extensions_by_measure(protective_measure_id).await {
            Ok(extensions) => {
                info!("[Service] Found {} extensions for protective measure: {}", extensions.len(), protective_measure_id);
                Ok(ApiResponse::success(extensions).into_response())
            }
            Err(e) => {
                error!("[Service] Database error: {:?}", e);
                Err(AppError::InternalServerError)
            }
        }
    }

    pub async fn update_extension_by_id(
        &self,
        id: Uuid,
        data: UpdateExtension,
        req: HttpRequest,
    ) -> Result<HttpResponse, AppError> {
        info!("[Service] Updating extension with ID: {}", id);

        let claims = extract_claims(&req)?;

        let extension = match self.extension_repository.get_extension_by_id(id).await {
            Ok(ext) => {
                info!("[Service] Extension found");
                ext
            }
            Err(sqlx::Error::RowNotFound) => {
                error!("[Service] Extension not found: {}", id);
                return Err(AppError::NotFound("Extension not found".to_string()));
            }
            Err(e) => {
                error!("[Service] Database error: {:?}", e);
                return Err(AppError::InternalServerError);
            }
        };

        let protective_measure = self.protective_measure_repository.get_protective_measure_by_id(extension.protective_measure_id).await
            .map_err(|e| {
                error!("[Service] Error fetching protective measure: {:?}", e);
                AppError::InternalServerError
            })?;

        let victim = self.victim_repository.get_victim_by_id(protective_measure.victim_id).await
            .map_err(|e| {
                error!("[Service] Error fetching victim: {:?}", e);
                AppError::InternalServerError
            })?;

        let policies = get_user_policies_with_defaults(&**self.user_repository, &claims).await?;
        check_policy(&claims, POLICY_UPDATE_PROTECTIVE_MEASURES, victim.city_id, &policies)?;

        match self.extension_repository.update_extension_by_id(data, id).await {
            Ok(extension) => {
                info!("[Service] Extension updated successfully with ID: {}", id);
                Ok(ApiResponse::success(extension).into_response())
            }
            Err(e) => {
                error!("[Service] Error updating extension: {:?}", e);
                Err(AppError::InternalServerError)
            }
        }
    }

    pub async fn delete_extension_by_id(&self, id: Uuid, req: HttpRequest) -> Result<HttpResponse, AppError> {
        info!("[Service] Deleting extension with ID: {}", id);

        let claims = extract_claims(&req)?;

        let extension = match self.extension_repository.get_extension_by_id(id).await {
            Ok(ext) => ext,
            Err(sqlx::Error::RowNotFound) => {
                error!("[Service] Extension not found: {}", id);
                return Err(AppError::NotFound("Extension not found".to_string()));
            }
            Err(e) => {
                error!("[Service] Database error: {:?}", e);
                return Err(AppError::InternalServerError);
            }
        };

        let protective_measure = self.protective_measure_repository.get_protective_measure_by_id(extension.protective_measure_id).await
            .map_err(|e| {
                error!("[Service] Error fetching protective measure: {:?}", e);
                AppError::InternalServerError
            })?;

        let victim = self.victim_repository.get_victim_by_id(protective_measure.victim_id).await
            .map_err(|e| {
                error!("[Service] Error fetching victim: {:?}", e);
                AppError::InternalServerError
            })?;

        let policies = get_user_policies_with_defaults(&**self.user_repository, &claims).await?;
        check_policy(&claims, POLICY_DELETE_PROTECTIVE_MEASURES, victim.city_id, &policies)?;

        match self.extension_repository.delete_extension_by_id(id).await {
            Ok(extension) => {
                info!("[Service] Extension deleted successfully with ID: {}", id);
                Ok(ApiResponse::success(extension).into_response())
            }
            Err(sqlx::Error::RowNotFound) => {
                error!("[Service] Extension not found: {}", id);
                Err(AppError::NotFound("Extension not found".to_string()))
            }
            Err(e) => {
                error!("[Service] Database error: {:?}", e);
                Err(AppError::InternalServerError)
            }
        }
    }
}
