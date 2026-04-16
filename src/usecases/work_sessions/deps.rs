use std::sync::Arc;

use crate::core::contracts::repository::users::UserRepository;
use crate::core::contracts::repository::work_sessions::{
    WorkSessionReadRepository, WorkSessionWriteRepository,
};

#[derive(Clone)]
pub struct WorkSessionUseCaseDependencies {
    pub work_session_read_repository: Arc<dyn WorkSessionReadRepository>,
    pub work_session_write_repository: Arc<dyn WorkSessionWriteRepository>,
    pub user_repository: Arc<dyn UserRepository>,
}

impl WorkSessionUseCaseDependencies {
    pub fn new(
        work_session_read_repository: Arc<dyn WorkSessionReadRepository>,
        work_session_write_repository: Arc<dyn WorkSessionWriteRepository>,
        user_repository: Arc<dyn UserRepository>,
    ) -> Self {
        Self {
            work_session_read_repository,
            work_session_write_repository,
            user_repository,
        }
    }
}
