use actix_web::{http::StatusCode, test};
use chrono::{NaiveDate, NaiveTime};
use sqlx::PgPool;
use uuid::Uuid;

use crate::common::{db_fixtures, test_helpers};

fn build_attendance_payload(pm_id: Uuid) -> serde_json::Value {
    serde_json::json!({
        "was_victim_present": true,
        "attendance_date": NaiveDate::from_ymd_opt(2025, 1, 1).unwrap(),
        "attendance_time": NaiveTime::from_hms_opt(10, 0, 0).unwrap(),
        "description": "Atendimento realizado",
        "latitude": serde_json::Value::Null,
        "longitude": serde_json::Value::Null,
        "address": serde_json::Value::Null,
        "protective_measure_id": pm_id,
        "is_remote": false,
        "risk_level": serde_json::Value::Null,
        "offender_freedom_status": serde_json::Value::Null,
        "offender_has_firearm_access": serde_json::Value::Null,
        "needs_legal_assistance": false,
        "needs_psychological_support": false,
        "was_instructed_about_protective_measure_procedures": false,
        "offender_violated_protective_measure": false,
    })
}

fn build_attendance_payload_with_address(pm_id: Uuid, city_id: Uuid) -> serde_json::Value {
    serde_json::json!({
        "was_victim_present": true,
        "attendance_date": NaiveDate::from_ymd_opt(2025, 1, 15).unwrap(),
        "attendance_time": NaiveTime::from_hms_opt(14, 30, 0).unwrap(),
        "description": "Visita domiciliar",
        "latitude": -23.5505,
        "longitude": -46.6333,
        "address": {
            "street": "Rua Atendimento",
            "number": "100",
            "district": "Centro",
            "city_id": city_id,
            "zip_code": "22222-222",
            "complement": "Casa"
        },
        "protective_measure_id": pm_id,
        "is_remote": false,
        "risk_level": serde_json::Value::Null,
        "offender_freedom_status": serde_json::Value::Null,
        "offender_has_firearm_access": serde_json::Value::Null,
        "needs_legal_assistance": false,
        "needs_psychological_support": false,
        "was_instructed_about_protective_measure_procedures": false,
        "offender_violated_protective_measure": false,
    })
}

#[sqlx::test]
async fn create_attendance_success_for_victim_in_own_city(pool: PgPool) {

    let config = test_helpers::build_test_config();
    let app = test_helpers::create_full_test_app(pool.clone(), config.clone()).await;

    let city = db_fixtures::insert_city(&pool, "Cidade A").await;
    let victim_id = db_fixtures::insert_victim(&pool, "Vitima", city).await;
    let offender_id = db_fixtures::insert_offender(&pool, "Agressor", city).await;
    let pm_id =
        db_fixtures::insert_protective_measure(&pool, victim_id, offender_id, city, "Valid").await;

    // Create user with default policies and work session
    let user_id =
        db_fixtures::insert_user(&pool, "100001", "admin@test.com", "CITY_ADMIN", Some(city)).await;
    test_helpers::create_work_session_for_user(&pool, user_id).await;

    // Create claims with the actual user_id
    let mut admin_claims = test_helpers::build_city_admin_claims(city);
    admin_claims.id = user_id.to_string();
    let admin_token = test_helpers::generate_jwt(&admin_claims, &config.jwt_secret);

    let payload = build_attendance_payload(pm_id);
    let req = test_helpers::with_auth_headers(
        test::TestRequest::post()
            .uri("/api/v1/attendance-victims")
            .set_json(&payload),
        &config,
        &admin_token,
    )
    .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), StatusCode::CREATED);

    let body: serde_json::Value = test::read_body_json(resp).await;
    assert_eq!(body["status"].as_u64().unwrap(), 201);
    assert_eq!(
        body["data"]["victim_id"].as_str().unwrap(),
        victim_id.to_string()
    );
}

#[sqlx::test]
async fn city_admin_cannot_create_attendance_for_other_city_victim(pool: PgPool) {

    let config = test_helpers::build_test_config();
    let app = test_helpers::create_full_test_app(pool.clone(), config.clone()).await;

    let city_a = db_fixtures::insert_city(&pool, "Cidade A").await;
    let city_b = db_fixtures::insert_city(&pool, "Cidade B").await;
    let victim_b = db_fixtures::insert_victim(&pool, "Vitima B", city_b).await;
    let offender_b = db_fixtures::insert_offender(&pool, "Agressor B", city_b).await;
    let pm_b =
        db_fixtures::insert_protective_measure(&pool, victim_b, offender_b, city_b, "Valid").await;

    let admin_a_claims = test_helpers::build_city_admin_claims(city_a);
    let admin_a_token = test_helpers::generate_jwt(&admin_a_claims, &config.jwt_secret);

    let payload = build_attendance_payload(pm_b);
    let req = test_helpers::with_auth_headers(
        test::TestRequest::post()
            .uri("/api/v1/attendance-victims")
            .set_json(&payload),
        &config,
        &admin_a_token,
    )
    .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), StatusCode::FORBIDDEN);
}

#[sqlx::test]
async fn list_attendance_victims_filtered_by_city(pool: PgPool) {

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

    // Create ROOT user with work session
    let root_user_id =
        db_fixtures::insert_user(&pool, "100001", "root@test.com", "ROOT", None).await;
    test_helpers::create_work_session_for_user(&pool, root_user_id).await;

    let mut root_claims = test_helpers::build_root_claims();
    root_claims.id = root_user_id.to_string();
    let root_token = test_helpers::generate_jwt(&root_claims, &config.jwt_secret);

    // Create attendance for both victims as ROOT
    for pm_id in [pm_a, pm_b] {
        let payload = build_attendance_payload(pm_id);
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

    // CITY_ADMIN A should only see its city's attendance
    let admin_a_claims = test_helpers::build_city_admin_claims(city_a);
    let admin_a_token = test_helpers::generate_jwt(&admin_a_claims, &config.jwt_secret);

    let list_req = test_helpers::with_auth_headers(
        test::TestRequest::get().uri("/api/v1/attendance-victims"),
        &config,
        &admin_a_token,
    )
    .to_request();
    let list_resp = test::call_service(&app, list_req).await;
    assert_eq!(list_resp.status(), StatusCode::OK);
    let body: serde_json::Value = test::read_body_json(list_resp).await;
    let atts = body["data"].as_array().unwrap();
    assert_eq!(atts.len(), 1);
    assert_eq!(atts[0]["victim_id"].as_str().unwrap(), victim_a.to_string());
}

#[sqlx::test]
async fn delete_attendance_soft_delete_and_not_listed(pool: PgPool) {

    let config = test_helpers::build_test_config();
    let app = test_helpers::create_full_test_app(pool.clone(), config.clone()).await;

    let city = db_fixtures::insert_city(&pool, "Cidade Del").await;
    let victim_id = db_fixtures::insert_victim(&pool, "Vitima", city).await;
    let offender_id = db_fixtures::insert_offender(&pool, "Agressor", city).await;
    let pm_id =
        db_fixtures::insert_protective_measure(&pool, victim_id, offender_id, city, "Valid").await;

    // Create user with work session
    let user_id =
        db_fixtures::insert_user(&pool, "100001", "admin@test.com", "CITY_ADMIN", Some(city)).await;
    test_helpers::create_work_session_for_user(&pool, user_id).await;

    let mut admin_claims = test_helpers::build_city_admin_claims(city);
    admin_claims.id = user_id.to_string();
    let admin_token = test_helpers::generate_jwt(&admin_claims, &config.jwt_secret);

    let payload = build_attendance_payload(pm_id);
    let create_req = test_helpers::with_auth_headers(
        test::TestRequest::post()
            .uri("/api/v1/attendance-victims")
            .set_json(&payload),
        &config,
        &admin_token,
    )
    .to_request();
    let create_resp = test::call_service(&app, create_req).await;
    assert_eq!(create_resp.status(), StatusCode::CREATED);
    let body: serde_json::Value = test::read_body_json(create_resp).await;
    let attendance_id = body["data"]["id"].as_str().unwrap();

    let delete_req = test_helpers::with_auth_headers(
        test::TestRequest::delete().uri(&format!("/api/v1/attendance-victims/{}", attendance_id)),
        &config,
        &admin_token,
    )
    .to_request();
    let delete_resp = test::call_service(&app, delete_req).await;
    assert_eq!(delete_resp.status(), StatusCode::OK);

    // GET by id -> 404
    let get_req = test_helpers::with_auth_headers(
        test::TestRequest::get().uri(&format!("/api/v1/attendance-victims/{}", attendance_id)),
        &config,
        &admin_token,
    )
    .to_request();
    let get_resp = test::call_service(&app, get_req).await;
    assert_eq!(get_resp.status(), StatusCode::NOT_FOUND);

    // List -> empty
    let list_req = test_helpers::with_auth_headers(
        test::TestRequest::get().uri("/api/v1/attendance-victims"),
        &config,
        &admin_token,
    )
    .to_request();
    let list_resp = test::call_service(&app, list_req).await;
    assert_eq!(list_resp.status(), StatusCode::OK);
    let list_body: serde_json::Value = test::read_body_json(list_resp).await;
    let atts = list_body["data"].as_array().unwrap();
    assert!(
        atts.iter()
            .all(|a| a["id"].as_str().unwrap() != attendance_id)
    );
}

#[sqlx::test]
async fn get_attendance_by_id_for_other_city_returns_forbidden(pool: PgPool) {

    let config = test_helpers::build_test_config();
    let app = test_helpers::create_full_test_app(pool.clone(), config.clone()).await;

    let city_a = db_fixtures::insert_city(&pool, "Cidade A").await;
    let city_b = db_fixtures::insert_city(&pool, "Cidade B").await;

    // Create ROOT user with work session
    let root_user_id =
        db_fixtures::insert_user(&pool, "100001", "root@test.com", "ROOT", None).await;
    test_helpers::create_work_session_for_user(&pool, root_user_id).await;

    let mut root_claims = test_helpers::build_root_claims();
    root_claims.id = root_user_id.to_string();
    let root_token = test_helpers::generate_jwt(&root_claims, &config.jwt_secret);

    let victim_b = db_fixtures::insert_victim(&pool, "Vitima B", city_b).await;
    let offender_b = db_fixtures::insert_offender(&pool, "Agressor B", city_b).await;
    let pm_b =
        db_fixtures::insert_protective_measure(&pool, victim_b, offender_b, city_b, "Valid").await;
    let payload = build_attendance_payload(pm_b);
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

    // CITY_ADMIN A should be forbidden from accessing attendance in city B
    let admin_a_claims = test_helpers::build_city_admin_claims(city_a);
    let admin_a_token = test_helpers::generate_jwt(&admin_a_claims, &config.jwt_secret);

    let get_req = test_helpers::with_auth_headers(
        test::TestRequest::get().uri(&format!("/api/v1/attendance-victims/{}", attendance_id)),
        &config,
        &admin_a_token,
    )
    .to_request();
    let get_resp = test::call_service(&app, get_req).await;
    assert_eq!(get_resp.status(), StatusCode::FORBIDDEN);
}

#[sqlx::test]
async fn create_attendance_with_address_in_single_request(pool: PgPool) {

    let config = test_helpers::build_test_config();
    let app = test_helpers::create_full_test_app(pool.clone(), config.clone()).await;

    let city = db_fixtures::insert_city(&pool, "Cidade Addr").await;
    let victim_id = db_fixtures::insert_victim(&pool, "Vitima", city).await;
    let offender_id = db_fixtures::insert_offender(&pool, "Agressor", city).await;
    let pm_id =
        db_fixtures::insert_protective_measure(&pool, victim_id, offender_id, city, "Valid").await;

    // Create ROOT user with work session
    let user_id = db_fixtures::insert_user(&pool, "100001", "root@test.com", "ROOT", None).await;
    test_helpers::create_work_session_for_user(&pool, user_id).await;

    let mut root_claims = test_helpers::build_root_claims();
    root_claims.id = user_id.to_string();
    let root_token = test_helpers::generate_jwt(&root_claims, &config.jwt_secret);

    // Create attendance with address
    let payload = build_attendance_payload_with_address(pm_id, city);
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

    // Verify attendance data
    assert_eq!(
        body["data"]["description"].as_str().unwrap(),
        "Visita domiciliar"
    );
    assert!(body["data"]["latitude"].is_number());
    assert!(body["data"]["longitude"].is_number());

    // Verify address data is included
    assert!(body["data"]["address"].is_object());
    assert_eq!(
        body["data"]["address"]["street"].as_str().unwrap(),
        "Rua Atendimento"
    );
    assert_eq!(body["data"]["address"]["number"].as_str().unwrap(), "100");
    assert_eq!(
        body["data"]["address"]["district"].as_str().unwrap(),
        "Centro"
    );
    assert_eq!(
        body["data"]["address"]["city_id"].as_str().unwrap(),
        city.to_string()
    );
}

#[sqlx::test]
async fn get_attendance_returns_address_when_exists(pool: PgPool) {

    let config = test_helpers::build_test_config();
    let app = test_helpers::create_full_test_app(pool.clone(), config.clone()).await;

    let city = db_fixtures::insert_city(&pool, "Cidade Get").await;
    let victim_id = db_fixtures::insert_victim(&pool, "Vitima", city).await;
    let offender_id = db_fixtures::insert_offender(&pool, "Agressor", city).await;
    let pm_id =
        db_fixtures::insert_protective_measure(&pool, victim_id, offender_id, city, "Valid").await;

    // Create ROOT user with work session
    let root_user_id =
        db_fixtures::insert_user(&pool, "100001", "root@test.com", "ROOT", None).await;
    test_helpers::create_work_session_for_user(&pool, root_user_id).await;

    let mut root_claims = test_helpers::build_root_claims();
    root_claims.id = root_user_id.to_string();
    let root_token = test_helpers::generate_jwt(&root_claims, &config.jwt_secret);

    // Create attendance with address
    let payload = build_attendance_payload_with_address(pm_id, city);
    let create_req = test_helpers::with_auth_headers(
        test::TestRequest::post()
            .uri("/api/v1/attendance-victims")
            .set_json(&payload),
        &config,
        &root_token,
    )
    .to_request();

    let create_resp = test::call_service(&app, create_req).await;
    let create_body: serde_json::Value = test::read_body_json(create_resp).await;
    let attendance_id = create_body["data"]["id"].as_str().unwrap();

    // GET attendance should include address
    let get_req = test_helpers::with_auth_headers(
        test::TestRequest::get().uri(&format!("/api/v1/attendance-victims/{}", attendance_id)),
        &config,
        &root_token,
    )
    .to_request();

    let get_resp = test::call_service(&app, get_req).await;
    assert_eq!(get_resp.status(), StatusCode::OK);

    let body: serde_json::Value = test::read_body_json(get_resp).await;
    assert!(body["data"]["address"].is_object());
    assert_eq!(
        body["data"]["address"]["street"].as_str().unwrap(),
        "Rua Atendimento"
    );
}

#[sqlx::test]
async fn update_attendance_can_add_or_update_address(pool: PgPool) {

    let config = test_helpers::build_test_config();
    let app = test_helpers::create_full_test_app(pool.clone(), config.clone()).await;

    let city = db_fixtures::insert_city(&pool, "Cidade Update").await;
    let victim_id = db_fixtures::insert_victim(&pool, "Vitima", city).await;
    let offender_id = db_fixtures::insert_offender(&pool, "Agressor", city).await;
    let pm_id =
        db_fixtures::insert_protective_measure(&pool, victim_id, offender_id, city, "Valid").await;

    // Create ROOT user with work session
    let root_user_id =
        db_fixtures::insert_user(&pool, "100001", "root@test.com", "ROOT", None).await;
    test_helpers::create_work_session_for_user(&pool, root_user_id).await;

    let mut root_claims = test_helpers::build_root_claims();
    root_claims.id = root_user_id.to_string();
    let root_token = test_helpers::generate_jwt(&root_claims, &config.jwt_secret);

    // Create attendance without address
    let payload = build_attendance_payload(pm_id);
    let create_req = test_helpers::with_auth_headers(
        test::TestRequest::post()
            .uri("/api/v1/attendance-victims")
            .set_json(&payload),
        &config,
        &root_token,
    )
    .to_request();

    let create_resp = test::call_service(&app, create_req).await;
    let create_body: serde_json::Value = test::read_body_json(create_resp).await;
    let attendance_id = create_body["data"]["id"].as_str().unwrap();

    // Verify no address initially
    assert!(create_body["data"]["address"].is_null());

    // Update attendance adding address
    let update_payload = serde_json::json!({
        "was_victim_present": false,
        "attendance_date": NaiveDate::from_ymd_opt(2025, 2, 1).unwrap(),
        "attendance_time": NaiveTime::from_hms_opt(16, 0, 0).unwrap(),
        "description": "Atendimento atualizado",
        "latitude": null,
        "longitude": null,
        "address": {
            "street": "Nova Rua",
            "number": "200",
            "district": "Bairro Novo",
            "city_id": city,
            "zip_code": "20000-000",
            "complement": null
        },
        "protective_measure_id": pm_id,
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
        &root_token,
    )
    .to_request();

    let update_resp = test::call_service(&app, update_req).await;
    assert_eq!(update_resp.status(), StatusCode::OK);

    let update_body: serde_json::Value = test::read_body_json(update_resp).await;
    assert_eq!(
        update_body["data"]["description"].as_str().unwrap(),
        "Atendimento atualizado"
    );
    assert!(!update_body["data"]["was_victim_present"].as_bool().unwrap());
    assert!(update_body["data"]["address"].is_object());
    assert_eq!(
        update_body["data"]["address"]["street"].as_str().unwrap(),
        "Nova Rua"
    );
    assert_eq!(
        update_body["data"]["address"]["city_id"].as_str().unwrap(),
        city.to_string()
    );
}

#[sqlx::test]
async fn city_admin_cannot_create_attendance_with_address_for_other_city_victim(pool: PgPool) {

    let config = test_helpers::build_test_config();
    let app = test_helpers::create_full_test_app(pool.clone(), config.clone()).await;

    let city_a = db_fixtures::insert_city(&pool, "Cidade A").await;
    let city_b = db_fixtures::insert_city(&pool, "Cidade B").await;
    let victim_b = db_fixtures::insert_victim(&pool, "Vitima B", city_b).await;
    let offender_b = db_fixtures::insert_offender(&pool, "Agressor B", city_b).await;
    let pm_b =
        db_fixtures::insert_protective_measure(&pool, victim_b, offender_b, city_b, "Valid").await;

    let admin_a_claims = test_helpers::build_city_admin_claims(city_a);
    let admin_a_token = test_helpers::generate_jwt(&admin_a_claims, &config.jwt_secret);

    // Try to create attendance with address for victim in city B
    let payload = build_attendance_payload_with_address(pm_b, city_b);
    let req = test_helpers::with_auth_headers(
        test::TestRequest::post()
            .uri("/api/v1/attendance-victims")
            .set_json(&payload),
        &config,
        &admin_a_token,
    )
    .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), StatusCode::FORBIDDEN);
}

#[sqlx::test]
async fn create_attendance_without_active_session_fails(pool: PgPool) {

    let config = test_helpers::build_test_config();
    let app = test_helpers::create_full_test_app(pool.clone(), config.clone()).await;

    let city_id = db_fixtures::insert_city(&pool, "Test City").await;
    let victim_id = db_fixtures::insert_victim(&pool, "Test Victim", city_id).await;
    let offender_id = db_fixtures::insert_offender(&pool, "Test Offender", city_id).await;
    let pm_id =
        db_fixtures::insert_protective_measure(&pool, victim_id, offender_id, city_id, "Valid")
            .await;

    // Create CITY_ADMIN user WITHOUT creating work session (this is the key difference)
    // CITY_ADMIN has the necessary policies, so we can test the work session requirement
    let user_id = db_fixtures::insert_user(
        &pool,
        "100099",
        "admin@test.com",
        "CITY_ADMIN",
        Some(city_id),
    )
    .await;

    // DO NOT create work session here - that's the whole point of this test
    // test_helpers::create_work_session_for_user(&pool, user_id).await; // ← OMITTED

    let mut claims = test_helpers::build_city_admin_claims(city_id);
    claims.id = user_id.to_string();
    let token = test_helpers::generate_jwt(&claims, &config.jwt_secret);

    let payload = build_attendance_payload(pm_id);
    let req = test_helpers::with_auth_headers(
        test::TestRequest::post()
            .uri("/api/v1/attendance-victims")
            .set_json(&payload),
        &config,
        &token,
    )
    .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), StatusCode::BAD_REQUEST);

    let body: serde_json::Value = test::read_body_json(resp).await;
    let message = body["message"].as_str().unwrap();
    assert!(
        message.contains("active work session") || message.contains("No active work session"),
        "Error message should mention active work session requirement, got: {}",
        message
    );
}

#[sqlx::test]
async fn get_attendance_victims_by_victim_id_success(pool: PgPool) {

    let config = test_helpers::build_test_config();
    let app = test_helpers::create_full_test_app(pool.clone(), config.clone()).await;

    let city_id = db_fixtures::insert_city(&pool, "Test City").await;
    let victim1_id = db_fixtures::insert_victim(&pool, "Victim 1", city_id).await;
    let victim2_id = db_fixtures::insert_victim(&pool, "Victim 2", city_id).await;
    let offender1_id = db_fixtures::insert_offender(&pool, "Offender 1", city_id).await;
    let offender2_id = db_fixtures::insert_offender(&pool, "Offender 2", city_id).await;
    let pm1 =
        db_fixtures::insert_protective_measure(&pool, victim1_id, offender1_id, city_id, "Valid")
            .await;
    let pm2 =
        db_fixtures::insert_protective_measure(&pool, victim2_id, offender2_id, city_id, "Valid")
            .await;

    // Create ROOT user with work session
    let root_user_id =
        db_fixtures::insert_user(&pool, "100100", "root@test.com", "ROOT", None).await;
    test_helpers::create_work_session_for_user(&pool, root_user_id).await;

    let mut root_claims = test_helpers::build_root_claims();
    root_claims.id = root_user_id.to_string();
    let root_token = test_helpers::generate_jwt(&root_claims, &config.jwt_secret);

    // Create 2 attendances for victim1
    for i in 0..2 {
        let payload = serde_json::json!({
            "protective_measure_id": pm1,
            "was_victim_present": true,
            "attendance_date": format!("2025-01-{:02}", i + 1),
            "attendance_time": "14:30:00",
            "is_remote": false,
            "needs_legal_assistance": false,
            "needs_psychological_support": false,
            "was_instructed_about_protective_measure_procedures": false,
            "offender_violated_protective_measure": false
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
        assert_eq!(resp.status(), StatusCode::CREATED);
    }

    // Create 1 attendance for victim2
    let payload2 = build_attendance_payload(pm2);
    let req2 = test_helpers::with_auth_headers(
        test::TestRequest::post()
            .uri("/api/v1/attendance-victims")
            .set_json(&payload2),
        &config,
        &root_token,
    )
    .to_request();

    let resp2 = test::call_service(&app, req2).await;
    assert_eq!(resp2.status(), StatusCode::CREATED);

    // Get attendances by victim1 ID
    let get_req = test_helpers::with_auth_headers(
        test::TestRequest::get().uri(&format!(
            "/api/v1/attendance-victims/by-victim/{}",
            victim1_id
        )),
        &config,
        &root_token,
    )
    .to_request();

    let get_resp = test::call_service(&app, get_req).await;
    assert_eq!(get_resp.status(), StatusCode::OK);

    let body: serde_json::Value = test::read_body_json(get_resp).await;
    let data = body["data"].as_array().unwrap();

    // Should return exactly 2 attendances for victim1
    assert_eq!(data.len(), 2, "Should return 2 attendances for victim 1");

    // Verify all returned attendances belong to victim1
    for attendance in data {
        assert_eq!(
            attendance["victim_id"].as_str().unwrap(),
            victim1_id.to_string(),
            "All returned attendances should belong to victim 1"
        );
    }
}

#[sqlx::test]
async fn get_attendance_victims_by_victim_different_city_forbidden(pool: PgPool) {

    let config = test_helpers::build_test_config();
    let app = test_helpers::create_full_test_app(pool.clone(), config.clone()).await;

    let city_a = db_fixtures::insert_city(&pool, "City A").await;
    let city_b = db_fixtures::insert_city(&pool, "City B").await;

    let victim_b = db_fixtures::insert_victim(&pool, "Victim B", city_b).await;
    let offender_b = db_fixtures::insert_offender(&pool, "Offender B", city_b).await;
    let pm_b =
        db_fixtures::insert_protective_measure(&pool, victim_b, offender_b, city_b, "Valid").await;

    // Create ROOT user with work session to create attendance in city B
    let root_user_id =
        db_fixtures::insert_user(&pool, "100101", "root@test.com", "ROOT", None).await;
    test_helpers::create_work_session_for_user(&pool, root_user_id).await;

    let mut root_claims = test_helpers::build_root_claims();
    root_claims.id = root_user_id.to_string();
    let root_token = test_helpers::generate_jwt(&root_claims, &config.jwt_secret);

    // Create attendance for victim in city B
    let payload = build_attendance_payload(pm_b);
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

    // Try to get attendances as CITY_ADMIN from city A
    let admin_a_claims = test_helpers::build_city_admin_claims(city_a);
    let admin_a_token = test_helpers::generate_jwt(&admin_a_claims, &config.jwt_secret);

    let get_req = test_helpers::with_auth_headers(
        test::TestRequest::get().uri(&format!(
            "/api/v1/attendance-victims/by-victim/{}",
            victim_b
        )),
        &config,
        &admin_a_token,
    )
    .to_request();

    let get_resp = test::call_service(&app, get_req).await;
    assert_eq!(get_resp.status(), StatusCode::FORBIDDEN);
}
