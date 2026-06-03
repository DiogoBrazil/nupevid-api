use actix_web::{App, Error, dev::Service, test, web};
use jsonwebtoken::{EncodingKey, Header, encode};
use nupevid_api::app_factory::AppDependencies;
use nupevid_api::config::config_env::Config;
use nupevid_api::core::entities::auth::UserClaims;
use nupevid_api::core::value_objects::profiles::Profile;
use nupevid_api::core::value_objects::ranks::Rank;
use nupevid_api::middleware::auth::AuthMiddleware;
use sqlx::PgPool;
use std::time::{SystemTime, UNIX_EPOCH};
use uuid::Uuid;

/// Build a Config instance suitable for tests.
pub fn build_test_config() -> Config {
    dotenv::dotenv().ok();

    let server_addr = std::env::var("SERVER_ADDR").unwrap_or_else(|_| "127.0.0.1:0".to_string());
    let jwt_secret = std::env::var("JWT_SECRET").unwrap_or_else(|_| "test-jwt-secret".to_string());
    let api_key = std::env::var("API_KEY").unwrap_or_else(|_| "test-api-key".to_string());
    let jwt_issuer = std::env::var("JWT_ISSUER").unwrap_or_else(|_| "nupevid-api".to_string());
    let jwt_audience =
        std::env::var("JWT_AUDIENCE").unwrap_or_else(|_| "nupevid-api".to_string());

    Config {
        database_url: String::new(),
        server_addr,
        jwt_secret,
        api_key,
        jwt_issuer,
        jwt_audience,
        db_max_connections: 5,
        enable_bootstrap_root: false,
        access_token_ttl_seconds: 900,
        refresh_token_ttl_seconds: 604800,
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
