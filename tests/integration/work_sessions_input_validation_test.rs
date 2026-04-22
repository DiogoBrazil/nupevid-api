use actix_web::{http::StatusCode, test};
use sqlx::PgPool;

use crate::common::{db_fixtures, test_helpers};

/// Phase 5 - Test 1: Create session with empty description (allowed since optional)
#[sqlx::test]
async fn create_session_with_empty_description(pool: PgPool) {
    let config = test_helpers::build_test_config();
    let app = test_helpers::create_full_test_app(pool.clone(), config.clone()).await;

    let city = db_fixtures::insert_city(&pool, "Test City").await;
    let user_id =
        db_fixtures::insert_user(&pool, "200001", "user@test.com", "CITY_USER", Some(city)).await;

    let mut claims = test_helpers::build_city_user_claims(city);
    claims.id = user_id.to_string();
    let token = test_helpers::generate_jwt(&claims, &config.jwt_secret);

    let payload = serde_json::json!({
        "description": "",
        "members": [
            {
                "user_id": user_id,
                "function": "Commander"
            }
        ]
    });

    let req = test_helpers::with_auth_headers(
        test::TestRequest::post()
            .uri("/api/v1/work-sessions")
            .set_json(&payload),
        &config,
        &token,
    )
    .to_request();

    let resp = test::call_service(&app, req).await;
    // Empty string is valid since description is Option<String>
    assert_eq!(resp.status(), StatusCode::CREATED);
}

/// Phase 5 - Test 2: Create session with null description (allowed since optional)
#[sqlx::test]
async fn create_session_with_null_description(pool: PgPool) {
    let config = test_helpers::build_test_config();
    let app = test_helpers::create_full_test_app(pool.clone(), config.clone()).await;

    let city = db_fixtures::insert_city(&pool, "Test City").await;
    let user_id =
        db_fixtures::insert_user(&pool, "200002", "user@test.com", "CITY_USER", Some(city)).await;

    let mut claims = test_helpers::build_city_user_claims(city);
    claims.id = user_id.to_string();
    let token = test_helpers::generate_jwt(&claims, &config.jwt_secret);

    let payload = serde_json::json!({
        "description": null,
        "members": [
            {
                "user_id": user_id,
                "function": "Commander"
            }
        ]
    });

    let req = test_helpers::with_auth_headers(
        test::TestRequest::post()
            .uri("/api/v1/work-sessions")
            .set_json(&payload),
        &config,
        &token,
    )
    .to_request();

    let resp = test::call_service(&app, req).await;
    // Null is valid since description is Option<String>
    assert_eq!(resp.status(), StatusCode::CREATED);
}

/// Phase 5 - Test 3: Add member with invalid function
#[sqlx::test]
async fn add_member_with_invalid_function(pool: PgPool) {
    let config = test_helpers::build_test_config();
    let app = test_helpers::create_full_test_app(pool.clone(), config.clone()).await;

    let city = db_fixtures::insert_city(&pool, "Test City").await;
    let creator_id = db_fixtures::insert_user(
        &pool,
        "200003",
        "creator@test.com",
        "CITY_ADMIN",
        Some(city),
    )
    .await;
    let member_id =
        db_fixtures::insert_user(&pool, "200004", "member@test.com", "CITY_USER", Some(city)).await;

    let mut creator_claims = test_helpers::build_city_admin_claims(city);
    creator_claims.id = creator_id.to_string();
    let creator_token = test_helpers::generate_jwt(&creator_claims, &config.jwt_secret);

    // Create session
    let create_payload = serde_json::json!({
        "description": "Test session",
        "members": [
            {
                "user_id": creator_id,
                "function": "Commander"
            }
        ]
    });

    let create_req = test_helpers::with_auth_headers(
        test::TestRequest::post()
            .uri("/api/v1/work-sessions")
            .set_json(&create_payload),
        &config,
        &creator_token,
    )
    .to_request();

    let create_resp = test::call_service(&app, create_req).await;
    let create_body: serde_json::Value = test::read_body_json(create_resp).await;
    let session_id = create_body["data"]["id"].as_str().unwrap();

    // Try to add member with invalid function
    let add_payload = serde_json::json!({
        "user_id": member_id,
        "function": "InvalidFunction"
    });

    let add_req = test_helpers::with_auth_headers(
        test::TestRequest::post()
            .uri(&format!("/api/v1/work-sessions/{}/members", session_id))
            .set_json(&add_payload),
        &config,
        &creator_token,
    )
    .to_request();

    let add_resp = test::call_service(&app, add_req).await;
    // Invalid function is rejected during JSON deserialization
    assert_eq!(add_resp.status(), StatusCode::UNPROCESSABLE_ENTITY);
}

/// Phase 5 - Test 4: Create session with member having invalid function
#[sqlx::test]
async fn create_session_with_invalid_member_function(pool: PgPool) {
    let config = test_helpers::build_test_config();
    let app = test_helpers::create_full_test_app(pool.clone(), config.clone()).await;

    let city = db_fixtures::insert_city(&pool, "Test City").await;
    let creator_id =
        db_fixtures::insert_user(&pool, "200005", "creator@test.com", "CITY_USER", Some(city))
            .await;
    let member_id =
        db_fixtures::insert_user(&pool, "200006", "member@test.com", "CITY_USER", Some(city)).await;

    let mut creator_claims = test_helpers::build_city_user_claims(city);
    creator_claims.id = creator_id.to_string();
    let creator_token = test_helpers::generate_jwt(&creator_claims, &config.jwt_secret);

    let payload = serde_json::json!({
        "description": "Test session",
        "members": [
            {
                "user_id": creator_id,
                "function": "Commander"
            },
            {
                "user_id": member_id,
                "function": "InvalidRole"
            }
        ]
    });

    let req = test_helpers::with_auth_headers(
        test::TestRequest::post()
            .uri("/api/v1/work-sessions")
            .set_json(&payload),
        &config,
        &creator_token,
    )
    .to_request();

    let resp = test::call_service(&app, req).await;
    // Invalid function is rejected during JSON deserialization
    assert_eq!(resp.status(), StatusCode::UNPROCESSABLE_ENTITY);
}

/// Phase 5 - Test 5: Create session with very long description (tests DB limits)
#[sqlx::test]
async fn create_session_with_very_long_description(pool: PgPool) {
    let config = test_helpers::build_test_config();
    let app = test_helpers::create_full_test_app(pool.clone(), config.clone()).await;

    let city = db_fixtures::insert_city(&pool, "Test City").await;
    let user_id =
        db_fixtures::insert_user(&pool, "200007", "user@test.com", "CITY_USER", Some(city)).await;

    let mut claims = test_helpers::build_city_user_claims(city);
    claims.id = user_id.to_string();
    let token = test_helpers::generate_jwt(&claims, &config.jwt_secret);

    // Create 10000 character description to test TEXT field limits
    let long_description = "A".repeat(10000);

    let payload = serde_json::json!({
        "description": long_description,
        "members": [
            {
                "user_id": user_id,
                "function": "Commander"
            }
        ]
    });

    let req = test_helpers::with_auth_headers(
        test::TestRequest::post()
            .uri("/api/v1/work-sessions")
            .set_json(&payload),
        &config,
        &token,
    )
    .to_request();

    let resp = test::call_service(&app, req).await;
    // TEXT field in postgres can handle very long strings
    // Should succeed unless there's explicit validation
    assert_eq!(resp.status(), StatusCode::CREATED);
}
