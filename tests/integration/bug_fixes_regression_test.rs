use actix_web::{http::StatusCode, test};
use chrono::NaiveDate;
use sqlx::PgPool;

use crate::common::{db_fixtures, test_helpers};

// ==================== B02: ROOT can operate on work sessions without city_id ====================

#[sqlx::test]
async fn b02_root_can_view_work_session_from_any_city(pool: PgPool) {
    let config = test_helpers::build_test_config();
    let app = test_helpers::create_full_test_app(pool.clone(), config.clone()).await;

    let city = db_fixtures::insert_city(&pool, "City B02 View").await;
    let user_id = db_fixtures::insert_user(
        &pool,
        "300001",
        "user_b02_view@test.com",
        "CITY_ADMIN",
        Some(city),
    )
    .await;
    let root_id =
        db_fixtures::insert_user(&pool, "300002", "root_b02_view@test.com", "ROOT", None).await;

    // User creates session via API
    let mut user_claims = test_helpers::build_city_admin_claims(city);
    user_claims.id = user_id.to_string();
    let user_token = test_helpers::generate_jwt(&user_claims, &config.jwt_secret);

    let create_payload = serde_json::json!({
        "description": "B02 regression session",
        "members": [{ "user_id": user_id, "function": "Commander" }]
    });

    let create_req = test_helpers::with_auth_headers(
        test::TestRequest::post()
            .uri("/api/v1/work-sessions")
            .set_json(&create_payload),
        &config,
        &user_token,
    )
    .to_request();

    let create_resp = test::call_service(&app, create_req).await;
    assert_eq!(create_resp.status(), StatusCode::CREATED);
    let create_body: serde_json::Value = test::read_body_json(create_resp).await;
    let session_id = create_body["data"]["id"].as_str().unwrap();

    // ROOT views the session
    let mut root_claims = test_helpers::build_root_claims();
    root_claims.id = root_id.to_string();
    let root_token = test_helpers::generate_jwt(&root_claims, &config.jwt_secret);

    let get_req = test_helpers::with_auth_headers(
        test::TestRequest::get().uri(&format!("/api/v1/work-sessions/{}", session_id)),
        &config,
        &root_token,
    )
    .to_request();

    let get_resp = test::call_service(&app, get_req).await;
    assert_eq!(get_resp.status(), StatusCode::OK);
    let body: serde_json::Value = test::read_body_json(get_resp).await;
    assert_eq!(body["data"]["id"].as_str().unwrap(), session_id);
}

#[sqlx::test]
async fn b02_root_can_add_member_to_session(pool: PgPool) {
    let config = test_helpers::build_test_config();
    let app = test_helpers::create_full_test_app(pool.clone(), config.clone()).await;

    let city = db_fixtures::insert_city(&pool, "City B02 Add").await;
    let user_a = db_fixtures::insert_user(
        &pool,
        "300003",
        "a_b02_add@test.com",
        "CITY_USER",
        Some(city),
    )
    .await;
    let user_b = db_fixtures::insert_user(
        &pool,
        "300004",
        "b_b02_add@test.com",
        "CITY_USER",
        Some(city),
    )
    .await;
    let root_id =
        db_fixtures::insert_user(&pool, "300005", "root_b02_add@test.com", "ROOT", None).await;

    // ROOT creates session with user_a
    let mut root_claims = test_helpers::build_root_claims();
    root_claims.id = root_id.to_string();
    let root_token = test_helpers::generate_jwt(&root_claims, &config.jwt_secret);

    let create_payload = serde_json::json!({
        "description": "B02 add member session",
        "members": [
            { "user_id": root_id, "function": "Commander" },
            { "user_id": user_a, "function": "Patroller" }
        ]
    });

    let create_req = test_helpers::with_auth_headers(
        test::TestRequest::post()
            .uri("/api/v1/work-sessions")
            .set_json(&create_payload),
        &config,
        &root_token,
    )
    .to_request();

    let create_resp = test::call_service(&app, create_req).await;
    assert_eq!(create_resp.status(), StatusCode::CREATED);
    let create_body: serde_json::Value = test::read_body_json(create_resp).await;
    let session_id = create_body["data"]["id"].as_str().unwrap();

    // ROOT adds user_b
    let add_payload = serde_json::json!({
        "user_id": user_b,
        "function": "Driver"
    });

    let add_req = test_helpers::with_auth_headers(
        test::TestRequest::post()
            .uri(&format!("/api/v1/work-sessions/{}/members", session_id))
            .set_json(&add_payload),
        &config,
        &root_token,
    )
    .to_request();

    let add_resp = test::call_service(&app, add_req).await;
    assert_eq!(add_resp.status(), StatusCode::OK);

    // Verify session now has 3 members (root + user_a + user_b)
    let get_req = test_helpers::with_auth_headers(
        test::TestRequest::get().uri(&format!("/api/v1/work-sessions/{}", session_id)),
        &config,
        &root_token,
    )
    .to_request();
    let get_resp = test::call_service(&app, get_req).await;
    let get_body: serde_json::Value = test::read_body_json(get_resp).await;
    let members = get_body["data"]["members"].as_array().unwrap();
    assert_eq!(
        members.len(),
        3,
        "Session should have 3 members after adding user_b"
    );
}

#[sqlx::test]
async fn b02_root_can_update_work_session(pool: PgPool) {
    let config = test_helpers::build_test_config();
    let app = test_helpers::create_full_test_app(pool.clone(), config.clone()).await;

    let city = db_fixtures::insert_city(&pool, "City B02 Upd").await;
    let user_a = db_fixtures::insert_user(
        &pool,
        "300006",
        "a_b02_upd@test.com",
        "CITY_USER",
        Some(city),
    )
    .await;
    let root_id =
        db_fixtures::insert_user(&pool, "300007", "root_b02_upd@test.com", "ROOT", None).await;

    let mut root_claims = test_helpers::build_root_claims();
    root_claims.id = root_id.to_string();
    let root_token = test_helpers::generate_jwt(&root_claims, &config.jwt_secret);

    // ROOT creates session
    let create_payload = serde_json::json!({
        "description": "Original description",
        "members": [{ "user_id": root_id, "function": "Commander" }]
    });

    let create_req = test_helpers::with_auth_headers(
        test::TestRequest::post()
            .uri("/api/v1/work-sessions")
            .set_json(&create_payload),
        &config,
        &root_token,
    )
    .to_request();

    let create_resp = test::call_service(&app, create_req).await;
    assert_eq!(create_resp.status(), StatusCode::CREATED);
    let create_body: serde_json::Value = test::read_body_json(create_resp).await;
    let session_id = create_body["data"]["id"].as_str().unwrap();

    // ROOT updates session
    let update_payload = serde_json::json!({
        "description": "Updated by ROOT",
        "members": [
            { "user_id": root_id, "function": "Commander" },
            { "user_id": user_a, "function": "Patroller" }
        ]
    });

    let update_req = test_helpers::with_auth_headers(
        test::TestRequest::put()
            .uri(&format!("/api/v1/work-sessions/{}", session_id))
            .set_json(&update_payload),
        &config,
        &root_token,
    )
    .to_request();

    let update_resp = test::call_service(&app, update_req).await;
    assert_eq!(update_resp.status(), StatusCode::OK);
    let body: serde_json::Value = test::read_body_json(update_resp).await;
    assert_eq!(
        body["data"]["description"].as_str().unwrap(),
        "Updated by ROOT"
    );
}

// ==================== B06: Update protective measure returns updated measure ====================

#[sqlx::test]
async fn b06_update_protective_measure_returns_updated_measure(pool: PgPool) {
    let config = test_helpers::build_test_config();
    let app = test_helpers::create_full_test_app(pool.clone(), config.clone()).await;

    let city = db_fixtures::insert_city(&pool, "City B06").await;
    let victim_id = db_fixtures::insert_victim(&pool, "Victim B06", city).await;
    let offender_id = db_fixtures::insert_offender(&pool, "Offender B06", city).await;

    let admin_claims = test_helpers::build_city_admin_claims(city);
    let admin_token = test_helpers::generate_jwt(&admin_claims, &config.jwt_secret);

    // Create protective measure
    let create_payload = serde_json::json!({
        "process_number": "B06-12345.2025",
        "issued_at": NaiveDate::from_ymd_opt(2025, 1, 1).unwrap(),
        "judicial_authority": "Juiz B06",
        "court_district_id": city,
        "status": "Revoked",
        "violence_types": ["Physical"],
        "relationship_to_victim": "Spouse",
        "assaults_children": false,
        "was_drunk_during_assault": false,
        "victim_id": victim_id,
        "offender_id": offender_id,
    });

    let create_req = test_helpers::with_auth_headers(
        test::TestRequest::post()
            .uri("/api/v1/protective-measures")
            .set_json(&create_payload),
        &config,
        &admin_token,
    )
    .to_request();

    let create_resp = test::call_service(&app, create_req).await;
    assert_eq!(create_resp.status(), StatusCode::CREATED);
    let create_body: serde_json::Value = test::read_body_json(create_resp).await;
    let measure_id = create_body["data"]["id"].as_str().unwrap();

    // Update protective measure
    let update_payload = serde_json::json!({
        "process_number": "B06-99999.2025",
        "issued_at": NaiveDate::from_ymd_opt(2025, 2, 1).unwrap(),
        "judicial_authority": "Juiz B06 Updated",
        "court_district_id": city,
        "status": "Revoked",
        "violence_types": ["Physical"],
        "relationship_to_victim": "Spouse",
        "assaults_children": false,
        "was_drunk_during_assault": false,
        "victim_id": victim_id,
        "offender_id": offender_id,
    });

    let update_req = test_helpers::with_auth_headers(
        test::TestRequest::put()
            .uri(&format!("/api/v1/protective-measures/{}", measure_id))
            .set_json(&update_payload),
        &config,
        &admin_token,
    )
    .to_request();

    let update_resp = test::call_service(&app, update_req).await;
    assert_eq!(update_resp.status(), StatusCode::OK);
    let body: serde_json::Value = test::read_body_json(update_resp).await;

    assert_eq!(
        body["data"]["process_number"].as_str().unwrap(),
        "B06-99999.2025"
    );
}

// ==================== B10: City validation when adding member ====================

#[sqlx::test]
async fn b10_add_member_from_different_city_fails(pool: PgPool) {
    let config = test_helpers::build_test_config();
    let app = test_helpers::create_full_test_app(pool.clone(), config.clone()).await;

    let city_a = db_fixtures::insert_city(&pool, "City B10 A").await;
    let city_b = db_fixtures::insert_city(&pool, "City B10 B").await;
    let user_a = db_fixtures::insert_user(
        &pool,
        "300010",
        "user_b10_a@test.com",
        "CITY_ADMIN",
        Some(city_a),
    )
    .await;
    let user_b = db_fixtures::insert_user(
        &pool,
        "300011",
        "user_b10_b@test.com",
        "CITY_USER",
        Some(city_b),
    )
    .await;

    let mut claims_a = test_helpers::build_city_admin_claims(city_a);
    claims_a.id = user_a.to_string();
    let token_a = test_helpers::generate_jwt(&claims_a, &config.jwt_secret);

    // User A creates session
    let create_payload = serde_json::json!({
        "description": "B10 session",
        "members": [{ "user_id": user_a, "function": "Commander" }]
    });

    let create_req = test_helpers::with_auth_headers(
        test::TestRequest::post()
            .uri("/api/v1/work-sessions")
            .set_json(&create_payload),
        &config,
        &token_a,
    )
    .to_request();

    let create_resp = test::call_service(&app, create_req).await;
    assert_eq!(create_resp.status(), StatusCode::CREATED);
    let create_body: serde_json::Value = test::read_body_json(create_resp).await;
    let session_id = create_body["data"]["id"].as_str().unwrap();

    // Try to add user B from city B → should fail
    let add_payload = serde_json::json!({
        "user_id": user_b,
        "function": "Patroller"
    });

    let add_req = test_helpers::with_auth_headers(
        test::TestRequest::post()
            .uri(&format!("/api/v1/work-sessions/{}/members", session_id))
            .set_json(&add_payload),
        &config,
        &token_a,
    )
    .to_request();

    let add_resp = test::call_service(&app, add_req).await;
    assert_eq!(add_resp.status(), StatusCode::BAD_REQUEST);
    let body: serde_json::Value = test::read_body_json(add_resp).await;
    assert!(
        body["message"].as_str().unwrap().contains("same city"),
        "Error should mention same city requirement"
    );
}

// ==================== B11: Cannot remove Commander ====================

#[sqlx::test]
async fn b11_cannot_remove_commander_from_session(pool: PgPool) {
    let config = test_helpers::build_test_config();
    let app = test_helpers::create_full_test_app(pool.clone(), config.clone()).await;

    let city = db_fixtures::insert_city(&pool, "City B11").await;
    let commander_id = db_fixtures::insert_user(
        &pool,
        "300012",
        "commander_b11@test.com",
        "CITY_ADMIN",
        Some(city),
    )
    .await;
    let patroller_id = db_fixtures::insert_user(
        &pool,
        "300013",
        "patroller_b11@test.com",
        "CITY_USER",
        Some(city),
    )
    .await;

    let mut claims = test_helpers::build_city_admin_claims(city);
    claims.id = commander_id.to_string();
    let token = test_helpers::generate_jwt(&claims, &config.jwt_secret);

    // Create session with Commander + Patroller
    let create_payload = serde_json::json!({
        "description": "B11 session",
        "members": [
            { "user_id": commander_id, "function": "Commander" },
            { "user_id": patroller_id, "function": "Patroller" }
        ]
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
    assert_eq!(create_resp.status(), StatusCode::CREATED);
    let create_body: serde_json::Value = test::read_body_json(create_resp).await;
    let session_id = create_body["data"]["id"].as_str().unwrap();

    // Try to remove Commander → should fail
    let remove_req = test_helpers::with_auth_headers(
        test::TestRequest::delete().uri(&format!(
            "/api/v1/work-sessions/{}/members/{}",
            session_id, commander_id
        )),
        &config,
        &token,
    )
    .to_request();

    let remove_resp = test::call_service(&app, remove_req).await;
    assert_eq!(remove_resp.status(), StatusCode::BAD_REQUEST);
    let body: serde_json::Value = test::read_body_json(remove_resp).await;
    assert!(
        body["message"]
            .as_str()
            .unwrap()
            .contains("Cannot remove the Commander"),
        "Error should mention Cannot remove the Commander"
    );
}

#[sqlx::test]
async fn b11_can_remove_non_commander_from_session(pool: PgPool) {
    let config = test_helpers::build_test_config();
    let app = test_helpers::create_full_test_app(pool.clone(), config.clone()).await;

    let city = db_fixtures::insert_city(&pool, "City B11b").await;
    let commander_id = db_fixtures::insert_user(
        &pool,
        "300014",
        "cmd_b11b@test.com",
        "CITY_ADMIN",
        Some(city),
    )
    .await;
    let patroller_id = db_fixtures::insert_user(
        &pool,
        "300015",
        "pat_b11b@test.com",
        "CITY_USER",
        Some(city),
    )
    .await;

    let mut claims = test_helpers::build_city_admin_claims(city);
    claims.id = commander_id.to_string();
    let token = test_helpers::generate_jwt(&claims, &config.jwt_secret);

    // Create session with Commander + Patroller
    let create_payload = serde_json::json!({
        "description": "B11b session",
        "members": [
            { "user_id": commander_id, "function": "Commander" },
            { "user_id": patroller_id, "function": "Patroller" }
        ]
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
    assert_eq!(create_resp.status(), StatusCode::CREATED);
    let create_body: serde_json::Value = test::read_body_json(create_resp).await;
    let session_id = create_body["data"]["id"].as_str().unwrap();

    // Remove Patroller → should succeed
    let remove_req = test_helpers::with_auth_headers(
        test::TestRequest::delete().uri(&format!(
            "/api/v1/work-sessions/{}/members/{}",
            session_id, patroller_id
        )),
        &config,
        &token,
    )
    .to_request();

    let remove_resp = test::call_service(&app, remove_req).await;
    assert_eq!(remove_resp.status(), StatusCode::OK);

    // Verify session now has 1 member (only commander)
    let get_req = test_helpers::with_auth_headers(
        test::TestRequest::get().uri(&format!("/api/v1/work-sessions/{}", session_id)),
        &config,
        &token,
    )
    .to_request();
    let get_resp = test::call_service(&app, get_req).await;
    let get_body: serde_json::Value = test::read_body_json(get_resp).await;
    let members = get_body["data"]["members"].as_array().unwrap();
    assert_eq!(
        members.len(),
        1,
        "Session should have only commander after removing patroller"
    );
}
