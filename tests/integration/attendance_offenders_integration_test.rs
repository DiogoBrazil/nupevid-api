use actix_web::{http::StatusCode, test};
use chrono::{NaiveDate, NaiveTime};
use uuid::Uuid;

use crate::common::{db_fixtures, test_helpers};

fn build_attendance_offender_payload(offender_id: Uuid, victim_id: Uuid) -> serde_json::Value {
    serde_json::json!({
        "offender_id": offender_id,
        "victim_id": victim_id,
        "protective_measure_id": serde_json::Value::Null,
        "was_offender_present": true,
        "attendance_date": NaiveDate::from_ymd_opt(2025, 1, 1).unwrap(),
        "attendance_time": NaiveTime::from_hms_opt(10, 0, 0).unwrap(),
        "address": serde_json::Value::Null,
        "is_remote": false,
        "assaults_children": true,
        "violence_aggravator": "AlcoholUse",
        "violence_aggravator_other": serde_json::Value::Null,
        "description": "Atendimento ao agressor",
    })
}

fn build_attendance_offender_payload_with_address(offender_id: Uuid, victim_id: Uuid, city_id: Uuid) -> serde_json::Value {
    serde_json::json!({
        "offender_id": offender_id,
        "victim_id": victim_id,
        "protective_measure_id": serde_json::Value::Null,
        "was_offender_present": true,
        "attendance_date": NaiveDate::from_ymd_opt(2025, 1, 15).unwrap(),
        "attendance_time": NaiveTime::from_hms_opt(14, 30, 0).unwrap(),
        "address": {
            "street": "Rua Atendimento Agressor",
            "number": "200",
            "district": "Centro",
            "city_id": city_id,
            "zip_code": "33333-333",
            "complement": "Delegacia"
        },
        "is_remote": false,
        "assaults_children": false,
        "violence_aggravator": "Other",
        "violence_aggravator_other": "Ciúmes excessivos",
        "description": "Atendimento presencial ao agressor",
    })
}

#[actix_rt::test]
async fn create_attendance_offender_success_for_offender_in_own_city() {
    let pool = test_helpers::setup_test_db().await;
    test_helpers::clean_database(&pool).await;

    let config = test_helpers::build_test_config();
    let app = test_helpers::create_full_test_app(pool.clone(), config.clone()).await;

    let city = db_fixtures::insert_city(&pool, "Cidade A").await;
    let victim_id = db_fixtures::insert_victim(&pool, "Vitima", city).await;
    let offender_id = db_fixtures::insert_offender(&pool, "Agressor", city, victim_id).await;

    // Create user with work session
    let user_id = db_fixtures::insert_user(&pool, "100001", "admin@test.com", "CITY_ADMIN", Some(city)).await;
    test_helpers::create_work_session_for_user(&pool, user_id).await;

    let mut admin_claims = test_helpers::build_city_admin_claims(city);
    admin_claims.id = user_id.to_string();
    let admin_token = test_helpers::generate_jwt(&admin_claims, &config.jwt_secret);

    let payload = build_attendance_offender_payload(offender_id, victim_id);
    let req = test_helpers::with_auth_headers(
        test::TestRequest::post()
            .uri("/api/v1/attendance-offenders")
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
        body["data"]["offender_id"].as_str().unwrap(),
        offender_id.to_string()
    );
    assert_eq!(
        body["data"]["victim_id"].as_str().unwrap(),
        victim_id.to_string()
    );
    assert_eq!(body["data"]["assaults_children"].as_bool().unwrap(), true);
    assert_eq!(body["data"]["violence_aggravator"].as_str().unwrap(), "AlcoholUse");
}

#[actix_rt::test]
async fn create_attendance_offender_with_address() {
    let pool = test_helpers::setup_test_db().await;
    test_helpers::clean_database(&pool).await;

    let config = test_helpers::build_test_config();
    let app = test_helpers::create_full_test_app(pool.clone(), config.clone()).await;

    let city = db_fixtures::insert_city(&pool, "Cidade A").await;
    let victim_id = db_fixtures::insert_victim(&pool, "Vitima", city).await;
    let offender_id = db_fixtures::insert_offender(&pool, "Agressor", city, victim_id).await;

    // Create user with work session
    let user_id = db_fixtures::insert_user(&pool, "100002", "admin2@test.com", "CITY_ADMIN", Some(city)).await;
    test_helpers::create_work_session_for_user(&pool, user_id).await;

    let mut admin_claims = test_helpers::build_city_admin_claims(city);
    admin_claims.id = user_id.to_string();
    let admin_token = test_helpers::generate_jwt(&admin_claims, &config.jwt_secret);

    let payload = build_attendance_offender_payload_with_address(offender_id, victim_id, city);
    let req = test_helpers::with_auth_headers(
        test::TestRequest::post()
            .uri("/api/v1/attendance-offenders")
            .set_json(&payload),
        &config,
        &admin_token,
    )
    .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), StatusCode::CREATED);

    let body: serde_json::Value = test::read_body_json(resp).await;
    assert_eq!(body["status"].as_u64().unwrap(), 201);
    assert!(body["data"]["address"].is_object());
    assert_eq!(body["data"]["address"]["street"].as_str().unwrap(), "Rua Atendimento Agressor");
    assert_eq!(body["data"]["violence_aggravator"].as_str().unwrap(), "Other");
    assert_eq!(body["data"]["violence_aggravator_other"].as_str().unwrap(), "Ciúmes excessivos");
}

#[actix_rt::test]
async fn city_admin_cannot_create_attendance_offender_for_other_city() {
    let pool = test_helpers::setup_test_db().await;
    test_helpers::clean_database(&pool).await;

    let config = test_helpers::build_test_config();
    let app = test_helpers::create_full_test_app(pool.clone(), config.clone()).await;

    let city_a = db_fixtures::insert_city(&pool, "Cidade A").await;
    let city_b = db_fixtures::insert_city(&pool, "Cidade B").await;
    let victim_b = db_fixtures::insert_victim(&pool, "Vitima B", city_b).await;
    let offender_b = db_fixtures::insert_offender(&pool, "Agressor B", city_b, victim_b).await;

    let admin_a_claims = test_helpers::build_city_admin_claims(city_a);
    let admin_a_token = test_helpers::generate_jwt(&admin_a_claims, &config.jwt_secret);

    let payload = build_attendance_offender_payload(offender_b, victim_b);
    let req = test_helpers::with_auth_headers(
        test::TestRequest::post()
            .uri("/api/v1/attendance-offenders")
            .set_json(&payload),
        &config,
        &admin_a_token,
    )
    .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), StatusCode::FORBIDDEN);
}

#[actix_rt::test]
async fn list_attendance_offenders_filtered_by_city() {
    let pool = test_helpers::setup_test_db().await;
    test_helpers::clean_database(&pool).await;

    let config = test_helpers::build_test_config();
    let app = test_helpers::create_full_test_app(pool.clone(), config.clone()).await;

    let city_a = db_fixtures::insert_city(&pool, "Cidade A").await;
    let city_b = db_fixtures::insert_city(&pool, "Cidade B").await;
    let victim_a = db_fixtures::insert_victim(&pool, "Vitima A", city_a).await;
    let victim_b = db_fixtures::insert_victim(&pool, "Vitima B", city_b).await;
    let offender_a = db_fixtures::insert_offender(&pool, "Agressor A", city_a, victim_a).await;
    let offender_b = db_fixtures::insert_offender(&pool, "Agressor B", city_b, victim_b).await;

    // Create ROOT user with work session
    let root_user_id = db_fixtures::insert_user(&pool, "100003", "root@test.com", "ROOT", None).await;
    test_helpers::create_work_session_for_user(&pool, root_user_id).await;

    let mut root_claims = test_helpers::build_root_claims();
    root_claims.id = root_user_id.to_string();
    let root_token = test_helpers::generate_jwt(&root_claims, &config.jwt_secret);

    // Create attendance for both offenders as ROOT
    for (offender_id, victim_id) in [(offender_a, victim_a), (offender_b, victim_b)] {
        let payload = build_attendance_offender_payload(offender_id, victim_id);
        let req = test_helpers::with_auth_headers(
            test::TestRequest::post()
                .uri("/api/v1/attendance-offenders")
                .set_json(&payload),
            &config,
            &root_token,
        )
        .to_request();
        test::call_service(&app, req).await;
    }

    // City A admin should only see their own
    let admin_a_claims = test_helpers::build_city_admin_claims(city_a);
    let admin_a_token = test_helpers::generate_jwt(&admin_a_claims, &config.jwt_secret);

    let req = test_helpers::with_auth_headers(
        test::TestRequest::get().uri("/api/v1/attendance-offenders"),
        &config,
        &admin_a_token,
    )
    .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), StatusCode::OK);

    let body: serde_json::Value = test::read_body_json(resp).await;
    let data = body["data"].as_array().unwrap();
    assert_eq!(data.len(), 1);
    assert_eq!(data[0]["offender_id"].as_str().unwrap(), offender_a.to_string());
}

#[actix_rt::test]
async fn get_attendance_offender_by_id() {
    let pool = test_helpers::setup_test_db().await;
    test_helpers::clean_database(&pool).await;

    let config = test_helpers::build_test_config();
    let app = test_helpers::create_full_test_app(pool.clone(), config.clone()).await;

    let city = db_fixtures::insert_city(&pool, "Cidade A").await;
    let victim_id = db_fixtures::insert_victim(&pool, "Vitima", city).await;
    let offender_id = db_fixtures::insert_offender(&pool, "Agressor", city, victim_id).await;

    // Create user with work session
    let user_id = db_fixtures::insert_user(&pool, "100004", "admin4@test.com", "CITY_ADMIN", Some(city)).await;
    test_helpers::create_work_session_for_user(&pool, user_id).await;

    let mut admin_claims = test_helpers::build_city_admin_claims(city);
    admin_claims.id = user_id.to_string();
    let admin_token = test_helpers::generate_jwt(&admin_claims, &config.jwt_secret);

    let payload = build_attendance_offender_payload(offender_id, victim_id);
    let req = test_helpers::with_auth_headers(
        test::TestRequest::post()
            .uri("/api/v1/attendance-offenders")
            .set_json(&payload),
        &config,
        &admin_token,
    )
    .to_request();

    let resp = test::call_service(&app, req).await;
    let body: serde_json::Value = test::read_body_json(resp).await;
    let attendance_id = body["data"]["id"].as_str().unwrap();

    // Get by ID
    let req = test_helpers::with_auth_headers(
        test::TestRequest::get().uri(&format!("/api/v1/attendance-offenders/{}", attendance_id)),
        &config,
        &admin_token,
    )
    .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), StatusCode::OK);

    let body: serde_json::Value = test::read_body_json(resp).await;
    assert_eq!(body["data"]["id"].as_str().unwrap(), attendance_id);
    assert_eq!(body["data"]["offender_id"].as_str().unwrap(), offender_id.to_string());
}

#[actix_rt::test]
async fn update_attendance_offender() {
    let pool = test_helpers::setup_test_db().await;
    test_helpers::clean_database(&pool).await;

    let config = test_helpers::build_test_config();
    let app = test_helpers::create_full_test_app(pool.clone(), config.clone()).await;

    let city = db_fixtures::insert_city(&pool, "Cidade A").await;
    let victim_id = db_fixtures::insert_victim(&pool, "Vitima", city).await;
    let offender_id = db_fixtures::insert_offender(&pool, "Agressor", city, victim_id).await;

    // Create user with work session
    let user_id = db_fixtures::insert_user(&pool, "100005", "admin5@test.com", "CITY_ADMIN", Some(city)).await;
    test_helpers::create_work_session_for_user(&pool, user_id).await;

    let mut admin_claims = test_helpers::build_city_admin_claims(city);
    admin_claims.id = user_id.to_string();
    let admin_token = test_helpers::generate_jwt(&admin_claims, &config.jwt_secret);

    let payload = build_attendance_offender_payload(offender_id, victim_id);
    let req = test_helpers::with_auth_headers(
        test::TestRequest::post()
            .uri("/api/v1/attendance-offenders")
            .set_json(&payload),
        &config,
        &admin_token,
    )
    .to_request();

    let resp = test::call_service(&app, req).await;
    let body: serde_json::Value = test::read_body_json(resp).await;
    let attendance_id = body["data"]["id"].as_str().unwrap();

    // Update
    let mut update_payload = build_attendance_offender_payload(offender_id, victim_id);
    update_payload["description"] = serde_json::Value::String("Descrição atualizada".to_string());
    update_payload["assaults_children"] = serde_json::Value::Bool(false);

    let req = test_helpers::with_auth_headers(
        test::TestRequest::put()
            .uri(&format!("/api/v1/attendance-offenders/{}", attendance_id))
            .set_json(&update_payload),
        &config,
        &admin_token,
    )
    .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), StatusCode::OK);

    let body: serde_json::Value = test::read_body_json(resp).await;
    assert_eq!(body["data"]["description"].as_str().unwrap(), "Descrição atualizada");
    assert_eq!(body["data"]["assaults_children"].as_bool().unwrap(), false);
}

#[actix_rt::test]
async fn delete_attendance_offender() {
    let pool = test_helpers::setup_test_db().await;
    test_helpers::clean_database(&pool).await;

    let config = test_helpers::build_test_config();
    let app = test_helpers::create_full_test_app(pool.clone(), config.clone()).await;

    let city = db_fixtures::insert_city(&pool, "Cidade A").await;
    let victim_id = db_fixtures::insert_victim(&pool, "Vitima", city).await;
    let offender_id = db_fixtures::insert_offender(&pool, "Agressor", city, victim_id).await;

    // Create user with work session
    let user_id = db_fixtures::insert_user(&pool, "100006", "admin6@test.com", "CITY_ADMIN", Some(city)).await;
    test_helpers::create_work_session_for_user(&pool, user_id).await;

    let mut admin_claims = test_helpers::build_city_admin_claims(city);
    admin_claims.id = user_id.to_string();
    let admin_token = test_helpers::generate_jwt(&admin_claims, &config.jwt_secret);

    let payload = build_attendance_offender_payload(offender_id, victim_id);
    let req = test_helpers::with_auth_headers(
        test::TestRequest::post()
            .uri("/api/v1/attendance-offenders")
            .set_json(&payload),
        &config,
        &admin_token,
    )
    .to_request();

    let resp = test::call_service(&app, req).await;
    let body: serde_json::Value = test::read_body_json(resp).await;
    let attendance_id = body["data"]["id"].as_str().unwrap();

    // Delete
    let req = test_helpers::with_auth_headers(
        test::TestRequest::delete().uri(&format!("/api/v1/attendance-offenders/{}", attendance_id)),
        &config,
        &admin_token,
    )
    .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), StatusCode::OK);

    // Try to get deleted attendance
    let req = test_helpers::with_auth_headers(
        test::TestRequest::get().uri(&format!("/api/v1/attendance-offenders/{}", attendance_id)),
        &config,
        &admin_token,
    )
    .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), StatusCode::NOT_FOUND);
}

#[actix_rt::test]
async fn get_attendance_offenders_by_offender_id() {
    let pool = test_helpers::setup_test_db().await;
    test_helpers::clean_database(&pool).await;

    let config = test_helpers::build_test_config();
    let app = test_helpers::create_full_test_app(pool.clone(), config.clone()).await;

    let city = db_fixtures::insert_city(&pool, "Cidade A").await;
    let victim_id = db_fixtures::insert_victim(&pool, "Vitima", city).await;
    let offender_id = db_fixtures::insert_offender(&pool, "Agressor", city, victim_id).await;

    // Create user with work session
    let user_id = db_fixtures::insert_user(&pool, "100007", "admin7@test.com", "CITY_ADMIN", Some(city)).await;
    test_helpers::create_work_session_for_user(&pool, user_id).await;

    let mut admin_claims = test_helpers::build_city_admin_claims(city);
    admin_claims.id = user_id.to_string();
    let admin_token = test_helpers::generate_jwt(&admin_claims, &config.jwt_secret);

    // Create 2 attendances for the same offender
    for _ in 0..2 {
        let payload = build_attendance_offender_payload(offender_id, victim_id);
        let req = test_helpers::with_auth_headers(
            test::TestRequest::post()
                .uri("/api/v1/attendance-offenders")
                .set_json(&payload),
            &config,
            &admin_token,
        )
        .to_request();
        test::call_service(&app, req).await;
    }

    // Get by offender ID
    let req = test_helpers::with_auth_headers(
        test::TestRequest::get().uri(&format!("/api/v1/attendance-offenders/by-offender/{}", offender_id)),
        &config,
        &admin_token,
    )
    .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), StatusCode::OK);

    let body: serde_json::Value = test::read_body_json(resp).await;
    let data = body["data"].as_array().unwrap();
    assert_eq!(data.len(), 2);
}

#[actix_rt::test]
async fn get_attendance_offenders_by_victim_id() {
    let pool = test_helpers::setup_test_db().await;
    test_helpers::clean_database(&pool).await;

    let config = test_helpers::build_test_config();
    let app = test_helpers::create_full_test_app(pool.clone(), config.clone()).await;

    let city = db_fixtures::insert_city(&pool, "Cidade A").await;
    let victim_id = db_fixtures::insert_victim(&pool, "Vitima", city).await;
    let offender_id = db_fixtures::insert_offender(&pool, "Agressor", city, victim_id).await;

    // Create user with work session
    let user_id = db_fixtures::insert_user(&pool, "100008", "admin8@test.com", "CITY_ADMIN", Some(city)).await;
    test_helpers::create_work_session_for_user(&pool, user_id).await;

    let mut admin_claims = test_helpers::build_city_admin_claims(city);
    admin_claims.id = user_id.to_string();
    let admin_token = test_helpers::generate_jwt(&admin_claims, &config.jwt_secret);

    // Create 3 attendances for the same victim
    for _ in 0..3 {
        let payload = build_attendance_offender_payload(offender_id, victim_id);
        let req = test_helpers::with_auth_headers(
            test::TestRequest::post()
                .uri("/api/v1/attendance-offenders")
                .set_json(&payload),
            &config,
            &admin_token,
        )
        .to_request();
        test::call_service(&app, req).await;
    }

    // Get by victim ID
    let req = test_helpers::with_auth_headers(
        test::TestRequest::get().uri(&format!("/api/v1/attendance-offenders/by-victim/{}", victim_id)),
        &config,
        &admin_token,
    )
    .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), StatusCode::OK);

    let body: serde_json::Value = test::read_body_json(resp).await;
    let data = body["data"].as_array().unwrap();
    assert_eq!(data.len(), 3);
}

#[actix_rt::test]
async fn create_attendance_offender_without_active_session_fails() {
    let pool = test_helpers::setup_test_db().await;
    test_helpers::clean_database(&pool).await;

    let config = test_helpers::build_test_config();
    let app = test_helpers::create_full_test_app(pool.clone(), config.clone()).await;

    let city_id = db_fixtures::insert_city(&pool, "Test City").await;
    let victim_id = db_fixtures::insert_victim(&pool, "Test Victim", city_id).await;
    let offender_id = db_fixtures::insert_offender(&pool, "Test Offender", city_id, victim_id).await;

    // Create CITY_ADMIN user WITHOUT creating work session (this is the key difference)
    // CITY_ADMIN has the necessary policies, so we can test the work session requirement
    let user_id = db_fixtures::insert_user(&pool, "100199", "admin@test.com", "CITY_ADMIN", Some(city_id)).await;

    // DO NOT create work session here - that's the whole point of this test
    // test_helpers::create_work_session_for_user(&pool, user_id).await; // ← OMITTED

    let mut claims = test_helpers::build_city_admin_claims(city_id);
    claims.id = user_id.to_string();
    let token = test_helpers::generate_jwt(&claims, &config.jwt_secret);

    let payload = build_attendance_offender_payload(offender_id, victim_id);
    let req = test_helpers::with_auth_headers(
        test::TestRequest::post()
            .uri("/api/v1/attendance-offenders")
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
