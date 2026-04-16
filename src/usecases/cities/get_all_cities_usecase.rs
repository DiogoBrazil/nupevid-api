use log::info;

use crate::core::entities::auth::UserClaims;
use crate::core::entities::cities::City;
use crate::core::pagination::PaginatedResult;
use crate::core::value_objects::policies::Policy;
use crate::core::application_error::ApplicationError as AppError;
use crate::core::auth_context::AuthContext;
use crate::usecases::cities::deps::CityUseCaseDependencies;
use crate::utils::pagination::Pagination;

pub struct GetAllCitiesUseCase {
    deps: CityUseCaseDependencies,
}

impl GetAllCitiesUseCase {
    pub fn new(deps: CityUseCaseDependencies) -> Self {
        Self { deps }
    }

    pub async fn execute(
        &self,
        pagination: Pagination,
        claims: &UserClaims,
    ) -> Result<PaginatedResult<City>, AppError> {
        let auth = AuthContext::load(self.deps.user_repository.as_ref(), claims).await?;
        let allowed_cities = auth.allowed_cities(&Policy::ReadCities);

        let total_items = self
            .deps
            .city_repository
            .count_cities(allowed_cities.as_deref())
            .await
            .map_err(|_| AppError::InternalServerError)?;

        let cities = self
            .deps
            .city_repository
            .get_cities_paginated(
                allowed_cities.as_deref(),
                pagination.page_size,
                pagination.offset,
            )
            .await
            .map_err(|_| AppError::InternalServerError)?;

        info!("[GetAllCitiesUseCase] Successfully retrieved {} cities", cities.len());

        Ok(PaginatedResult {
            items: cities,
            page: pagination.page,
            page_size: pagination.page_size,
            total_items,
        })
    }
}
