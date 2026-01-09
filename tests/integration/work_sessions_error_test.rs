use actix_web::{test, http::StatusCode};
use chrono::{NaiveDate, NaiveTime};

use crate::common::{test_helpers, db_fixtures};

/// Phase 1 - Test 1: Create victim attendance without active session should fail
#[actix_rt::test]
async fn create_victim_attendance_without_active_session_fails() {
    let pool = test_helpers::setup_test_db().await;
    test_helpers::clean_database(&pool).await;

    let config = test_helpers::build_test_config();
    let app = test_helpers::create_full_test_app(pool.clone(), config.clone()).await;

    let city = db_fixtures::insert_city(&pool, "Test City").await;
    let victim_id = db_fixtures::insert_victim(&pool, "Victim", city).await;
    let user_id = db_fixtures::insert_user(&pool, "100001", "user@test.com", "CITY_USER", Some(city)).await;

    // Create JWT token but NO work session
    let mut user_claims = test_helpers::build_city_user_claims(city);
    user_claims.id = user_id.to_string();
    let user_token = test_helpers::generate_jwt(&user_claims, &config.jwt_secret);

    let payload = serde_json::json!({
        "victim_id": victim_id,
        "was_victim_present": true,
        "attendance_date": NaiveDate::from_ymd_opt(2025, 1, 1).unwrap(),
        "attendance_time": NaiveTime::from_hms_opt(10, 0, 0).unwrap(),
        "description": "Test attendance",
        "latitude": null,
        "longitude": null,
        "address": null,
        "offender_id": null,
        "protective_measure_id": null,
        "is_remote": false,
        "risk_level": null,
        "offender_freedom_status": null,
        "offender_has_firearm_access": null,
        "needs_legal_assistance": false,
        "needs_psychological_support": false,
        "was_instructed_about_protective_measure_procedures": false,
        "offender_violated_protective_measure": false,
    });

    let req = test_helpers::with_auth_headers(
        test::TestRequest::post()
            .uri("/api/v1/attendance-victims")
            .set_json(&payload),
        &config,
        &user_token,
    )
    .to_request();

    let resp = test::call_service(&app, req).await;
    // Returns FORBIDDEN (403) because permission check happens before work session check
    assert_eq!(resp.status(), StatusCode::FORBIDDEN);
}

/// Phase 1 - Test 2: Create offender attendance without active session should fail
#[actix_rt::test]
async fn create_offender_attendance_without_active_session_fails() {
    let pool = test_helpers::setup_test_db().await;
    test_helpers::clean_database(&pool).await;

    let config = test_helpers::build_test_config();
    let app = test_helpers::create_full_test_app(pool.clone(), config.clone()).await;

    let city = db_fixtures::insert_city(&pool, "Test City").await;
    let victim_id = db_fixtures::insert_victim(&pool, "Victim", city).await;
    let offender_id = db_fixtures::insert_offender(&pool, "Offender", city).await;
    let user_id = db_fixtures::insert_user(&pool, "100002", "user2@test.com", "CITY_USER", Some(city)).await;

    // Create JWT token but NO work session
    let mut user_claims = test_helpers::build_city_user_claims(city);
    user_claims.id = user_id.to_string();
    let user_token = test_helpers::generate_jwt(&user_claims, &config.jwt_secret);

    let payload = serde_json::json!({
        "offender_id": offender_id,
        "victim_id": victim_id,
        "protective_measure_id": null,
        "was_offender_present": true,
        "attendance_date": NaiveDate::from_ymd_opt(2025, 1, 1).unwrap(),
        "attendance_time": NaiveTime::from_hms_opt(10, 0, 0).unwrap(),
        "address": null,
        "is_remote": false,
        "assaults_children": false,
        "violence_aggravator": "AlcoholUse",
        "violence_aggravator_other": null,
        "description": "Test attendance",
    });

    let req = test_helpers::with_auth_headers(
        test::TestRequest::post()
            .uri("/api/v1/attendance-offenders")
            .set_json(&payload),
        &config,
        &user_token,
    )
    .to_request();

    let resp = test::call_service(&app, req).await;
    // Returns FORBIDDEN (403) because permission check happens before work session check
    assert_eq!(resp.status(), StatusCode::FORBIDDEN);
}

/// Phase 1 - Test 3: Verify attendance members are tracked correctly
#[actix_rt::test]
async fn attendance_tracks_all_session_members() {
    let pool = test_helpers::setup_test_db().await;
    test_helpers::clean_database(&pool).await;

    let config = test_helpers::build_test_config();
    let app = test_helpers::create_full_test_app(pool.clone(), config.clone()).await;

    let city = db_fixtures::insert_city(&pool, "Test City").await;
    let victim_id = db_fixtures::insert_victim(&pool, "Victim", city).await;
    
    // Create 3 users (use CITY_ADMIN for commander to have create_attendances permission)
    let commander_id = db_fixtures::insert_user(&pool, "100003", "commander@test.com", "CITY_ADMIN", Some(city)).await;
    let driver_id = db_fixtures::insert_user(&pool, "100004", "driver@test.com", "CITY_USER", Some(city)).await;
    let patroller_id = db_fixtures::insert_user(&pool, "100005", "patroller@test.com", "CITY_USER", Some(city)).await;

    let mut commander_claims = test_helpers::build_city_admin_claims(city);
    commander_claims.id = commander_id.to_string();
    let commander_token = test_helpers::generate_jwt(&commander_claims, &config.jwt_secret);

    // Create work session with Driver and Patroller
    // Note: Commander (creator) is added automatically, so only add other members
    let create_session_payload = serde_json::json!({
        "description": "Team session",
        "members": [
            {
                "user_id": driver_id,
                "function": "Driver"
            },
            {
                "user_id": patroller_id,
                "function": "Patroller"
            }
        ]
    });

    let create_session_req = test_helpers::with_auth_headers(
        test::TestRequest::post()
            .uri("/api/v1/work-sessions")
            .set_json(&create_session_payload),
        &config,
        &commander_token,
    )
    .to_request();

    let session_resp = test::call_service(&app, create_session_req).await;
    let status = session_resp.status();
    let session_body: serde_json::Value = test::read_body_json(session_resp).await;
    if status != StatusCode::CREATED {
        eprintln!("Session creation failed with status {}: {:?}", status, session_body);
    }
    assert_eq!(status, StatusCode::CREATED);
    let session_id = session_body["data"]["id"].as_str().unwrap();

    // Create victim attendance
    let attendance_payload = serde_json::json!({
        "victim_id": victim_id,
        "was_victim_present": true,
        "attendance_date": NaiveDate::from_ymd_opt(2025, 1, 1).unwrap(),
        "attendance_time": NaiveTime::from_hms_opt(10, 0, 0).unwrap(),
        "description": "Team attendance",
        "latitude": null,
        "longitude": null,
        "address": null,
        "offender_id": null,
        "protective_measure_id": null,
        "is_remote": false,
        "risk_level": null,
        "offender_freedom_status": null,
        "offender_has_firearm_access": null,
        "needs_legal_assistance": false,
        "needs_psychological_support": false,
        "was_instructed_about_protective_measure_procedures": false,
        "offender_violated_protective_measure": false,
    });

    let attendance_req = test_helpers::with_auth_headers(
        test::TestRequest::post()
            .uri("/api/v1/attendance-victims")
            .set_json(&attendance_payload),
        &config,
        &commander_token,
    )
    .to_request();

    let attendance_resp = test::call_service(&app, attendance_req).await;
    let att_status = attendance_resp.status();
    let attendance_body: serde_json::Value = test::read_body_json(attendance_resp).await;
    if att_status != StatusCode::CREATED {
        eprintln!("Attendance creation failed with status {}: {:?}", att_status, attendance_body);
    }
    assert_eq!(att_status, StatusCode::CREATED);
    let attendance_id = attendance_body["data"]["id"].as_str().unwrap();

    // Verify all 3 members are tracked (Commander + Driver + Patroller)
    let member_count: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM attendance_victim_members WHERE attendance_victim_id = $1"
    )
    .bind(uuid::Uuid::parse_str(attendance_id).unwrap())
    .fetch_one(&pool)
    .await
    .expect("Failed to count attendance members");

    assert_eq!(member_count, 3, "Should track all session members (Commander + Driver + Patroller)");

    // Verify session_id is tracked
    let tracked_sessions: Vec<uuid::Uuid> = sqlx::query_scalar(
        "SELECT DISTINCT work_session_id FROM attendance_victim_members WHERE attendance_victim_id = $1"
    )
    .bind(uuid::Uuid::parse_str(attendance_id).unwrap())
    .fetch_all(&pool)
    .await
    .expect("Failed to get tracked sessions");

    assert_eq!(tracked_sessions.len(), 1);
    assert_eq!(tracked_sessions[0].to_string(), session_id);
}
