use crate::common::{fixtures, test_helpers};
use actix_web::{http::StatusCode, test};
use sqlx::PgPool;

#[sqlx::test]
async fn test_update_password_success(pool: PgPool) {
    let config = test_helpers::build_test_config();
    let app = test_helpers::create_full_test_app(pool.clone(), config.clone()).await;

    let claims = test_helpers::build_root_claims();
    let token = test_helpers::generate_jwt(&claims, &config.jwt_secret);

    // Create a user
    let user = fixtures::valid_create_user();
    let create_req = test_helpers::with_auth_headers(
        test::TestRequest::post()
            .uri("/api/v1/users")
            .set_json(&user),
        &config,
        &token,
    )
    .to_request();
    let create_resp = test::call_service(&app, create_req).await;
    let create_body: serde_json::Value = test::read_body_json(create_resp).await;
    let user_id = create_body["data"]["id"].as_str().unwrap();

    // User logs in
    let login_payload = serde_json::json!({
        "email": user.email,
        "password": user.password
    });

    let login_req = test::TestRequest::post()
        .uri("/api/v1/auth/login")
        .insert_header(("api_key", config.api_key.clone()))
        .set_json(&login_payload)
        .to_request();

    let login_resp = test::call_service(&app, login_req).await;
    let login_body: serde_json::Value = test::read_body_json(login_resp).await;
    let user_token = login_body["data"]["token"].as_str().unwrap();

    // Update password
    let password_data = fixtures::valid_update_password();
    let update_req = test_helpers::with_auth_headers(
        test::TestRequest::patch()
            .uri("/api/v1/users/password")
            .set_json(&password_data),
        &config,
        user_token,
    )
    .to_request();

    let update_resp = test::call_service(&app, update_req).await;
    assert_eq!(update_resp.status(), StatusCode::OK);

    let update_body: serde_json::Value = test::read_body_json(update_resp).await;
    assert_eq!(update_body["status"].as_u64().unwrap(), 200);
    assert_eq!(update_body["data"]["id"].as_str().unwrap(), user_id);
}

#[sqlx::test]
async fn test_update_password_incorrect_current(pool: PgPool) {
    let config = test_helpers::build_test_config();
    let app = test_helpers::create_full_test_app(pool.clone(), config.clone()).await;

    let city = crate::common::db_fixtures::insert_city(&pool, "Test City").await;

    let root_claims = test_helpers::build_root_claims();
    let root_token = test_helpers::generate_jwt(&root_claims, &config.jwt_secret);

    // Create a CITY_USER (non-ROOT users must provide correct current_password)
    let user_payload = serde_json::json!({
        "rank": "SD PM",
        "registration": "100012345",
        "full_name": "Test User",
        "profile": "CITY_USER",
        "email": "testuser@test.com",
        "password": "correctPassword123",
        "city_id": city,
        "permission_policies": null
    });

    let create_req = test_helpers::with_auth_headers(
        test::TestRequest::post()
            .uri("/api/v1/users")
            .set_json(&user_payload),
        &config,
        &root_token,
    )
    .to_request();
    let create_resp = test::call_service(&app, create_req).await;
    let create_body: serde_json::Value = test::read_body_json(create_resp).await;
    let _user_id = create_body["data"]["id"].as_str().unwrap();

    // User logs in
    let login_payload = serde_json::json!({
        "email": "testuser@test.com",
        "password": "correctPassword123"
    });

    let login_req = test::TestRequest::post()
        .uri("/api/v1/auth/login")
        .insert_header(("api_key", config.api_key.clone()))
        .set_json(&login_payload)
        .to_request();

    let login_resp = test::call_service(&app, login_req).await;
    let login_body: serde_json::Value = test::read_body_json(login_resp).await;
    let user_token = login_body["data"]["token"].as_str().unwrap();

    // Try to update password with INCORRECT current password
    let password_data = fixtures::invalid_update_password();
    let update_req = test_helpers::with_auth_headers(
        test::TestRequest::patch()
            .uri("/api/v1/users/password")
            .set_json(&password_data),
        &config,
        user_token,
    )
    .to_request();

    let update_resp = test::call_service(&app, update_req).await;
    assert_eq!(update_resp.status(), StatusCode::BAD_REQUEST);

    let update_body: serde_json::Value = test::read_body_json(update_resp).await;
    assert_eq!(update_body["status_code"].as_u64().unwrap(), 400);
    assert!(
        update_body["message"]
            .as_str()
            .unwrap()
            .contains("incorrect")
    );
}

#[sqlx::test]
async fn test_update_password_does_not_affect_other_users(pool: PgPool) {
    let config = test_helpers::build_test_config();
    let app = test_helpers::create_full_test_app(pool.clone(), config.clone()).await;

    let claims = test_helpers::build_root_claims();
    let token = test_helpers::generate_jwt(&claims, &config.jwt_secret);

    let user = fixtures::valid_create_user();
    let other_user = fixtures::valid_create_user_2();
    let create_req = test_helpers::with_auth_headers(
        test::TestRequest::post()
            .uri("/api/v1/users")
            .set_json(&user),
        &config,
        &token,
    )
    .to_request();
    let create_resp = test::call_service(&app, create_req).await;
    let _create_body: serde_json::Value = test::read_body_json(create_resp).await;

    let create_other_req = test_helpers::with_auth_headers(
        test::TestRequest::post()
            .uri("/api/v1/users")
            .set_json(&other_user),
        &config,
        &token,
    )
    .to_request();
    let create_other_resp = test::call_service(&app, create_other_req).await;
    let _create_other_body: serde_json::Value = test::read_body_json(create_other_resp).await;

    let login_payload = serde_json::json!({
        "email": user.email,
        "password": user.password
    });

    let login_req = test::TestRequest::post()
        .uri("/api/v1/auth/login")
        .insert_header(("api_key", config.api_key.clone()))
        .set_json(&login_payload)
        .to_request();

    let login_resp = test::call_service(&app, login_req).await;
    let login_body: serde_json::Value = test::read_body_json(login_resp).await;
    let user_token = login_body["data"]["token"].as_str().unwrap();

    let password_data = fixtures::valid_update_password();
    let req = test_helpers::with_auth_headers(
        test::TestRequest::patch()
            .uri("/api/v1/users/password")
            .set_json(&password_data),
        &config,
        user_token,
    )
    .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), StatusCode::OK);

    let other_login_payload = serde_json::json!({
        "email": other_user.email,
        "password": other_user.password
    });

    let other_login_req = test::TestRequest::post()
        .uri("/api/v1/auth/login")
        .insert_header(("api_key", config.api_key.clone()))
        .set_json(&other_login_payload)
        .to_request();

    let other_login_resp = test::call_service(&app, other_login_req).await;
    assert_eq!(other_login_resp.status(), StatusCode::OK);
}

#[sqlx::test]
async fn test_update_password_empty_fields(pool: PgPool) {
    let config = test_helpers::build_test_config();
    let app = test_helpers::create_full_test_app(pool.clone(), config.clone()).await;

    let claims = test_helpers::build_root_claims();
    let token = test_helpers::generate_jwt(&claims, &config.jwt_secret);

    // Create a user
    let user = fixtures::valid_create_user();
    let create_req = test_helpers::with_auth_headers(
        test::TestRequest::post()
            .uri("/api/v1/users")
            .set_json(&user),
        &config,
        &token,
    )
    .to_request();
    let create_resp = test::call_service(&app, create_req).await;
    let create_body: serde_json::Value = test::read_body_json(create_resp).await;
    let _user_id = create_body["data"]["id"].as_str().unwrap();

    // Try to update with empty password fields
    let password_data = serde_json::json!({
        "current_password": "",
        "new_password": ""
    });
    let login_payload = serde_json::json!({
        "email": user.email,
        "password": user.password
    });

    let login_req = test::TestRequest::post()
        .uri("/api/v1/auth/login")
        .insert_header(("api_key", config.api_key.clone()))
        .set_json(&login_payload)
        .to_request();

    let login_resp = test::call_service(&app, login_req).await;
    let login_body: serde_json::Value = test::read_body_json(login_resp).await;
    let user_token = login_body["data"]["token"].as_str().unwrap();

    let update_req = test_helpers::with_auth_headers(
        test::TestRequest::patch()
            .uri("/api/v1/users/password")
            .set_json(&password_data),
        &config,
        user_token,
    )
    .to_request();

    let update_resp = test::call_service(&app, update_req).await;
    assert_eq!(update_resp.status(), StatusCode::BAD_REQUEST);

    let update_body: serde_json::Value = test::read_body_json(update_resp).await;
    assert_eq!(update_body["status_code"].as_u64().unwrap(), 400);
    assert!(
        update_body["message"]
            .as_str()
            .unwrap()
            .contains("required")
    );
}

#[sqlx::test]
async fn test_non_root_must_provide_current_password(pool: PgPool) {
    let config = test_helpers::build_test_config();
    let app = test_helpers::create_full_test_app(pool.clone(), config.clone()).await;

    let city = crate::common::db_fixtures::insert_city(&pool, "Test City").await;

    let root_claims = test_helpers::build_root_claims();
    let root_token = test_helpers::generate_jwt(&root_claims, &config.jwt_secret);

    // Create a CITY_USER
    let user_payload = serde_json::json!({
        "rank": "SD PM",
        "registration": "100012345",
        "full_name": "City User",
        "profile": "CITY_USER",
        "email": "cityuser@test.com",
        "password": "userPassword123",
        "city_id": city,
        "permission_policies": null
    });

    let create_req = test_helpers::with_auth_headers(
        test::TestRequest::post()
            .uri("/api/v1/users")
            .set_json(&user_payload),
        &config,
        &root_token,
    )
    .to_request();
    let create_resp = test::call_service(&app, create_req).await;
    let create_body: serde_json::Value = test::read_body_json(create_resp).await;
    let _user_id = create_body["data"]["id"].as_str().unwrap();

    // User logs in
    let login_payload = serde_json::json!({
        "email": "cityuser@test.com",
        "password": "userPassword123"
    });

    let login_req = test::TestRequest::post()
        .uri("/api/v1/auth/login")
        .insert_header(("api_key", config.api_key.clone()))
        .set_json(&login_payload)
        .to_request();

    let login_resp = test::call_service(&app, login_req).await;
    let login_body: serde_json::Value = test::read_body_json(login_resp).await;
    let user_token = login_body["data"]["token"].as_str().unwrap();

    // Try to update password WITHOUT providing current_password
    let password_data = serde_json::json!({
        "new_password": "newPassword456"
    });

    let update_req = test_helpers::with_auth_headers(
        test::TestRequest::patch()
            .uri("/api/v1/users/password")
            .set_json(&password_data),
        &config,
        user_token,
    )
    .to_request();

    let update_resp = test::call_service(&app, update_req).await;

    // Should fail because non-ROOT users must provide current_password
    assert_eq!(update_resp.status(), StatusCode::BAD_REQUEST);

    let update_body: serde_json::Value = test::read_body_json(update_resp).await;
    assert_eq!(update_body["status_code"].as_u64().unwrap(), 400);
    assert!(
        update_body["message"]
            .as_str()
            .unwrap()
            .contains("current_password is required")
    );
}
