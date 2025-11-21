use actix_web::{test, web, App, dev::Service, Error};
use nupevid_api::adapters::password_hasher::Argon2PasswordHasher;
use nupevid_api::config::database::init_database;
use nupevid_api::repositories::users::PgUserRepository;
use nupevid_api::routes::users::configure_routes;
use nupevid_api::services::users::UserService;
use sqlx::PgPool;
use std::env;

/// Setup test database pool
pub async fn setup_test_db() -> PgPool {
    dotenv::dotenv().ok();

    let database_url = env::var("DATABASE_TEST_URL")
        .unwrap_or_else(|_| "postgres://postgres:postgres@localhost:5432/nupevid_test".to_string());

    let pool = init_database(&database_url).await;

    // Run migrations
    sqlx::migrate!("./migrations")
        .run(&pool)
        .await
        .expect("Failed to run migrations");

    pool
}

/// Clean all users from database
pub async fn clean_users_table(pool: &PgPool) {
    sqlx::query("DELETE FROM users")
        .execute(pool)
        .await
        .expect("Failed to clean users table");
}

/// Create test app with all dependencies
pub async fn create_test_app(pool: PgPool) -> impl Service<actix_http::Request, Response = actix_web::dev::ServiceResponse, Error = Error> {
    let user_repository = web::Data::new(PgUserRepository::new(pool.clone()));
    let password_hasher = Box::new(Argon2PasswordHasher::new());
    let user_service = web::Data::new(UserService::new(
        user_repository.clone(),
        password_hasher,
    ));

    test::init_service(
        App::new()
            .app_data(user_repository)
            .app_data(user_service)
            .configure(configure_routes),
    )
    .await
}
