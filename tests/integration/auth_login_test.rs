use actix_web::{test, http::StatusCode};
use nupevid_api::adapters::password_hasher::{Argon2PasswordHasher, PasswordHasherPort};
use sqlx::PgPool;
use uuid::Uuid;

use crate::common::test_helpers;

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
    assert_eq!(body["data"]["email"].as_str().unwrap(), email);
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
    assert!(body["message"].as_str().unwrap().contains("Invalid credentials"));
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
    assert!(body["message"].as_str().unwrap().contains("Invalid credentials"));
}