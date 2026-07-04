use crate::core::application_error::ApplicationError as AppError;
use crate::core::contracts::adapters::system_metrics::MachineInformation;
use crate::core::entities::auth::UserClaims;
use crate::core::value_objects::profiles::Profile;

use super::deps::MachineInformationUseCaseDependencies;

pub struct GetMachineInformationUseCase {
    deps: MachineInformationUseCaseDependencies,
}

impl GetMachineInformationUseCase {
    pub fn new(deps: MachineInformationUseCaseDependencies) -> Self {
        Self { deps }
    }

    pub async fn execute(&self, claims: &UserClaims) -> Result<MachineInformation, AppError> {
        if claims.profile != Profile::Root {
            return Err(AppError::Forbidden(
                "Only ROOT can access machine information".to_string(),
            ));
        }

        Ok(self.deps.system_metrics.collect().await)
    }
}
