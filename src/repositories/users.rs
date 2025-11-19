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
}
