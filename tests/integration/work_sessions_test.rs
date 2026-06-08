use actix_web::{http::StatusCode, test};
use sqlx::PgPool;

use crate::common::{db_fixtures, test_helpers};

#[sqlx::test]
async fn create_work_session_success(pool: PgPool) {
    let config = test_helpers::build_test_config();
    let app = test_helpers::create_full_test_app(pool.clone(), config.clone()).await;

    // Create city and user
    let city_id = db_fixtures::insert_city(&pool, "Test City").await;
    let user_id =
        db_fixtures::insert_user(&pool, "100001", "user@test.com", "CITY_USER", Some(city_id))
            .await;
    let member1_id = db_fixtures::insert_user(
        &pool,
        "100002",
        "member1@test.com",
        "CITY_USER",
        Some(city_id),
    )
    .await;
    let member2_id = db_fixtures::insert_user(
        &pool,
        "100003",
        "member2@test.com",
        "CITY_USER",
        Some(city_id),
    )
    .await;

    // Generate JWT token
    let mut claims = test_helpers::build_city_user_claims(city_id);
    claims.id = user_id.to_string();
    let token = test_helpers::generate_jwt(&claims, &config.jwt_secret);

    let payload = serde_json::json!({
        "description": "Test session",
        "members": [
            {
                "user_id": user_id,
                "function": "Commander"
            },
            {
                "user_id": member1_id,
                "function": "Driver"
            },
            {
                "user_id": member2_id,
                "function": "Patroller"
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
    let status = resp.status();
    let body: serde_json::Value = test::read_body_json(resp).await;
    assert_eq!(status, StatusCode::CREATED);
    assert_eq!(body["status"].as_u64().unwrap(), 201);
    assert!(body["data"]["id"].as_str().is_some());
    assert_eq!(body["data"]["is_active"].as_bool().unwrap(), true);
    assert_eq!(
        body["data"]["description"].as_str().unwrap(),
        "Test session"
    );

    // Verify members (Commander + Driver + Patroller = 3)
    let members = body["data"]["members"].as_array().unwrap();
    assert_eq!(members.len(), 3);
}

#[sqlx::test]
async fn create_work_session_without_commander_fails(pool: PgPool) {
    let config = test_helpers::build_test_config();
    let app = test_helpers::create_full_test_app(pool.clone(), config.clone()).await;

    let city_id = db_fixtures::insert_city(&pool, "Test City").await;
    let user_id =
        db_fixtures::insert_user(&pool, "100001", "user@test.com", "CITY_USER", Some(city_id))
            .await;
    let member1_id = db_fixtures::insert_user(
        &pool,
        "100002",
        "member1@test.com",
        "CITY_USER",
        Some(city_id),
    )
    .await;

    let mut claims = test_helpers::build_city_user_claims(city_id);
    claims.id = user_id.to_string();
    let token = test_helpers::generate_jwt(&claims, &config.jwt_secret);

    // Requesting user must be included, but no Commander should fail
    let payload = serde_json::json!({
        "description": "Session with Driver",
        "members": [
            {
                "user_id": user_id,
                "function": "Driver"
            },
            {
                "user_id": member1_id,
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
    assert_eq!(resp.status(), StatusCode::BAD_REQUEST);

    let body: serde_json::Value = test::read_body_json(resp).await;
    assert!(body["message"].as_str().unwrap().contains("Commander"));
}

#[sqlx::test]
async fn create_work_session_with_two_commanders_fails(pool: PgPool) {
    let config = test_helpers::build_test_config();
    let app = test_helpers::create_full_test_app(pool.clone(), config.clone()).await;

    let city_id = db_fixtures::insert_city(&pool, "Test City").await;
    let user_id =
        db_fixtures::insert_user(&pool, "100001", "user@test.com", "CITY_USER", Some(city_id))
            .await;
    let member1_id = db_fixtures::insert_user(
        &pool,
        "100002",
        "member1@test.com",
        "CITY_USER",
        Some(city_id),
    )
    .await;

    let mut claims = test_helpers::build_city_user_claims(city_id);
    claims.id = user_id.to_string();
    let token = test_helpers::generate_jwt(&claims, &config.jwt_secret);

    // With requester as Commander, adding one more Commander = 2 total
    let payload = serde_json::json!({
        "description": "Invalid session - two commanders",
        "members": [
            {
                "user_id": user_id,
                "function": "Commander"
            },
            {
                "user_id": member1_id,
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
    assert_eq!(resp.status(), StatusCode::BAD_REQUEST);

    let body: serde_json::Value = test::read_body_json(resp).await;
    assert!(body["message"].as_str().unwrap().contains("Commander"));
}

#[sqlx::test]
async fn get_active_session_success(pool: PgPool) {
    let config = test_helpers::build_test_config();
    let app = test_helpers::create_full_test_app(pool.clone(), config.clone()).await;

    let city_id = db_fixtures::insert_city(&pool, "Test City").await;
    let user_id =
        db_fixtures::insert_user(&pool, "100001", "user@test.com", "CITY_USER", Some(city_id))
            .await;

    // Create session directly in DB
    let session_id = test_helpers::create_work_session_for_user(&pool, user_id).await;

    let mut claims = test_helpers::build_city_user_claims(city_id);
    claims.id = user_id.to_string();
    let token = test_helpers::generate_jwt(&claims, &config.jwt_secret);

    let req = test_helpers::with_auth_headers(
        test::TestRequest::get().uri("/api/v1/work-sessions/active"),
        &config,
        &token,
    )
    .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), StatusCode::OK);

    let body: serde_json::Value = test::read_body_json(resp).await;
    assert_eq!(body["data"]["id"].as_str().unwrap(), session_id.to_string());
    assert_eq!(body["data"]["is_active"].as_bool().unwrap(), true);
}

#[sqlx::test]
async fn get_active_session_when_none_returns_404(pool: PgPool) {
    let config = test_helpers::build_test_config();
    let app = test_helpers::create_full_test_app(pool.clone(), config.clone()).await;

    let city_id = db_fixtures::insert_city(&pool, "Test City").await;
    let user_id =
        db_fixtures::insert_user(&pool, "100001", "user@test.com", "CITY_USER", Some(city_id))
            .await;

    let mut claims = test_helpers::build_city_user_claims(city_id);
    claims.id = user_id.to_string();
    let token = test_helpers::generate_jwt(&claims, &config.jwt_secret);

    let req = test_helpers::with_auth_headers(
        test::TestRequest::get().uri("/api/v1/work-sessions/active"),
        &config,
        &token,
    )
    .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), StatusCode::NOT_FOUND);
}

#[sqlx::test]
async fn end_session_success(pool: PgPool) {
    let config = test_helpers::build_test_config();
    let app = test_helpers::create_full_test_app(pool.clone(), config.clone()).await;

    let city_id = db_fixtures::insert_city(&pool, "Test City").await;
    let user_id =
        db_fixtures::insert_user(&pool, "100001", "user@test.com", "CITY_USER", Some(city_id))
            .await;

    // Create session
    let session_id = test_helpers::create_work_session_for_user(&pool, user_id).await;

    let mut claims = test_helpers::build_city_user_claims(city_id);
    claims.id = user_id.to_string();
    let token = test_helpers::generate_jwt(&claims, &config.jwt_secret);

    let req = test_helpers::with_auth_headers(
        test::TestRequest::post().uri("/api/v1/work-sessions/end"),
        &config,
        &token,
    )
    .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), StatusCode::OK);

    // Verify session is no longer active
    let is_active: bool = sqlx::query_scalar("SELECT is_active FROM work_sessions WHERE id = $1")
        .bind(session_id)
        .fetch_one(&pool)
        .await
        .expect("Failed to check session status");

    assert_eq!(is_active, false);
}

#[sqlx::test]
async fn user_cannot_have_two_active_sessions(pool: PgPool) {
    let config = test_helpers::build_test_config();
    let app = test_helpers::create_full_test_app(pool.clone(), config.clone()).await;

    let city_id = db_fixtures::insert_city(&pool, "Test City").await;
    let user_id =
        db_fixtures::insert_user(&pool, "100001", "user@test.com", "CITY_USER", Some(city_id))
            .await;
    let member_id = db_fixtures::insert_user(
        &pool,
        "100002",
        "member@test.com",
        "CITY_USER",
        Some(city_id),
    )
    .await;

    // Create first session
    test_helpers::create_work_session_for_user(&pool, user_id).await;

    let mut claims = test_helpers::build_city_user_claims(city_id);
    claims.id = user_id.to_string();
    let token = test_helpers::generate_jwt(&claims, &config.jwt_secret);

    // Try to create second session
    let payload = serde_json::json!({
        "description": "Second session",
        "members": [
            {
                "user_id": user_id,
                "function": "Commander"
            },
            {
                "user_id": member_id,
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
    assert_eq!(resp.status(), StatusCode::CONFLICT);

    let body: serde_json::Value = test::read_body_json(resp).await;
    assert!(
        body["message"]
            .as_str()
            .unwrap()
            .contains("already has an active work session")
    );
}
