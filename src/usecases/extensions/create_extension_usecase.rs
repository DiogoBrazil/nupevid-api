use log::{error, info};
use uuid::Uuid;

use crate::core::application_error::ApplicationError as AppError;
use crate::core::commands::protective_measures::CreateExtension;
use crate::core::contracts::repository::error::RepositoryError;
use crate::core::entities::auth::UserClaims;
use crate::core::entities::protective_measures::ProtectiveMeasureExtension;
use crate::core::value_objects::policies::Policy;
use crate::usecases::extensions::deps::ExtensionUseCaseDependencies;
use crate::usecases::extensions::helpers::authorize_extension_access;

pub struct CreateExtensionUseCase {
    deps: ExtensionUseCaseDependencies,
}

impl CreateExtensionUseCase {
    pub fn new(deps: ExtensionUseCaseDependencies) -> Self {
        Self { deps }
    }

    pub async fn execute(
        &self,
        protective_measure_id: Uuid,
        data: CreateExtension,
        claims: &UserClaims,
    ) -> Result<ProtectiveMeasureExtension, AppError> {
        info!(
            "[CreateExtensionUseCase] Creating extension {} for protective measure: {}",
            data.extension_number, protective_measure_id
        );

        authorize_extension_access(
            &*self.deps.protective_measure_repository,
            &*self.deps.victim_repository,
            &*self.deps.user_repository,
            protective_measure_id,
            &Policy::CreateProtectiveMeasures,
            claims,
            "CreateExtensionUseCase",
        )
        .await?;

        match self
            .deps
            .extension_repository
            .create_extension(protective_measure_id, data)
            .await
        {
            Ok(extension) => {
                info!(
                    "[CreateExtensionUseCase] Extension created successfully with ID: {}",
                    extension.id
                );
                Ok(extension)
            }
            Err(RepositoryError::ReferencedEntityNotFound(msg)) => Err(AppError::BadRequest(msg)),
            Err(e) => {
                error!("[CreateExtensionUseCase] Error creating extension: {:?}", e);
                Err(AppError::InternalServerError)
            }
        }
    }
}
