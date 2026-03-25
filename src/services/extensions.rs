use log::{error, info};
use std::sync::Arc;
use uuid::Uuid;

use crate::core::commands::protective_measures::{CreateExtension, UpdateExtension};
use crate::core::contracts::repository::extensions::ExtensionRepository;
use crate::core::contracts::repository::protective_measures::ProtectiveMeasureReadRepository;
use crate::core::contracts::repository::users::UserRepository;
use crate::core::contracts::repository::victims::VictimReadRepository;
use crate::core::entities::auth::ClaimsToUserToken;
use crate::core::entities::protective_measures::ProtectiveMeasureExtension;
use crate::utils::errors::AppError;
use crate::core::contracts::repository::error::RepositoryError;
use crate::services::auth_context::AuthContext;
use crate::services::error_mapping::map_constraint;
use crate::validators::common::{
    POLICY_CREATE_PROTECTIVE_MEASURES, POLICY_DELETE_PROTECTIVE_MEASURES,
    POLICY_READ_PROTECTIVE_MEASURES, POLICY_UPDATE_PROTECTIVE_MEASURES,
};

pub struct ExtensionService {
    extension_repository: Arc<dyn ExtensionRepository>,
    protective_measure_repository: Arc<dyn ProtectiveMeasureReadRepository>,
    victim_repository: Arc<dyn VictimReadRepository>,
    user_repository: Arc<dyn UserRepository>,
}

impl ExtensionService {
    pub fn new(
        extension_repository: Arc<dyn ExtensionRepository>,
        protective_measure_repository: Arc<dyn ProtectiveMeasureReadRepository>,
        victim_repository: Arc<dyn VictimReadRepository>,
        user_repository: Arc<dyn UserRepository>,
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
        claims: &ClaimsToUserToken,
    ) -> Result<ProtectiveMeasureExtension, AppError> {
        info!(
            "[Service] Creating extension {} for protective measure: {}",
            data.extension_number, protective_measure_id
        );

        let protective_measure = match self
            .protective_measure_repository
            .get_protective_measure_by_id(protective_measure_id)
            .await
        {
            Ok(pm) => {
                info!("[Service] Protective measure found");
                pm
            }
            Err(RepositoryError::NotFound) => {
                error!(
                    "[Service] Protective measure not found: {}",
                    protective_measure_id
                );
                return Err(AppError::NotFound(
                    "Protective measure not found".to_string(),
                ));
            }
            Err(e) => {
                error!(
                    "[Service] Database error checking protective measure: {:?}",
                    e
                );
                return Err(AppError::InternalServerError);
            }
        };

        let victim = match self
            .victim_repository
            .get_victim_by_id(protective_measure.victim_id)
            .await
        {
            Ok(v) => v,
            Err(RepositoryError::NotFound) => {
                error!(
                    "[Service] Victim not found: {}",
                    protective_measure.victim_id
                );
                return Err(AppError::NotFound(format!(
                    "Victim with id '{}' not found",
                    protective_measure.victim_id
                )));
            }
            Err(e) => {
                error!("[Service] Error checking victim: {:?}", e);
                return Err(AppError::InternalServerError);
            }
        };

        let auth = AuthContext::load(self.user_repository.as_ref(), claims).await?;
        auth.check_policy(POLICY_CREATE_PROTECTIVE_MEASURES, victim.city_id)?;

        match self
            .extension_repository
            .create_extension(protective_measure_id, data)
            .await
        {
            Ok(extension) => {
                info!(
                    "[Service] Extension created successfully with ID: {}",
                    extension.id
                );
                Ok(extension)
            }
            Err(e) => {
                if let RepositoryError::UniqueViolation { constraint }
                | RepositoryError::ForeignKeyViolation { constraint } = &e
                    && let Some(app_err) = map_constraint(
                        constraint.as_deref(),
                        &[(
                            "fk_extensions_protective_measure",
                            "Error adding extension: protective_measure_id not found",
                        )],
                    )
                {
                    return Err(app_err);
                }
                error!("[Service] Error creating extension: {:?}", e);
                Err(AppError::InternalServerError)
            }
        }
    }

    pub async fn get_extension_by_id(
        &self,
        id: Uuid,
        claims: &ClaimsToUserToken,
    ) -> Result<ProtectiveMeasureExtension, AppError> {
        info!("[Service] Getting extension with ID: {}", id);

        let extension = match self.extension_repository.get_extension_by_id(id).await {
            Ok(ext) => ext,
            Err(RepositoryError::NotFound) => {
                error!("[Service] Extension not found with ID: {}", id);
                return Err(AppError::NotFound("Extension not found".to_string()));
            }
            Err(e) => {
                error!("[Service] Database error: {:?}", e);
                return Err(AppError::InternalServerError);
            }
        };

        let protective_measure = self
            .protective_measure_repository
            .get_protective_measure_by_id(extension.protective_measure_id)
            .await
            .map_err(|e| {
                error!("[Service] Error fetching protective measure: {:?}", e);
                AppError::InternalServerError
            })?;

        let victim = self
            .victim_repository
            .get_victim_by_id(protective_measure.victim_id)
            .await
            .map_err(|e| {
                error!("[Service] Error fetching victim: {:?}", e);
                AppError::InternalServerError
            })?;

        let auth = AuthContext::load(&*self.user_repository, claims).await?;
        auth.check_policy(POLICY_READ_PROTECTIVE_MEASURES, victim.city_id)?;

        info!("[Service] Extension found with ID: {}", id);
        Ok(extension)
    }

    pub async fn get_extensions_by_measure(
        &self,
        protective_measure_id: Uuid,
        claims: &ClaimsToUserToken,
    ) -> Result<Vec<ProtectiveMeasureExtension>, AppError> {
        info!(
            "[Service] Getting extensions for protective measure: {}",
            protective_measure_id
        );

        let protective_measure = match self
            .protective_measure_repository
            .get_protective_measure_by_id(protective_measure_id)
            .await
        {
            Ok(pm) => pm,
            Err(RepositoryError::NotFound) => {
                error!(
                    "[Service] Protective measure not found: {}",
                    protective_measure_id
                );
                return Err(AppError::NotFound(
                    "Protective measure not found".to_string(),
                ));
            }
            Err(e) => {
                error!("[Service] Database error: {:?}", e);
                return Err(AppError::InternalServerError);
            }
        };

        let victim = self
            .victim_repository
            .get_victim_by_id(protective_measure.victim_id)
            .await
            .map_err(|e| {
                error!("[Service] Error fetching victim: {:?}", e);
                AppError::InternalServerError
            })?;

        let auth = AuthContext::load(&*self.user_repository, claims).await?;
        auth.check_policy(POLICY_READ_PROTECTIVE_MEASURES, victim.city_id)?;

        match self
            .extension_repository
            .get_extensions_by_measure(protective_measure_id)
            .await
        {
            Ok(extensions) => {
                info!(
                    "[Service] Found {} extensions for protective measure: {}",
                    extensions.len(),
                    protective_measure_id
                );
                Ok(extensions)
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
        claims: &ClaimsToUserToken,
    ) -> Result<ProtectiveMeasureExtension, AppError> {
        info!("[Service] Updating extension with ID: {}", id);

        let extension = match self.extension_repository.get_extension_by_id(id).await {
            Ok(ext) => {
                info!("[Service] Extension found");
                ext
            }
            Err(RepositoryError::NotFound) => {
                error!("[Service] Extension not found: {}", id);
                return Err(AppError::NotFound("Extension not found".to_string()));
            }
            Err(e) => {
                error!("[Service] Database error: {:?}", e);
                return Err(AppError::InternalServerError);
            }
        };

        let protective_measure = self
            .protective_measure_repository
            .get_protective_measure_by_id(extension.protective_measure_id)
            .await
            .map_err(|e| {
                error!("[Service] Error fetching protective measure: {:?}", e);
                AppError::InternalServerError
            })?;

        let victim = self
            .victim_repository
            .get_victim_by_id(protective_measure.victim_id)
            .await
            .map_err(|e| {
                error!("[Service] Error fetching victim: {:?}", e);
                AppError::InternalServerError
            })?;

        let auth = AuthContext::load(&*self.user_repository, claims).await?;
        auth.check_policy(POLICY_UPDATE_PROTECTIVE_MEASURES, victim.city_id)?;

        match self
            .extension_repository
            .update_extension_by_id(data, id)
            .await
        {
            Ok(extension) => {
                info!("[Service] Extension updated successfully with ID: {}", id);
                Ok(extension)
            }
            Err(e) => {
                error!("[Service] Error updating extension: {:?}", e);
                Err(AppError::InternalServerError)
            }
        }
    }

    pub async fn delete_extension_by_id(
        &self,
        id: Uuid,
        claims: &ClaimsToUserToken,
    ) -> Result<ProtectiveMeasureExtension, AppError> {
        info!("[Service] Deleting extension with ID: {}", id);

        let extension = match self.extension_repository.get_extension_by_id(id).await {
            Ok(ext) => ext,
            Err(RepositoryError::NotFound) => {
                error!("[Service] Extension not found: {}", id);
                return Err(AppError::NotFound("Extension not found".to_string()));
            }
            Err(e) => {
                error!("[Service] Database error: {:?}", e);
                return Err(AppError::InternalServerError);
            }
        };

        let protective_measure = self
            .protective_measure_repository
            .get_protective_measure_by_id(extension.protective_measure_id)
            .await
            .map_err(|e| {
                error!("[Service] Error fetching protective measure: {:?}", e);
                AppError::InternalServerError
            })?;

        let victim = self
            .victim_repository
            .get_victim_by_id(protective_measure.victim_id)
            .await
            .map_err(|e| {
                error!("[Service] Error fetching victim: {:?}", e);
                AppError::InternalServerError
            })?;

        let auth = AuthContext::load(&*self.user_repository, claims).await?;
        auth.check_policy(POLICY_DELETE_PROTECTIVE_MEASURES, victim.city_id)?;

        match self.extension_repository.delete_extension_by_id(id).await {
            Ok(extension) => {
                info!("[Service] Extension deleted successfully with ID: {}", id);
                Ok(extension)
            }
            Err(RepositoryError::NotFound) => {
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
