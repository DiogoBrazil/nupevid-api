use std::sync::Arc;

use crate::core::contracts::adapters::password_hasher::PasswordHasherPort;
use crate::core::contracts::repository::refresh_tokens::RefreshTokenRepository;
use crate::core::contracts::repository::users::UserRepository;

#[derive(Clone)]
pub struct UserUseCaseDependencies {
    pub user_repository: Arc<dyn UserRepository>,
    pub password_hasher: Arc<dyn PasswordHasherPort>,
    pub refresh_token_repository: Arc<dyn RefreshTokenRepository>,
}

impl UserUseCaseDependencies {
    pub fn new(
        user_repository: Arc<dyn UserRepository>,
        password_hasher: Arc<dyn PasswordHasherPort>,
        refresh_token_repository: Arc<dyn RefreshTokenRepository>,
    ) -> Self {
        Self {
            user_repository,
            password_hasher,
            refresh_token_repository,
        }
    }
}
