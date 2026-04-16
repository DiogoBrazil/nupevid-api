use actix_web::{http::StatusCode, test};
use chrono::{NaiveDate, NaiveTime};
use uuid::Uuid;

use crate::common::{db_fixtures, test_helpers};

fn build_victim_attendance_payload(
    victim_id: Uuid,
    protective_measure_id: Option<Uuid>,
) -> serde_json::Value {
    serde_json::json!({
        "victim_id": victim_id,
        "was_victim_present": true,
        "attendance_date": NaiveDate::from_ymd_opt(2025, 1, 1).unwrap(),
        "attendance_time": NaiveTime::from_hms_opt(10, 0, 0).unwrap(),
        "is_remote": false,
        "protective_measure_id": protective_measure_id,
        "needs_legal_assistance": false,
        "needs_psychological_support": false,
        "was_instructed_about_protective_measure_procedures": false,
        "offender_violated_protective_measure": false,
    })
}

fn build_offender_attendance_payload(
    offender_id: Uuid,
    victim_id: Uuid,
    protective_measure_id: Option<Uuid>,
) -> serde_json::Value {
    serde_json::json!({
        "offender_id": offender_id,
        "victim_id": victim_id,
        "protective_measure_id": protective_measure_id,
        "was_offender_present": true,
        "attendance_date": NaiveDate::from_ymd_opt(2025, 1, 1).unwrap(),
        "attendance_time": NaiveTime::from_hms_opt(10, 0, 0).unwrap(),
        "is_remote": false,
        "assaults_children": false,
        "violence_aggravator": "AlcoholUse",
        "description": "Atendimento ao agressor",
    })
}

async fn setup_test_data() -> (
    sqlx::PgPool,
    nupevid_api::config::config_env::Config,
    impl actix_web::dev::Service<
        actix_http::Request,
        Response = actix_web::dev::ServiceResponse<actix_web::body::BoxBody>,
        Error = actix_web::Error,
    >,
    String,
    Uuid,
    Uuid,
    Uuid,
    Uuid,
    Uuid,
) {
    let pool = test_helpers::setup_test_db().await;
    test_helpers::clean_database(&pool).await;

    let config = test_helpers::build_test_config();
    let app = test_helpers::create_full_test_app(pool.clone(), config.clone()).await;

    let city_id = db_fixtures::insert_city(&pool, "Test City").await;
    let victim_id = db_fixtures::insert_victim(&pool, "Test Victim", city_id).await;
    let offender_id = db_fixtures::insert_offender(&pool, "Test Offender", city_id).await;

    let measure_a =
        db_fixtures::insert_protective_measure(&pool, victim_id, offender_id, city_id, "Valid")
            .await;
    let measure_b =
        db_fixtures::insert_protective_measure(&pool, victim_id, offender_id, city_id, "Revoked")
            .await;

    let root_user_id =
        db_fixtures::insert_user(&pool, "100001", "root@test.com", "ROOT", None).await;
    test_helpers::create_work_session_for_user(&pool, root_user_id).await;

    let mut root_claims = test_helpers::build_root_claims();
    root_claims.id = root_user_id.to_string();
    let root_token = test_helpers::generate_jwt(&root_claims, &config.jwt_secret);

    (
        pool,
        config,
        app,
        root_token,
        city_id,
        victim_id,
        offender_id,
        measure_a,
        measure_b,
    )
}

// ──────────────────────────────────────────────────────────────────────────────
// Attendance Victims - filter by protective_measure_id
// ──────────────────────────────────────────────────────────────────────────────

#[actix_rt::test]
async fn attendance_victims_by_victim_filter_by_measure_returns_only_matching() {
    let (_pool, config, app, root_token, _city_id, victim_id, _offender_id, measure_a, measure_b) =
        setup_test_data().await;

    // Create 2 attendances with measure_a, 1 with measure_b, 1 without measure
    for measure in [Some(measure_a), Some(measure_a), Some(measure_b), None] {
        let payload = build_victim_attendance_payload(victim_id, measure);
        let req = test_helpers::with_auth_headers(
            test::TestRequest::post()
                .uri("/api/v1/attendance-victims")
                .set_json(&payload),
            &config,
            &root_token,
        )
        .to_request();
        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status(), StatusCode::CREATED);
    }

    // Filter by measure_a -> should return 2
    let req = test_helpers::with_auth_headers(
        test::TestRequest::get().uri(&format!(
            "/api/v1/attendance-victims/by-victim/{}?protective_measure_id={}",
            victim_id, measure_a
        )),
        &config,
        &root_token,
    )
    .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), StatusCode::OK);

    let body: serde_json::Value = test::read_body_json(resp).await;
    let data = body["data"].as_array().unwrap();
    assert_eq!(data.len(), 2, "Should return 2 attendances for measure_a");

    for attendance in data {
        assert_eq!(
            attendance["protective_measure_id"].as_str().unwrap(),
            measure_a.to_string()
        );
    }
}

#[actix_rt::test]
async fn attendance_victims_by_victim_filter_by_measure_b_returns_one() {
    let (_pool, config, app, root_token, _city_id, victim_id, _offender_id, measure_a, measure_b) =
        setup_test_data().await;

    for measure in [Some(measure_a), Some(measure_b)] {
        let payload = build_victim_attendance_payload(victim_id, measure);
        let req = test_helpers::with_auth_headers(
            test::TestRequest::post()
                .uri("/api/v1/attendance-victims")
                .set_json(&payload),
            &config,
            &root_token,
        )
        .to_request();
        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status(), StatusCode::CREATED);
    }

    // Filter by measure_b -> should return 1
    let req = test_helpers::with_auth_headers(
        test::TestRequest::get().uri(&format!(
            "/api/v1/attendance-victims/by-victim/{}?protective_measure_id={}",
            victim_id, measure_b
        )),
        &config,
        &root_token,
    )
    .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), StatusCode::OK);

    let body: serde_json::Value = test::read_body_json(resp).await;
    let data = body["data"].as_array().unwrap();
    assert_eq!(data.len(), 1, "Should return 1 attendance for measure_b");
    assert_eq!(
        data[0]["protective_measure_id"].as_str().unwrap(),
        measure_b.to_string()
    );
}

#[actix_rt::test]
async fn attendance_victims_by_victim_without_filter_returns_all() {
    let (_pool, config, app, root_token, _city_id, victim_id, _offender_id, measure_a, _measure_b) =
        setup_test_data().await;

    // Create 1 with measure, 1 without
    for measure in [Some(measure_a), None] {
        let payload = build_victim_attendance_payload(victim_id, measure);
        let req = test_helpers::with_auth_headers(
            test::TestRequest::post()
                .uri("/api/v1/attendance-victims")
                .set_json(&payload),
            &config,
            &root_token,
        )
        .to_request();
        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status(), StatusCode::CREATED);
    }

    // No filter -> should return all 2
    let req = test_helpers::with_auth_headers(
        test::TestRequest::get().uri(&format!(
            "/api/v1/attendance-victims/by-victim/{}",
            victim_id
        )),
        &config,
        &root_token,
    )
    .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), StatusCode::OK);

    let body: serde_json::Value = test::read_body_json(resp).await;
    let data = body["data"].as_array().unwrap();
    assert_eq!(data.len(), 2, "Without filter should return all attendances");

    // Verify both types are present: one with measure and one without
    let has_with_measure = data
        .iter()
        .any(|a| a["protective_measure_id"].as_str() == Some(&measure_a.to_string()));
    let has_without_measure = data
        .iter()
        .any(|a| a["protective_measure_id"].is_null());
    assert!(has_with_measure, "Should include attendance with measure");
    assert!(has_without_measure, "Should include attendance without measure");
}

#[actix_rt::test]
async fn attendance_victims_by_victim_filter_nonexistent_measure_returns_empty() {
    let (_pool, config, app, root_token, _city_id, victim_id, _offender_id, measure_a, _measure_b) =
        setup_test_data().await;

    let payload = build_victim_attendance_payload(victim_id, Some(measure_a));
    let req = test_helpers::with_auth_headers(
        test::TestRequest::post()
            .uri("/api/v1/attendance-victims")
            .set_json(&payload),
        &config,
        &root_token,
    )
    .to_request();
    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), StatusCode::CREATED);

    // Filter by a random UUID that doesn't match any measure
    let random_uuid = Uuid::new_v4();
    let req = test_helpers::with_auth_headers(
        test::TestRequest::get().uri(&format!(
            "/api/v1/attendance-victims/by-victim/{}?protective_measure_id={}",
            victim_id, random_uuid
        )),
        &config,
        &root_token,
    )
    .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), StatusCode::OK);

    let body: serde_json::Value = test::read_body_json(resp).await;
    let data = body["data"].as_array().unwrap();
    assert_eq!(data.len(), 0, "Filter with nonexistent measure should return empty");
}

// ──────────────────────────────────────────────────────────────────────────────
// Attendance Offenders - filter by protective_measure_id (by-offender)
// ──────────────────────────────────────────────────────────────────────────────

#[actix_rt::test]
async fn attendance_offenders_by_offender_filter_by_measure_returns_only_matching() {
    let (_pool, config, app, root_token, _city_id, victim_id, offender_id, measure_a, measure_b) =
        setup_test_data().await;

    for measure in [Some(measure_a), Some(measure_a), Some(measure_b), None] {
        let payload = build_offender_attendance_payload(offender_id, victim_id, measure);
        let req = test_helpers::with_auth_headers(
            test::TestRequest::post()
                .uri("/api/v1/attendance-offenders")
                .set_json(&payload),
            &config,
            &root_token,
        )
        .to_request();
        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status(), StatusCode::CREATED);
    }

    // Filter by measure_a -> should return 2
    let req = test_helpers::with_auth_headers(
        test::TestRequest::get().uri(&format!(
            "/api/v1/attendance-offenders/by-offender/{}?protective_measure_id={}",
            offender_id, measure_a
        )),
        &config,
        &root_token,
    )
    .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), StatusCode::OK);

    let body: serde_json::Value = test::read_body_json(resp).await;
    let data = body["data"].as_array().unwrap();
    assert_eq!(data.len(), 2, "Should return 2 attendances for measure_a");

    for attendance in data {
        assert_eq!(
            attendance["protective_measure_id"].as_str().unwrap(),
            measure_a.to_string()
        );
    }
}

#[actix_rt::test]
async fn attendance_offenders_by_offender_without_filter_returns_all() {
    let (_pool, config, app, root_token, _city_id, victim_id, offender_id, measure_a, _measure_b) =
        setup_test_data().await;

    for measure in [Some(measure_a), None] {
        let payload = build_offender_attendance_payload(offender_id, victim_id, measure);
        let req = test_helpers::with_auth_headers(
            test::TestRequest::post()
                .uri("/api/v1/attendance-offenders")
                .set_json(&payload),
            &config,
            &root_token,
        )
        .to_request();
        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status(), StatusCode::CREATED);
    }

    let req = test_helpers::with_auth_headers(
        test::TestRequest::get().uri(&format!(
            "/api/v1/attendance-offenders/by-offender/{}",
            offender_id
        )),
        &config,
        &root_token,
    )
    .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), StatusCode::OK);

    let body: serde_json::Value = test::read_body_json(resp).await;
    let data = body["data"].as_array().unwrap();
    assert_eq!(data.len(), 2, "Without filter should return all attendances");

    let has_with_measure = data
        .iter()
        .any(|a| a["protective_measure_id"].as_str() == Some(&measure_a.to_string()));
    let has_without_measure = data
        .iter()
        .any(|a| a["protective_measure_id"].is_null());
    assert!(has_with_measure, "Should include attendance with measure");
    assert!(has_without_measure, "Should include attendance without measure");
}

// ──────────────────────────────────────────────────────────────────────────────
// Attendance Offenders - filter by protective_measure_id (by-victim)
// ──────────────────────────────────────────────────────────────────────────────

#[actix_rt::test]
async fn attendance_offenders_by_victim_filter_by_measure_returns_only_matching() {
    let (_pool, config, app, root_token, _city_id, victim_id, offender_id, measure_a, measure_b) =
        setup_test_data().await;

    for measure in [Some(measure_a), Some(measure_b), None] {
        let payload = build_offender_attendance_payload(offender_id, victim_id, measure);
        let req = test_helpers::with_auth_headers(
            test::TestRequest::post()
                .uri("/api/v1/attendance-offenders")
                .set_json(&payload),
            &config,
            &root_token,
        )
        .to_request();
        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status(), StatusCode::CREATED);
    }

    // Filter by measure_a -> should return 1
    let req = test_helpers::with_auth_headers(
        test::TestRequest::get().uri(&format!(
            "/api/v1/attendance-offenders/by-victim/{}?protective_measure_id={}",
            victim_id, measure_a
        )),
        &config,
        &root_token,
    )
    .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), StatusCode::OK);

    let body: serde_json::Value = test::read_body_json(resp).await;
    let data = body["data"].as_array().unwrap();
    assert_eq!(data.len(), 1, "Should return 1 attendance for measure_a");
    assert_eq!(
        data[0]["protective_measure_id"].as_str().unwrap(),
        measure_a.to_string()
    );
}

#[actix_rt::test]
async fn attendance_offenders_by_victim_without_filter_returns_all() {
    let (_pool, config, app, root_token, _city_id, victim_id, offender_id, measure_a, _measure_b) =
        setup_test_data().await;

    for measure in [Some(measure_a), None] {
        let payload = build_offender_attendance_payload(offender_id, victim_id, measure);
        let req = test_helpers::with_auth_headers(
            test::TestRequest::post()
                .uri("/api/v1/attendance-offenders")
                .set_json(&payload),
            &config,
            &root_token,
        )
        .to_request();
        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status(), StatusCode::CREATED);
    }

    let req = test_helpers::with_auth_headers(
        test::TestRequest::get().uri(&format!(
            "/api/v1/attendance-offenders/by-victim/{}",
            victim_id
        )),
        &config,
        &root_token,
    )
    .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), StatusCode::OK);

    let body: serde_json::Value = test::read_body_json(resp).await;
    let data = body["data"].as_array().unwrap();
    assert_eq!(data.len(), 2, "Without filter should return all attendances");

    let has_with_measure = data
        .iter()
        .any(|a| a["protective_measure_id"].as_str() == Some(&measure_a.to_string()));
    let has_without_measure = data
        .iter()
        .any(|a| a["protective_measure_id"].is_null());
    assert!(has_with_measure, "Should include attendance with measure");
    assert!(has_without_measure, "Should include attendance without measure");
}

#[actix_rt::test]
async fn attendance_offenders_by_victim_filter_nonexistent_measure_returns_empty() {
    let (_pool, config, app, root_token, _city_id, victim_id, offender_id, measure_a, _measure_b) =
        setup_test_data().await;

    let payload = build_offender_attendance_payload(offender_id, victim_id, Some(measure_a));
    let req = test_helpers::with_auth_headers(
        test::TestRequest::post()
            .uri("/api/v1/attendance-offenders")
            .set_json(&payload),
        &config,
        &root_token,
    )
    .to_request();
    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), StatusCode::CREATED);

    let random_uuid = Uuid::new_v4();
    let req = test_helpers::with_auth_headers(
        test::TestRequest::get().uri(&format!(
            "/api/v1/attendance-offenders/by-victim/{}?protective_measure_id={}",
            victim_id, random_uuid
        )),
        &config,
        &root_token,
    )
    .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), StatusCode::OK);

    let body: serde_json::Value = test::read_body_json(resp).await;
    let data = body["data"].as_array().unwrap();
    assert_eq!(data.len(), 0, "Filter with nonexistent measure should return empty");
}
