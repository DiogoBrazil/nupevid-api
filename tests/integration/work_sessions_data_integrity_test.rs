use actix_web::{http::StatusCode, test};
use chrono::Utc;

use crate::common::{db_fixtures, test_helpers};

/// Phase 7 - Test 1: Verify ended_at is null when session is created
#[actix_rt::test]
async fn new_session_has_null_ended_at() {
    let pool = test_helpers::setup_test_db().await;
    test_helpers::clean_database(&pool).await;

    let config = test_helpers::build_test_config();
    let app = test_helpers::create_full_test_app(pool.clone(), config.clone()).await;

    let city = db_fixtures::insert_city(&pool, "Test City").await;
    let user_id =
        db_fixtures::insert_user(&pool, "400001", "user@test.com", "CITY_USER", Some(city)).await;

    let mut claims = test_helpers::build_city_user_claims(city);
    claims.id = user_id.to_string();
    let token = test_helpers::generate_jwt(&claims, &config.jwt_secret);

    let payload = serde_json::json!({
        "description": "Test session",
        "members": []
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
    assert_eq!(resp.status(), StatusCode::CREATED);

    let body: serde_json::Value = test::read_body_json(resp).await;
    let session_id = body["data"]["id"].as_str().unwrap();

    // Verify ended_at is null in database
    let ended_at: Option<chrono::DateTime<Utc>> =
        sqlx::query_scalar("SELECT ended_at FROM work_sessions WHERE id = $1")
            .bind(uuid::Uuid::parse_str(session_id).unwrap())
            .fetch_one(&pool)
            .await
            .expect("Failed to fetch ended_at");

    assert!(
        ended_at.is_none(),
        "ended_at should be null for new session"
    );
}

/// Phase 7 - Test 2: Verify ended_at is set when session is ended
#[actix_rt::test]
async fn ended_session_has_ended_at_timestamp() {
    let pool = test_helpers::setup_test_db().await;
    test_helpers::clean_database(&pool).await;

    let config = test_helpers::build_test_config();
    let app = test_helpers::create_full_test_app(pool.clone(), config.clone()).await;

    let city = db_fixtures::insert_city(&pool, "Test City").await;
    let user_id =
        db_fixtures::insert_user(&pool, "400002", "user@test.com", "CITY_USER", Some(city)).await;

    let mut claims = test_helpers::build_city_user_claims(city);
    claims.id = user_id.to_string();
    let token = test_helpers::generate_jwt(&claims, &config.jwt_secret);

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
        &token,
    )
    .to_request();

    let create_resp = test::call_service(&app, create_req).await;
    let create_body: serde_json::Value = test::read_body_json(create_resp).await;
    let session_id = create_body["data"]["id"].as_str().unwrap();

    let before_end = Utc::now();

    // End session
    let end_req = test_helpers::with_auth_headers(
        test::TestRequest::post().uri("/api/v1/work-sessions/end"),
        &config,
        &token,
    )
    .to_request();

    test::call_service(&app, end_req).await;

    let after_end = Utc::now();

    // Verify ended_at is set and within reasonable time range
    let ended_at: Option<chrono::DateTime<Utc>> =
        sqlx::query_scalar("SELECT ended_at FROM work_sessions WHERE id = $1")
            .bind(uuid::Uuid::parse_str(session_id).unwrap())
            .fetch_one(&pool)
            .await
            .expect("Failed to fetch ended_at");

    assert!(
        ended_at.is_some(),
        "ended_at should be set after ending session"
    );
    let ended_timestamp = ended_at.unwrap();
    assert!(
        ended_timestamp >= before_end && ended_timestamp <= after_end,
        "ended_at timestamp should be between before_end and after_end"
    );
}

/// Phase 7 - Test 3: Verify created_at and updated_at are set on creation
#[actix_rt::test]
async fn session_has_timestamps_on_creation() {
    let pool = test_helpers::setup_test_db().await;
    test_helpers::clean_database(&pool).await;

    let config = test_helpers::build_test_config();
    let app = test_helpers::create_full_test_app(pool.clone(), config.clone()).await;

    let city = db_fixtures::insert_city(&pool, "Test City").await;
    let user_id =
        db_fixtures::insert_user(&pool, "400003", "user@test.com", "CITY_USER", Some(city)).await;

    let mut claims = test_helpers::build_city_user_claims(city);
    claims.id = user_id.to_string();
    let token = test_helpers::generate_jwt(&claims, &config.jwt_secret);

    let before_create = Utc::now();

    let payload = serde_json::json!({
        "description": "Test session",
        "members": []
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
    let after_create = Utc::now();

    let body: serde_json::Value = test::read_body_json(resp).await;
    let session_id = body["data"]["id"].as_str().unwrap();

    // Verify timestamps
    let (created_at, updated_at): (chrono::DateTime<Utc>, chrono::DateTime<Utc>) =
        sqlx::query_as("SELECT created_at, updated_at FROM work_sessions WHERE id = $1")
            .bind(uuid::Uuid::parse_str(session_id).unwrap())
            .fetch_one(&pool)
            .await
            .expect("Failed to fetch timestamps");

    assert!(
        created_at >= before_create && created_at <= after_create,
        "created_at should be set at session creation time"
    );
    assert!(
        updated_at >= before_create && updated_at <= after_create,
        "updated_at should be set at session creation time"
    );
    // On creation, both timestamps should be very close or equal
    assert!(
        (updated_at - created_at).num_seconds().abs() <= 1,
        "created_at and updated_at should be nearly equal on creation"
    );
}

/// Phase 7 - Test 4: Verify started_at is set on creation
#[actix_rt::test]
async fn session_has_started_at_timestamp() {
    let pool = test_helpers::setup_test_db().await;
    test_helpers::clean_database(&pool).await;

    let config = test_helpers::build_test_config();
    let app = test_helpers::create_full_test_app(pool.clone(), config.clone()).await;

    let city = db_fixtures::insert_city(&pool, "Test City").await;
    let user_id =
        db_fixtures::insert_user(&pool, "400004", "user@test.com", "CITY_USER", Some(city)).await;

    let mut claims = test_helpers::build_city_user_claims(city);
    claims.id = user_id.to_string();
    let token = test_helpers::generate_jwt(&claims, &config.jwt_secret);

    let before_create = Utc::now();

    let payload = serde_json::json!({
        "description": "Test session",
        "members": []
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
    let after_create = Utc::now();

    let body: serde_json::Value = test::read_body_json(resp).await;
    let session_id = body["data"]["id"].as_str().unwrap();

    // Verify started_at
    let started_at: chrono::DateTime<Utc> =
        sqlx::query_scalar("SELECT started_at FROM work_sessions WHERE id = $1")
            .bind(uuid::Uuid::parse_str(session_id).unwrap())
            .fetch_one(&pool)
            .await
            .expect("Failed to fetch started_at");

    assert!(
        started_at >= before_create && started_at <= after_create,
        "started_at should be set at session creation time"
    );
}
