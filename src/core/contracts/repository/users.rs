use async_trait::async_trait;
use serde_json::Value as JsonValue;
use uuid::Uuid;

use super::error::RepositoryError;
use crate::core::commands::users::{CreateUser, UpdateUser};
use crate::core::entities::users::User;

#[cfg_attr(test, mockall::automock)]
#[async_trait]
pub trait UserRepository: Send + Sync + 'static {
    async fn create_user(&self, data: CreateUser) -> Result<User, RepositoryError>;
    async fn get_user_by_id(&self, id: Uuid) -> Result<User, RepositoryError>;
    async fn get_all_users(&self) -> Result<Vec<User>, RepositoryError>;
    async fn get_users_by_name(&self, name: &str) -> Result<Vec<User>, RepositoryError>;
    async fn get_users_by_registration(
        &self,
        registration: &str,
    ) -> Result<Vec<User>, RepositoryError>;
    async fn get_users_paginated<'a>(
        &'a self,
        allowed_cities: Option<&'a [Uuid]>,
        exclude_root: bool,
        limit: i64,
        offset: i64,
    ) -> Result<Vec<User>, RepositoryError>
    where
        Self: 'a;
    async fn count_users<'a>(
        &'a self,
        allowed_cities: Option<&'a [Uuid]>,
        exclude_root: bool,
    ) -> Result<i64, RepositoryError>
    where
        Self: 'a;
    async fn update_user_by_id(&self, data: UpdateUser, id: Uuid) -> Result<User, RepositoryError>;
    async fn delete_user_by_id(&self, id: Uuid) -> Result<User, RepositoryError>;
    async fn check_user_exists_by_email(&self, email: &str) -> Result<bool, RepositoryError>;
    async fn check_email_exists_for_other_user(
        &self,
        email: &str,
        id: Uuid,
    ) -> Result<bool, RepositoryError>;
    async fn get_user_password_by_id(&self, id: Uuid) -> Result<String, RepositoryError>;
    async fn update_user_password_by_id(
        &self,
        id: Uuid,
        new_password: String,
    ) -> Result<User, RepositoryError>;
    async fn reset_user_password_by_id(
        &self,
        id: Uuid,
        new_password: String,
        expires_at: chrono::DateTime<chrono::Utc>,
    ) -> Result<User, RepositoryError>;
    async fn check_city_admin_exists_for_city(
        &self,
        city_id: Uuid,
        exclude_user_id: Uuid,
    ) -> Result<bool, RepositoryError>;
    async fn get_user_policies_json_by_id(&self, id: Uuid) -> Result<JsonValue, RepositoryError>;
    async fn update_user_policies_by_id(
        &self,
        id: Uuid,
        policies: JsonValue,
    ) -> Result<User, RepositoryError>;
}
