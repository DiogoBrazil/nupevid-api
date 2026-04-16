use log::{error, info};
use uuid::Uuid;

use crate::core::commands::protective_measures::UpdateExtension;
use crate::core::entities::auth::UserClaims;
use crate::core::entities::protective_measures::ProtectiveMeasureExtension;
use crate::core::value_objects::policies::Policy;
use crate::core::application_error::ApplicationError as AppError;
use crate::usecases::extensions::deps::ExtensionUseCaseDependencies;
use crate::usecases::extensions::helpers::load_auth_and_check_extension;

pub struct UpdateExtensionByIdUseCase {
    deps: ExtensionUseCaseDependencies,
}

impl UpdateExtensionByIdUseCase {
    pub fn new(deps: ExtensionUseCaseDependencies) -> Self {
        Self { deps }
    }

    pub async fn execute(
        &self,
        id: Uuid,
        data: UpdateExtension,
        claims: &UserClaims,
    ) -> Result<ProtectiveMeasureExtension, AppError> {
        info!("[UpdateExtensionByIdUseCase] Updating extension with ID: {}", id);

        load_auth_and_check_extension(
            &self.deps,
            id,
            &Policy::UpdateProtectiveMeasures,
            claims,
            "UpdateExtensionByIdUseCase",
        )
        .await?;

        match self
            .deps
            .extension_repository
            .update_extension_by_id(data, id)
            .await
        {
            Ok(extension) => {
                info!(
                    "[UpdateExtensionByIdUseCase] Extension updated successfully with ID: {}",
                    id
                );
                Ok(extension)
            }
            Err(e) => {
                error!(
                    "[UpdateExtensionByIdUseCase] Error updating extension: {:?}",
                    e
                );
                Err(AppError::InternalServerError)
            }
        }
    }
}
