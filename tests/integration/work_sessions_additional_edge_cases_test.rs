use actix_web::{test, http::StatusCode};
use uuid::Uuid;

use crate::common::{test_helpers, db_fixtures};

/// Phase 9 - Test 1: Try to end session twice
#[actix_rt::test]
async fn cannot_end_session_twice() {
    let pool = test_helpers::setup_test_db().await;
    test_helpers::clean_database(&pool).await;

    let config = test_helpers::build_test_config();
    let app = test_helpers::create_full_test_app(pool.clone(), config.clone()).await;

    let city = db_fixtures::insert_city(&pool, "Test City").await;
    let user_id = db_fixtures::insert_user(&pool, "600001", "user@test.com", "CITY_USER", Some(city)).await;

    let mut claims = test_helpers::build_city_user_claims(city);
    claims.id = user_id.to_string();
    let token = test_helpers::generate_jwt(&claims, &config.jwt_secret);

    // Create session
    test_helpers::create_work_session_for_user(&pool, user_id).await;

    // End session first time
    let end_req1 = test_helpers::with_auth_headers(
        test::TestRequest::post()
            .uri("/api/v1/work-sessions/end"),
        &config,
        &token,
    )
    .to_request();

    let end_resp1 = test::call_service(&app, end_req1).await;
    assert_eq!(end_resp1.status(), StatusCode::OK);

    // Try to end again
    let end_req2 = test_helpers::with_auth_headers(
        test::TestRequest::post()
            .uri("/api/v1/work-sessions/end"),
        &config,
        &token,
    )
    .to_request();

    let end_resp2 = test::call_service(&app, end_req2).await;
    // Should fail - no active session
    assert_eq!(end_resp2.status(), StatusCode::NOT_FOUND);
}

/// Phase 9 - Test 2: GET session with invalid UUID format
#[actix_rt::test]
async fn get_session_with_invalid_uuid() {
    let pool = test_helpers::setup_test_db().await;
    test_helpers::clean_database(&pool).await;

    let config = test_helpers::build_test_config();
    let app = test_helpers::create_full_test_app(pool.clone(), config.clone()).await;

    let city = db_fixtures::insert_city(&pool, "Test City").await;
    let user_id = db_fixtures::insert_user(&pool, "600002", "user@test.com", "CITY_ADMIN", Some(city)).await;

    let mut claims = test_helpers::build_city_admin_claims(city);
    claims.id = user_id.to_string();
    let token = test_helpers::generate_jwt(&claims, &config.jwt_secret);

    // Try with invalid UUID
    let req = test_helpers::with_auth_headers(
        test::TestRequest::get()
            .uri("/api/v1/work-sessions/not-a-uuid"),
        &config,
        &token,
    )
    .to_request();

    let resp = test::call_service(&app, req).await;
    // Actix-web route matching fails so returns NOT_FOUND
    assert_eq!(resp.status(), StatusCode::NOT_FOUND);
}

/// Phase 9 - Test 3: Update members with empty array
#[actix_rt::test]
async fn update_members_with_empty_array_fails() {
    let pool = test_helpers::setup_test_db().await;
    test_helpers::clean_database(&pool).await;

    let config = test_helpers::build_test_config();
    let app = test_helpers::create_full_test_app(pool.clone(), config.clone()).await;

    let city = db_fixtures::insert_city(&pool, "Test City").await;
    let creator_id = db_fixtures::insert_user(&pool, "600003", "creator@test.com", "CITY_ADMIN", Some(city)).await;

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

    // Try to update with empty members array
    let update_payload = serde_json::json!({
        "members": []
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
    // Should fail - need at least one member
    assert_eq!(update_resp.status(), StatusCode::BAD_REQUEST);
}

/// Phase 9 - Test 4: Try to remove Commander from session
#[actix_rt::test]
async fn cannot_remove_commander_from_session() {
    let pool = test_helpers::setup_test_db().await;
    test_helpers::clean_database(&pool).await;

    let config = test_helpers::build_test_config();
    let app = test_helpers::create_full_test_app(pool.clone(), config.clone()).await;

    let city = db_fixtures::insert_city(&pool, "Test City").await;
    let creator_id = db_fixtures::insert_user(&pool, "600004", "creator@test.com", "CITY_ADMIN", Some(city)).await;
    let member_id = db_fixtures::insert_user(&pool, "600005", "member@test.com", "CITY_USER", Some(city)).await;

    let mut creator_claims = test_helpers::build_city_admin_claims(city);
    creator_claims.id = creator_id.to_string();
    let creator_token = test_helpers::generate_jwt(&creator_claims, &config.jwt_secret);

    // Create session with extra member
    let create_payload = serde_json::json!({
        "description": "Test session",
        "members": [
            {
                "user_id": member_id,
                "function": "Driver"
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

    // Try to update members removing the Commander (creator)
    let update_payload = serde_json::json!({
        "members": [
            {
                "user_id": member_id,
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
    // Should fail - need exactly one Commander
    assert_eq!(update_resp.status(), StatusCode::BAD_REQUEST);
}

/// Phase 9 - Test 5: GET non-existent session returns 404
#[actix_rt::test]
async fn get_nonexistent_session_returns_404() {
    let pool = test_helpers::setup_test_db().await;
    test_helpers::clean_database(&pool).await;

    let config = test_helpers::build_test_config();
    let app = test_helpers::create_full_test_app(pool.clone(), config.clone()).await;

    let city = db_fixtures::insert_city(&pool, "Test City").await;
    let user_id = db_fixtures::insert_user(&pool, "600006", "user@test.com", "CITY_ADMIN", Some(city)).await;

    let mut claims = test_helpers::build_city_admin_claims(city);
    claims.id = user_id.to_string();
    let token = test_helpers::generate_jwt(&claims, &config.jwt_secret);

    // Valid UUID but non-existent session
    let nonexistent_id = Uuid::new_v4();

    let req = test_helpers::with_auth_headers(
        test::TestRequest::get()
            .uri(&format!("/api/v1/work-sessions/{}", nonexistent_id)),
        &config,
        &token,
    )
    .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), StatusCode::NOT_FOUND);
}

/// Phase 9 - Test 6: Add member with malformed UUID
#[actix_rt::test]
async fn add_member_with_malformed_uuid() {
    let pool = test_helpers::setup_test_db().await;
    test_helpers::clean_database(&pool).await;

    let config = test_helpers::build_test_config();
    let app = test_helpers::create_full_test_app(pool.clone(), config.clone()).await;

    let city = db_fixtures::insert_city(&pool, "Test City").await;
    let creator_id = db_fixtures::insert_user(&pool, "600007", "creator@test.com", "CITY_ADMIN", Some(city)).await;

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

    // Try to add member with malformed UUID
    let add_payload = serde_json::json!({
        "user_id": "not-a-valid-uuid",
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
    // Should fail during deserialization
    let status = add_resp.status();
    assert!(status == StatusCode::BAD_REQUEST || status == StatusCode::UNPROCESSABLE_ENTITY);
}
