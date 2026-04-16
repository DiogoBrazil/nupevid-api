use log::{error, info};
use uuid::Uuid;

use crate::core::commands::cities::UpdateCity;
use crate::core::contracts::repository::error::RepositoryError;
use crate::core::entities::auth::UserClaims;
use crate::core::entities::cities::City;
use crate::core::value_objects::profiles::Profile;
use crate::core::application_error::ApplicationError as AppError;
use crate::usecases::cities::deps::CityUseCaseDependencies;
use crate::validators::city_validator::CityValidator;

pub struct UpdateCityByIdUseCase {
    deps: CityUseCaseDependencies,
}

impl UpdateCityByIdUseCase {
    pub fn new(deps: CityUseCaseDependencies) -> Self {
        Self { deps }
    }

    pub async fn execute(
        &self,
        data: UpdateCity,
        id: Uuid,
        claims: &UserClaims,
    ) -> Result<City, AppError> {
        info!("[UpdateCityByIdUseCase] Starting city update for id: {}", id);

        if claims.profile != Profile::Root {
            return Err(AppError::Forbidden(
                "Only ROOT can update cities".to_string(),
            ));
        }

        CityValidator::validate_fields(
            &data.name,
            &data.state,
            &data.battalion,
            "Error updating city",
        )?;

        match self
            .deps
            .city_repository
            .get_city_by_name_and_battalion(&data.name, &data.battalion)
            .await
        {
            Ok(Some(existing_city)) if existing_city.id != id => {
                return Err(AppError::BadRequest(format!(
                    "Error updating city: a city with name '{}' and battalion '{}' already exists",
                    data.name, data.battalion
                )));
            }
            Ok(_) => {}
            Err(error) => {
                error!(
                    "[UpdateCityByIdUseCase] Error checking duplicate city: {:?}",
                    error
                );
                return Err(AppError::InternalServerError);
            }
        }

        match self.deps.city_repository.update_city_by_id(data, id).await {
            Ok(city) => Ok(city),
            Err(RepositoryError::NotFound) => {
                Err(AppError::NotFound(format!("City with id '{}' not found", id)))
            }
            Err(error) => {
                error!(
                    "[UpdateCityByIdUseCase] Error updating city in database: {:?}",
                    error
                );
                Err(AppError::InternalServerError)
            }
        }
    }
}
