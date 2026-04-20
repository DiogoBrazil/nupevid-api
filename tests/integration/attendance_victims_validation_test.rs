use actix_web::{http::StatusCode, test};
use chrono::{NaiveDate, NaiveTime};
use uuid::Uuid;

use crate::common::{db_fixtures, test_helpers};

#[actix_rt::test]
async fn create_attendance_with_nonexistent_protective_measure_returns_404() {
    let pool = test_helpers::setup_test_db().await;
    test_helpers::clean_database(&pool).await;

    let config = test_helpers::build_test_config();
    let app = test_helpers::create_full_test_app(pool.clone(), config.clone()).await;

    let root_claims = test_helpers::build_root_claims();
    let root_token = test_helpers::generate_jwt(&root_claims, &config.jwt_secret);

    let random_pm_id = Uuid::new_v4();

    let payload = serde_json::json!({
        "protective_measure_id": random_pm_id,
        "was_victim_present": true,
        "attendance_date": NaiveDate::from_ymd_opt(2025, 1, 1).unwrap(),
        "attendance_time": NaiveTime::from_hms_opt(10, 0, 0).unwrap(),
        "description": "Test",
        "latitude": null,
        "longitude": null,
        "address": null,
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
        &root_token,
    )
    .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), StatusCode::NOT_FOUND);
}

#[actix_rt::test]
async fn update_attendance_change_victim_requires_permission_on_both() {
    let pool = test_helpers::setup_test_db().await;
    test_helpers::clean_database(&pool).await;

    let config = test_helpers::build_test_config();
    let app = test_helpers::create_full_test_app(pool.clone(), config.clone()).await;

    let city_a = db_fixtures::insert_city(&pool, "Cidade A").await;
    let city_b = db_fixtures::insert_city(&pool, "Cidade B").await;
    let victim_a = db_fixtures::insert_victim(&pool, "Vitima A", city_a).await;
    let victim_b = db_fixtures::insert_victim(&pool, "Vitima B", city_b).await;
    let offender_a = db_fixtures::insert_offender(&pool, "Agressor A", city_a).await;
    let offender_b = db_fixtures::insert_offender(&pool, "Agressor B", city_b).await;
    let pm_a =
        db_fixtures::insert_protective_measure(&pool, victim_a, offender_a, city_a, "Valid").await;
    let pm_b =
        db_fixtures::insert_protective_measure(&pool, victim_b, offender_b, city_b, "Valid").await;

    // Create root user in database
    let root_user_id =
        db_fixtures::insert_user(&pool, "100000001", "root@test.com", "ROOT", None).await;
    let mut root_claims = test_helpers::build_root_claims();
    root_claims.id = root_user_id.to_string();
    let root_token = test_helpers::generate_jwt(&root_claims, &config.jwt_secret);

    // Create work session for root user
    test_helpers::create_work_session_for_user(&pool, root_user_id).await;

    // Create attendance for victim_a
    let payload = serde_json::json!({
        "protective_measure_id": pm_a,
        "was_victim_present": true,
        "attendance_date": NaiveDate::from_ymd_opt(2025, 1, 1).unwrap(),
        "attendance_time": NaiveTime::from_hms_opt(10, 0, 0).unwrap(),
        "description": "Atendimento A",
        "latitude": null,
        "longitude": null,
        "address": null,
        "is_remote": false,
        "risk_level": null,
        "offender_freedom_status": null,
        "offender_has_firearm_access": null,
        "needs_legal_assistance": false,
        "needs_psychological_support": false,
        "was_instructed_about_protective_measure_procedures": false,
        "offender_violated_protective_measure": false,
    });

    let create_req = test_helpers::with_auth_headers(
        test::TestRequest::post()
            .uri("/api/v1/attendance-victims")
            .set_json(&payload),
        &config,
        &root_token,
    )
    .to_request();

    let create_resp = test::call_service(&app, create_req).await;
    assert_eq!(create_resp.status(), StatusCode::CREATED);
    let body: serde_json::Value = test::read_body_json(create_resp).await;
    let attendance_id = body["data"]["id"].as_str().unwrap();

    // CITY_ADMIN for city_a tries to change PM to one in city_b (which derives victim_b)
    let admin_a_claims = test_helpers::build_city_admin_claims(city_a);
    let admin_a_token = test_helpers::generate_jwt(&admin_a_claims, &config.jwt_secret);

    let update_payload = serde_json::json!({
        "protective_measure_id": pm_b,
        "was_victim_present": false,
        "attendance_date": NaiveDate::from_ymd_opt(2025, 2, 1).unwrap(),
        "attendance_time": NaiveTime::from_hms_opt(14, 0, 0).unwrap(),
        "description": "Atendimento Updated",
        "latitude": null,
        "longitude": null,
        "address": null,
        "is_remote": false,
        "risk_level": null,
        "offender_freedom_status": null,
        "offender_has_firearm_access": null,
        "needs_legal_assistance": false,
        "needs_psychological_support": false,
        "was_instructed_about_protective_measure_procedures": false,
        "offender_violated_protective_measure": false,
    });

    let update_req = test_helpers::with_auth_headers(
        test::TestRequest::put()
            .uri(&format!("/api/v1/attendance-victims/{}", attendance_id))
            .set_json(&update_payload),
        &config,
        &admin_a_token,
    )
    .to_request();

    let update_resp = test::call_service(&app, update_req).await;
    assert_eq!(update_resp.status(), StatusCode::FORBIDDEN);
}

#[actix_rt::test]
async fn update_nonexistent_attendance_returns_404() {
    let pool = test_helpers::setup_test_db().await;
    test_helpers::clean_database(&pool).await;

    let config = test_helpers::build_test_config();
    let app = test_helpers::create_full_test_app(pool.clone(), config.clone()).await;

    let city = db_fixtures::insert_city(&pool, "Test City").await;
    let victim_id = db_fixtures::insert_victim(&pool, "Vitima", city).await;
    let offender_id = db_fixtures::insert_offender(&pool, "Agressor", city).await;
    let pm_id =
        db_fixtures::insert_protective_measure(&pool, victim_id, offender_id, city, "Valid").await;

    let root_claims = test_helpers::build_root_claims();
    let root_token = test_helpers::generate_jwt(&root_claims, &config.jwt_secret);

    let random_id = Uuid::new_v4();

    let update_payload = serde_json::json!({
        "protective_measure_id": pm_id,
        "was_victim_present": true,
        "attendance_date": NaiveDate::from_ymd_opt(2025, 1, 1).unwrap(),
        "attendance_time": NaiveTime::from_hms_opt(10, 0, 0).unwrap(),
        "description": "Updated",
        "latitude": null,
        "longitude": null,
        "address": null,
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
        test::TestRequest::put()
            .uri(&format!("/api/v1/attendance-victims/{}", random_id))
            .set_json(&update_payload),
        &config,
        &root_token,
    )
    .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), StatusCode::NOT_FOUND);
}
