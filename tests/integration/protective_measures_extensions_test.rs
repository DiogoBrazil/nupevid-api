use actix_web::{test, http::StatusCode};
use chrono::NaiveDate;
use uuid::Uuid;

use crate::common::{test_helpers, db_fixtures};

fn build_measure_payload(victim_id: Uuid, court_district_id: Uuid, offender_id: Uuid, is_active: bool) -> serde_json::Value {
    serde_json::json!({
        "process_number": "12345-67.2025.8.26.0000",
        "issued_at": NaiveDate::from_ymd_opt(2025, 1, 1).unwrap(),
        "judicial_authority": "Juiz A",
        "court_district_id": court_district_id,
        "is_active": is_active,
        "victim_id": victim_id,
        "offender_id": offender_id,
    })
}

fn build_extension_payload(number: i32) -> serde_json::Value {
    serde_json::json!({
        "extension_number": number,
        "extension_date": NaiveDate::from_ymd_opt(2025, 6, 1).unwrap(),
        "new_valid_until": NaiveDate::from_ymd_opt(2025, 12, 1).unwrap(),
        "notes": "Prorrogação solicitada"
    })
}

#[actix_rt::test]
async fn test_protective_measure_includes_extensions() {
    let pool = test_helpers::setup_test_db().await;
    test_helpers::clean_database(&pool).await;

    let config = test_helpers::build_test_config();
    let app = test_helpers::create_full_test_app(pool.clone(), config.clone()).await;

    // 1. Setup Data
    let city = db_fixtures::insert_city(&pool, "Cidade Ext").await;
    let victim_id = db_fixtures::insert_victim(&pool, "Vitima Ext", city).await;
    let offender_id = db_fixtures::insert_offender(&pool, "Agressor Ext", city, victim_id).await;

    let admin_claims = test_helpers::build_city_admin_claims(city);
    let admin_token = test_helpers::generate_jwt(&admin_claims, &config.jwt_secret);

    // 2. Create Protective Measure
    let pm_payload = build_measure_payload(victim_id, city, offender_id, true);
    let create_pm_req = test_helpers::with_auth_headers(
        test::TestRequest::post()
            .uri("/api/v1/protective-measures")
            .set_json(&pm_payload),
        &config,
        &admin_token,
    )
    .to_request();
    let create_pm_resp = test::call_service(&app, create_pm_req).await;
    assert_eq!(create_pm_resp.status(), StatusCode::CREATED);
    
    let pm_body: serde_json::Value = test::read_body_json(create_pm_resp).await;
    let pm_id = pm_body["data"]["id"].as_str().unwrap();

    // 3. Create Extension
    let ext_payload = build_extension_payload(1);
    let create_ext_req = test_helpers::with_auth_headers(
        test::TestRequest::post()
            .uri(&format!("/api/v1/protective-measures/{}/extensions", pm_id))
            .set_json(&ext_payload),
        &config,
        &admin_token,
    )
    .to_request();
    let create_ext_resp = test::call_service(&app, create_ext_req).await;
    assert_eq!(create_ext_resp.status(), StatusCode::CREATED);

    // 4. Get Measure By ID - Should include extensions
    let get_pm_req = test_helpers::with_auth_headers(
        test::TestRequest::get()
            .uri(&format!("/api/v1/protective-measures/{}", pm_id)),
        &config,
        &admin_token,
    )
    .to_request();
    let get_pm_resp = test::call_service(&app, get_pm_req).await;
    assert_eq!(get_pm_resp.status(), StatusCode::OK);
    
    let get_pm_body: serde_json::Value = test::read_body_json(get_pm_resp).await;
    let measure = &get_pm_body["data"];
    
    assert!(measure.get("extensions").is_some(), "Response should have 'extensions' field");
    let extensions = measure["extensions"].as_array().unwrap();
    assert_eq!(extensions.len(), 1, "Should have 1 extension");
    assert_eq!(extensions[0]["extension_number"].as_i64().unwrap(), 1);

    // 5. List Measures - Should include extensions
    let list_req = test_helpers::with_auth_headers(
        test::TestRequest::get()
            .uri("/api/v1/protective-measures"),
        &config,
        &admin_token,
    )
    .to_request();
    let list_resp = test::call_service(&app, list_req).await;
    assert_eq!(list_resp.status(), StatusCode::OK);
    
    let list_body: serde_json::Value = test::read_body_json(list_resp).await;
    let list = list_body["data"].as_array().unwrap();
    assert_eq!(list.len(), 1);
    
    assert!(list[0].get("extensions").is_some());
    assert_eq!(list[0]["extensions"].as_array().unwrap().len(), 1);

    // 6. List by Victim - Should include extensions
    let victim_list_req = test_helpers::with_auth_headers(
        test::TestRequest::get()
            .uri(&format!("/api/v1/protective-measures/victim/{}", victim_id)),
        &config,
        &admin_token,
    )
    .to_request();
    let victim_list_resp = test::call_service(&app, victim_list_req).await;
    assert_eq!(victim_list_resp.status(), StatusCode::OK);
    
    let victim_list_body: serde_json::Value = test::read_body_json(victim_list_resp).await;
    let v_list = victim_list_body["data"].as_array().unwrap();
    assert_eq!(v_list.len(), 1);
    
    assert!(v_list[0].get("extensions").is_some());
    assert_eq!(v_list[0]["extensions"].as_array().unwrap().len(), 1);
}

// ==================== CREATE EXTENSION POLICY TESTS ====================

#[actix_rt::test]
async fn test_city_admin_can_create_extension_in_own_city() {
    let pool = test_helpers::setup_test_db().await;
    test_helpers::clean_database(&pool).await;

    let config = test_helpers::build_test_config();
    let app = test_helpers::create_full_test_app(pool.clone(), config.clone()).await;

    let city = db_fixtures::insert_city(&pool, "City A").await;
    let victim_id = db_fixtures::insert_victim(&pool, "Victim A", city).await;
    let offender_id = db_fixtures::insert_offender(&pool, "Offender A", city, victim_id).await;

    let admin_claims = test_helpers::build_city_admin_claims(city);
    let admin_token = test_helpers::generate_jwt(&admin_claims, &config.jwt_secret);

    // Create protective measure
    let pm_payload = build_measure_payload(victim_id, city, offender_id, true);
    let create_pm_req = test_helpers::with_auth_headers(
        test::TestRequest::post()
            .uri("/api/v1/protective-measures")
            .set_json(&pm_payload),
        &config,
        &admin_token,
    )
    .to_request();
    let create_pm_resp = test::call_service(&app, create_pm_req).await;
    assert_eq!(create_pm_resp.status(), StatusCode::CREATED);

    let pm_body: serde_json::Value = test::read_body_json(create_pm_resp).await;
    let pm_id = pm_body["data"]["id"].as_str().unwrap();

    // Create extension - should succeed
    let ext_payload = build_extension_payload(1);
    let create_ext_req = test_helpers::with_auth_headers(
        test::TestRequest::post()
            .uri(&format!("/api/v1/protective-measures/{}/extensions", pm_id))
            .set_json(&ext_payload),
        &config,
        &admin_token,
    )
    .to_request();
    let create_ext_resp = test::call_service(&app, create_ext_req).await;
    assert_eq!(create_ext_resp.status(), StatusCode::CREATED);
}

#[actix_rt::test]
async fn test_city_admin_cannot_create_extension_in_other_city() {
    let pool = test_helpers::setup_test_db().await;
    test_helpers::clean_database(&pool).await;

    let config = test_helpers::build_test_config();
    let app = test_helpers::create_full_test_app(pool.clone(), config.clone()).await;

    let city_a = db_fixtures::insert_city(&pool, "City A").await;
    let city_b = db_fixtures::insert_city(&pool, "City B").await;
    let victim_b = db_fixtures::insert_victim(&pool, "Victim B", city_b).await;
    let offender_b = db_fixtures::insert_offender(&pool, "Offender B", city_b, victim_b).await;

    let admin_b_claims = test_helpers::build_city_admin_claims(city_b);
    let admin_b_token = test_helpers::generate_jwt(&admin_b_claims, &config.jwt_secret);

    // Create protective measure in city B
    let pm_payload = build_measure_payload(victim_b, city_b, offender_b, true);
    let create_pm_req = test_helpers::with_auth_headers(
        test::TestRequest::post()
            .uri("/api/v1/protective-measures")
            .set_json(&pm_payload),
        &config,
        &admin_b_token,
    )
    .to_request();
    let create_pm_resp = test::call_service(&app, create_pm_req).await;
    assert_eq!(create_pm_resp.status(), StatusCode::CREATED);

    let pm_body: serde_json::Value = test::read_body_json(create_pm_resp).await;
    let pm_id = pm_body["data"]["id"].as_str().unwrap();

    // Try to create extension as admin from city A - should fail
    let admin_a_claims = test_helpers::build_city_admin_claims(city_a);
    let admin_a_token = test_helpers::generate_jwt(&admin_a_claims, &config.jwt_secret);

    let ext_payload = build_extension_payload(1);
    let create_ext_req = test_helpers::with_auth_headers(
        test::TestRequest::post()
            .uri(&format!("/api/v1/protective-measures/{}/extensions", pm_id))
            .set_json(&ext_payload),
        &config,
        &admin_a_token,
    )
    .to_request();
    let create_ext_resp = test::call_service(&app, create_ext_req).await;
    assert_eq!(create_ext_resp.status(), StatusCode::FORBIDDEN);
}

#[actix_rt::test]
async fn test_city_user_cannot_create_extension_without_permission() {
    let pool = test_helpers::setup_test_db().await;
    test_helpers::clean_database(&pool).await;

    let config = test_helpers::build_test_config();
    let app = test_helpers::create_full_test_app(pool.clone(), config.clone()).await;

    let city = db_fixtures::insert_city(&pool, "City Test").await;
    let victim_id = db_fixtures::insert_victim(&pool, "Victim Test", city).await;
    let offender_id = db_fixtures::insert_offender(&pool, "Offender Test", city, victim_id).await;

    let admin_claims = test_helpers::build_city_admin_claims(city);
    let admin_token = test_helpers::generate_jwt(&admin_claims, &config.jwt_secret);

    // Create protective measure
    let pm_payload = build_measure_payload(victim_id, city, offender_id, true);
    let create_pm_req = test_helpers::with_auth_headers(
        test::TestRequest::post()
            .uri("/api/v1/protective-measures")
            .set_json(&pm_payload),
        &config,
        &admin_token,
    )
    .to_request();
    let create_pm_resp = test::call_service(&app, create_pm_req).await;
    assert_eq!(create_pm_resp.status(), StatusCode::CREATED);

    let pm_body: serde_json::Value = test::read_body_json(create_pm_resp).await;
    let pm_id = pm_body["data"]["id"].as_str().unwrap();

    // Try to create extension as CITY_USER - should fail
    let user_claims = test_helpers::build_city_user_claims(city);
    let user_token = test_helpers::generate_jwt(&user_claims, &config.jwt_secret);

    let ext_payload = build_extension_payload(1);
    let create_ext_req = test_helpers::with_auth_headers(
        test::TestRequest::post()
            .uri(&format!("/api/v1/protective-measures/{}/extensions", pm_id))
            .set_json(&ext_payload),
        &config,
        &user_token,
    )
    .to_request();
    let create_ext_resp = test::call_service(&app, create_ext_req).await;
    assert_eq!(create_ext_resp.status(), StatusCode::FORBIDDEN);
}

// ==================== READ EXTENSION POLICY TESTS ====================

#[actix_rt::test]
async fn test_city_user_can_read_extension_in_own_city() {
    let pool = test_helpers::setup_test_db().await;
    test_helpers::clean_database(&pool).await;

    let config = test_helpers::build_test_config();
    let app = test_helpers::create_full_test_app(pool.clone(), config.clone()).await;

    let city = db_fixtures::insert_city(&pool, "City Read").await;
    let victim_id = db_fixtures::insert_victim(&pool, "Victim Read", city).await;
    let offender_id = db_fixtures::insert_offender(&pool, "Offender Read", city, victim_id).await;

    let admin_claims = test_helpers::build_city_admin_claims(city);
    let admin_token = test_helpers::generate_jwt(&admin_claims, &config.jwt_secret);

    // Create protective measure and extension
    let pm_payload = build_measure_payload(victim_id, city, offender_id, true);
    let create_pm_req = test_helpers::with_auth_headers(
        test::TestRequest::post()
            .uri("/api/v1/protective-measures")
            .set_json(&pm_payload),
        &config,
        &admin_token,
    )
    .to_request();
    let create_pm_resp = test::call_service(&app, create_pm_req).await;
    let pm_body: serde_json::Value = test::read_body_json(create_pm_resp).await;
    let pm_id = pm_body["data"]["id"].as_str().unwrap();

    let ext_payload = build_extension_payload(1);
    let create_ext_req = test_helpers::with_auth_headers(
        test::TestRequest::post()
            .uri(&format!("/api/v1/protective-measures/{}/extensions", pm_id))
            .set_json(&ext_payload),
        &config,
        &admin_token,
    )
    .to_request();
    let create_ext_resp = test::call_service(&app, create_ext_req).await;
    let ext_body: serde_json::Value = test::read_body_json(create_ext_resp).await;
    let ext_id = ext_body["data"]["id"].as_str().unwrap();

    // CITY_USER should be able to read extension
    let user_claims = test_helpers::build_city_user_claims(city);
    let user_token = test_helpers::generate_jwt(&user_claims, &config.jwt_secret);

    let get_ext_req = test_helpers::with_auth_headers(
        test::TestRequest::get()
            .uri(&format!("/api/v1/protective-measures/{}/extensions/{}", pm_id, ext_id)),
        &config,
        &user_token,
    )
    .to_request();
    let get_ext_resp = test::call_service(&app, get_ext_req).await;
    assert_eq!(get_ext_resp.status(), StatusCode::OK);
}

#[actix_rt::test]
async fn test_city_user_cannot_read_extension_from_other_city() {
    let pool = test_helpers::setup_test_db().await;
    test_helpers::clean_database(&pool).await;

    let config = test_helpers::build_test_config();
    let app = test_helpers::create_full_test_app(pool.clone(), config.clone()).await;

    let city_a = db_fixtures::insert_city(&pool, "City A Read").await;
    let city_b = db_fixtures::insert_city(&pool, "City B Read").await;
    let victim_b = db_fixtures::insert_victim(&pool, "Victim B", city_b).await;
    let offender_b = db_fixtures::insert_offender(&pool, "Offender B", city_b, victim_b).await;

    let admin_b_claims = test_helpers::build_city_admin_claims(city_b);
    let admin_b_token = test_helpers::generate_jwt(&admin_b_claims, &config.jwt_secret);

    // Create protective measure and extension in city B
    let pm_payload = build_measure_payload(victim_b, city_b, offender_b, true);
    let create_pm_req = test_helpers::with_auth_headers(
        test::TestRequest::post()
            .uri("/api/v1/protective-measures")
            .set_json(&pm_payload),
        &config,
        &admin_b_token,
    )
    .to_request();
    let create_pm_resp = test::call_service(&app, create_pm_req).await;
    let pm_body: serde_json::Value = test::read_body_json(create_pm_resp).await;
    let pm_id = pm_body["data"]["id"].as_str().unwrap();

    let ext_payload = build_extension_payload(1);
    let create_ext_req = test_helpers::with_auth_headers(
        test::TestRequest::post()
            .uri(&format!("/api/v1/protective-measures/{}/extensions", pm_id))
            .set_json(&ext_payload),
        &config,
        &admin_b_token,
    )
    .to_request();
    let create_ext_resp = test::call_service(&app, create_ext_req).await;
    let ext_body: serde_json::Value = test::read_body_json(create_ext_resp).await;
    let ext_id = ext_body["data"]["id"].as_str().unwrap();

    // CITY_USER from city A should NOT be able to read extension from city B
    let user_a_claims = test_helpers::build_city_user_claims(city_a);
    let user_a_token = test_helpers::generate_jwt(&user_a_claims, &config.jwt_secret);

    let get_ext_req = test_helpers::with_auth_headers(
        test::TestRequest::get()
            .uri(&format!("/api/v1/protective-measures/{}/extensions/{}", pm_id, ext_id)),
        &config,
        &user_a_token,
    )
    .to_request();
    let get_ext_resp = test::call_service(&app, get_ext_req).await;
    assert_eq!(get_ext_resp.status(), StatusCode::FORBIDDEN);
}

#[actix_rt::test]
async fn test_city_user_can_list_extensions_in_own_city() {
    let pool = test_helpers::setup_test_db().await;
    test_helpers::clean_database(&pool).await;

    let config = test_helpers::build_test_config();
    let app = test_helpers::create_full_test_app(pool.clone(), config.clone()).await;

    let city = db_fixtures::insert_city(&pool, "City List").await;
    let victim_id = db_fixtures::insert_victim(&pool, "Victim List", city).await;
    let offender_id = db_fixtures::insert_offender(&pool, "Offender List", city, victim_id).await;

    let admin_claims = test_helpers::build_city_admin_claims(city);
    let admin_token = test_helpers::generate_jwt(&admin_claims, &config.jwt_secret);

    // Create protective measure and extension
    let pm_payload = build_measure_payload(victim_id, city, offender_id, true);
    let create_pm_req = test_helpers::with_auth_headers(
        test::TestRequest::post()
            .uri("/api/v1/protective-measures")
            .set_json(&pm_payload),
        &config,
        &admin_token,
    )
    .to_request();
    let create_pm_resp = test::call_service(&app, create_pm_req).await;
    let pm_body: serde_json::Value = test::read_body_json(create_pm_resp).await;
    let pm_id = pm_body["data"]["id"].as_str().unwrap();

    let ext_payload = build_extension_payload(1);
    let create_ext_req = test_helpers::with_auth_headers(
        test::TestRequest::post()
            .uri(&format!("/api/v1/protective-measures/{}/extensions", pm_id))
            .set_json(&ext_payload),
        &config,
        &admin_token,
    )
    .to_request();
    test::call_service(&app, create_ext_req).await;

    // CITY_USER should be able to list extensions
    let user_claims = test_helpers::build_city_user_claims(city);
    let user_token = test_helpers::generate_jwt(&user_claims, &config.jwt_secret);

    let list_ext_req = test_helpers::with_auth_headers(
        test::TestRequest::get()
            .uri(&format!("/api/v1/protective-measures/{}/extensions", pm_id)),
        &config,
        &user_token,
    )
    .to_request();
    let list_ext_resp = test::call_service(&app, list_ext_req).await;
    assert_eq!(list_ext_resp.status(), StatusCode::OK);

    let list_body: serde_json::Value = test::read_body_json(list_ext_resp).await;
    assert_eq!(list_body["data"].as_array().unwrap().len(), 1);
}

// ==================== UPDATE EXTENSION POLICY TESTS ====================

#[actix_rt::test]
async fn test_city_admin_can_update_extension_in_own_city() {
    let pool = test_helpers::setup_test_db().await;
    test_helpers::clean_database(&pool).await;

    let config = test_helpers::build_test_config();
    let app = test_helpers::create_full_test_app(pool.clone(), config.clone()).await;

    let city = db_fixtures::insert_city(&pool, "City Update").await;
    let victim_id = db_fixtures::insert_victim(&pool, "Victim Update", city).await;
    let offender_id = db_fixtures::insert_offender(&pool, "Offender Update", city, victim_id).await;

    let admin_claims = test_helpers::build_city_admin_claims(city);
    let admin_token = test_helpers::generate_jwt(&admin_claims, &config.jwt_secret);

    // Create protective measure and extension
    let pm_payload = build_measure_payload(victim_id, city, offender_id, true);
    let create_pm_req = test_helpers::with_auth_headers(
        test::TestRequest::post()
            .uri("/api/v1/protective-measures")
            .set_json(&pm_payload),
        &config,
        &admin_token,
    )
    .to_request();
    let create_pm_resp = test::call_service(&app, create_pm_req).await;
    let pm_body: serde_json::Value = test::read_body_json(create_pm_resp).await;
    let pm_id = pm_body["data"]["id"].as_str().unwrap();

    let ext_payload = build_extension_payload(1);
    let create_ext_req = test_helpers::with_auth_headers(
        test::TestRequest::post()
            .uri(&format!("/api/v1/protective-measures/{}/extensions", pm_id))
            .set_json(&ext_payload),
        &config,
        &admin_token,
    )
    .to_request();
    let create_ext_resp = test::call_service(&app, create_ext_req).await;
    let ext_body: serde_json::Value = test::read_body_json(create_ext_resp).await;
    let ext_id = ext_body["data"]["id"].as_str().unwrap();

    // Update extension - should succeed
    let update_payload = build_extension_payload(2);
    let update_req = test_helpers::with_auth_headers(
        test::TestRequest::put()
            .uri(&format!("/api/v1/protective-measures/{}/extensions/{}", pm_id, ext_id))
            .set_json(&update_payload),
        &config,
        &admin_token,
    )
    .to_request();
    let update_resp = test::call_service(&app, update_req).await;
    assert_eq!(update_resp.status(), StatusCode::OK);
}

#[actix_rt::test]
async fn test_city_admin_cannot_update_extension_in_other_city() {
    let pool = test_helpers::setup_test_db().await;
    test_helpers::clean_database(&pool).await;

    let config = test_helpers::build_test_config();
    let app = test_helpers::create_full_test_app(pool.clone(), config.clone()).await;

    let city_a = db_fixtures::insert_city(&pool, "City A Update").await;
    let city_b = db_fixtures::insert_city(&pool, "City B Update").await;
    let victim_b = db_fixtures::insert_victim(&pool, "Victim B", city_b).await;
    let offender_b = db_fixtures::insert_offender(&pool, "Offender B", city_b, victim_b).await;

    let admin_b_claims = test_helpers::build_city_admin_claims(city_b);
    let admin_b_token = test_helpers::generate_jwt(&admin_b_claims, &config.jwt_secret);

    // Create protective measure and extension in city B
    let pm_payload = build_measure_payload(victim_b, city_b, offender_b, true);
    let create_pm_req = test_helpers::with_auth_headers(
        test::TestRequest::post()
            .uri("/api/v1/protective-measures")
            .set_json(&pm_payload),
        &config,
        &admin_b_token,
    )
    .to_request();
    let create_pm_resp = test::call_service(&app, create_pm_req).await;
    let pm_body: serde_json::Value = test::read_body_json(create_pm_resp).await;
    let pm_id = pm_body["data"]["id"].as_str().unwrap();

    let ext_payload = build_extension_payload(1);
    let create_ext_req = test_helpers::with_auth_headers(
        test::TestRequest::post()
            .uri(&format!("/api/v1/protective-measures/{}/extensions", pm_id))
            .set_json(&ext_payload),
        &config,
        &admin_b_token,
    )
    .to_request();
    let create_ext_resp = test::call_service(&app, create_ext_req).await;
    let ext_body: serde_json::Value = test::read_body_json(create_ext_resp).await;
    let ext_id = ext_body["data"]["id"].as_str().unwrap();

    // Try to update extension as admin from city A - should fail
    let admin_a_claims = test_helpers::build_city_admin_claims(city_a);
    let admin_a_token = test_helpers::generate_jwt(&admin_a_claims, &config.jwt_secret);

    let update_payload = build_extension_payload(2);
    let update_req = test_helpers::with_auth_headers(
        test::TestRequest::put()
            .uri(&format!("/api/v1/protective-measures/{}/extensions/{}", pm_id, ext_id))
            .set_json(&update_payload),
        &config,
        &admin_a_token,
    )
    .to_request();
    let update_resp = test::call_service(&app, update_req).await;
    assert_eq!(update_resp.status(), StatusCode::FORBIDDEN);
}

#[actix_rt::test]
async fn test_city_user_cannot_update_extension() {
    let pool = test_helpers::setup_test_db().await;
    test_helpers::clean_database(&pool).await;

    let config = test_helpers::build_test_config();
    let app = test_helpers::create_full_test_app(pool.clone(), config.clone()).await;

    let city = db_fixtures::insert_city(&pool, "City User Update").await;
    let victim_id = db_fixtures::insert_victim(&pool, "Victim", city).await;
    let offender_id = db_fixtures::insert_offender(&pool, "Offender", city, victim_id).await;

    let admin_claims = test_helpers::build_city_admin_claims(city);
    let admin_token = test_helpers::generate_jwt(&admin_claims, &config.jwt_secret);

    // Create protective measure and extension
    let pm_payload = build_measure_payload(victim_id, city, offender_id, true);
    let create_pm_req = test_helpers::with_auth_headers(
        test::TestRequest::post()
            .uri("/api/v1/protective-measures")
            .set_json(&pm_payload),
        &config,
        &admin_token,
    )
    .to_request();
    let create_pm_resp = test::call_service(&app, create_pm_req).await;
    let pm_body: serde_json::Value = test::read_body_json(create_pm_resp).await;
    let pm_id = pm_body["data"]["id"].as_str().unwrap();

    let ext_payload = build_extension_payload(1);
    let create_ext_req = test_helpers::with_auth_headers(
        test::TestRequest::post()
            .uri(&format!("/api/v1/protective-measures/{}/extensions", pm_id))
            .set_json(&ext_payload),
        &config,
        &admin_token,
    )
    .to_request();
    let create_ext_resp = test::call_service(&app, create_ext_req).await;
    let ext_body: serde_json::Value = test::read_body_json(create_ext_resp).await;
    let ext_id = ext_body["data"]["id"].as_str().unwrap();

    // Try to update extension as CITY_USER - should fail
    let user_claims = test_helpers::build_city_user_claims(city);
    let user_token = test_helpers::generate_jwt(&user_claims, &config.jwt_secret);

    let update_payload = build_extension_payload(2);
    let update_req = test_helpers::with_auth_headers(
        test::TestRequest::put()
            .uri(&format!("/api/v1/protective-measures/{}/extensions/{}", pm_id, ext_id))
            .set_json(&update_payload),
        &config,
        &user_token,
    )
    .to_request();
    let update_resp = test::call_service(&app, update_req).await;
    assert_eq!(update_resp.status(), StatusCode::FORBIDDEN);
}

// ==================== DELETE EXTENSION POLICY TESTS ====================

#[actix_rt::test]
async fn test_city_admin_can_delete_extension_in_own_city() {
    let pool = test_helpers::setup_test_db().await;
    test_helpers::clean_database(&pool).await;

    let config = test_helpers::build_test_config();
    let app = test_helpers::create_full_test_app(pool.clone(), config.clone()).await;

    let city = db_fixtures::insert_city(&pool, "City Delete").await;
    let victim_id = db_fixtures::insert_victim(&pool, "Victim Delete", city).await;
    let offender_id = db_fixtures::insert_offender(&pool, "Offender Delete", city, victim_id).await;

    let admin_claims = test_helpers::build_city_admin_claims(city);
    let admin_token = test_helpers::generate_jwt(&admin_claims, &config.jwt_secret);

    // Create protective measure and extension
    let pm_payload = build_measure_payload(victim_id, city, offender_id, true);
    let create_pm_req = test_helpers::with_auth_headers(
        test::TestRequest::post()
            .uri("/api/v1/protective-measures")
            .set_json(&pm_payload),
        &config,
        &admin_token,
    )
    .to_request();
    let create_pm_resp = test::call_service(&app, create_pm_req).await;
    let pm_body: serde_json::Value = test::read_body_json(create_pm_resp).await;
    let pm_id = pm_body["data"]["id"].as_str().unwrap();

    let ext_payload = build_extension_payload(1);
    let create_ext_req = test_helpers::with_auth_headers(
        test::TestRequest::post()
            .uri(&format!("/api/v1/protective-measures/{}/extensions", pm_id))
            .set_json(&ext_payload),
        &config,
        &admin_token,
    )
    .to_request();
    let create_ext_resp = test::call_service(&app, create_ext_req).await;
    let ext_body: serde_json::Value = test::read_body_json(create_ext_resp).await;
    let ext_id = ext_body["data"]["id"].as_str().unwrap();

    // Delete extension - should succeed
    let delete_req = test_helpers::with_auth_headers(
        test::TestRequest::delete()
            .uri(&format!("/api/v1/protective-measures/{}/extensions/{}", pm_id, ext_id)),
        &config,
        &admin_token,
    )
    .to_request();
    let delete_resp = test::call_service(&app, delete_req).await;
    assert_eq!(delete_resp.status(), StatusCode::OK);
}

#[actix_rt::test]
async fn test_city_admin_cannot_delete_extension_in_other_city() {
    let pool = test_helpers::setup_test_db().await;
    test_helpers::clean_database(&pool).await;

    let config = test_helpers::build_test_config();
    let app = test_helpers::create_full_test_app(pool.clone(), config.clone()).await;

    let city_a = db_fixtures::insert_city(&pool, "City A Delete").await;
    let city_b = db_fixtures::insert_city(&pool, "City B Delete").await;
    let victim_b = db_fixtures::insert_victim(&pool, "Victim B", city_b).await;
    let offender_b = db_fixtures::insert_offender(&pool, "Offender B", city_b, victim_b).await;

    let admin_b_claims = test_helpers::build_city_admin_claims(city_b);
    let admin_b_token = test_helpers::generate_jwt(&admin_b_claims, &config.jwt_secret);

    // Create protective measure and extension in city B
    let pm_payload = build_measure_payload(victim_b, city_b, offender_b, true);
    let create_pm_req = test_helpers::with_auth_headers(
        test::TestRequest::post()
            .uri("/api/v1/protective-measures")
            .set_json(&pm_payload),
        &config,
        &admin_b_token,
    )
    .to_request();
    let create_pm_resp = test::call_service(&app, create_pm_req).await;
    let pm_body: serde_json::Value = test::read_body_json(create_pm_resp).await;
    let pm_id = pm_body["data"]["id"].as_str().unwrap();

    let ext_payload = build_extension_payload(1);
    let create_ext_req = test_helpers::with_auth_headers(
        test::TestRequest::post()
            .uri(&format!("/api/v1/protective-measures/{}/extensions", pm_id))
            .set_json(&ext_payload),
        &config,
        &admin_b_token,
    )
    .to_request();
    let create_ext_resp = test::call_service(&app, create_ext_req).await;
    let ext_body: serde_json::Value = test::read_body_json(create_ext_resp).await;
    let ext_id = ext_body["data"]["id"].as_str().unwrap();

    // Try to delete extension as admin from city A - should fail
    let admin_a_claims = test_helpers::build_city_admin_claims(city_a);
    let admin_a_token = test_helpers::generate_jwt(&admin_a_claims, &config.jwt_secret);

    let delete_req = test_helpers::with_auth_headers(
        test::TestRequest::delete()
            .uri(&format!("/api/v1/protective-measures/{}/extensions/{}", pm_id, ext_id)),
        &config,
        &admin_a_token,
    )
    .to_request();
    let delete_resp = test::call_service(&app, delete_req).await;
    assert_eq!(delete_resp.status(), StatusCode::FORBIDDEN);
}

#[actix_rt::test]
async fn test_city_user_cannot_delete_extension() {
    let pool = test_helpers::setup_test_db().await;
    test_helpers::clean_database(&pool).await;

    let config = test_helpers::build_test_config();
    let app = test_helpers::create_full_test_app(pool.clone(), config.clone()).await;

    let city = db_fixtures::insert_city(&pool, "City User Delete").await;
    let victim_id = db_fixtures::insert_victim(&pool, "Victim", city).await;
    let offender_id = db_fixtures::insert_offender(&pool, "Offender", city, victim_id).await;

    let admin_claims = test_helpers::build_city_admin_claims(city);
    let admin_token = test_helpers::generate_jwt(&admin_claims, &config.jwt_secret);

    // Create protective measure and extension
    let pm_payload = build_measure_payload(victim_id, city, offender_id, true);
    let create_pm_req = test_helpers::with_auth_headers(
        test::TestRequest::post()
            .uri("/api/v1/protective-measures")
            .set_json(&pm_payload),
        &config,
        &admin_token,
    )
    .to_request();
    let create_pm_resp = test::call_service(&app, create_pm_req).await;
    let pm_body: serde_json::Value = test::read_body_json(create_pm_resp).await;
    let pm_id = pm_body["data"]["id"].as_str().unwrap();

    let ext_payload = build_extension_payload(1);
    let create_ext_req = test_helpers::with_auth_headers(
        test::TestRequest::post()
            .uri(&format!("/api/v1/protective-measures/{}/extensions", pm_id))
            .set_json(&ext_payload),
        &config,
        &admin_token,
    )
    .to_request();
    let create_ext_resp = test::call_service(&app, create_ext_req).await;
    let ext_body: serde_json::Value = test::read_body_json(create_ext_resp).await;
    let ext_id = ext_body["data"]["id"].as_str().unwrap();

    // Try to delete extension as CITY_USER - should fail
    let user_claims = test_helpers::build_city_user_claims(city);
    let user_token = test_helpers::generate_jwt(&user_claims, &config.jwt_secret);

    let delete_req = test_helpers::with_auth_headers(
        test::TestRequest::delete()
            .uri(&format!("/api/v1/protective-measures/{}/extensions/{}", pm_id, ext_id)),
        &config,
        &user_token,
    )
    .to_request();
    let delete_resp = test::call_service(&app, delete_req).await;
    assert_eq!(delete_resp.status(), StatusCode::FORBIDDEN);
}
