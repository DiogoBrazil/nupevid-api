use actix_web::{App, Error, dev::Service, test, web};
use jsonwebtoken::{EncodingKey, Header, encode};
use nupevid_api::adapters::{
    password_hasher::Argon2PasswordHasher, token_generator::JwtTokenGenerator,
};
use nupevid_api::config::{config_env::Config, database::init_database};
use nupevid_api::core::entities::auth::ClaimsToUserToken;
use nupevid_api::middleware::auth::AuthMiddleware;
use nupevid_api::repositories::{
    attendance_members::PgAttendanceMemberRepository,
    attendance_offenders::PgAttendanceOffenderRepository,
    attendance_victims::PgAttendanceVictimRepository, auth::PgAuthRepository,
    cities::PgCityRepository, extensions::PgExtensionRepository, offenders::PgOffenderRepository,
    protective_measures::PgProtectiveMeasureRepository, users::PgUserRepository,
    victims::PgVictimRepository, work_sessions::PgWorkSessionRepository,
};
use nupevid_api::routes::config::base_routes::configure_routes as configure_base_routes;
use nupevid_api::services::{
    attendance_offenders::AttendanceOffenderService, attendance_victims::AttendanceVictimService,
    auth::AuthService, cities::CityService, extensions::ExtensionService,
    offenders::OffenderService, protective_measures::ProtectiveMeasureService, users::UserService,
    victims::VictimService, work_sessions::WorkSessionService,
};
use sqlx::PgPool;
use std::env;
use std::time::{SystemTime, UNIX_EPOCH};
use uuid::Uuid;

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

/// Build a Config instance suitable for tests, using DATABASE_TEST_URL and
/// default values for SERVER_ADDR / JWT_SECRET / API_KEY when not provided.
pub fn build_test_config() -> Config {
    dotenv::dotenv().ok();

    let database_url = env::var("DATABASE_TEST_URL")
        .unwrap_or_else(|_| "postgres://postgres:postgres@localhost:5432/nupevid_test".to_string());

    let sercer_addr = env::var("SERVER_ADDR").unwrap_or_else(|_| "127.0.0.1:0".to_string());
    let jwt_secret = env::var("JWT_SECRET").unwrap_or_else(|_| "test-jwt-secret".to_string());
    let api_key = env::var("API_KEY").unwrap_or_else(|_| "test-api-key".to_string());

    Config {
        database_url,
        sercer_addr,
        jwt_secret,
        api_key,
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
    // Adapters
    let password_hasher = Box::new(Argon2PasswordHasher::new());
    let token_generator = Box::new(JwtTokenGenerator::new());

    // Repositories
    let user_repository = web::Data::new(PgUserRepository::new(pool.clone()));
    let auth_repository = web::Data::new(PgAuthRepository::new(pool.clone()));
    let city_repository = web::Data::new(PgCityRepository::new(pool.clone()));
    let victim_repository = web::Data::new(PgVictimRepository::new(pool.clone()));
    let protective_measure_repository =
        web::Data::new(PgProtectiveMeasureRepository::new(pool.clone()));
    let extension_repository = web::Data::new(PgExtensionRepository::new(pool.clone()));
    let attendance_repository = web::Data::new(PgAttendanceVictimRepository::new(pool.clone()));
    let attendance_offender_repository =
        web::Data::new(PgAttendanceOffenderRepository::new(pool.clone()));
    let offender_repository = web::Data::new(PgOffenderRepository::new(pool.clone()));
    let work_session_repository = web::Data::new(PgWorkSessionRepository::new(pool.clone()));
    let attendance_member_repository =
        web::Data::new(PgAttendanceMemberRepository::new(pool.clone()));

    // Shared config
    let config_data = web::Data::new(config.clone());

    // Services
    let user_service = web::Data::new(UserService::new(
        user_repository.clone(),
        password_hasher.clone(),
    ));
    let auth_service = web::Data::new(AuthService::new(
        auth_repository.clone(),
        work_session_repository.clone(),
        config_data.clone(),
        password_hasher.clone(),
        token_generator.clone(),
    ));
    let city_service = web::Data::new(CityService::new(
        city_repository.clone(),
        user_repository.clone(),
    ));
    let victim_service = web::Data::new(VictimService::new(
        victim_repository.clone(),
        user_repository.clone(),
    ));
    let protective_measure_service = web::Data::new(ProtectiveMeasureService::new(
        protective_measure_repository.clone(),
        victim_repository.clone(),
        offender_repository.clone(),
        user_repository.clone(),
        extension_repository.clone(),
    ));
    let extension_service = web::Data::new(ExtensionService::new(
        extension_repository.clone(),
        protective_measure_repository.clone(),
        victim_repository.clone(),
        user_repository.clone(),
    ));
    let attendance_service = web::Data::new(AttendanceVictimService::new(
        attendance_repository.clone(),
        victim_repository.clone(),
        user_repository.clone(),
        work_session_repository.clone(),
        attendance_member_repository.clone(),
    ));
    let offender_service = web::Data::new(OffenderService::new(
        offender_repository.clone(),
        user_repository.clone(),
    ));
    let attendance_offender_service = web::Data::new(AttendanceOffenderService::new(
        attendance_offender_repository.clone(),
        offender_repository.clone(),
        victim_repository.clone(),
        protective_measure_repository.clone(),
        user_repository.clone(),
        work_session_repository.clone(),
        attendance_member_repository.clone(),
    ));
    let work_session_service = web::Data::new(WorkSessionService::new(
        work_session_repository.clone(),
        user_repository.clone(),
    ));

    test::init_service(
        App::new()
            .wrap(AuthMiddleware)
            .app_data(user_repository.clone())
            .app_data(auth_repository.clone())
            .app_data(city_repository.clone())
            .app_data(victim_repository.clone())
            .app_data(protective_measure_repository.clone())
            .app_data(extension_repository.clone())
            .app_data(attendance_repository.clone())
            .app_data(attendance_offender_repository.clone())
            .app_data(offender_repository.clone())
            .app_data(work_session_repository.clone())
            .app_data(attendance_member_repository.clone())
            .app_data(user_service.clone())
            .app_data(auth_service.clone())
            .app_data(city_service.clone())
            .app_data(victim_service.clone())
            .app_data(protective_measure_service.clone())
            .app_data(extension_service.clone())
            .app_data(attendance_service.clone())
            .app_data(attendance_offender_service.clone())
            .app_data(offender_service.clone())
            .app_data(work_session_service.clone())
            .app_data(config_data.clone())
            .configure(configure_base_routes),
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

pub fn build_root_claims() -> ClaimsToUserToken {
    ClaimsToUserToken {
        id: Uuid::new_v4().to_string(),
        exp: default_exp(),
        rank: "CEL PM".to_string(),
        registration: "100000001".to_string(),
        full_name: "Root User".to_string(),
        profile: "ROOT".to_string(),
        email: "root@test.com".to_string(),
        city_id: None,
    }
}

pub fn build_city_admin_claims(city_id: Uuid) -> ClaimsToUserToken {
    ClaimsToUserToken {
        id: Uuid::new_v4().to_string(),
        exp: default_exp(),
        rank: "CAP PM".to_string(),
        registration: "100000002".to_string(),
        full_name: "City Admin".to_string(),
        profile: "CITY_ADMIN".to_string(),
        email: "city.admin@test.com".to_string(),
        city_id: Some(city_id.to_string()),
    }
}

pub fn build_city_user_claims(city_id: Uuid) -> ClaimsToUserToken {
    ClaimsToUserToken {
        id: Uuid::new_v4().to_string(),
        exp: default_exp(),
        rank: "SD PM".to_string(),
        registration: "100000003".to_string(),
        full_name: "City User".to_string(),
        profile: "CITY_USER".to_string(),
        email: "city.user@test.com".to_string(),
        city_id: Some(city_id.to_string()),
    }
}

/// Generate a signed JWT for the given claims and secret.
pub fn generate_jwt(claims: &ClaimsToUserToken, secret: &str) -> String {
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
