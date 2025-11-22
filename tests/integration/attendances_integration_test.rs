use actix_web::{test, http::StatusCode};
use chrono::{NaiveDate, NaiveTime};
use uuid::Uuid;

use crate::common::{test_helpers, db_fixtures};

fn build_attendance_payload(victim_id: Uuid) -> serde_json::Value {
    serde_json::json!({
        "victim_id": victim_id,
        "was_victim_present": true,
        "attendance_date": NaiveDate::from_ymd_opt(2025, 1, 1).unwrap(),
        "attendance_time": NaiveTime::from_hms_opt(10, 0, 0).unwrap(),
        "description": "Atendimento realizado",
        "latitude": serde_json::Value::Null,
        "longitude": serde_json::Value::Null,
    })
}

fn build_attendance_address_payload(attendance_id: &str) -> serde_json::Value {
    serde_json::json!({
        "attendance_id": attendance_id,
        "street": "Rua Atendimento",
        "number": "100",
        "district": "Centro",
        "city_name": "Cidade A",
        "state": "SP",
        "zip_code": "22222-222",
        "complement": "Casa"
    })
}

#[actix_rt::test]
async fn create_attendance_success_for_victim_in_own_city() {
    let pool = test_helpers::setup_test_db().await;
    test_helpers::clean_database(&pool).await;

    let config = test_helpers::build_test_config();
    let app = test_helpers::create_full_test_app(pool.clone(), config.clone()).await;

    let city = db_fixtures::insert_city(&pool, "Cidade A").await;
    let victim_id = db_fixtures::insert_victim(&pool, "Vitima", city).await;

    let admin_claims = test_helpers::build_city_admin_claims(city);
    let admin_token = test_helpers::generate_jwt(&admin_claims, &config.jwt_secret);

    let payload = build_attendance_payload(victim_id);
    let req = test_helpers::with_auth_headers(
        test::TestRequest::post().uri("/api/v1/attendances").set_json(&payload),
        &config,
        &admin_token,
    )
    .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), StatusCode::CREATED);

    let body: serde_json::Value = test::read_body_json(resp).await;
    assert_eq!(body["status"].as_u64().unwrap(), 201);
    assert_eq!(body["data"]["victim_id"].as_str().unwrap(), victim_id.to_string());
}

#[actix_rt::test]
async fn city_admin_cannot_create_attendance_for_other_city_victim() {
    let pool = test_helpers::setup_test_db().await;
    test_helpers::clean_database(&pool).await;

    let config = test_helpers::build_test_config();
    let app = test_helpers::create_full_test_app(pool.clone(), config.clone()).await;

    let city_a = db_fixtures::insert_city(&pool, "Cidade A").await;
    let city_b = db_fixtures::insert_city(&pool, "Cidade B").await;
    let victim_b = db_fixtures::insert_victim(&pool, "Vitima B", city_b).await;

    let admin_a_claims = test_helpers::build_city_admin_claims(city_a);
    let admin_a_token = test_helpers::generate_jwt(&admin_a_claims, &config.jwt_secret);

    let payload = build_attendance_payload(victim_b);
    let req = test_helpers::with_auth_headers(
        test::TestRequest::post().uri("/api/v1/attendances").set_json(&payload),
        &config,
        &admin_a_token,
    )
    .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), StatusCode::FORBIDDEN);
}

#[actix_rt::test]
async fn list_attendances_filtered_by_city() {
    let pool = test_helpers::setup_test_db().await;
    test_helpers::clean_database(&pool).await;

    let config = test_helpers::build_test_config();
    let app = test_helpers::create_full_test_app(pool.clone(), config.clone()).await;

    let city_a = db_fixtures::insert_city(&pool, "Cidade A").await;
    let city_b = db_fixtures::insert_city(&pool, "Cidade B").await;
    let victim_a = db_fixtures::insert_victim(&pool, "Vitima A", city_a).await;
    let victim_b = db_fixtures::insert_victim(&pool, "Vitima B", city_b).await;

    let root_claims = test_helpers::build_root_claims();
    let root_token = test_helpers::generate_jwt(&root_claims, &config.jwt_secret);

    // Create attendance for both victims as ROOT
    for victim_id in [victim_a, victim_b] {
        let payload = build_attendance_payload(victim_id);
        let req = test_helpers::with_auth_headers(
            test::TestRequest::post().uri("/api/v1/attendances").set_json(&payload),
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
        test::TestRequest::get().uri("/api/v1/attendances"),
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

#[actix_rt::test]
async fn delete_attendance_soft_delete_and_not_listed() {
    let pool = test_helpers::setup_test_db().await;
    test_helpers::clean_database(&pool).await;

    let config = test_helpers::build_test_config();
    let app = test_helpers::create_full_test_app(pool.clone(), config.clone()).await;

    let city = db_fixtures::insert_city(&pool, "Cidade Del").await;
    let victim_id = db_fixtures::insert_victim(&pool, "Vitima", city).await;

    let admin_claims = test_helpers::build_city_admin_claims(city);
    let admin_token = test_helpers::generate_jwt(&admin_claims, &config.jwt_secret);

    let payload = build_attendance_payload(victim_id);
    let create_req = test_helpers::with_auth_headers(
        test::TestRequest::post().uri("/api/v1/attendances").set_json(&payload),
        &config,
        &admin_token,
    )
    .to_request();
    let create_resp = test::call_service(&app, create_req).await;
    assert_eq!(create_resp.status(), StatusCode::CREATED);
    let body: serde_json::Value = test::read_body_json(create_resp).await;
    let attendance_id = body["data"]["id"].as_str().unwrap();

    let delete_req = test_helpers::with_auth_headers(
        test::TestRequest::delete().uri(&format!("/api/v1/attendances/{}", attendance_id)),
        &config,
        &admin_token,
    )
    .to_request();
    let delete_resp = test::call_service(&app, delete_req).await;
    assert_eq!(delete_resp.status(), StatusCode::OK);

    // GET by id -> 404
    let get_req = test_helpers::with_auth_headers(
        test::TestRequest::get().uri(&format!("/api/v1/attendances/{}", attendance_id)),
        &config,
        &admin_token,
    )
    .to_request();
    let get_resp = test::call_service(&app, get_req).await;
    assert_eq!(get_resp.status(), StatusCode::NOT_FOUND);

    // List -> empty
    let list_req = test_helpers::with_auth_headers(
        test::TestRequest::get().uri("/api/v1/attendances"),
        &config,
        &admin_token,
    )
    .to_request();
    let list_resp = test::call_service(&app, list_req).await;
    assert_eq!(list_resp.status(), StatusCode::OK);
    let list_body: serde_json::Value = test::read_body_json(list_resp).await;
    let atts = list_body["data"].as_array().unwrap();
    assert!(atts.iter().all(|a| a["id"].as_str().unwrap() != attendance_id));
}

#[actix_rt::test]
async fn get_attendance_by_id_for_other_city_returns_forbidden() {
    let pool = test_helpers::setup_test_db().await;
    test_helpers::clean_database(&pool).await;

    let config = test_helpers::build_test_config();
    let app = test_helpers::create_full_test_app(pool.clone(), config.clone()).await;

    let city_a = db_fixtures::insert_city(&pool, "Cidade A").await;
    let city_b = db_fixtures::insert_city(&pool, "Cidade B").await;

    let root_claims = test_helpers::build_root_claims();
    let root_token = test_helpers::generate_jwt(&root_claims, &config.jwt_secret);

    let victim_b = db_fixtures::insert_victim(&pool, "Vitima B", city_b).await;
    let payload = build_attendance_payload(victim_b);
    let create_req = test_helpers::with_auth_headers(
        test::TestRequest::post().uri("/api/v1/attendances").set_json(&payload),
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
        test::TestRequest::get().uri(&format!("/api/v1/attendances/{}", attendance_id)),
        &config,
        &admin_a_token,
    )
    .to_request();
    let get_resp = test::call_service(&app, get_req).await;
    assert_eq!(get_resp.status(), StatusCode::FORBIDDEN);
}

#[actix_rt::test]
async fn attendance_addresses_respect_city_access() {
    let pool = test_helpers::setup_test_db().await;
    test_helpers::clean_database(&pool).await;

    let config = test_helpers::build_test_config();
    let app = test_helpers::create_full_test_app(pool.clone(), config.clone()).await;

    let city_a = db_fixtures::insert_city(&pool, "Cidade A").await;
    let city_b = db_fixtures::insert_city(&pool, "Cidade B").await;

    let root_claims = test_helpers::build_root_claims();
    let root_token = test_helpers::generate_jwt(&root_claims, &config.jwt_secret);

    // Create victim in B and attendance as ROOT
    let victim_b = db_fixtures::insert_victim(&pool, "Vitima B", city_b).await;
    let att_payload = build_attendance_payload(victim_b);
    let create_att_req = test_helpers::with_auth_headers(
        test::TestRequest::post().uri("/api/v1/attendances").set_json(&att_payload),
        &config,
        &root_token,
    )
    .to_request();
    let create_att_resp = test::call_service(&app, create_att_req).await;
    assert_eq!(create_att_resp.status(), StatusCode::CREATED);
    let att_body: serde_json::Value = test::read_body_json(create_att_resp).await;
    let attendance_id = att_body["data"]["id"].as_str().unwrap();

    // CITY_ADMIN A
    let admin_a_claims = test_helpers::build_city_admin_claims(city_a);
    let admin_a_token = test_helpers::generate_jwt(&admin_a_claims, &config.jwt_secret);

    // CITY_ADMIN A cannot create address for attendance in city B
    let addr_payload = build_attendance_address_payload(attendance_id);
    let create_addr_req = test_helpers::with_auth_headers(
        test::TestRequest::post().uri("/api/v1/attendance-addresses").set_json(&addr_payload),
        &config,
        &admin_a_token,
    )
    .to_request();
    let create_addr_resp = test::call_service(&app, create_addr_req).await;
    assert_eq!(create_addr_resp.status(), StatusCode::FORBIDDEN);
}