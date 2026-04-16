use log::info;
use uuid::Uuid;

use crate::core::application_error::ApplicationError as AppError;
use crate::core::entities::auth::UserClaims;
use crate::core::entities::protective_measures::ProtectiveMeasureExtension;
use crate::core::value_objects::policies::Policy;
use crate::usecases::extensions::deps::ExtensionUseCaseDependencies;
use crate::usecases::extensions::helpers::load_auth_and_check_extension;

pub struct GetExtensionByIdUseCase {
    deps: ExtensionUseCaseDependencies,
}

impl GetExtensionByIdUseCase {
    pub fn new(deps: ExtensionUseCaseDependencies) -> Self {
        Self { deps }
    }

    pub async fn execute(
        &self,
        id: Uuid,
        claims: &UserClaims,
    ) -> Result<ProtectiveMeasureExtension, AppError> {
        info!(
            "[GetExtensionByIdUseCase] Getting extension with ID: {}",
            id
        );

        let extension = load_auth_and_check_extension(
            &self.deps,
            id,
            &Policy::ReadProtectiveMeasures,
            claims,
            "GetExtensionByIdUseCase",
        )
        .await?;

        info!("[GetExtensionByIdUseCase] Extension found with ID: {}", id);
        Ok(extension)
    }
}
