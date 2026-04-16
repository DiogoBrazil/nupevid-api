use log::{error, info};

use crate::core::commands::cities::CreateCity;
use crate::core::entities::auth::UserClaims;
use crate::core::entities::cities::City;
use crate::core::value_objects::profiles::Profile;
use crate::core::application_error::ApplicationError as AppError;
use crate::usecases::cities::deps::CityUseCaseDependencies;
use crate::validators::city_validator::CityValidator;

pub struct CreateCityUseCase {
    deps: CityUseCaseDependencies,
}

impl CreateCityUseCase {
    pub fn new(deps: CityUseCaseDependencies) -> Self {
        Self { deps }
    }

    pub async fn execute(
        &self,
        city: CreateCity,
        claims: &UserClaims,
    ) -> Result<City, AppError> {
        info!("[CreateCityUseCase] Starting city creation: {}", city.name);

        if claims.profile != Profile::Root {
            return Err(AppError::Forbidden(
                "Only ROOT can create cities".to_string(),
            ));
        }

        CityValidator::validate_fields(
            &city.name,
            &city.state,
            &city.battalion,
            "Error adding city",
        )?;

        match self
            .deps
            .city_repository
            .get_city_by_name_and_battalion(&city.name, &city.battalion)
            .await
        {
            Ok(Some(_)) => Err(AppError::BadRequest(format!(
                "Error adding city: a city with name '{}' and battalion '{}' already exists",
                city.name, city.battalion
            ))),
            Ok(None) => self
                .deps
                .city_repository
                .create_city(city)
                .await
                .map_err(|error| {
                    error!("[CreateCityUseCase] Failed to save city: {:?}", error);
                    AppError::InternalServerError
                }),
            Err(error) => {
                error!(
                    "[CreateCityUseCase] Error checking duplicate city: {:?}",
                    error
                );
                Err(AppError::InternalServerError)
            }
        }
    }
}
