use log::{error, info};
use uuid::Uuid;

use crate::core::application_error::ApplicationError as AppError;
use crate::core::contracts::repository::error::RepositoryError;
use crate::core::entities::auth::UserClaims;
use crate::core::entities::protective_measures::ProtectiveMeasureExtension;
use crate::core::value_objects::policies::Policy;
use crate::usecases::extensions::deps::ExtensionUseCaseDependencies;
use crate::usecases::extensions::helpers::load_auth_and_check_extension;

pub struct DeleteExtensionByIdUseCase {
    deps: ExtensionUseCaseDependencies,
}

impl DeleteExtensionByIdUseCase {
    pub fn new(deps: ExtensionUseCaseDependencies) -> Self {
        Self { deps }
    }

    pub async fn execute(
        &self,
        id: Uuid,
        claims: &UserClaims,
    ) -> Result<ProtectiveMeasureExtension, AppError> {
        info!(
            "[DeleteExtensionByIdUseCase] Deleting extension with ID: {}",
            id
        );

        load_auth_and_check_extension(
            &self.deps,
            id,
            &Policy::DeleteProtectiveMeasures,
            claims,
            "DeleteExtensionByIdUseCase",
        )
        .await?;

        match self
            .deps
            .extension_repository
            .delete_extension_by_id(id)
            .await
        {
            Ok(extension) => {
                info!(
                    "[DeleteExtensionByIdUseCase] Extension deleted successfully with ID: {}",
                    id
                );
                Ok(extension)
            }
            Err(RepositoryError::NotFound) => {
                error!("[DeleteExtensionByIdUseCase] Extension not found: {}", id);
                Err(AppError::NotFound("Extension not found".to_string()))
            }
            Err(e) => {
                error!("[DeleteExtensionByIdUseCase] Database error: {:?}", e);
                Err(AppError::InternalServerError)
            }
        }
    }
}
