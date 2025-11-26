use actix_web::{http::StatusCode, test};

use crate::common::test_helpers;

#[actix_rt::test]
async fn login_with_empty_email_returns_unauthorized() {
    let pool = test_helpers::setup_test_db().await;
    test_helpers::clean_database(&pool).await;

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

#[actix_rt::test]
async fn login_with_empty_password_returns_unauthorized() {
    let pool = test_helpers::setup_test_db().await;
    test_helpers::clean_database(&pool).await;

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

#[actix_rt::test]
async fn login_with_invalid_email_format_returns_unauthorized() {
    let pool = test_helpers::setup_test_db().await;
    test_helpers::clean_database(&pool).await;

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

#[actix_rt::test]
async fn login_without_api_key_header_returns_unauthorized() {
    let pool = test_helpers::setup_test_db().await;
    test_helpers::clean_database(&pool).await;

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
    assert!(result.is_err(), "Expected unauthorized error without api_key");
}

#[actix_rt::test]
async fn login_with_invalid_api_key_returns_unauthorized() {
    let pool = test_helpers::setup_test_db().await;
    test_helpers::clean_database(&pool).await;

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
    assert!(result.is_err(), "Expected unauthorized error with invalid api_key");
}
