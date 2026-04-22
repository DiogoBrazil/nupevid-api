use async_trait::async_trait;
use log::info;
use serde_json::{Value as JsonValue, json};
use sqlx::PgPool;
use uuid::Uuid;

use crate::config::querys::users::UsersQueries;
use crate::core::{
    contracts::repository::users::UserRepository,
    entities::users::{CreateUser, UpdateUser, UserDataCreatedWithoutPassword},
};

#[derive(Clone)]
pub struct PgUserRepository {
    pool: PgPool,
}

impl PgUserRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl UserRepository for PgUserRepository {
    async fn create_user(
        &self,
        user: CreateUser,
    ) -> Result<UserDataCreatedWithoutPassword, sqlx::Error> {
        let id: Uuid = Uuid::new_v4();

        info!(
            "[Repository] Executing SQL query to create user with email: {} and ID: {}",
            user.email, id
        );

        let permission_policies_json = match &user.permission_policies {
            Some(policies) => json!(policies),
            None => json!({}),
        };

        let user_created: UserDataCreatedWithoutPassword =
            sqlx::query_as(UsersQueries::CREATE_USER)
                .bind(id)
                .bind(user.rank)
                .bind(user.registration)
                .bind(user.full_name)
                .bind(user.profile)
                .bind(user.email)
                .bind(user.password)
                .bind(user.city_id)
                .bind(permission_policies_json)
                .fetch_one(&self.pool)
                .await?;

        info!(
            "[Repository] User successfully inserted into database with ID: {}",
            user_created.id
        );

        Ok(user_created)
    }

    async fn update_user_by_id(
        &self,
        data: UpdateUser,
        id: Uuid,
    ) -> Result<UserDataCreatedWithoutPassword, sqlx::Error> {
        info!(
            "[Repository] Executing SQL query to update user with ID: {}",
            id
        );

        let permission_policies_json = match &data.permission_policies {
            Some(policies) => json!(policies),
            None => json!({}),
        };

        let user_updated: UserDataCreatedWithoutPassword =
            sqlx::query_as(UsersQueries::UPDATE_USER_BY_ID)
                .bind(id)
                .bind(data.rank)
                .bind(data.registration)
                .bind(data.full_name)
                .bind(data.profile)
                .bind(data.email)
                .bind(data.city_id)
                .bind(permission_policies_json)
                .fetch_one(&self.pool)
                .await?;

        info!(
            "[Repository] User successfully updated into database with ID: {}",
            id
        );

        Ok(user_updated)
    }

    async fn get_user_by_id(
        &self,
        id: Uuid,
    ) -> Result<UserDataCreatedWithoutPassword, sqlx::Error> {
        info!(
            "[Repository] Executing SQL query to get user with id: {}",
            id
        );

        let user: UserDataCreatedWithoutPassword = sqlx::query_as(UsersQueries::GET_USER_BY_ID)
            .bind(id)
            .fetch_one(&self.pool)
            .await?;

        info!(
            "[Repository] User successfully found in the database with IDD: {}",
            id
        );

        Ok(user)
    }

    async fn check_user_exists_by_email(&self, email: &str) -> Result<bool, sqlx::Error> {
        info!(
            "[Repository] Executing SQL query to check if user with email {} exists",
            email
        );

        let user_exists: bool = sqlx::query_scalar(UsersQueries::CHECK_USER_EXISTS_BY_EMAIL)
            .bind(email)
            .fetch_one(&self.pool)
            .await?;

        info!(
            "[Repository] User exists with email {}: {}",
            email, user_exists
        );

        Ok(user_exists)
    }

    async fn check_email_exists_for_other_user(
        &self,
        email: &str,
        id: Uuid,
    ) -> Result<bool, sqlx::Error> {
        info!(
            "[Repository] Executing SQL query to check if email {} exists for other user",
            email
        );

        let result: bool = sqlx::query_scalar(UsersQueries::CHECK_EMAIL_EXISTS_FOR_OTHER_USER)
            .bind(email)
            .bind(id)
            .fetch_one(&self.pool)
            .await?;

        Ok(result)
    }

    async fn get_all_users(&self) -> Result<Vec<UserDataCreatedWithoutPassword>, sqlx::Error> {
        info!("[Repository] Executing SQL query to get all users");

        let users: Vec<UserDataCreatedWithoutPassword> =
            sqlx::query_as(UsersQueries::GET_ALL_USERS)
                .fetch_all(&self.pool)
                .await?;

        info!("[Repository] Found {} users in database", users.len());

        Ok(users)
    }

    async fn get_users_by_name(
        &self,
        name: &str,
    ) -> Result<Vec<UserDataCreatedWithoutPassword>, sqlx::Error> {
        let pattern = format!("%{}%", name);
        info!(
            "[Repository] Executing SQL query to get users by name pattern: {}",
            pattern
        );

        let users: Vec<UserDataCreatedWithoutPassword> =
            sqlx::query_as(UsersQueries::GET_USERS_BY_NAME)
                .bind(pattern)
                .fetch_all(&self.pool)
                .await?;

        info!("[Repository] Found {} users by name", users.len());

        Ok(users)
    }

    async fn get_users_by_registration(
        &self,
        registration: &str,
    ) -> Result<Vec<UserDataCreatedWithoutPassword>, sqlx::Error> {
        info!(
            "[Repository] Executing SQL query to get users by registration: {}",
            registration
        );

        let users: Vec<UserDataCreatedWithoutPassword> =
            sqlx::query_as(UsersQueries::GET_USERS_BY_REGISTRATION)
                .bind(registration)
                .fetch_all(&self.pool)
                .await?;

        info!("[Repository] Found {} users by registration", users.len());

        Ok(users)
    }

    async fn get_users_paginated(
        &self,
        allowed_cities: Option<&[Uuid]>,
        exclude_root: bool,
        limit: i64,
        offset: i64,
    ) -> Result<Vec<UserDataCreatedWithoutPassword>, sqlx::Error> {
        info!("[Repository] Executing SQL query to get paginated users");

        let users: Vec<UserDataCreatedWithoutPassword> = match allowed_cities {
            Some(city_ids) => {
                sqlx::query_as(UsersQueries::GET_USERS_PAGED_BY_CITIES)
                    .bind(city_ids)
                    .bind(exclude_root)
                    .bind(limit)
                    .bind(offset)
                    .fetch_all(&self.pool)
                    .await?
            }
            None => {
                sqlx::query_as(UsersQueries::GET_USERS_PAGED)
                    .bind(exclude_root)
                    .bind(limit)
                    .bind(offset)
                    .fetch_all(&self.pool)
                    .await?
            }
        };

        info!("[Repository] Found {} users in database", users.len());

        Ok(users)
    }

    async fn count_users(
        &self,
        allowed_cities: Option<&[Uuid]>,
        exclude_root: bool,
    ) -> Result<i64, sqlx::Error> {
        let count: i64 = match allowed_cities {
            Some(city_ids) => {
                sqlx::query_scalar(UsersQueries::COUNT_USERS_BY_CITIES)
                    .bind(city_ids)
                    .bind(exclude_root)
                    .fetch_one(&self.pool)
                    .await?
            }
            None => {
                sqlx::query_scalar(UsersQueries::COUNT_USERS)
                    .bind(exclude_root)
                    .fetch_one(&self.pool)
                    .await?
            }
        };

        Ok(count)
    }

    async fn delete_user_by_id(
        &self,
        id: Uuid,
    ) -> Result<UserDataCreatedWithoutPassword, sqlx::Error> {
        info!(
            "[Repository] Executing SQL query to delete user with id: {}",
            id
        );

        let deleted_user: UserDataCreatedWithoutPassword =
            sqlx::query_as(UsersQueries::DELETE_USER_BY_ID)
                .bind(id)
                .fetch_one(&self.pool)
                .await?;

        info!(
            "[Repository] User successfully deleted from database with ID: {}",
            id
        );

        Ok(deleted_user)
    }

    async fn get_user_password_by_id(&self, id: Uuid) -> Result<String, sqlx::Error> {
        info!(
            "[Repository] Executing SQL query to get password for user with id: {}",
            id
        );

        let password: String = sqlx::query_scalar(UsersQueries::GET_USER_PASSWORD_BY_ID)
            .bind(id)
            .fetch_one(&self.pool)
            .await?;

        info!(
            "[Repository] Password retrieved successfully for user with ID: {}",
            id
        );

        Ok(password)
    }

    async fn update_user_password_by_id(
        &self,
        id: Uuid,
        new_password: String,
    ) -> Result<UserDataCreatedWithoutPassword, sqlx::Error> {
        info!(
            "[Repository] Executing SQL query to update password for user with id: {}",
            id
        );

        let updated_user: UserDataCreatedWithoutPassword =
            sqlx::query_as(UsersQueries::UPDATE_USER_PASSWORD_BY_ID)
                .bind(id)
                .bind(new_password)
                .fetch_one(&self.pool)
                .await?;

        info!(
            "[Repository] Password updated successfully for user with ID: {}",
            id
        );

        Ok(updated_user)
    }

    async fn reset_user_password_by_id(
        &self,
        id: Uuid,
        new_password: String,
        expires_at: chrono::DateTime<chrono::Utc>,
    ) -> Result<UserDataCreatedWithoutPassword, sqlx::Error> {
        info!(
            "[Repository] Executing SQL query to reset password for user with id: {}",
            id
        );

        let updated_user: UserDataCreatedWithoutPassword =
            sqlx::query_as(UsersQueries::RESET_USER_PASSWORD_BY_ID)
                .bind(id)
                .bind(new_password)
                .bind(expires_at)
                .fetch_one(&self.pool)
                .await?;

        info!(
            "[Repository] Password reset successfully for user with ID: {}",
            id
        );

        Ok(updated_user)
    }

    async fn check_city_admin_exists_for_city(
        &self,
        city_id: Uuid,
        exclude_user_id: Uuid,
    ) -> Result<bool, sqlx::Error> {
        info!(
            "[Repository] Executing SQL query to check if CITY_ADMIN exists for city: {} excluding user: {}",
            city_id, exclude_user_id
        );

        let result: bool = sqlx::query_scalar(UsersQueries::CHECK_CITY_ADMIN_EXISTS_FOR_CITY)
            .bind(city_id)
            .bind(exclude_user_id)
            .fetch_one(&self.pool)
            .await?;

        info!(
            "[Repository] CITY_ADMIN exists for city {}: {}",
            city_id, result
        );

        Ok(result)
    }

    async fn get_user_policies_json_by_id(&self, id: Uuid) -> Result<JsonValue, sqlx::Error> {
        info!(
            "[Repository] Executing SQL query to get permission_policies for user {}",
            id
        );

        let policies: JsonValue = sqlx::query_scalar(UsersQueries::GET_USER_POLICIES_BY_ID)
            .bind(id)
            .fetch_one(&self.pool)
            .await?;

        info!("[Repository] Retrieved permission_policies for user {}", id);

        Ok(policies)
    }

    async fn update_user_policies_by_id(
        &self,
        id: Uuid,
        policies: JsonValue,
    ) -> Result<UserDataCreatedWithoutPassword, sqlx::Error> {
        info!(
            "[Repository] Executing SQL query to update permission_policies for user {}",
            id
        );
        let user: UserDataCreatedWithoutPassword =
            sqlx::query_as(UsersQueries::UPDATE_USER_POLICIES_BY_ID)
                .bind(id)
                .bind(policies)
                .fetch_one(&self.pool)
                .await?;
        info!("[Repository] Updated permission_policies for user {}", id);
        Ok(user)
    }
}
