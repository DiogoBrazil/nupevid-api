use std::sync::Arc;

use crate::core::contracts::adapters::system_metrics::SystemMetricsPort;

#[derive(Clone)]
pub struct MachineInformationUseCaseDependencies {
    pub system_metrics: Arc<dyn SystemMetricsPort>,
}

impl MachineInformationUseCaseDependencies {
    pub fn new(system_metrics: Arc<dyn SystemMetricsPort>) -> Self {
        Self { system_metrics }
    }
}
