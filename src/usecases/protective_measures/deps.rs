use std::sync::Arc;

use crate::core::contracts::repository::cities::CityRepository;
use crate::core::contracts::repository::extensions::ExtensionRepository;
use crate::core::contracts::repository::offenders::OffenderReadRepository;
use crate::core::contracts::repository::protective_measures::{
    ProtectiveMeasureReadRepository, ProtectiveMeasureWriteRepository,
};
use crate::core::contracts::repository::users::UserRepository;
use crate::core::contracts::repository::victims::VictimReadRepository;

#[derive(Clone)]
pub struct ProtectiveMeasureUseCaseDependencies {
    pub measure_read_repository: Arc<dyn ProtectiveMeasureReadRepository>,
    pub measure_write_repository: Arc<dyn ProtectiveMeasureWriteRepository>,
    pub victim_repository: Arc<dyn VictimReadRepository>,
    pub offender_repository: Arc<dyn OffenderReadRepository>,
    pub user_repository: Arc<dyn UserRepository>,
    pub extension_repository: Arc<dyn ExtensionRepository>,
    pub city_repository: Arc<dyn CityRepository>,
}

impl ProtectiveMeasureUseCaseDependencies {
    pub fn new(
        measure_read_repository: Arc<dyn ProtectiveMeasureReadRepository>,
        measure_write_repository: Arc<dyn ProtectiveMeasureWriteRepository>,
        victim_repository: Arc<dyn VictimReadRepository>,
        offender_repository: Arc<dyn OffenderReadRepository>,
        user_repository: Arc<dyn UserRepository>,
        extension_repository: Arc<dyn ExtensionRepository>,
        city_repository: Arc<dyn CityRepository>,
    ) -> Self {
        Self {
            measure_read_repository,
            measure_write_repository,
            victim_repository,
            offender_repository,
            user_repository,
            extension_repository,
            city_repository,
        }
    }
}
