use actix_web::{App, Error, dev::Service, test, web};
use jsonwebtoken::{EncodingKey, Header, encode};
use nupevid_api::app_factory::AppDependencies;
use nupevid_api::config::{config_env::Config, database::init_database};
use nupevid_api::core::entities::auth::UserClaims;
use nupevid_api::core::value_objects::profiles::Profile;
use nupevid_api::core::value_objects::ranks::Rank;
use nupevid_api::middleware::auth::AuthMiddleware;
use sqlx::PgPool;
use std::env;
use std::time::{SystemTime, UNIX_EPOCH};
use uuid::Uuid;

/// Setup test database pool
pub async fn setup_test_db() -> PgPool {
    dotenv::dotenv().ok();

    let database_url = env::var("DATABASE_TEST_URL")
        .unwrap_or_else(|_| "postgres://postgres:postgres@localhost:5432/nupevid_test".to_string());

    let pool = init_database(&database_url, 5).await;

    // Run migrations
    sqlx::migrate!("./migrations")
        .run(&pool)
        .await
        .expect("Failed to run migrations");

    pool
}

/// Build a Config instance suitable for tests, using DATABASE_TEST_URL and
/// default values for SERVER_ADDR / JWT_SECRET / API_KEY when not provided.
pub fn build_test_config() -> Config {
    dotenv::dotenv().ok();

    let database_url = env::var("DATABASE_TEST_URL")
        .unwrap_or_else(|_| "postgres://postgres:postgres@localhost:5432/nupevid_test".to_string());

    let server_addr = env::var("SERVER_ADDR").unwrap_or_else(|_| "127.0.0.1:0".to_string());
    let jwt_secret = env::var("JWT_SECRET").unwrap_or_else(|_| "test-jwt-secret".to_string());
    let api_key = env::var("API_KEY").unwrap_or_else(|_| "test-api-key".to_string());
    let jwt_issuer = env::var("JWT_ISSUER").unwrap_or_else(|_| "nupevid-api".to_string());
    let jwt_audience = env::var("JWT_AUDIENCE").unwrap_or_else(|_| "nupevid-api".to_string());

    Config {
        database_url,
        server_addr,
        jwt_secret,
        api_key,
        jwt_issuer,
        jwt_audience,
        db_max_connections: 5,
        enable_bootstrap_root: false,
    }
}

/// Clean all domain tables from database in foreign-key safe order.
pub async fn clean_database(pool: &PgPool) {
    // Child tables first, then parents
    for sql in [
        "DELETE FROM attendance_victim_members",
        "DELETE FROM attendance_offender_members",
        "DELETE FROM attendance_offender_addresses",
        "DELETE FROM attendance_offenders",
        "DELETE FROM attendance_victim_addresses",
        "DELETE FROM attendance_victims",
        "DELETE FROM work_session_members",
        "DELETE FROM work_sessions",
        "DELETE FROM protective_measure_extensions",
        "DELETE FROM protective_measures",
        "DELETE FROM offender_phones",
        "DELETE FROM offender_addresses",
        "DELETE FROM offenders",
        "DELETE FROM victim_phones",
        "DELETE FROM victim_addresses",
        "DELETE FROM victims",
        "DELETE FROM users",
        "DELETE FROM cities",
    ] {
        sqlx::query(sql)
            .execute(pool)
            .await
            .expect("Failed to clean table");
    }
}

/// Create a full test app mirroring main.rs, with AuthMiddleware and /api/v1 routes.
pub async fn create_full_test_app(
    pool: PgPool,
    config: Config,
) -> impl Service<actix_http::Request, Response = actix_web::dev::ServiceResponse, Error = Error> {
    let deps = AppDependencies::new(pool, config);

    test::init_service(
        App::new()
            .wrap(AuthMiddleware)
            .configure(|cfg: &mut web::ServiceConfig| deps.configure(cfg)),
    )
    .await
}

/// Build common JWT claims helpers for tests.
fn default_exp() -> usize {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs() as usize
        + 3600
}

pub fn build_root_claims() -> UserClaims {
    UserClaims {
        id: Uuid::new_v4().to_string(),
        exp: default_exp(),
        iss: "nupevid-api".to_string(),
        aud: "nupevid-api".to_string(),
        rank: Rank::CelPm,
        registration: "100000001".to_string(),
        full_name: "Root User".to_string(),
        profile: Profile::Root,
        email: "root@test.com".to_string(),
        city_id: None,
    }
}

pub fn build_city_admin_claims(city_id: Uuid) -> UserClaims {
    UserClaims {
        id: Uuid::new_v4().to_string(),
        exp: default_exp(),
        iss: "nupevid-api".to_string(),
        aud: "nupevid-api".to_string(),
        rank: Rank::CapPm,
        registration: "100000002".to_string(),
        full_name: "City Admin".to_string(),
        profile: Profile::CityAdmin,
        email: "city.admin@test.com".to_string(),
        city_id: Some(city_id.to_string()),
    }
}

pub fn build_city_user_claims(city_id: Uuid) -> UserClaims {
    UserClaims {
        id: Uuid::new_v4().to_string(),
        exp: default_exp(),
        iss: "nupevid-api".to_string(),
        aud: "nupevid-api".to_string(),
        rank: Rank::SdPm,
        registration: "100000003".to_string(),
        full_name: "City User".to_string(),
        profile: Profile::CityUser,
        email: "city.user@test.com".to_string(),
        city_id: Some(city_id.to_string()),
    }
}

/// Generate a signed JWT for the given claims and secret.
pub fn generate_jwt(claims: &UserClaims, secret: &str) -> String {
    encode(
        &Header::default(),
        claims,
        &EncodingKey::from_secret(secret.as_bytes()),
    )
    .expect("Failed to encode JWT for tests")
}

/// Convenience helper to add api_key and Authorization headers to a TestRequest.
pub fn with_auth_headers(
    req: test::TestRequest,
    config: &Config,
    token: &str,
) -> test::TestRequest {
    req.insert_header(("api_key", config.api_key.clone()))
        .insert_header(("Authorization", format!("Bearer {}", token)))
}

/// Helper to create a work session directly in the database for testing
pub async fn create_work_session_for_user(pool: &PgPool, user_id: Uuid) -> Uuid {
    let session_id = Uuid::new_v4();
    let session_member_registration_id = Uuid::new_v4();

    // Create work session
    sqlx::query(
        "INSERT INTO work_sessions (id, created_by_user_id, description) VALUES ($1, $2, $3)",
    )
    .bind(session_id)
    .bind(user_id)
    .bind("Test session")
    .execute(pool)
    .await
    .expect("Failed to create work session for test");

    // Add user as Commander
    sqlx::query(
        "INSERT INTO work_session_members (id, work_session_id, user_id, function) VALUES ($1, $2, $3, $4::team_member_function)"
    )
    .bind(session_member_registration_id)
    .bind(session_id)
    .bind(user_id)
    .bind("Commander")
    .execute(pool)
    .await
    .expect("Failed to add user to work session for test");

    session_id
}
