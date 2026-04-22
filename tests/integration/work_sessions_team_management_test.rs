use actix_web::{http::StatusCode, test};
use sqlx::PgPool;

use crate::common::{db_fixtures, test_helpers};

/// Phase 2 - Test 2: Add member to existing session
#[sqlx::test]
async fn add_member_to_session_success(pool: PgPool) {
    let config = test_helpers::build_test_config();
    let app = test_helpers::create_full_test_app(pool.clone(), config.clone()).await;

    let city = db_fixtures::insert_city(&pool, "Test City").await;
    let creator_id = db_fixtures::insert_user(
        &pool,
        "100001",
        "creator@test.com",
        "CITY_ADMIN",
        Some(city),
    )
    .await;
    let new_member_id =
        db_fixtures::insert_user(&pool, "100002", "member@test.com", "CITY_USER", Some(city)).await;

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
    assert_eq!(create_resp.status(), StatusCode::CREATED);
    let create_body: serde_json::Value = test::read_body_json(create_resp).await;
    let session_id = create_body["data"]["id"].as_str().unwrap();

    // Add new member as Driver
    let add_payload = serde_json::json!({
        "user_id": new_member_id,
        "function": "Driver"
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
    assert_eq!(add_resp.status(), StatusCode::OK);

    // Verify session now has 2 members
    let get_req = test_helpers::with_auth_headers(
        test::TestRequest::get().uri(&format!("/api/v1/work-sessions/{}", session_id)),
        &config,
        &creator_token,
    )
    .to_request();

    let get_resp = test::call_service(&app, get_req).await;
    let get_body: serde_json::Value = test::read_body_json(get_resp).await;
    let members = get_body["data"]["members"].as_array().unwrap();
    assert_eq!(members.len(), 2);
}

/// Phase 2 - Test 3: Remove member from session
#[sqlx::test]
async fn remove_member_from_session_success(pool: PgPool) {
    let config = test_helpers::build_test_config();
    let app = test_helpers::create_full_test_app(pool.clone(), config.clone()).await;

    let city = db_fixtures::insert_city(&pool, "Test City").await;
    let creator_id = db_fixtures::insert_user(
        &pool,
        "100003",
        "creator@test.com",
        "CITY_ADMIN",
        Some(city),
    )
    .await;
    let member_id =
        db_fixtures::insert_user(&pool, "100004", "member@test.com", "CITY_USER", Some(city)).await;

    let mut creator_claims = test_helpers::build_city_admin_claims(city);
    creator_claims.id = creator_id.to_string();
    let creator_token = test_helpers::generate_jwt(&creator_claims, &config.jwt_secret);

    // Create session with one extra member
    let create_payload = serde_json::json!({
        "description": "Test session",
        "members": [
            {
                "user_id": creator_id,
                "function": "Commander"
            },
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

    // Remove the Driver using user_id (not member record id)
    let remove_req = test_helpers::with_auth_headers(
        test::TestRequest::delete().uri(&format!(
            "/api/v1/work-sessions/{}/members/{}",
            session_id, member_id
        )),
        &config,
        &creator_token,
    )
    .to_request();

    let remove_resp = test::call_service(&app, remove_req).await;
    assert_eq!(remove_resp.status(), StatusCode::OK);

    // Verify only 1 member remains (the creator/Commander)
    let get_req = test_helpers::with_auth_headers(
        test::TestRequest::get().uri(&format!("/api/v1/work-sessions/{}", session_id)),
        &config,
        &creator_token,
    )
    .to_request();

    let get_resp = test::call_service(&app, get_req).await;
    let get_body: serde_json::Value = test::read_body_json(get_resp).await;
    let members = get_body["data"]["members"].as_array().unwrap();
    assert_eq!(members.len(), 1);
}

/// Phase 2 - Test 4: Update session members list
#[sqlx::test]
async fn update_session_members_success(pool: PgPool) {
    let config = test_helpers::build_test_config();
    let app = test_helpers::create_full_test_app(pool.clone(), config.clone()).await;

    let city = db_fixtures::insert_city(&pool, "Test City").await;
    let creator_id = db_fixtures::insert_user(
        &pool,
        "100005",
        "creator@test.com",
        "CITY_ADMIN",
        Some(city),
    )
    .await;
    let member1_id =
        db_fixtures::insert_user(&pool, "100006", "member1@test.com", "CITY_USER", Some(city))
            .await;
    let member2_id =
        db_fixtures::insert_user(&pool, "100007", "member2@test.com", "CITY_USER", Some(city))
            .await;

    let mut creator_claims = test_helpers::build_city_admin_claims(city);
    creator_claims.id = creator_id.to_string();
    let creator_token = test_helpers::generate_jwt(&creator_claims, &config.jwt_secret);

    // Create session with one member
    let create_payload = serde_json::json!({
        "description": "Test session",
        "members": [
            {
                "user_id": creator_id,
                "function": "Commander"
            },
            {
                "user_id": member1_id,
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

    // Update members: change member1 to Patroller and add member2 as Driver
    let update_payload = serde_json::json!({
        "members": [
            {
                "user_id": creator_id,
                "function": "Commander"
            },
            {
                "user_id": member1_id,
                "function": "Patroller"
            },
            {
                "user_id": member2_id,
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
    assert_eq!(update_resp.status(), StatusCode::OK);

    let update_body: serde_json::Value = test::read_body_json(update_resp).await;
    let members = update_body["data"]["members"].as_array().unwrap();
    assert_eq!(members.len(), 3);

    // Verify functions are correct
    let commander_count = members
        .iter()
        .filter(|m| m["function"].as_str() == Some("Commander"))
        .count();
    let driver_count = members
        .iter()
        .filter(|m| m["function"].as_str() == Some("Driver"))
        .count();
    let patroller_count = members
        .iter()
        .filter(|m| m["function"].as_str() == Some("Patroller"))
        .count();

    assert_eq!(commander_count, 1);
    assert_eq!(driver_count, 1);
    assert_eq!(patroller_count, 1);
}

/// Phase 2 - Test 5: Cannot add second Driver
#[sqlx::test]
async fn cannot_add_second_driver(pool: PgPool) {
    let config = test_helpers::build_test_config();
    let app = test_helpers::create_full_test_app(pool.clone(), config.clone()).await;

    let city = db_fixtures::insert_city(&pool, "Test City").await;
    let creator_id = db_fixtures::insert_user(
        &pool,
        "100008",
        "creator@test.com",
        "CITY_ADMIN",
        Some(city),
    )
    .await;
    let driver1_id =
        db_fixtures::insert_user(&pool, "100009", "driver1@test.com", "CITY_USER", Some(city))
            .await;
    let driver2_id =
        db_fixtures::insert_user(&pool, "100010", "driver2@test.com", "CITY_USER", Some(city))
            .await;

    let mut creator_claims = test_helpers::build_city_admin_claims(city);
    creator_claims.id = creator_id.to_string();
    let creator_token = test_helpers::generate_jwt(&creator_claims, &config.jwt_secret);

    // Create session with one Driver
    let create_payload = serde_json::json!({
        "description": "Test session",
        "members": [
            {
                "user_id": creator_id,
                "function": "Commander"
            },
            {
                "user_id": driver1_id,
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

    // Try to add second Driver
    let add_payload = serde_json::json!({
        "user_id": driver2_id,
        "function": "Driver"
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
    assert_eq!(add_resp.status(), StatusCode::BAD_REQUEST);

    let body: serde_json::Value = test::read_body_json(add_resp).await;
    assert!(body["message"].as_str().unwrap().contains("Driver"));
}

/// Phase 2 - Test 6: Cannot remove last member (would leave session empty)
#[sqlx::test]
async fn cannot_remove_last_member(pool: PgPool) {
    let config = test_helpers::build_test_config();
    let app = test_helpers::create_full_test_app(pool.clone(), config.clone()).await;

    let city = db_fixtures::insert_city(&pool, "Test City").await;
    let creator_id = db_fixtures::insert_user(
        &pool,
        "100011",
        "creator@test.com",
        "CITY_ADMIN",
        Some(city),
    )
    .await;

    let mut creator_claims = test_helpers::build_city_admin_claims(city);
    creator_claims.id = creator_id.to_string();
    let creator_token = test_helpers::generate_jwt(&creator_claims, &config.jwt_secret);

    // Create session with only creator
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

    // Verify only creator exists
    let members = create_body["data"]["members"].as_array().unwrap();
    assert_eq!(members.len(), 1);

    // Try to remove the only member (creator) using creator_id
    let remove_req = test_helpers::with_auth_headers(
        test::TestRequest::delete().uri(&format!(
            "/api/v1/work-sessions/{}/members/{}",
            session_id, creator_id
        )),
        &config,
        &creator_token,
    )
    .to_request();

    let remove_resp = test::call_service(&app, remove_req).await;
    assert_eq!(remove_resp.status(), StatusCode::BAD_REQUEST);

    let body: serde_json::Value = test::read_body_json(remove_resp).await;
    assert!(
        body["message"]
            .as_str()
            .unwrap()
            .contains("at least one member")
    );
}
