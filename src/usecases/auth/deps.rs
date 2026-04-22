use std::sync::Arc;

use crate::config::config_env::Config;
use crate::core::contracts::adapters::password_hasher::PasswordHasherPort;
use crate::core::contracts::adapters::token_generator::TokenGeneratorPort;
use crate::core::contracts::repository::auth::AuthRepository;
use crate::core::contracts::repository::work_sessions::{
    WorkSessionReadRepository, WorkSessionWriteRepository,
};

#[derive(Clone)]
pub struct AuthUseCaseDependencies {
    pub auth_repository: Arc<dyn AuthRepository>,
    pub work_session_read_repository: Arc<dyn WorkSessionReadRepository>,
    pub work_session_write_repository: Arc<dyn WorkSessionWriteRepository>,
    pub config: Arc<Config>,
    pub password_hasher: Arc<dyn PasswordHasherPort>,
    pub token_generator: Arc<dyn TokenGeneratorPort>,
}

impl AuthUseCaseDependencies {
    pub fn new(
        auth_repository: Arc<dyn AuthRepository>,
        work_session_read_repository: Arc<dyn WorkSessionReadRepository>,
        work_session_write_repository: Arc<dyn WorkSessionWriteRepository>,
        config: Arc<Config>,
        password_hasher: Arc<dyn PasswordHasherPort>,
        token_generator: Arc<dyn TokenGeneratorPort>,
    ) -> Self {
        Self {
            auth_repository,
            work_session_read_repository,
            work_session_write_repository,
            config,
            password_hasher,
            token_generator,
        }
    }
}
