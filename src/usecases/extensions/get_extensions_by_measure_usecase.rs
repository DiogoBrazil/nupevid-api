use log::{error, info};
use uuid::Uuid;

use crate::core::entities::auth::UserClaims;
use crate::core::entities::protective_measures::ProtectiveMeasureExtension;
use crate::core::value_objects::policies::Policy;
use crate::core::application_error::ApplicationError as AppError;
use crate::usecases::extensions::deps::ExtensionUseCaseDependencies;
use crate::usecases::extensions::helpers::authorize_extension_access;

pub struct GetExtensionsByMeasureUseCase {
    deps: ExtensionUseCaseDependencies,
}

impl GetExtensionsByMeasureUseCase {
    pub fn new(deps: ExtensionUseCaseDependencies) -> Self {
        Self { deps }
    }

    pub async fn execute(
        &self,
        protective_measure_id: Uuid,
        claims: &UserClaims,
    ) -> Result<Vec<ProtectiveMeasureExtension>, AppError> {
        info!(
            "[GetExtensionsByMeasureUseCase] Getting extensions for protective measure: {}",
            protective_measure_id
        );

        authorize_extension_access(
            &*self.deps.protective_measure_repository,
            &*self.deps.victim_repository,
            &*self.deps.user_repository,
            protective_measure_id,
            &Policy::ReadProtectiveMeasures,
            claims,
            "GetExtensionsByMeasureUseCase",
        )
        .await?;

        match self
            .deps
            .extension_repository
            .get_extensions_by_measure(protective_measure_id)
            .await
        {
            Ok(extensions) => {
                info!(
                    "[GetExtensionsByMeasureUseCase] Found {} extensions for protective measure: {}",
                    extensions.len(),
                    protective_measure_id
                );
                Ok(extensions)
            }
            Err(e) => {
                error!(
                    "[GetExtensionsByMeasureUseCase] Database error: {:?}",
                    e
                );
                Err(AppError::InternalServerError)
            }
        }
    }
}
