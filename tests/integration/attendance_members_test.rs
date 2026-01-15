use actix_web::{http::StatusCode, test};

use crate::common::{db_fixtures, test_helpers};

// ==================== ATTENDANCE VICTIMS MEMBERS TESTS ====================

#[actix_rt::test]
async fn get_victim_attendance_members_success() {
    let pool = test_helpers::setup_test_db().await;
    test_helpers::clean_database(&pool).await;

    let config = test_helpers::build_test_config();
    let app = test_helpers::create_full_test_app(pool.clone(), config.clone()).await;

    let city_id = db_fixtures::insert_city(&pool, "Test City").await;
    let user_id = db_fixtures::insert_user(
        &pool,
        "100020",
        "admin@test.com",
        "CITY_ADMIN",
        Some(city_id),
    )
    .await;
    let victim_id = db_fixtures::insert_victim(&pool, "Test Victim", city_id).await;

    // Create work session for user
    let _session_id = test_helpers::create_work_session_for_user(&pool, user_id).await;

    let mut claims = test_helpers::build_city_admin_claims(city_id);
    claims.id = user_id.to_string();
    let token = test_helpers::generate_jwt(&claims, &config.jwt_secret);

    // Create attendance victim
    let create_payload = serde_json::json!({
        "victim_id": victim_id,
        "was_victim_present": true,
        "attendance_date": "2025-01-15",
        "attendance_time": "14:30:00",
        "is_remote": false,
        "needs_legal_assistance": false,
        "needs_psychological_support": false,
        "was_instructed_about_protective_measure_procedures": false,
        "offender_violated_protective_measure": false
    });

    let create_req = test_helpers::with_auth_headers(
        test::TestRequest::post()
            .uri("/api/v1/attendance-victims")
            .set_json(&create_payload),
        &config,
        &token,
    )
    .to_request();

    let create_resp = test::call_service(&app, create_req).await;
    assert_eq!(create_resp.status(), StatusCode::CREATED);
    let create_body: serde_json::Value = test::read_body_json(create_resp).await;
    let attendance_id = create_body["data"]["id"].as_str().unwrap();

    // Get members
    let get_req = test_helpers::with_auth_headers(
        test::TestRequest::get().uri(&format!(
            "/api/v1/attendance-victims/{}/members",
            attendance_id
        )),
        &config,
        &token,
    )
    .to_request();

    let get_resp = test::call_service(&app, get_req).await;
    assert_eq!(get_resp.status(), StatusCode::OK);
    let get_body: serde_json::Value = test::read_body_json(get_resp).await;
    let members = get_body["data"].as_array().unwrap();

    // Should have user automatically added
    assert!(members.len() >= 1);
}

#[actix_rt::test]
async fn add_victim_attendance_member_success() {
    let pool = test_helpers::setup_test_db().await;
    test_helpers::clean_database(&pool).await;

    let config = test_helpers::build_test_config();
    let app = test_helpers::create_full_test_app(pool.clone(), config.clone()).await;

    let city_id = db_fixtures::insert_city(&pool, "Test City").await;
    let user_id = db_fixtures::insert_user(
        &pool,
        "100021",
        "admin@test.com",
        "CITY_ADMIN",
        Some(city_id),
    )
    .await;
    let member_id = db_fixtures::insert_user(
        &pool,
        "100022",
        "member@test.com",
        "CITY_USER",
        Some(city_id),
    )
    .await;
    let victim_id = db_fixtures::insert_victim(&pool, "Test Victim", city_id).await;

    // Create work session
    let _session_id = test_helpers::create_work_session_for_user(&pool, user_id).await;

    let mut claims = test_helpers::build_city_admin_claims(city_id);
    claims.id = user_id.to_string();
    let token = test_helpers::generate_jwt(&claims, &config.jwt_secret);

    // Create attendance
    let create_payload = serde_json::json!({
        "victim_id": victim_id,
        "was_victim_present": true,
        "attendance_date": "2025-01-15",
        "attendance_time": "14:30:00",
        "is_remote": false,
        "needs_legal_assistance": false,
        "needs_psychological_support": false,
        "was_instructed_about_protective_measure_procedures": false,
        "offender_violated_protective_measure": false
    });

    let create_req = test_helpers::with_auth_headers(
        test::TestRequest::post()
            .uri("/api/v1/attendance-victims")
            .set_json(&create_payload),
        &config,
        &token,
    )
    .to_request();

    let create_resp = test::call_service(&app, create_req).await;
    let create_body: serde_json::Value = test::read_body_json(create_resp).await;
    let attendance_id = create_body["data"]["id"].as_str().unwrap();

    // Add new member
    let add_payload = serde_json::json!({
        "user_id": member_id
    });

    let add_req = test_helpers::with_auth_headers(
        test::TestRequest::post()
            .uri(&format!(
                "/api/v1/attendance-victims/{}/members",
                attendance_id
            ))
            .set_json(&add_payload),
        &config,
        &token,
    )
    .to_request();

    let add_resp = test::call_service(&app, add_req).await;
    assert_eq!(add_resp.status(), StatusCode::OK);

    // Verify member was added
    let get_req = test_helpers::with_auth_headers(
        test::TestRequest::get().uri(&format!(
            "/api/v1/attendance-victims/{}/members",
            attendance_id
        )),
        &config,
        &token,
    )
    .to_request();

    let get_resp = test::call_service(&app, get_req).await;
    let get_body: serde_json::Value = test::read_body_json(get_resp).await;
    let members = get_body["data"].as_array().unwrap();

    let has_member = members
        .iter()
        .any(|m| m["user_id"].as_str().unwrap() == member_id.to_string());
    assert!(has_member);
}

#[actix_rt::test]
async fn add_victim_attendance_member_different_city_fails() {
    let pool = test_helpers::setup_test_db().await;
    test_helpers::clean_database(&pool).await;

    let config = test_helpers::build_test_config();
    let app = test_helpers::create_full_test_app(pool.clone(), config.clone()).await;

    let city1_id = db_fixtures::insert_city(&pool, "City 1").await;
    let city2_id = db_fixtures::insert_city(&pool, "City 2").await;
    let user_id = db_fixtures::insert_user(
        &pool,
        "100023",
        "admin@test.com",
        "CITY_ADMIN",
        Some(city1_id),
    )
    .await;
    let other_city_user_id = db_fixtures::insert_user(
        &pool,
        "100024",
        "other@test.com",
        "CITY_USER",
        Some(city2_id),
    )
    .await;
    let victim_id = db_fixtures::insert_victim(&pool, "Test Victim", city1_id).await;

    // Create work session
    let _session_id = test_helpers::create_work_session_for_user(&pool, user_id).await;

    let mut claims = test_helpers::build_city_admin_claims(city1_id);
    claims.id = user_id.to_string();
    let token = test_helpers::generate_jwt(&claims, &config.jwt_secret);

    // Create attendance
    let create_payload = serde_json::json!({
        "victim_id": victim_id,
        "was_victim_present": true,
        "attendance_date": "2025-01-15",
        "attendance_time": "14:30:00",
        "is_remote": false,
        "needs_legal_assistance": false,
        "needs_psychological_support": false,
        "was_instructed_about_protective_measure_procedures": false,
        "offender_violated_protective_measure": false
    });

    let create_req = test_helpers::with_auth_headers(
        test::TestRequest::post()
            .uri("/api/v1/attendance-victims")
            .set_json(&create_payload),
        &config,
        &token,
    )
    .to_request();

    let create_resp = test::call_service(&app, create_req).await;
    let create_body: serde_json::Value = test::read_body_json(create_resp).await;
    let attendance_id = create_body["data"]["id"].as_str().unwrap();

    // Try to add member from different city
    let add_payload = serde_json::json!({
        "user_id": other_city_user_id
    });

    let add_req = test_helpers::with_auth_headers(
        test::TestRequest::post()
            .uri(&format!(
                "/api/v1/attendance-victims/{}/members",
                attendance_id
            ))
            .set_json(&add_payload),
        &config,
        &token,
    )
    .to_request();

    let add_resp = test::call_service(&app, add_req).await;
    assert_eq!(add_resp.status(), StatusCode::BAD_REQUEST);
}

#[actix_rt::test]
async fn remove_victim_attendance_member_success() {
    let pool = test_helpers::setup_test_db().await;
    test_helpers::clean_database(&pool).await;

    let config = test_helpers::build_test_config();
    let app = test_helpers::create_full_test_app(pool.clone(), config.clone()).await;

    let city_id = db_fixtures::insert_city(&pool, "Test City").await;
    let user_id = db_fixtures::insert_user(
        &pool,
        "100025",
        "admin@test.com",
        "CITY_ADMIN",
        Some(city_id),
    )
    .await;
    let member_id = db_fixtures::insert_user(
        &pool,
        "100026",
        "member@test.com",
        "CITY_USER",
        Some(city_id),
    )
    .await;
    let victim_id = db_fixtures::insert_victim(&pool, "Test Victim", city_id).await;

    // Create work session
    let _session_id = test_helpers::create_work_session_for_user(&pool, user_id).await;

    let mut claims = test_helpers::build_city_admin_claims(city_id);
    claims.id = user_id.to_string();
    let token = test_helpers::generate_jwt(&claims, &config.jwt_secret);

    // Create attendance
    let create_payload = serde_json::json!({
        "victim_id": victim_id,
        "was_victim_present": true,
        "attendance_date": "2025-01-15",
        "attendance_time": "14:30:00",
        "is_remote": false,
        "needs_legal_assistance": false,
        "needs_psychological_support": false,
        "was_instructed_about_protective_measure_procedures": false,
        "offender_violated_protective_measure": false
    });

    let create_req = test_helpers::with_auth_headers(
        test::TestRequest::post()
            .uri("/api/v1/attendance-victims")
            .set_json(&create_payload),
        &config,
        &token,
    )
    .to_request();

    let create_resp = test::call_service(&app, create_req).await;
    let create_body: serde_json::Value = test::read_body_json(create_resp).await;
    let attendance_id = create_body["data"]["id"].as_str().unwrap();

    // Add member first
    let add_payload = serde_json::json!({
        "user_id": member_id
    });

    let add_req = test_helpers::with_auth_headers(
        test::TestRequest::post()
            .uri(&format!(
                "/api/v1/attendance-victims/{}/members",
                attendance_id
            ))
            .set_json(&add_payload),
        &config,
        &token,
    )
    .to_request();

    test::call_service(&app, add_req).await;

    // Remove member
    let remove_req = test_helpers::with_auth_headers(
        test::TestRequest::delete().uri(&format!(
            "/api/v1/attendance-victims/{}/members/{}",
            attendance_id, member_id
        )),
        &config,
        &token,
    )
    .to_request();

    let remove_resp = test::call_service(&app, remove_req).await;
    assert_eq!(remove_resp.status(), StatusCode::OK);

    // Verify member was removed
    let get_req = test_helpers::with_auth_headers(
        test::TestRequest::get().uri(&format!(
            "/api/v1/attendance-victims/{}/members",
            attendance_id
        )),
        &config,
        &token,
    )
    .to_request();

    let get_resp = test::call_service(&app, get_req).await;
    let get_body: serde_json::Value = test::read_body_json(get_resp).await;
    let members = get_body["data"].as_array().unwrap();

    let has_member = members
        .iter()
        .any(|m| m["user_id"].as_str().unwrap() == member_id.to_string());
    assert!(!has_member);
}

// ==================== ATTENDANCE OFFENDERS MEMBERS TESTS ====================

#[actix_rt::test]
async fn get_offender_attendance_members_success() {
    let pool = test_helpers::setup_test_db().await;
    test_helpers::clean_database(&pool).await;

    let config = test_helpers::build_test_config();
    let app = test_helpers::create_full_test_app(pool.clone(), config.clone()).await;

    let city_id = db_fixtures::insert_city(&pool, "Test City").await;
    let user_id = db_fixtures::insert_user(
        &pool,
        "100027",
        "admin@test.com",
        "CITY_ADMIN",
        Some(city_id),
    )
    .await;
    let victim_id = db_fixtures::insert_victim(&pool, "Test Victim", city_id).await;
    let offender_id = db_fixtures::insert_offender(&pool, "Test Offender", city_id).await;

    // Create work session
    let _session_id = test_helpers::create_work_session_for_user(&pool, user_id).await;

    let mut claims = test_helpers::build_city_admin_claims(city_id);
    claims.id = user_id.to_string();
    let token = test_helpers::generate_jwt(&claims, &config.jwt_secret);

    // Create attendance offender
    let create_payload = serde_json::json!({
        "offender_id": offender_id,
        "victim_id": victim_id,
        "protective_measure_id": serde_json::Value::Null,
        "was_offender_present": true,
        "attendance_date": "2025-01-15",
        "attendance_time": "14:30:00",
        "address": serde_json::Value::Null,
        "is_remote": false,
        "assaults_children": false,
        "violence_aggravator": "AlcoholUse",
        "violence_aggravator_other": serde_json::Value::Null,
        "description": "Test attendance"
    });

    let create_req = test_helpers::with_auth_headers(
        test::TestRequest::post()
            .uri("/api/v1/attendance-offenders")
            .set_json(&create_payload),
        &config,
        &token,
    )
    .to_request();

    let create_resp = test::call_service(&app, create_req).await;
    assert_eq!(create_resp.status(), StatusCode::CREATED);
    let create_body: serde_json::Value = test::read_body_json(create_resp).await;
    let attendance_id = create_body["data"]["id"].as_str().unwrap();

    // Get members
    let get_req = test_helpers::with_auth_headers(
        test::TestRequest::get().uri(&format!(
            "/api/v1/attendance-offenders/{}/members",
            attendance_id
        )),
        &config,
        &token,
    )
    .to_request();

    let get_resp = test::call_service(&app, get_req).await;
    assert_eq!(get_resp.status(), StatusCode::OK);
    let get_body: serde_json::Value = test::read_body_json(get_resp).await;
    let members = get_body["data"].as_array().unwrap();

    // Should have user automatically added
    assert!(members.len() >= 1);
}

#[actix_rt::test]
async fn add_offender_attendance_member_success() {
    let pool = test_helpers::setup_test_db().await;
    test_helpers::clean_database(&pool).await;

    let config = test_helpers::build_test_config();
    let app = test_helpers::create_full_test_app(pool.clone(), config.clone()).await;

    let city_id = db_fixtures::insert_city(&pool, "Test City").await;
    let user_id = db_fixtures::insert_user(
        &pool,
        "100028",
        "admin@test.com",
        "CITY_ADMIN",
        Some(city_id),
    )
    .await;
    let member_id = db_fixtures::insert_user(
        &pool,
        "100029",
        "member@test.com",
        "CITY_USER",
        Some(city_id),
    )
    .await;
    let victim_id = db_fixtures::insert_victim(&pool, "Test Victim", city_id).await;
    let offender_id = db_fixtures::insert_offender(&pool, "Test Offender", city_id).await;

    // Create work session
    let _session_id = test_helpers::create_work_session_for_user(&pool, user_id).await;

    let mut claims = test_helpers::build_city_admin_claims(city_id);
    claims.id = user_id.to_string();
    let token = test_helpers::generate_jwt(&claims, &config.jwt_secret);

    // Create attendance
    let create_payload = serde_json::json!({
        "offender_id": offender_id,
        "victim_id": victim_id,
        "protective_measure_id": serde_json::Value::Null,
        "was_offender_present": true,
        "attendance_date": "2025-01-15",
        "attendance_time": "14:30:00",
        "address": serde_json::Value::Null,
        "is_remote": false,
        "assaults_children": false,
        "violence_aggravator": "AlcoholUse",
        "violence_aggravator_other": serde_json::Value::Null,
        "description": "Test attendance"
    });

    let create_req = test_helpers::with_auth_headers(
        test::TestRequest::post()
            .uri("/api/v1/attendance-offenders")
            .set_json(&create_payload),
        &config,
        &token,
    )
    .to_request();

    let create_resp = test::call_service(&app, create_req).await;
    assert_eq!(create_resp.status(), StatusCode::CREATED);
    let create_body: serde_json::Value = test::read_body_json(create_resp).await;
    let attendance_id = create_body["data"]["id"].as_str().unwrap();

    // Add new member
    let add_payload = serde_json::json!({
        "user_id": member_id
    });

    let add_req = test_helpers::with_auth_headers(
        test::TestRequest::post()
            .uri(&format!(
                "/api/v1/attendance-offenders/{}/members",
                attendance_id
            ))
            .set_json(&add_payload),
        &config,
        &token,
    )
    .to_request();

    let add_resp = test::call_service(&app, add_req).await;
    assert_eq!(add_resp.status(), StatusCode::OK);

    // Verify member was added
    let get_req = test_helpers::with_auth_headers(
        test::TestRequest::get().uri(&format!(
            "/api/v1/attendance-offenders/{}/members",
            attendance_id
        )),
        &config,
        &token,
    )
    .to_request();

    let get_resp = test::call_service(&app, get_req).await;
    let get_body: serde_json::Value = test::read_body_json(get_resp).await;
    let members = get_body["data"].as_array().unwrap();

    let has_member = members
        .iter()
        .any(|m| m["user_id"].as_str().unwrap() == member_id.to_string());
    assert!(has_member);
}

#[actix_rt::test]
async fn remove_offender_attendance_member_success() {
    let pool = test_helpers::setup_test_db().await;
    test_helpers::clean_database(&pool).await;

    let config = test_helpers::build_test_config();
    let app = test_helpers::create_full_test_app(pool.clone(), config.clone()).await;

    let city_id = db_fixtures::insert_city(&pool, "Test City").await;
    let user_id = db_fixtures::insert_user(
        &pool,
        "100030",
        "admin@test.com",
        "CITY_ADMIN",
        Some(city_id),
    )
    .await;
    let member_id = db_fixtures::insert_user(
        &pool,
        "100031",
        "member@test.com",
        "CITY_USER",
        Some(city_id),
    )
    .await;
    let victim_id = db_fixtures::insert_victim(&pool, "Test Victim", city_id).await;
    let offender_id = db_fixtures::insert_offender(&pool, "Test Offender", city_id).await;

    // Create work session
    let _session_id = test_helpers::create_work_session_for_user(&pool, user_id).await;

    let mut claims = test_helpers::build_city_admin_claims(city_id);
    claims.id = user_id.to_string();
    let token = test_helpers::generate_jwt(&claims, &config.jwt_secret);

    // Create attendance
    let create_payload = serde_json::json!({
        "offender_id": offender_id,
        "victim_id": victim_id,
        "protective_measure_id": serde_json::Value::Null,
        "was_offender_present": true,
        "attendance_date": "2025-01-15",
        "attendance_time": "14:30:00",
        "address": serde_json::Value::Null,
        "is_remote": false,
        "assaults_children": false,
        "violence_aggravator": "AlcoholUse",
        "violence_aggravator_other": serde_json::Value::Null,
        "description": "Test attendance"
    });

    let create_req = test_helpers::with_auth_headers(
        test::TestRequest::post()
            .uri("/api/v1/attendance-offenders")
            .set_json(&create_payload),
        &config,
        &token,
    )
    .to_request();

    let create_resp = test::call_service(&app, create_req).await;
    assert_eq!(create_resp.status(), StatusCode::CREATED);
    let create_body: serde_json::Value = test::read_body_json(create_resp).await;
    let attendance_id = create_body["data"]["id"].as_str().unwrap();

    // Add member first
    let add_payload = serde_json::json!({
        "user_id": member_id
    });

    let add_req = test_helpers::with_auth_headers(
        test::TestRequest::post()
            .uri(&format!(
                "/api/v1/attendance-offenders/{}/members",
                attendance_id
            ))
            .set_json(&add_payload),
        &config,
        &token,
    )
    .to_request();

    test::call_service(&app, add_req).await;

    // Remove member
    let remove_req = test_helpers::with_auth_headers(
        test::TestRequest::delete().uri(&format!(
            "/api/v1/attendance-offenders/{}/members/{}",
            attendance_id, member_id
        )),
        &config,
        &token,
    )
    .to_request();

    let remove_resp = test::call_service(&app, remove_req).await;
    assert_eq!(remove_resp.status(), StatusCode::OK);

    // Verify member was removed
    let get_req = test_helpers::with_auth_headers(
        test::TestRequest::get().uri(&format!(
            "/api/v1/attendance-offenders/{}/members",
            attendance_id
        )),
        &config,
        &token,
    )
    .to_request();

    let get_resp = test::call_service(&app, get_req).await;
    let get_body: serde_json::Value = test::read_body_json(get_resp).await;
    let members = get_body["data"].as_array().unwrap();

    let has_member = members
        .iter()
        .any(|m| m["user_id"].as_str().unwrap() == member_id.to_string());
    assert!(!has_member);
}

// ==================== ALL SESSION MEMBERS AUTO-ADD TESTS ====================

#[actix_rt::test]
async fn create_attendance_adds_all_session_members_automatically() {
    let pool = test_helpers::setup_test_db().await;
    test_helpers::clean_database(&pool).await;

    let config = test_helpers::build_test_config();
    let app = test_helpers::create_full_test_app(pool.clone(), config.clone()).await;

    let city_id = db_fixtures::insert_city(&pool, "Test City").await;
    let creator_id = db_fixtures::insert_user(
        &pool,
        "100050",
        "creator@test.com",
        "CITY_ADMIN",
        Some(city_id),
    )
    .await;
    let member1_id = db_fixtures::insert_user(
        &pool,
        "100051",
        "member1@test.com",
        "CITY_USER",
        Some(city_id),
    )
    .await;
    let member2_id = db_fixtures::insert_user(
        &pool,
        "100052",
        "member2@test.com",
        "CITY_USER",
        Some(city_id),
    )
    .await;
    let victim_id = db_fixtures::insert_victim(&pool, "Test Victim", city_id).await;

    let mut claims = test_helpers::build_city_admin_claims(city_id);
    claims.id = creator_id.to_string();
    let token = test_helpers::generate_jwt(&claims, &config.jwt_secret);

    // Create work session with 3 members total (creator + 2 additional)
    let create_session_payload = serde_json::json!({
        "description": "Session with 3 members",
        "members": [
            {
                "user_id": creator_id,
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

    let create_session_req = test_helpers::with_auth_headers(
        test::TestRequest::post()
            .uri("/api/v1/work-sessions")
            .set_json(&create_session_payload),
        &config,
        &token,
    )
    .to_request();

    let create_session_resp = test::call_service(&app, create_session_req).await;
    assert_eq!(create_session_resp.status(), StatusCode::CREATED);
    let session_body: serde_json::Value = test::read_body_json(create_session_resp).await;
    let work_session_id = session_body["data"]["id"].as_str().unwrap();

    // Verify session has 3 members
    let session_members = session_body["data"]["members"].as_array().unwrap();
    assert_eq!(
        session_members.len(),
        3,
        "Work session should have 3 members (creator + driver + patroller)"
    );

    // Create attendance victim
    let create_attendance_payload = serde_json::json!({
        "victim_id": victim_id,
        "was_victim_present": true,
        "attendance_date": "2025-01-15",
        "attendance_time": "14:30:00",
        "is_remote": false,
        "needs_legal_assistance": false,
        "needs_psychological_support": false,
        "was_instructed_about_protective_measure_procedures": false,
        "offender_violated_protective_measure": false
    });

    let create_attendance_req = test_helpers::with_auth_headers(
        test::TestRequest::post()
            .uri("/api/v1/attendance-victims")
            .set_json(&create_attendance_payload),
        &config,
        &token,
    )
    .to_request();

    let create_attendance_resp = test::call_service(&app, create_attendance_req).await;
    assert_eq!(create_attendance_resp.status(), StatusCode::CREATED);
    let attendance_body: serde_json::Value = test::read_body_json(create_attendance_resp).await;
    let attendance_id = attendance_body["data"]["id"].as_str().unwrap();

    // Get attendance members
    let get_members_req = test_helpers::with_auth_headers(
        test::TestRequest::get().uri(&format!(
            "/api/v1/attendance-victims/{}/members",
            attendance_id
        )),
        &config,
        &token,
    )
    .to_request();

    let get_members_resp = test::call_service(&app, get_members_req).await;
    assert_eq!(get_members_resp.status(), StatusCode::OK);
    let members_body: serde_json::Value = test::read_body_json(get_members_resp).await;
    let members = members_body["data"].as_array().unwrap();

    // CRITICAL ASSERTION: All 3 work session members should be auto-added
    assert_eq!(
        members.len(),
        3,
        "All 3 work session members should be automatically added to attendance. Found: {}",
        members.len()
    );

    // Verify each specific member is present
    let creator_present = members
        .iter()
        .any(|m| m["user_id"].as_str().unwrap() == creator_id.to_string());
    let member1_present = members
        .iter()
        .any(|m| m["user_id"].as_str().unwrap() == member1_id.to_string());
    let member2_present = members
        .iter()
        .any(|m| m["user_id"].as_str().unwrap() == member2_id.to_string());

    assert!(creator_present, "Creator should be in attendance members");
    assert!(
        member1_present,
        "Member 1 (Driver) should be in attendance members"
    );
    assert!(
        member2_present,
        "Member 2 (Patroller) should be in attendance members"
    );

    // Verify work_session_id is set for all members
    for member in members {
        let ws_id = member["work_session_id"].as_str();
        assert!(
            ws_id.is_some(),
            "work_session_id should be set for all members"
        );
        assert_eq!(
            ws_id.unwrap(),
            work_session_id,
            "work_session_id should match the active work session"
        );
    }
}
