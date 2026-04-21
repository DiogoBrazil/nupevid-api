use actix_web::{http::StatusCode, test};
use sqlx::PgPool;

use crate::common::{db_fixtures, test_helpers};

/// Phase 4 - Test 1: Cannot add members to inactive session
#[sqlx::test]
async fn cannot_add_members_to_inactive_session(pool: PgPool) {
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

    // Create and end session
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

    // End session
    let end_req = test_helpers::with_auth_headers(
        test::TestRequest::post().uri("/api/v1/work-sessions/end"),
        &config,
        &creator_token,
    )
    .to_request();

    test::call_service(&app, end_req).await;

    // Try to add member to ended session
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
    assert_eq!(add_resp.status(), StatusCode::BAD_REQUEST);

    let body: serde_json::Value = test::read_body_json(add_resp).await;
    assert!(body["message"].as_str().unwrap().contains("inactive"));
}

/// Phase 4 - Test 2: Cannot remove members from inactive session
#[sqlx::test]
async fn cannot_remove_members_from_inactive_session(pool: PgPool) {
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

    // Create session with member
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

    // End session
    let end_req = test_helpers::with_auth_headers(
        test::TestRequest::post().uri("/api/v1/work-sessions/end"),
        &config,
        &creator_token,
    )
    .to_request();

    test::call_service(&app, end_req).await;

    // Try to remove member from ended session
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
    assert_eq!(remove_resp.status(), StatusCode::BAD_REQUEST);

    let body: serde_json::Value = test::read_body_json(remove_resp).await;
    assert!(body["message"].as_str().unwrap().contains("inactive"));
}

/// Phase 4 - Test 3: After ending session, user can create new session
#[sqlx::test]
async fn can_create_new_session_after_ending_previous(pool: PgPool) {
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

    let mut creator_claims = test_helpers::build_city_admin_claims(city);
    creator_claims.id = creator_id.to_string();
    let creator_token = test_helpers::generate_jwt(&creator_claims, &config.jwt_secret);

    // Create first session
    let create1_payload = serde_json::json!({
        "description": "First session",
        "members": [
            {
                "user_id": creator_id,
                "function": "Commander"
            }
        ]
    });

    let create1_req = test_helpers::with_auth_headers(
        test::TestRequest::post()
            .uri("/api/v1/work-sessions")
            .set_json(&create1_payload),
        &config,
        &creator_token,
    )
    .to_request();

    let create1_resp = test::call_service(&app, create1_req).await;
    assert_eq!(create1_resp.status(), StatusCode::CREATED);

    // End first session
    let end_req = test_helpers::with_auth_headers(
        test::TestRequest::post().uri("/api/v1/work-sessions/end"),
        &config,
        &creator_token,
    )
    .to_request();

    let end_resp = test::call_service(&app, end_req).await;
    assert_eq!(end_resp.status(), StatusCode::OK);

    // Create second session
    let create2_payload = serde_json::json!({
        "description": "Second session",
        "members": [
            {
                "user_id": creator_id,
                "function": "Commander"
            }
        ]
    });

    let create2_req = test_helpers::with_auth_headers(
        test::TestRequest::post()
            .uri("/api/v1/work-sessions")
            .set_json(&create2_payload),
        &config,
        &creator_token,
    )
    .to_request();

    let create2_resp = test::call_service(&app, create2_req).await;
    assert_eq!(create2_resp.status(), StatusCode::CREATED);

    let create2_body: serde_json::Value = test::read_body_json(create2_resp).await;
    assert_eq!(create2_body["data"]["is_active"].as_bool().unwrap(), true);
    assert_eq!(
        create2_body["data"]["description"].as_str().unwrap(),
        "Second session"
    );
}

/// Phase 4 - Test 4: Verify ended session is_active=false
#[sqlx::test]
async fn ended_session_has_is_active_false(pool: PgPool) {
    let config = test_helpers::build_test_config();
    let app = test_helpers::create_full_test_app(pool.clone(), config.clone()).await;

    let city = db_fixtures::insert_city(&pool, "Test City").await;
    let creator_id = db_fixtures::insert_user(
        &pool,
        "100006",
        "creator@test.com",
        "CITY_ADMIN",
        Some(city),
    )
    .await;

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

    // Verify initially active
    assert_eq!(create_body["data"]["is_active"].as_bool().unwrap(), true);

    // End session
    let end_req = test_helpers::with_auth_headers(
        test::TestRequest::post().uri("/api/v1/work-sessions/end"),
        &config,
        &creator_token,
    )
    .to_request();

    test::call_service(&app, end_req).await;

    // Verify in database
    let is_active: bool = sqlx::query_scalar("SELECT is_active FROM work_sessions WHERE id = $1")
        .bind(uuid::Uuid::parse_str(session_id).unwrap())
        .fetch_one(&pool)
        .await
        .expect("Failed to check session status");

    assert_eq!(is_active, false);
}

/// Phase 4 - Test 5: Can add multiple Patrollers (no limit)
#[sqlx::test]
async fn can_add_multiple_patrollers(pool: PgPool) {
    let config = test_helpers::build_test_config();
    let app = test_helpers::create_full_test_app(pool.clone(), config.clone()).await;

    let city = db_fixtures::insert_city(&pool, "Test City").await;
    let creator_id = db_fixtures::insert_user(
        &pool,
        "100007",
        "creator@test.com",
        "CITY_ADMIN",
        Some(city),
    )
    .await;
    let patroller1_id = db_fixtures::insert_user(
        &pool,
        "100008",
        "patroller1@test.com",
        "CITY_USER",
        Some(city),
    )
    .await;
    let patroller2_id = db_fixtures::insert_user(
        &pool,
        "100009",
        "patroller2@test.com",
        "CITY_USER",
        Some(city),
    )
    .await;
    let patroller3_id = db_fixtures::insert_user(
        &pool,
        "100010",
        "patroller3@test.com",
        "CITY_USER",
        Some(city),
    )
    .await;

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

    // Add three Patrollers
    for patroller_id in [patroller1_id, patroller2_id, patroller3_id] {
        let add_payload = serde_json::json!({
            "user_id": patroller_id,
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
        assert_eq!(add_resp.status(), StatusCode::OK);
    }

    // Verify session has 4 members (Commander + 3 Patrollers)
    let get_req = test_helpers::with_auth_headers(
        test::TestRequest::get().uri(&format!("/api/v1/work-sessions/{}", session_id)),
        &config,
        &creator_token,
    )
    .to_request();

    let get_resp = test::call_service(&app, get_req).await;
    let get_body: serde_json::Value = test::read_body_json(get_resp).await;
    let members = get_body["data"]["members"].as_array().unwrap();
    assert_eq!(members.len(), 4);

    let patroller_count = members
        .iter()
        .filter(|m| m["function"].as_str() == Some("Patroller"))
        .count();
    assert_eq!(patroller_count, 3);
}

/// Phase 4 - Test 6: Get active session returns 404 after ending
#[sqlx::test]
async fn get_active_session_returns_404_after_ending(pool: PgPool) {
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

    test::call_service(&app, create_req).await;

    // Verify active session exists
    let get_before_req = test_helpers::with_auth_headers(
        test::TestRequest::get().uri("/api/v1/work-sessions/active"),
        &config,
        &creator_token,
    )
    .to_request();

    let get_before_resp = test::call_service(&app, get_before_req).await;
    assert_eq!(get_before_resp.status(), StatusCode::OK);

    // End session
    let end_req = test_helpers::with_auth_headers(
        test::TestRequest::post().uri("/api/v1/work-sessions/end"),
        &config,
        &creator_token,
    )
    .to_request();

    test::call_service(&app, end_req).await;

    // Verify no active session
    let get_after_req = test_helpers::with_auth_headers(
        test::TestRequest::get().uri("/api/v1/work-sessions/active"),
        &config,
        &creator_token,
    )
    .to_request();

    let get_after_resp = test::call_service(&app, get_after_req).await;
    assert_eq!(get_after_resp.status(), StatusCode::NOT_FOUND);
}

/// Phase 4 - cannot_reopen_ended_session_via_update_payload
/// Ensures that once a session is ended, a PUT /work-sessions/{id} cannot be
/// used to revive/modify it. Backend rejects with 400 BadRequest and a message
/// containing "inactive".
#[sqlx::test]
async fn cannot_reopen_ended_session_via_update_payload(pool: PgPool) {
    let config = test_helpers::build_test_config();
    let app = test_helpers::create_full_test_app(pool.clone(), config.clone()).await;

    let city = db_fixtures::insert_city(&pool, "Test City").await;
    let creator_id = db_fixtures::insert_user(
        &pool,
        "100012",
        "creator-reopen@test.com",
        "CITY_ADMIN",
        Some(city),
    )
    .await;

    let mut creator_claims = test_helpers::build_city_admin_claims(city);
    creator_claims.id = creator_id.to_string();
    let creator_token = test_helpers::generate_jwt(&creator_claims, &config.jwt_secret);

    // Create session
    let create_payload = serde_json::json!({
        "description": "Initial session",
        "members": [
            { "user_id": creator_id, "function": "Commander" }
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
    assert!(
        create_resp.status().is_success(),
        "create should succeed, got {}",
        create_resp.status()
    );
    let create_body: serde_json::Value = test::read_body_json(create_resp).await;
    let session_id = create_body["data"]["id"].as_str().unwrap().to_string();

    // End the session
    let end_req = test_helpers::with_auth_headers(
        test::TestRequest::post().uri("/api/v1/work-sessions/end"),
        &config,
        &creator_token,
    )
    .to_request();
    let end_resp = test::call_service(&app, end_req).await;
    assert!(
        end_resp.status().is_success(),
        "end should succeed, got {}",
        end_resp.status()
    );

    // Attempt to "reopen"/modify via PUT with a fresh payload
    let update_payload = serde_json::json!({
        "description": "Reopened session",
        "members": [
            { "user_id": creator_id, "function": "Commander" }
        ]
    });

    let update_req = test_helpers::with_auth_headers(
        test::TestRequest::put()
            .uri(&format!("/api/v1/work-sessions/{}", session_id))
            .set_json(&update_payload),
        &config,
        &creator_token,
    )
    .to_request();

    let update_resp = test::call_service(&app, update_req).await;
    assert_eq!(update_resp.status(), StatusCode::BAD_REQUEST);

    let body: serde_json::Value = test::read_body_json(update_resp).await;
    assert!(
        body["message"]
            .as_str()
            .unwrap_or_default()
            .to_lowercase()
            .contains("inactive"),
        "expected message to mention 'inactive', got: {:?}",
        body["message"]
    );

    // Verify through /active that there still is no active session for this user
    let get_active_req = test_helpers::with_auth_headers(
        test::TestRequest::get().uri("/api/v1/work-sessions/active"),
        &config,
        &creator_token,
    )
    .to_request();
    let get_active_resp = test::call_service(&app, get_active_req).await;
    assert_eq!(get_active_resp.status(), StatusCode::NOT_FOUND);
}
