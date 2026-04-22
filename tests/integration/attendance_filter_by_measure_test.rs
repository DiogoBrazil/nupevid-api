use actix_web::{http::StatusCode, test};
use chrono::{NaiveDate, NaiveTime};
use uuid::Uuid;

use crate::common::{db_fixtures, test_helpers};

fn build_victim_attendance_payload(protective_measure_id: Uuid) -> serde_json::Value {
    serde_json::json!({
        "protective_measure_id": protective_measure_id,
        "was_victim_present": true,
        "attendance_date": NaiveDate::from_ymd_opt(2025, 1, 1).unwrap(),
        "attendance_time": NaiveTime::from_hms_opt(10, 0, 0).unwrap(),
        "is_remote": false,
        "needs_legal_assistance": false,
        "needs_psychological_support": false,
        "was_instructed_about_protective_measure_procedures": false,
        "offender_violated_protective_measure": false,
    })
}

fn build_offender_attendance_payload(protective_measure_id: Uuid) -> serde_json::Value {
    serde_json::json!({
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

async fn set_victim_attendance_created_at(pool: &sqlx::PgPool, id: Uuid, timestamp: &str) {
    sqlx::query(
        "UPDATE attendance_victims
         SET created_at = $2::timestamptz, updated_at = $2::timestamptz
         WHERE id = $1",
    )
    .bind(id)
    .bind(timestamp)
    .execute(pool)
    .await
    .expect("failed to update victim attendance created_at");
}

async fn set_offender_attendance_created_at(pool: &sqlx::PgPool, id: Uuid, timestamp: &str) {
    sqlx::query(
        "UPDATE attendance_offenders
         SET created_at = $2::timestamptz, updated_at = $2::timestamptz
         WHERE id = $1",
    )
    .bind(id)
    .bind(timestamp)
    .execute(pool)
    .await
    .expect("failed to update offender attendance created_at");
}

#[actix_rt::test]
async fn attendance_victims_by_measure_returns_matching_in_created_order() {
    let (pool, config, app, root_token, _city_id, _victim_id, _offender_id, measure_a, measure_b) =
        setup_test_data().await;

    let payload = build_victim_attendance_payload(measure_a);
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
    let first_body: serde_json::Value = test::read_body_json(resp).await;
    let first_id = Uuid::parse_str(first_body["data"]["id"].as_str().unwrap()).unwrap();

    let payload = build_victim_attendance_payload(measure_a);
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
    let second_body: serde_json::Value = test::read_body_json(resp).await;
    let second_id = Uuid::parse_str(second_body["data"]["id"].as_str().unwrap()).unwrap();

    let payload = build_victim_attendance_payload(measure_b);
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

    set_victim_attendance_created_at(&pool, first_id, "2025-01-02T00:00:00Z").await;
    set_victim_attendance_created_at(&pool, second_id, "2025-01-01T00:00:00Z").await;

    let req = test_helpers::with_auth_headers(
        test::TestRequest::get().uri(&format!(
            "/api/v1/attendance-victims/by-measure/{}",
            measure_a
        )),
        &config,
        &root_token,
    )
    .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), StatusCode::OK);

    let body: serde_json::Value = test::read_body_json(resp).await;
    let data = body["data"].as_array().unwrap();
    assert_eq!(data.len(), 2);
    assert_eq!(data[0]["id"].as_str().unwrap(), second_id.to_string());
    assert_eq!(data[1]["id"].as_str().unwrap(), first_id.to_string());
    let measure_a_str = measure_a.to_string();
    assert!(data.iter().all(|attendance| {
        attendance["protective_measure_id"].as_str() == Some(measure_a_str.as_str())
    }));
}

#[actix_rt::test]
async fn attendance_victims_by_measure_without_attendances_returns_empty() {
    let (_pool, config, app, root_token, _city_id, _victim_id, _offender_id, measure_a, _measure_b) =
        setup_test_data().await;

    let req = test_helpers::with_auth_headers(
        test::TestRequest::get().uri(&format!(
            "/api/v1/attendance-victims/by-measure/{}",
            measure_a
        )),
        &config,
        &root_token,
    )
    .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), StatusCode::OK);

    let body: serde_json::Value = test::read_body_json(resp).await;
    assert_eq!(body["data"].as_array().unwrap().len(), 0);
}

#[actix_rt::test]
async fn attendance_victims_by_measure_nonexistent_measure_returns_not_found() {
    let (
        _pool,
        config,
        app,
        root_token,
        _city_id,
        _victim_id,
        _offender_id,
        _measure_a,
        _measure_b,
    ) = setup_test_data().await;

    let req = test_helpers::with_auth_headers(
        test::TestRequest::get().uri(&format!(
            "/api/v1/attendance-victims/by-measure/{}",
            Uuid::new_v4()
        )),
        &config,
        &root_token,
    )
    .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), StatusCode::NOT_FOUND);
}

#[actix_rt::test]
async fn attendance_victims_by_victim_returns_all_without_measure_filter() {
    let (_pool, config, app, root_token, _city_id, victim_id, _offender_id, measure_a, measure_b) =
        setup_test_data().await;

    for measure in [measure_a, measure_b] {
        let payload = build_victim_attendance_payload(measure);
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
    assert_eq!(data.len(), 2);
    let measure_a_str = measure_a.to_string();
    let measure_b_str = measure_b.to_string();
    assert!(
        data.iter()
            .any(|a| a["protective_measure_id"].as_str() == Some(measure_a_str.as_str()))
    );
    assert!(
        data.iter()
            .any(|a| a["protective_measure_id"].as_str() == Some(measure_b_str.as_str()))
    );
}

#[actix_rt::test]
async fn attendance_offenders_by_measure_returns_matching_in_created_order() {
    let (pool, config, app, root_token, _city_id, _victim_id, _offender_id, measure_a, measure_b) =
        setup_test_data().await;

    let payload = build_offender_attendance_payload(measure_a);
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
    let first_body: serde_json::Value = test::read_body_json(resp).await;
    let first_id = Uuid::parse_str(first_body["data"]["id"].as_str().unwrap()).unwrap();

    let payload = build_offender_attendance_payload(measure_a);
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
    let second_body: serde_json::Value = test::read_body_json(resp).await;
    let second_id = Uuid::parse_str(second_body["data"]["id"].as_str().unwrap()).unwrap();

    let payload = build_offender_attendance_payload(measure_b);
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

    set_offender_attendance_created_at(&pool, first_id, "2025-01-02T00:00:00Z").await;
    set_offender_attendance_created_at(&pool, second_id, "2025-01-01T00:00:00Z").await;

    let req = test_helpers::with_auth_headers(
        test::TestRequest::get().uri(&format!(
            "/api/v1/attendance-offenders/by-measure/{}",
            measure_a
        )),
        &config,
        &root_token,
    )
    .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), StatusCode::OK);

    let body: serde_json::Value = test::read_body_json(resp).await;
    let data = body["data"].as_array().unwrap();
    assert_eq!(data.len(), 2);
    assert_eq!(data[0]["id"].as_str().unwrap(), second_id.to_string());
    assert_eq!(data[1]["id"].as_str().unwrap(), first_id.to_string());
    let measure_a_str = measure_a.to_string();
    assert!(data.iter().all(|attendance| {
        attendance["protective_measure_id"].as_str() == Some(measure_a_str.as_str())
    }));
}

#[actix_rt::test]
async fn attendance_offenders_by_measure_without_attendances_returns_empty() {
    let (_pool, config, app, root_token, _city_id, _victim_id, _offender_id, measure_a, _measure_b) =
        setup_test_data().await;

    let req = test_helpers::with_auth_headers(
        test::TestRequest::get().uri(&format!(
            "/api/v1/attendance-offenders/by-measure/{}",
            measure_a
        )),
        &config,
        &root_token,
    )
    .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), StatusCode::OK);

    let body: serde_json::Value = test::read_body_json(resp).await;
    assert_eq!(body["data"].as_array().unwrap().len(), 0);
}

#[actix_rt::test]
async fn attendance_offenders_by_measure_nonexistent_measure_returns_not_found() {
    let (
        _pool,
        config,
        app,
        root_token,
        _city_id,
        _victim_id,
        _offender_id,
        _measure_a,
        _measure_b,
    ) = setup_test_data().await;

    let req = test_helpers::with_auth_headers(
        test::TestRequest::get().uri(&format!(
            "/api/v1/attendance-offenders/by-measure/{}",
            Uuid::new_v4()
        )),
        &config,
        &root_token,
    )
    .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), StatusCode::NOT_FOUND);
}

#[actix_rt::test]
async fn attendance_offenders_by_offender_returns_all_without_measure_filter() {
    let (_pool, config, app, root_token, _city_id, _victim_id, offender_id, measure_a, measure_b) =
        setup_test_data().await;

    for measure in [measure_a, measure_b] {
        let payload = build_offender_attendance_payload(measure);
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
    assert_eq!(data.len(), 2);
    let measure_a_str = measure_a.to_string();
    let measure_b_str = measure_b.to_string();
    assert!(
        data.iter()
            .any(|a| a["protective_measure_id"].as_str() == Some(measure_a_str.as_str()))
    );
    assert!(
        data.iter()
            .any(|a| a["protective_measure_id"].as_str() == Some(measure_b_str.as_str()))
    );
}

#[actix_rt::test]
async fn attendance_offenders_by_victim_returns_all_without_measure_filter() {
    let (_pool, config, app, root_token, _city_id, victim_id, _offender_id, measure_a, measure_b) =
        setup_test_data().await;

    for measure in [measure_a, measure_b] {
        let payload = build_offender_attendance_payload(measure);
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
    assert_eq!(data.len(), 2);
    let measure_a_str = measure_a.to_string();
    let measure_b_str = measure_b.to_string();
    assert!(
        data.iter()
            .any(|a| a["protective_measure_id"].as_str() == Some(measure_a_str.as_str()))
    );
    assert!(
        data.iter()
            .any(|a| a["protective_measure_id"].as_str() == Some(measure_b_str.as_str()))
    );
}
