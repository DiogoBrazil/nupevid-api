use std::sync::Arc;

use crate::core::contracts::repository::cities::CityRepository;
use crate::core::contracts::repository::users::UserRepository;

#[derive(Clone)]
pub struct CityUseCaseDependencies {
    pub city_repository: Arc<dyn CityRepository>,
    pub user_repository: Arc<dyn UserRepository>,
}

impl CityUseCaseDependencies {
    pub fn new(
        city_repository: Arc<dyn CityRepository>,
        user_repository: Arc<dyn UserRepository>,
    ) -> Self {
        Self {
            city_repository,
            user_repository,
        }
    }
}
