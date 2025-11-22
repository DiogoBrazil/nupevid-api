use actix_web::{test, http::StatusCode};
use chrono::{NaiveDate};
use uuid::Uuid;

use crate::common::{test_helpers, db_fixtures};

fn build_measure_payload(victim_id: Uuid, is_active: bool) -> serde_json::Value {
    serde_json::json!({
        "process_number": "12345-67.2025.8.26.0000",
        "issued_at": NaiveDate::from_ymd_opt(2025, 1, 1).unwrap(),
        "judicial_authority": "Juiz A",
        "court_district": "Comarca X",
        "is_active": is_active,
        "victim_id": victim_id,
    })
}

#[actix_rt::test]
async fn create_protective_measure_success_for_victim_in_own_city() {
    let pool = test_helpers::setup_test_db().await;
    test_helpers::clean_database(&pool).await;

    let config = test_helpers::build_test_config();
    let app = test_helpers::create_full_test_app(pool.clone(), config.clone()).await;

    let city = db_fixtures::insert_city(&pool, "Cidade Medida").await;
    let victim_id = db_fixtures::insert_victim(&pool, "Vitima", city).await;

    let admin_claims = test_helpers::build_city_admin_claims(city);
    let admin_token = test_helpers::generate_jwt(&admin_claims, &config.jwt_secret);

    let payload = build_measure_payload(victim_id, true);

    let req = test_helpers::with_auth_headers(
        test::TestRequest::post()
            .uri("/api/v1/protective-measures")
            .set_json(&payload),
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
async fn cannot_create_second_active_measure_for_same_victim() {
    let pool = test_helpers::setup_test_db().await;
    test_helpers::clean_database(&pool).await;

    let config = test_helpers::build_test_config();
    let app = test_helpers::create_full_test_app(pool.clone(), config.clone()).await;

    let city = db_fixtures::insert_city(&pool, "Cidade Regra").await;
    let victim_id = db_fixtures::insert_victim(&pool, "Vitima", city).await;

    let admin_claims = test_helpers::build_city_admin_claims(city);
    let admin_token = test_helpers::generate_jwt(&admin_claims, &config.jwt_secret);

    let payload = build_measure_payload(victim_id, true);

    // First active measure
    let req1 = test_helpers::with_auth_headers(
        test::TestRequest::post()
            .uri("/api/v1/protective-measures")
            .set_json(&payload),
        &config,
        &admin_token,
    )
    .to_request();
    let resp1 = test::call_service(&app, req1).await;
    assert_eq!(resp1.status(), StatusCode::CREATED);

    // Second active measure should fail
    let req2 = test_helpers::with_auth_headers(
        test::TestRequest::post()
            .uri("/api/v1/protective-measures")
            .set_json(&payload),
        &config,
        &admin_token,
    )
    .to_request();
    let resp2 = test::call_service(&app, req2).await;
    assert_eq!(resp2.status(), StatusCode::BAD_REQUEST);

    let body: serde_json::Value = test::read_body_json(resp2).await;
    assert_eq!(body["status_code"].as_u64().unwrap(), 400);
    assert!(body["message"].as_str().unwrap().contains("already has an active protective measure"));
}

#[actix_rt::test]
async fn city_admin_cannot_create_measure_for_victim_in_other_city() {
    let pool = test_helpers::setup_test_db().await;
    test_helpers::clean_database(&pool).await;

    let config = test_helpers::build_test_config();
    let app = test_helpers::create_full_test_app(pool.clone(), config.clone()).await;

    let city_a = db_fixtures::insert_city(&pool, "Cidade A").await;
    let city_b = db_fixtures::insert_city(&pool, "Cidade B").await;
    let victim_in_b = db_fixtures::insert_victim(&pool, "Vitima B", city_b).await;

    let admin_a_claims = test_helpers::build_city_admin_claims(city_a);
    let admin_a_token = test_helpers::generate_jwt(&admin_a_claims, &config.jwt_secret);

    let payload = build_measure_payload(victim_in_b, true);

    let req = test_helpers::with_auth_headers(
        test::TestRequest::post()
            .uri("/api/v1/protective-measures")
            .set_json(&payload),
        &config,
        &admin_a_token,
    )
    .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), StatusCode::FORBIDDEN);
}

#[actix_rt::test]
async fn list_measures_by_victim_success() {
    let pool = test_helpers::setup_test_db().await;
    test_helpers::clean_database(&pool).await;

    let config = test_helpers::build_test_config();
    let app = test_helpers::create_full_test_app(pool.clone(), config.clone()).await;

    let city = db_fixtures::insert_city(&pool, "Cidade Lista").await;
    let victim_id = db_fixtures::insert_victim(&pool, "Vitima", city).await;

    let admin_claims = test_helpers::build_city_admin_claims(city);
    let admin_token = test_helpers::generate_jwt(&admin_claims, &config.jwt_secret);

    let payload = build_measure_payload(victim_id, true);
    let create_req = test_helpers::with_auth_headers(
        test::TestRequest::post()
            .uri("/api/v1/protective-measures")
            .set_json(&payload),
        &config,
        &admin_token,
    )
    .to_request();
    let create_resp = test::call_service(&app, create_req).await;
    assert_eq!(create_resp.status(), StatusCode::CREATED);

    let list_req = test_helpers::with_auth_headers(
        test::TestRequest::get()
            .uri(&format!("/api/v1/protective-measures/victim/{}", victim_id)),
        &config,
        &admin_token,
    )
    .to_request();

    let list_resp = test::call_service(&app, list_req).await;
    assert_eq!(list_resp.status(), StatusCode::OK);
    let body: serde_json::Value = test::read_body_json(list_resp).await;
    assert_eq!(body["data"].as_array().unwrap().len(), 1);
}

#[actix_rt::test]
async fn create_protective_measure_with_nonexistent_victim_returns_404() {
    let pool = test_helpers::setup_test_db().await;
    test_helpers::clean_database(&pool).await;

    let config = test_helpers::build_test_config();
    let app = test_helpers::create_full_test_app(pool.clone(), config.clone()).await;

    let city = db_fixtures::insert_city(&pool, "Cidade NF").await;

    let admin_claims = test_helpers::build_city_admin_claims(city);
    let admin_token = test_helpers::generate_jwt(&admin_claims, &config.jwt_secret);

    // Random victim id that does not exist
    let random_victim_id = Uuid::new_v4();
    let payload = build_measure_payload(random_victim_id, true);

    let req = test_helpers::with_auth_headers(
        test::TestRequest::post()
            .uri("/api/v1/protective-measures")
            .set_json(&payload),
        &config,
        &admin_token,
    )
    .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), StatusCode::NOT_FOUND);
}

#[actix_rt::test]
async fn delete_protective_measure_soft_delete() {
    let pool = test_helpers::setup_test_db().await;
    test_helpers::clean_database(&pool).await;

    let config = test_helpers::build_test_config();
    let app = test_helpers::create_full_test_app(pool.clone(), config.clone()).await;

    let city = db_fixtures::insert_city(&pool, "Cidade Del").await;
    let victim_id = db_fixtures::insert_victim(&pool, "Vitima", city).await;

    let admin_claims = test_helpers::build_city_admin_claims(city);
    let admin_token = test_helpers::generate_jwt(&admin_claims, &config.jwt_secret);

    let payload = build_measure_payload(victim_id, true);
    let create_req = test_helpers::with_auth_headers(
        test::TestRequest::post()
            .uri("/api/v1/protective-measures")
            .set_json(&payload),
        &config,
        &admin_token,
    )
    .to_request();
    let create_resp = test::call_service(&app, create_req).await;
    assert_eq!(create_resp.status(), StatusCode::CREATED);
    let body: serde_json::Value = test::read_body_json(create_resp).await;
    let measure_id = body["data"]["id"].as_str().unwrap();

    let delete_req = test_helpers::with_auth_headers(
        test::TestRequest::delete()
            .uri(&format!("/api/v1/protective-measures/{}", measure_id)),
        &config,
        &admin_token,
    )
    .to_request();
    let delete_resp = test::call_service(&app, delete_req).await;
    assert_eq!(delete_resp.status(), StatusCode::OK);

    // GET by id should be 404
    let get_req = test_helpers::with_auth_headers(
        test::TestRequest::get()
            .uri(&format!("/api/v1/protective-measures/{}", measure_id)),
        &config,
        &admin_token,
    )
    .to_request();
    let get_resp = test::call_service(&app, get_req).await;
    assert_eq!(get_resp.status(), StatusCode::NOT_FOUND);
}