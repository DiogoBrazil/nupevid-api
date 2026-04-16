use log::{error, info};
use uuid::Uuid;

use crate::core::application_error::ApplicationError as AppError;
use crate::core::contracts::repository::error::RepositoryError;
use crate::core::entities::auth::UserClaims;
use crate::core::entities::cities::City;
use crate::core::value_objects::profiles::Profile;
use crate::usecases::cities::deps::CityUseCaseDependencies;

pub struct DeleteCityByIdUseCase {
    deps: CityUseCaseDependencies,
}

impl DeleteCityByIdUseCase {
    pub fn new(deps: CityUseCaseDependencies) -> Self {
        Self { deps }
    }

    pub async fn execute(&self, id: Uuid, claims: &UserClaims) -> Result<City, AppError> {
        info!(
            "[DeleteCityByIdUseCase] Starting city deletion for id: {}",
            id
        );

        if claims.profile != Profile::Root {
            return Err(AppError::Forbidden(
                "Only ROOT can delete cities".to_string(),
            ));
        }

        match self.deps.city_repository.delete_city_by_id(id).await {
            Ok(city) => Ok(city),
            Err(RepositoryError::NotFound) => Err(AppError::NotFound(format!(
                "City with id '{}' not found",
                id
            ))),
            Err(error) => {
                error!(
                    "[DeleteCityByIdUseCase] Error deleting city in database: {:?}",
                    error
                );
                Err(AppError::InternalServerError)
            }
        }
    }
}
