use actix_web::{test, http::StatusCode};
use uuid::Uuid;

use crate::common::{test_helpers, db_fixtures};

/// Phase 6 - Test 1: Create session with non-existent member
#[actix_rt::test]
async fn create_session_with_nonexistent_member() {
    let pool = test_helpers::setup_test_db().await;
    test_helpers::clean_database(&pool).await;

    let config = test_helpers::build_test_config();
    let app = test_helpers::create_full_test_app(pool.clone(), config.clone()).await;

    let city = db_fixtures::insert_city(&pool, "Test City").await;
    let creator_id = db_fixtures::insert_user(&pool, "300001", "creator@test.com", "CITY_USER", Some(city)).await;

    let mut claims = test_helpers::build_city_user_claims(city);
    claims.id = creator_id.to_string();
    let token = test_helpers::generate_jwt(&claims, &config.jwt_secret);

    // Generate a valid UUID but for a user that doesn't exist
    let nonexistent_user_id = Uuid::new_v4();

    let payload = serde_json::json!({
        "description": "Session with ghost member",
        "members": [
            {
                "user_id": nonexistent_user_id,
                "function": "Driver"
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
    // Should fail - user doesn't exist
    let status = resp.status();
    assert!(status == StatusCode::BAD_REQUEST || status == StatusCode::NOT_FOUND);
}

/// Phase 6 - Test 2: Add non-existent member to existing session
#[actix_rt::test]
async fn add_nonexistent_member_to_session() {
    let pool = test_helpers::setup_test_db().await;
    test_helpers::clean_database(&pool).await;

    let config = test_helpers::build_test_config();
    let app = test_helpers::create_full_test_app(pool.clone(), config.clone()).await;

    let city = db_fixtures::insert_city(&pool, "Test City").await;
    let creator_id = db_fixtures::insert_user(&pool, "300002", "creator@test.com", "CITY_ADMIN", Some(city)).await;

    let mut creator_claims = test_helpers::build_city_admin_claims(city);
    creator_claims.id = creator_id.to_string();
    let creator_token = test_helpers::generate_jwt(&creator_claims, &config.jwt_secret);

    // Create session
    let create_payload = serde_json::json!({
        "description": "Test session",
        "members": []
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

    // Try to add non-existent member
    let nonexistent_user_id = Uuid::new_v4();

    let add_payload = serde_json::json!({
        "user_id": nonexistent_user_id,
        "function": "Patroller"
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
    let status = add_resp.status();
    assert!(status == StatusCode::BAD_REQUEST || status == StatusCode::NOT_FOUND);
}

/// Phase 6 - Test 3: Update members with non-existent user
#[actix_rt::test]
async fn update_members_with_nonexistent_user() {
    let pool = test_helpers::setup_test_db().await;
    test_helpers::clean_database(&pool).await;

    let config = test_helpers::build_test_config();
    let app = test_helpers::create_full_test_app(pool.clone(), config.clone()).await;

    let city = db_fixtures::insert_city(&pool, "Test City").await;
    let creator_id = db_fixtures::insert_user(&pool, "300003", "creator@test.com", "CITY_ADMIN", Some(city)).await;

    let mut creator_claims = test_helpers::build_city_admin_claims(city);
    creator_claims.id = creator_id.to_string();
    let creator_token = test_helpers::generate_jwt(&creator_claims, &config.jwt_secret);

    // Create session
    let create_payload = serde_json::json!({
        "description": "Test session",
        "members": []
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

    // Try to update with non-existent member
    let nonexistent_user_id = Uuid::new_v4();

    let update_payload = serde_json::json!({
        "members": [
            {
                "user_id": creator_id,
                "function": "Commander"
            },
            {
                "user_id": nonexistent_user_id,
                "function": "Driver"
            }
        ]
    });

    let update_req = test_helpers::with_auth_headers(
        test::TestRequest::put()
            .uri(&format!("/api/v1/work-sessions/{}/members", session_id))
            .set_json(&update_payload),
        &config,
        &creator_token,
    )
    .to_request();

    let update_resp = test::call_service(&app, update_req).await;
    let status = update_resp.status();
    assert!(status == StatusCode::BAD_REQUEST || status == StatusCode::NOT_FOUND);
}

/// Phase 6 - Test 4: Remove non-existent member from session
#[actix_rt::test]
async fn remove_nonexistent_member_from_session() {
    let pool = test_helpers::setup_test_db().await;
    test_helpers::clean_database(&pool).await;

    let config = test_helpers::build_test_config();
    let app = test_helpers::create_full_test_app(pool.clone(), config.clone()).await;

    let city = db_fixtures::insert_city(&pool, "Test City").await;
    let creator_id = db_fixtures::insert_user(&pool, "300004", "creator@test.com", "CITY_ADMIN", Some(city)).await;

    let mut creator_claims = test_helpers::build_city_admin_claims(city);
    creator_claims.id = creator_id.to_string();
    let creator_token = test_helpers::generate_jwt(&creator_claims, &config.jwt_secret);

    // Create session
    let create_payload = serde_json::json!({
        "description": "Test session",
        "members": []
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

    // Try to remove non-existent member
    let nonexistent_user_id = Uuid::new_v4();

    let remove_req = test_helpers::with_auth_headers(
        test::TestRequest::delete()
            .uri(&format!("/api/v1/work-sessions/{}/members/{}", session_id, nonexistent_user_id)),
        &config,
        &creator_token,
    )
    .to_request();

    let remove_resp = test::call_service(&app, remove_req).await;
    let status = remove_resp.status();
    // Should fail - member doesn't exist
    assert!(status == StatusCode::BAD_REQUEST || status == StatusCode::NOT_FOUND);
}
