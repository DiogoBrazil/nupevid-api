use std::sync::Arc;

use crate::core::contracts::repository::users::UserRepository;
use crate::core::contracts::repository::victims::{VictimReadRepository, VictimWriteRepository};

#[derive(Clone)]
pub struct VictimUseCaseDependencies {
    pub victim_read_repository: Arc<dyn VictimReadRepository>,
    pub victim_write_repository: Arc<dyn VictimWriteRepository>,
    pub user_repository: Arc<dyn UserRepository>,
}

impl VictimUseCaseDependencies {
    pub fn new(
        victim_read_repository: Arc<dyn VictimReadRepository>,
        victim_write_repository: Arc<dyn VictimWriteRepository>,
        user_repository: Arc<dyn UserRepository>,
    ) -> Self {
        Self {
            victim_read_repository,
            victim_write_repository,
            user_repository,
        }
    }
}
