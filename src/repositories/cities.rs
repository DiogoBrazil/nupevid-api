use async_trait::async_trait;
use log::info;
use sqlx::PgPool;
use uuid::Uuid;

use crate::core::{
    contracts::repository::cities::CityRepository,
    entities::cities::{CreateCity, UpdateCity, City}
};
use crate::config::querys::cities::CitiesQueries;

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
    async fn create_city(&self, city: CreateCity) -> Result<City, sqlx::Error> {
        let id: Uuid = Uuid::new_v4();

        info!("[Repository] Executing SQL query to create city: {} with ID: {}", city.name, id);

        let city_created: City = sqlx::query_as(CitiesQueries::CREATE_CITY)
            .bind(id)
            .bind(city.name)
            .bind(city.state)
            .bind(city.battalion)
            .fetch_one(&self.pool)
            .await?;

        info!("[Repository] City successfully inserted into database with ID: {}", city_created.id);

        Ok(city_created)
    }

    async fn get_city_by_id(&self, id: Uuid) -> Result<City, sqlx::Error> {
        info!("[Repository] Executing SQL query to get city with id: {}", id);

        let city: City = sqlx::query_as(CitiesQueries::GET_CITY_BY_ID)
            .bind(id)
            .fetch_one(&self.pool)
            .await?;

        info!("[Repository] City successfully found in the database with ID: {}", id);

        Ok(city)
    }

    async fn get_all_cities(&self) -> Result<Vec<City>, sqlx::Error> {
        info!("[Repository] Executing SQL query to get all cities");

        let cities: Vec<City> = sqlx::query_as(CitiesQueries::GET_ALL_CITIES)
            .fetch_all(&self.pool)
            .await?;

        info!("[Repository] Found {} cities in database", cities.len());

        Ok(cities)
    }

    async fn update_city_by_id(&self, data: UpdateCity, id: Uuid) -> Result<City, sqlx::Error> {
        info!("[Repository] Executing SQL query to update city with ID: {}", id);

        let city_updated: City = sqlx::query_as(CitiesQueries::UPDATE_CITY_BY_ID)
            .bind(id)
            .bind(data.name)
            .bind(data.state)
            .bind(data.battalion)
            .fetch_one(&self.pool)
            .await?;

        info!("[Repository] City successfully updated in database with ID: {}", id);

        Ok(city_updated)
    }

    async fn delete_city_by_id(&self, id: Uuid) -> Result<City, sqlx::Error> {
        info!("[Repository] Executing SQL query to soft delete city with id: {}", id);

        let deleted_city: City = sqlx::query_as(CitiesQueries::DELETE_CITY_BY_ID)
            .bind(id)
            .fetch_one(&self.pool)
            .await?;

        info!("[Repository] City successfully soft deleted from database with ID: {}", id);

        Ok(deleted_city)
    }

    async fn get_city_by_name_and_battalion(&self, name: &str, battalion: &str) -> Result<Option<City>, sqlx::Error> {
        info!("[Repository] Executing SQL query to check if city exists with name: {} and battalion: {}", name, battalion);

        let city: Option<City> = sqlx::query_as(CitiesQueries::GET_CITY_BY_NAME_AND_BATTALION)
            .bind(name)
            .bind(battalion)
            .fetch_optional(&self.pool)
            .await?;

        if city.is_some() {
            info!("[Repository] City already exists with name: {} and battalion: {}", name, battalion);
        } else {
            info!("[Repository] No city found with name: {} and battalion: {}", name, battalion);
        }

        Ok(city)
    }
}
