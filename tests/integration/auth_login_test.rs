use actix_web::{http::StatusCode, test};
use nupevid_api::adapters::password_hasher::{Argon2PasswordHasher, PasswordHasherPort};
use sqlx::PgPool;
use uuid::Uuid;

use crate::common::{db_fixtures, test_helpers};

async fn insert_test_user(pool: &PgPool, email: &str, plain_password: &str) {
    let hasher = Argon2PasswordHasher::new();
    let password_hash = hasher
        .hash_password(plain_password)
        .expect("Failed to hash password for test user");

    sqlx::query(
        "INSERT INTO users (id, rank, registration, full_name, profile, email, password, is_deleted) \
         VALUES ($1, $2, $3, $4, $5, $6, $7, false)",
    )
    .bind(Uuid::new_v4())
    .bind("Sargento")
    .bind(12345_i32)
    .bind("Test User")
    .bind("ROOT")
    .bind(email)
    .bind(password_hash)
    .execute(pool)
    .await
    .expect("Failed to insert test user");
}

#[actix_rt::test]
async fn login_success_root_user() {
    let pool = test_helpers::setup_test_db().await;
    test_helpers::clean_database(&pool).await;

    let config = test_helpers::build_test_config();
    let app = test_helpers::create_full_test_app(pool.clone(), config.clone()).await;

    let email = "login.success@test.com";
    let password = "StrongPassword123";
    insert_test_user(&pool, email, password).await;

    let payload = serde_json::json!({
        "email": email,
        "password": password,
    });

    let req = test::TestRequest::post()
        .uri("/api/v1/auth/login")
        .insert_header(("api_key", config.api_key.clone()))
        .set_json(&payload)
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), StatusCode::OK);

    let body: serde_json::Value = test::read_body_json(resp).await;
    assert_eq!(body["status"].as_u64().unwrap(), 200);
    assert!(body["data"]["token"].as_str().is_some());
    assert!(body["data"]["id"].as_str().is_some());
    assert_eq!(body["data"]["email"].as_str().unwrap(), email);
    assert_eq!(body["data"]["full_name"].as_str().unwrap(), "Test User");
    assert_eq!(body["data"]["rank"].as_str().unwrap(), "Sargento");
    assert_eq!(body["data"]["registration"].as_str().unwrap(), "12345");
    assert_eq!(body["data"]["profile"].as_str().unwrap(), "ROOT");
}

#[actix_rt::test]
async fn login_invalid_email_returns_unauthorized() {
    let pool = test_helpers::setup_test_db().await;
    test_helpers::clean_database(&pool).await;

    let config = test_helpers::build_test_config();
    let app = test_helpers::create_full_test_app(pool.clone(), config.clone()).await;

    let payload = serde_json::json!({
        "email": "non.existent@test.com",
        "password": "some-password",
    });

    let req = test::TestRequest::post()
        .uri("/api/v1/auth/login")
        .insert_header(("api_key", config.api_key.clone()))
        .set_json(&payload)
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), StatusCode::UNAUTHORIZED);

    let body: serde_json::Value = test::read_body_json(resp).await;
    assert_eq!(body["status_code"].as_u64().unwrap(), 401);
    assert!(
        body["message"]
            .as_str()
            .unwrap()
            .contains("Invalid credentials")
    );
}

#[actix_rt::test]
async fn login_invalid_password_returns_unauthorized() {
    let pool = test_helpers::setup_test_db().await;
    test_helpers::clean_database(&pool).await;

    let config = test_helpers::build_test_config();
    let app = test_helpers::create_full_test_app(pool.clone(), config.clone()).await;

    let email = "wrong.password@test.com";
    insert_test_user(&pool, email, "CorrectPassword123").await;

    let payload = serde_json::json!({
        "email": email,
        "password": "WrongPassword",
    });

    let req = test::TestRequest::post()
        .uri("/api/v1/auth/login")
        .insert_header(("api_key", config.api_key.clone()))
        .set_json(&payload)
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), StatusCode::UNAUTHORIZED);

    let body: serde_json::Value = test::read_body_json(resp).await;
    assert_eq!(body["status_code"].as_u64().unwrap(), 401);
    assert!(
        body["message"]
            .as_str()
            .unwrap()
            .contains("Invalid credentials")
    );
}

#[actix_rt::test]
async fn login_with_auto_create_session_true_creates_work_session() {
    let pool = test_helpers::setup_test_db().await;
    test_helpers::clean_database(&pool).await;

    let config = test_helpers::build_test_config();
    let app = test_helpers::create_full_test_app(pool.clone(), config.clone()).await;

    // Create city and user
    let city_id = db_fixtures::insert_city(&pool, "Test City").await;
    let user_id =
        db_fixtures::insert_user(&pool, "100001", "user@test.com", "CITY_USER", Some(city_id))
            .await;

    let payload = serde_json::json!({
        "email": "user@test.com",
        "password": "senha123",
        "auto_create_session": true,
    });

    let req = test::TestRequest::post()
        .uri("/api/v1/auth/login")
        .insert_header(("api_key", config.api_key.clone()))
        .set_json(&payload)
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), StatusCode::OK);

    let body: serde_json::Value = test::read_body_json(resp).await;
    assert_eq!(body["status"].as_u64().unwrap(), 200);
    assert!(body["data"]["token"].as_str().is_some());
    assert!(
        body["data"]["work_session"].is_object(),
        "work_session should be an object"
    );

    // Verify work session was created in database
    let session_id = body["data"]["work_session"]["id"].as_str().unwrap();
    let session_uuid = Uuid::parse_str(session_id).expect("Invalid session UUID");

    let session_exists: bool = sqlx::query_scalar(
        "SELECT EXISTS(SELECT 1 FROM work_sessions WHERE id = $1 AND created_by_user_id = $2)",
    )
    .bind(session_uuid)
    .bind(user_id)
    .fetch_one(&pool)
    .await
    .expect("Failed to check work session");

    assert!(session_exists, "Work session should exist in database");

    // Verify user is added as Commander
    let member_function: String = sqlx::query_scalar(
        "SELECT function::TEXT FROM work_session_members WHERE work_session_id = $1 AND user_id = $2"
    )
    .bind(session_uuid)
    .bind(user_id)
    .fetch_one(&pool)
    .await
    .expect("Failed to fetch member function");

    assert_eq!(member_function, "Commander");
}

#[actix_rt::test]
async fn login_with_auto_create_session_false_does_not_create_session() {
    let pool = test_helpers::setup_test_db().await;
    test_helpers::clean_database(&pool).await;

    let config = test_helpers::build_test_config();
    let app = test_helpers::create_full_test_app(pool.clone(), config.clone()).await;

    // Create city and user
    let city_id = db_fixtures::insert_city(&pool, "Test City").await;
    let user_id = db_fixtures::insert_user(
        &pool,
        "100002",
        "user2@test.com",
        "CITY_USER",
        Some(city_id),
    )
    .await;

    let payload = serde_json::json!({
        "email": "user2@test.com",
        "password": "senha123",
        "auto_create_session": false,
    });

    let req = test::TestRequest::post()
        .uri("/api/v1/auth/login")
        .insert_header(("api_key", config.api_key.clone()))
        .set_json(&payload)
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), StatusCode::OK);

    let body: serde_json::Value = test::read_body_json(resp).await;
    assert_eq!(body["status"].as_u64().unwrap(), 200);
    assert!(body["data"]["token"].as_str().is_some());
    assert!(
        body["data"]["work_session"].is_null(),
        "work_session should be null when auto_create_session is false"
    );

    // Verify no work session was created
    let session_count: i64 =
        sqlx::query_scalar("SELECT COUNT(*) FROM work_sessions WHERE created_by_user_id = $1")
            .bind(user_id)
            .fetch_one(&pool)
            .await
            .expect("Failed to count work sessions");

    assert_eq!(session_count, 0, "No work session should be created");
}

#[actix_rt::test]
async fn login_with_auto_create_session_returns_existing_active_session() {
    let pool = test_helpers::setup_test_db().await;
    test_helpers::clean_database(&pool).await;

    let config = test_helpers::build_test_config();
    let app = test_helpers::create_full_test_app(pool.clone(), config.clone()).await;

    // Create city and user
    let city_id = db_fixtures::insert_city(&pool, "Test City").await;
    let user_id = db_fixtures::insert_user(
        &pool,
        "100003",
        "user3@test.com",
        "CITY_USER",
        Some(city_id),
    )
    .await;

    // Create an existing active work session
    let existing_session_id = test_helpers::create_work_session_for_user(&pool, user_id).await;

    let payload = serde_json::json!({
        "email": "user3@test.com",
        "password": "senha123",
        "auto_create_session": true,
    });

    let req = test::TestRequest::post()
        .uri("/api/v1/auth/login")
        .insert_header(("api_key", config.api_key.clone()))
        .set_json(&payload)
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), StatusCode::OK);

    let body: serde_json::Value = test::read_body_json(resp).await;
    assert_eq!(body["status"].as_u64().unwrap(), 200);
    assert!(body["data"]["token"].as_str().is_some());
    assert!(
        body["data"]["work_session"].is_object(),
        "work_session should be an object"
    );

    // Verify it returns the existing session, not a new one
    let returned_session_id = body["data"]["work_session"]["id"].as_str().unwrap();
    assert_eq!(returned_session_id, existing_session_id.to_string());

    // Verify only one session exists
    let session_count: i64 =
        sqlx::query_scalar("SELECT COUNT(*) FROM work_sessions WHERE created_by_user_id = $1")
            .bind(user_id)
            .fetch_one(&pool)
            .await
            .expect("Failed to count work sessions");

    assert_eq!(
        session_count, 1,
        "Should return existing session, not create new one"
    );
}
