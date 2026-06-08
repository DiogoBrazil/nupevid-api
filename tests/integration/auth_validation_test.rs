use actix_web::{http::StatusCode, test};
use chrono::{Duration, Utc};
use sqlx::PgPool;

use crate::common::{db_fixtures, test_helpers};

#[sqlx::test]
async fn login_with_empty_email_returns_unauthorized(pool: PgPool) {
    let config = test_helpers::build_test_config();
    let app = test_helpers::create_full_test_app(pool.clone(), config.clone()).await;

    let payload = serde_json::json!({
        "email": "",
        "password": "somepassword"
    });

    let req = test::TestRequest::post()
        .uri("/api/v1/auth/login")
        .insert_header(("api_key", config.api_key.clone()))
        .set_json(&payload)
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), StatusCode::UNAUTHORIZED);
}

#[sqlx::test]
async fn login_with_empty_password_returns_unauthorized(pool: PgPool) {
    let config = test_helpers::build_test_config();
    let app = test_helpers::create_full_test_app(pool.clone(), config.clone()).await;

    let payload = serde_json::json!({
        "email": "test@test.com",
        "password": ""
    });

    let req = test::TestRequest::post()
        .uri("/api/v1/auth/login")
        .insert_header(("api_key", config.api_key.clone()))
        .set_json(&payload)
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), StatusCode::UNAUTHORIZED);
}

#[sqlx::test]
async fn login_with_invalid_email_format_returns_unauthorized(pool: PgPool) {
    let config = test_helpers::build_test_config();
    let app = test_helpers::create_full_test_app(pool.clone(), config.clone()).await;

    let payload = serde_json::json!({
        "email": "not-an-email",
        "password": "somepassword"
    });

    let req = test::TestRequest::post()
        .uri("/api/v1/auth/login")
        .insert_header(("api_key", config.api_key.clone()))
        .set_json(&payload)
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), StatusCode::UNAUTHORIZED);
}

#[sqlx::test]
async fn login_without_api_key_header_returns_unauthorized(pool: PgPool) {
    let config = test_helpers::build_test_config();
    let app = test_helpers::create_full_test_app(pool.clone(), config.clone()).await;

    let payload = serde_json::json!({
        "email": "test@test.com",
        "password": "password123"
    });

    let req = test::TestRequest::post()
        .uri("/api/v1/auth/login")
        .set_json(&payload)
        .to_request();

    let result = test::try_call_service(&app, req).await;
    assert!(
        result.is_err(),
        "Expected unauthorized error without api_key"
    );
}

#[sqlx::test]
async fn login_with_invalid_api_key_returns_unauthorized(pool: PgPool) {
    let config = test_helpers::build_test_config();
    let app = test_helpers::create_full_test_app(pool.clone(), config.clone()).await;

    let payload = serde_json::json!({
        "email": "test@test.com",
        "password": "password123"
    });

    let req = test::TestRequest::post()
        .uri("/api/v1/auth/login")
        .insert_header(("api_key", "wrong-api-key"))
        .set_json(&payload)
        .to_request();

    let result = test::try_call_service(&app, req).await;
    assert!(
        result.is_err(),
        "Expected unauthorized error with invalid api_key"
    );
}

#[sqlx::test]
async fn protected_route_without_authorization_header_returns_unauthorized(pool: PgPool) {
    let config = test_helpers::build_test_config();
    let app = test_helpers::create_full_test_app(pool.clone(), config.clone()).await;

    let req = test::TestRequest::get()
        .uri("/api/v1/users")
        .insert_header(("api_key", config.api_key.clone()))
        .to_request();

    let err = test::try_call_service(&app, req)
        .await
        .expect_err("protected route without Authorization must fail");

    assert!(err.to_string().contains("Invalid authorization header"));
}

#[sqlx::test]
async fn protected_route_with_malformed_authorization_header_returns_unauthorized(pool: PgPool) {
    let config = test_helpers::build_test_config();
    let app = test_helpers::create_full_test_app(pool.clone(), config.clone()).await;

    let req = test::TestRequest::get()
        .uri("/api/v1/users")
        .insert_header(("api_key", config.api_key.clone()))
        .insert_header(("Authorization", "Token malformed"))
        .to_request();

    let err = test::try_call_service(&app, req)
        .await
        .expect_err("protected route with malformed Authorization must fail");

    assert!(err.to_string().contains("Invalid authorization header"));
}

#[sqlx::test]
async fn protected_route_with_invalid_bearer_token_returns_unauthorized(pool: PgPool) {
    let config = test_helpers::build_test_config();
    let app = test_helpers::create_full_test_app(pool.clone(), config.clone()).await;

    let req = test::TestRequest::get()
        .uri("/api/v1/users")
        .insert_header(("api_key", config.api_key.clone()))
        .insert_header(("Authorization", "Bearer definitely-invalid-token"))
        .to_request();

    let err = test::try_call_service(&app, req)
        .await
        .expect_err("protected route with invalid bearer token must fail");

    assert!(err.to_string().contains("Invalid token"));
}

#[sqlx::test]
async fn login_with_expired_temporary_password_returns_unauthorized_with_specific_message(pool: PgPool) {
    let config = test_helpers::build_test_config();
    let app = test_helpers::create_full_test_app(pool.clone(), config.clone()).await;

    let city = db_fixtures::insert_city(&pool, "Cidade Auth").await;
    let user_id = db_fixtures::insert_user(
        &pool,
        "100055555",
        "temporary.expired@test.com",
        "CITY_USER",
        Some(city),
    )
    .await;

    let root_claims = test_helpers::build_root_claims();
    let root_token = test_helpers::generate_jwt(&root_claims, &config.jwt_secret);

    let reset_req = test_helpers::with_auth_headers(
        test::TestRequest::post().uri(&format!("/api/v1/users/{}/password/reset", user_id)),
        &config,
        &root_token,
    )
    .to_request();
    let reset_resp = test::call_service(&app, reset_req).await;
    assert_eq!(reset_resp.status(), StatusCode::OK);
    let reset_body: serde_json::Value = test::read_body_json(reset_resp).await;
    let temporary_password = reset_body["data"]["temporary_password"]
        .as_str()
        .unwrap()
        .to_string();

    sqlx::query(
        "UPDATE users SET temporary_password_expires_at = $2 WHERE id = $1 AND is_deleted = false",
    )
    .bind(user_id)
    .bind(Utc::now() - Duration::hours(2))
    .execute(&pool)
    .await
    .expect("failed to expire temporary password in test");

    let login_payload = serde_json::json!({
        "email": "temporary.expired@test.com",
        "password": temporary_password
    });

    let login_req = test::TestRequest::post()
        .uri("/api/v1/auth/login")
        .insert_header(("api_key", config.api_key.clone()))
        .set_json(&login_payload)
        .to_request();

    let login_resp = test::call_service(&app, login_req).await;
    assert_eq!(login_resp.status(), StatusCode::UNAUTHORIZED);
    let body: serde_json::Value = test::read_body_json(login_resp).await;
    assert_eq!(
        body["message"].as_str().unwrap(),
        "Unauthorized: Temporary password expired"
    );
}

#[sqlx::test]
async fn login_with_temporary_password_missing_expiration_returns_unauthorized(pool: PgPool) {
    let config = test_helpers::build_test_config();
    let app = test_helpers::create_full_test_app(pool.clone(), config.clone()).await;

    let city = db_fixtures::insert_city(&pool, "Cidade Auth 2").await;
    let user_id = db_fixtures::insert_user(
        &pool,
        "100055556",
        "temporary.no-expiration@test.com",
        "CITY_USER",
        Some(city),
    )
    .await;

    let root_claims = test_helpers::build_root_claims();
    let root_token = test_helpers::generate_jwt(&root_claims, &config.jwt_secret);

    let reset_req = test_helpers::with_auth_headers(
        test::TestRequest::post().uri(&format!("/api/v1/users/{}/password/reset", user_id)),
        &config,
        &root_token,
    )
    .to_request();
    let reset_resp = test::call_service(&app, reset_req).await;
    assert_eq!(reset_resp.status(), StatusCode::OK);
    let reset_body: serde_json::Value = test::read_body_json(reset_resp).await;
    let temporary_password = reset_body["data"]["temporary_password"]
        .as_str()
        .unwrap()
        .to_string();

    sqlx::query(
        "UPDATE users SET temporary_password_expires_at = NULL WHERE id = $1 AND is_deleted = false",
    )
    .bind(user_id)
    .execute(&pool)
    .await
    .expect("failed to clear temporary password expiration in test");

    let login_payload = serde_json::json!({
        "email": "temporary.no-expiration@test.com",
        "password": temporary_password
    });

    let login_req = test::TestRequest::post()
        .uri("/api/v1/auth/login")
        .insert_header(("api_key", config.api_key.clone()))
        .set_json(&login_payload)
        .to_request();

    let login_resp = test::call_service(&app, login_req).await;
    assert_eq!(login_resp.status(), StatusCode::UNAUTHORIZED);
    let body: serde_json::Value = test::read_body_json(login_resp).await;
    assert_eq!(
        body["message"].as_str().unwrap(),
        "Unauthorized: Temporary password expired"
    );
}
