use async_trait::async_trait;
use log::info;
use sqlx::PgPool;
use uuid::Uuid;

use crate::repositories::queries::cities::CitiesQueries;
use crate::core::{
    commands::cities::{CreateCity, UpdateCity},
    contracts::repository::{cities::CityRepository, error::RepositoryError},
    entities::cities::City,
};
use crate::repositories::error_mapper::map_sqlx_error;

use super::models::cities::CityRow;

#[derive(Clone)]
pub struct PgCityRepository {
    pool: PgPool,
}

impl PgCityRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl CityRepository for PgCityRepository {
    async fn create_city(&self, city: CreateCity) -> Result<City, RepositoryError> {
        let id: Uuid = Uuid::new_v4();

        info!(
            "[Repository] Executing SQL query to create city: {} with ID: {}",
            city.name, id
        );

        let city_created: City = sqlx::query_as::<_, CityRow>(CitiesQueries::CREATE_CITY)
            .bind(id)
            .bind(city.name)
            .bind(city.state)
            .bind(city.battalion)
            .fetch_one(&self.pool)
            .await
            .map_err(map_sqlx_error)?
            .into();

        info!(
            "[Repository] City successfully inserted into database with ID: {}",
            city_created.id
        );

        Ok(city_created)
    }

    async fn get_city_by_id(&self, id: Uuid) -> Result<City, RepositoryError> {
        info!(
            "[Repository] Executing SQL query to get city with id: {}",
            id
        );

        let city: City = sqlx::query_as::<_, CityRow>(CitiesQueries::GET_CITY_BY_ID)
            .bind(id)
            .fetch_one(&self.pool)
            .await
            .map_err(map_sqlx_error)?
            .into();

        info!(
            "[Repository] City successfully found in the database with ID: {}",
            id
        );

        Ok(city)
    }

    async fn get_all_cities(&self) -> Result<Vec<City>, RepositoryError> {
        info!("[Repository] Executing SQL query to get all cities");

        let cities: Vec<City> = sqlx::query_as::<_, CityRow>(CitiesQueries::GET_ALL_CITIES)
            .fetch_all(&self.pool)
            .await
            .map_err(map_sqlx_error)?
            .into_iter()
            .map(Into::into)
            .collect();

        info!("[Repository] Found {} cities in database", cities.len());

        Ok(cities)
    }

    async fn get_cities_paginated(
        &self,
        allowed_cities: Option<&[Uuid]>,
        limit: i64,
        offset: i64,
    ) -> Result<Vec<City>, RepositoryError> {
        info!("[Repository] Executing SQL query to get paginated cities");

        let cities: Vec<City> = match allowed_cities {
            Some(city_ids) => sqlx::query_as::<_, CityRow>(CitiesQueries::GET_CITIES_PAGED_BY_IDS)
                .bind(city_ids)
                .bind(limit)
                .bind(offset)
                .fetch_all(&self.pool)
                .await
                .map_err(map_sqlx_error)?
                .into_iter()
                .map(Into::into)
                .collect(),
            None => sqlx::query_as::<_, CityRow>(CitiesQueries::GET_CITIES_PAGED)
                .bind(limit)
                .bind(offset)
                .fetch_all(&self.pool)
                .await
                .map_err(map_sqlx_error)?
                .into_iter()
                .map(Into::into)
                .collect(),
        };

        info!("[Repository] Found {} cities in database", cities.len());

        Ok(cities)
    }

    async fn count_cities(&self, allowed_cities: Option<&[Uuid]>) -> Result<i64, RepositoryError> {
        let count: i64 = match allowed_cities {
            Some(city_ids) => sqlx::query_scalar(CitiesQueries::COUNT_CITIES_BY_IDS)
                .bind(city_ids)
                .fetch_one(&self.pool)
                .await
                .map_err(map_sqlx_error)?,
            None => sqlx::query_scalar(CitiesQueries::COUNT_CITIES)
                .fetch_one(&self.pool)
                .await
                .map_err(map_sqlx_error)?,
        };

        Ok(count)
    }

    async fn update_city_by_id(&self, data: UpdateCity, id: Uuid) -> Result<City, RepositoryError> {
        info!(
            "[Repository] Executing SQL query to update city with ID: {}",
            id
        );

        let city_updated: City = sqlx::query_as::<_, CityRow>(CitiesQueries::UPDATE_CITY_BY_ID)
            .bind(id)
            .bind(data.name)
            .bind(data.state)
            .bind(data.battalion)
            .fetch_one(&self.pool)
            .await
            .map_err(map_sqlx_error)?
            .into();

        info!(
            "[Repository] City successfully updated in database with ID: {}",
            id
        );

        Ok(city_updated)
    }

    async fn delete_city_by_id(&self, id: Uuid) -> Result<City, RepositoryError> {
        info!(
            "[Repository] Executing SQL query to soft delete city with id: {}",
            id
        );

        let deleted_city: City = sqlx::query_as::<_, CityRow>(CitiesQueries::DELETE_CITY_BY_ID)
            .bind(id)
            .fetch_one(&self.pool)
            .await
            .map_err(map_sqlx_error)?
            .into();

        info!(
            "[Repository] City successfully soft deleted from database with ID: {}",
            id
        );

        Ok(deleted_city)
    }

    async fn get_city_by_name_and_battalion(
        &self,
        name: &str,
        battalion: &str,
    ) -> Result<Option<City>, RepositoryError> {
        info!(
            "[Repository] Executing SQL query to check if city exists with name: {} and battalion: {}",
            name, battalion
        );

        let city: Option<City> = sqlx::query_as::<_, CityRow>(CitiesQueries::GET_CITY_BY_NAME_AND_BATTALION)
            .bind(name)
            .bind(battalion)
            .fetch_optional(&self.pool)
            .await
            .map_err(map_sqlx_error)?
            .map(|row| row.into());

        if city.is_some() {
            info!(
                "[Repository] City already exists with name: {} and battalion: {}",
                name, battalion
            );
        } else {
            info!(
                "[Repository] No city found with name: {} and battalion: {}",
                name, battalion
            );
        }

        Ok(city)
    }
}
