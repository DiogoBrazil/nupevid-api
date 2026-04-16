use std::sync::Arc;

use crate::core::contracts::repository::extensions::ExtensionRepository;
use crate::core::contracts::repository::protective_measures::ProtectiveMeasureReadRepository;
use crate::core::contracts::repository::users::UserRepository;
use crate::core::contracts::repository::victims::VictimReadRepository;

#[derive(Clone)]
pub struct ExtensionUseCaseDependencies {
    pub extension_repository: Arc<dyn ExtensionRepository>,
    pub protective_measure_repository: Arc<dyn ProtectiveMeasureReadRepository>,
    pub victim_repository: Arc<dyn VictimReadRepository>,
    pub user_repository: Arc<dyn UserRepository>,
}

impl ExtensionUseCaseDependencies {
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
}
