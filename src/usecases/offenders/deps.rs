use std::sync::Arc;

use crate::core::contracts::repository::offenders::{
    OffenderReadRepository, OffenderWriteRepository,
};
use crate::core::contracts::repository::users::UserRepository;

#[derive(Clone)]
pub struct OffenderUseCaseDependencies {
    pub offender_read_repository: Arc<dyn OffenderReadRepository>,
    pub offender_write_repository: Arc<dyn OffenderWriteRepository>,
    pub user_repository: Arc<dyn UserRepository>,
}

impl OffenderUseCaseDependencies {
    pub fn new(
        offender_read_repository: Arc<dyn OffenderReadRepository>,
        offender_write_repository: Arc<dyn OffenderWriteRepository>,
        user_repository: Arc<dyn UserRepository>,
    ) -> Self {
        Self {
            offender_read_repository,
            offender_write_repository,
            user_repository,
        }
    }
}
