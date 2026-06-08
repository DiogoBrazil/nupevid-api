use crate::core::application_error::ApplicationError as AppError;
use crate::core::contracts::adapters::system_metrics::MachineInformation;

use super::deps::MachineInformationUseCaseDependencies;

pub struct GetMachineInformationUseCase {
    deps: MachineInformationUseCaseDependencies,
}

impl GetMachineInformationUseCase {
    pub fn new(deps: MachineInformationUseCaseDependencies) -> Self {
        Self { deps }
    }

    pub async fn execute(&self) -> Result<MachineInformation, AppError> {
        Ok(self.deps.system_metrics.collect().await)
    }
}
