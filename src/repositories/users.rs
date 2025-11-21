use async_trait::async_trait;
use log::info;
use sqlx::PgPool;
use uuid::Uuid;

use crate::core::{
    contracts::repository::users::{
        UserRepository
    },
    entities::users::{
        CreateUser,
        UpdateUser,
        UserDataCreatedWithoutPassword
    }
};
use crate::config::querys::users::UsersQueries;

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
    async fn create_user(&self, user: CreateUser) -> Result<UserDataCreatedWithoutPassword, sqlx::Error> {
        let id: Uuid = Uuid::new_v4();

        info!("[Repository] Executing SQL query to create user with email: {} and ID: {}", user.email, id);

        let user_created: UserDataCreatedWithoutPassword = sqlx::query_as(UsersQueries::CREATE_USER)
            .bind(id)
            .bind(user.rank)
            .bind(user.registration)
            .bind(user.full_name)
            .bind(user.profile)
            .bind(user.email)
            .bind(user.password)
            .fetch_one(&self.pool)
            .await?;

        info!("[Repository] User successfully inserted into database with ID: {}", user_created.id);

        Ok(user_created)
    }

    async fn update_user_by_id(&self, data: UpdateUser, id: Uuid) -> Result<UserDataCreatedWithoutPassword, sqlx::Error> {
        info!("[Repository] Executing SQL query to update user with ID: {}", id);

        let user_updated: UserDataCreatedWithoutPassword = sqlx::query_as(UsersQueries::UPDATE_USER_BY_ID)
            .bind(id)
            .bind(data.rank)
            .bind(data.registration)
            .bind(data.full_name)
            .bind(data.profile)
            .bind(data.email)
            .fetch_one(&self.pool)
            .await?;

        info!("[Repository] User successfully updated into database with ID: {}", id);

        Ok(user_updated)
    }

    async fn get_user_by_id(&self, id: Uuid) -> Result<UserDataCreatedWithoutPassword, sqlx::Error> {
        info!("[Repository] Executing SQL query to get user with id: {}", id);

        let user: UserDataCreatedWithoutPassword = sqlx::query_as(UsersQueries::GET_USER_BY_ID)
            .bind(id)
            .fetch_one(&self.pool)
            .await?;

        info!("[Repository] User successfully found in the database with IDD: {}", id);

        Ok(user)

    }

    async fn check_user_exists_by_email(&self, email: String) -> Result<bool, sqlx::Error> {
        info!("[Repository] Executing SQL query to check if user with email {} exists", email);

        let user_exists: bool = sqlx::query_scalar(UsersQueries::CHECK_USER_EXISTS_BY_EMAIL)
            .bind(email.clone())
            .fetch_one(&self.pool)
            .await?;

        info!("[Repository] User exists with email {}: {}", email, user_exists);

        Ok(user_exists)
    }

    async fn check_email_exists_for_other_user(&self, email: &str, id: Uuid) -> Result<bool, sqlx::Error> {

        info!("[Repository] Executing SQL query to check if email {} exists for other user", email);

        let result: bool = sqlx::query_scalar(UsersQueries::CHECK_EMAIL_EXISTS_FOR_OTHER_USER)
            .bind(email)
            .bind(id)
            .fetch_one(&self.pool)
            .await?;

        Ok(result)
    }

    async fn get_all_users(&self) -> Result<Vec<UserDataCreatedWithoutPassword>, sqlx::Error> {
        info!("[Repository] Executing SQL query to get all users");

        let users: Vec<UserDataCreatedWithoutPassword> = sqlx::query_as(UsersQueries::GET_ALL_USERS)
            .fetch_all(&self.pool)
            .await?;

        info!("[Repository] Found {} users in database", users.len());

        Ok(users)
    }

    async fn delete_user_by_id(&self, id: Uuid) -> Result<UserDataCreatedWithoutPassword, sqlx::Error> {
        info!("[Repository] Executing SQL query to delete user with id: {}", id);

        let deleted_user: UserDataCreatedWithoutPassword = sqlx::query_as(UsersQueries::DELETE_USER_BY_ID)
            .bind(id)
            .fetch_one(&self.pool)
            .await?;

        info!("[Repository] User successfully deleted from database with ID: {}", id);

        Ok(deleted_user)
    }

    async fn get_user_password_by_id(&self, id: Uuid) -> Result<String, sqlx::Error> {
        info!("[Repository] Executing SQL query to get password for user with id: {}", id);

        let password: String = sqlx::query_scalar(UsersQueries::GET_USER_PASSWORD_BY_ID)
            .bind(id)
            .fetch_one(&self.pool)
            .await?;

        info!("[Repository] Password retrieved successfully for user with ID: {}", id);

        Ok(password)
    }

    async fn update_user_password_by_id(&self, id: Uuid, new_password: String) -> Result<UserDataCreatedWithoutPassword, sqlx::Error> {
        info!("[Repository] Executing SQL query to update password for user with id: {}", id);

        let updated_user: UserDataCreatedWithoutPassword = sqlx::query_as(UsersQueries::UPDATE_USER_PASSWORD_BY_ID)
            .bind(id)
            .bind(new_password)
            .fetch_one(&self.pool)
            .await?;

        info!("[Repository] Password updated successfully for user with ID: {}", id);

        Ok(updated_user)
    }

}
