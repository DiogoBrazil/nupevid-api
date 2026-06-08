use async_trait::async_trait;
use uuid::Uuid;

use super::error::RepositoryError;
use crate::core::commands::cities::{CreateCity, UpdateCity};
use crate::core::entities::cities::City;

#[async_trait]
pub trait CityRepository: Send + Sync {
    async fn create_city(&self, city: CreateCity) -> Result<City, RepositoryError>;
    async fn get_city_by_id(&self, id: Uuid) -> Result<City, RepositoryError>;
    async fn get_all_cities(&self) -> Result<Vec<City>, RepositoryError>;
    async fn get_cities_paginated(
        &self,
        allowed_cities: Option<&[Uuid]>,
        limit: i64,
        offset: i64,
    ) -> Result<Vec<City>, RepositoryError>;
    async fn count_cities(&self, allowed_cities: Option<&[Uuid]>) -> Result<i64, RepositoryError>;
    async fn update_city_by_id(&self, data: UpdateCity, id: Uuid) -> Result<City, RepositoryError>;
    async fn delete_city_by_id(&self, id: Uuid) -> Result<City, RepositoryError>;
    async fn get_city_by_name_and_battalion(
        &self,
        name: &str,
        battalion: &str,
    ) -> Result<Option<City>, RepositoryError>;
}
