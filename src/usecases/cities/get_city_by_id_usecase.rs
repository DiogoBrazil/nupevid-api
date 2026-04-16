use log::{error, info};
use uuid::Uuid;

use crate::core::contracts::repository::error::RepositoryError;
use crate::core::entities::auth::UserClaims;
use crate::core::entities::cities::City;
use crate::core::value_objects::policies::Policy;
use crate::core::value_objects::profiles::Profile;
use crate::core::application_error::ApplicationError as AppError;
use crate::core::auth_context::AuthContext;
use crate::usecases::cities::deps::CityUseCaseDependencies;

pub struct GetCityByIdUseCase {
    deps: CityUseCaseDependencies,
}

impl GetCityByIdUseCase {
    pub fn new(deps: CityUseCaseDependencies) -> Self {
        Self { deps }
    }

    pub async fn execute(
        &self,
        id: Uuid,
        claims: &UserClaims,
    ) -> Result<City, AppError> {
        let auth = AuthContext::load(self.deps.user_repository.as_ref(), claims).await?;

        match self.deps.city_repository.get_city_by_id(id).await {
            Ok(city) => {
                if claims.profile != Profile::Root {
                    auth.check_policy(&Policy::ReadCities, city.id)?;
                }
                info!("[GetCityByIdUseCase] City with id {} found", id);
                Ok(city)
            }
            Err(RepositoryError::NotFound) => {
                Err(AppError::NotFound(format!("City with id '{}' not found", id)))
            }
            Err(error) => {
                error!(
                    "[GetCityByIdUseCase] Database error while finding city: {:?}",
                    error
                );
                Err(AppError::InternalServerError)
            }
        }
    }
}
