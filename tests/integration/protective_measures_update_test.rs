use actix_web::{http::StatusCode, test};
use chrono::NaiveDate;
use uuid::Uuid;

use crate::common::{db_fixtures, test_helpers};

fn build_measure_payload(
    victim_id: Uuid,
    court_district_id: Uuid,
    offender_id: Uuid,
    status: &str,
) -> serde_json::Value {
    serde_json::json!({
        "process_number": "12345-67.2025.8.26.0000",
        "issued_at": NaiveDate::from_ymd_opt(2025, 1, 1).unwrap(),
        "judicial_authority": "Juiz A",
        "court_district_id": court_district_id,
        "status": status,
        "violence_types": ["Physical"],
        "relationship_to_victim": "Spouse",
        "assaults_children": false,
        "was_drunk_during_assault": false,
        "victim_id": victim_id,
        "offender_id": offender_id,
    })
}

#[actix_rt::test]
async fn update_protective_measure_success() {
    let pool = test_helpers::setup_test_db().await;
    test_helpers::clean_database(&pool).await;

    let config = test_helpers::build_test_config();
    let app = test_helpers::create_full_test_app(pool.clone(), config.clone()).await;

    let city = db_fixtures::insert_city(&pool, "Cidade Update").await;
    let victim_id = db_fixtures::insert_victim(&pool, "Vitima", city).await;
    let offender_id = db_fixtures::insert_offender(&pool, "Agressor", city).await;

    let admin_claims = test_helpers::build_city_admin_claims(city);
    let admin_token = test_helpers::generate_jwt(&admin_claims, &config.jwt_secret);

    // Create initial measure
    let payload = build_measure_payload(victim_id, city, offender_id, "Revoked");
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

    // Update measure
    let update_payload = serde_json::json!({
        "process_number": "99999-99.2025.8.26.0000",
        "issued_at": NaiveDate::from_ymd_opt(2025, 2, 1).unwrap(),
        "judicial_authority": "Juiz B Atualizado",
        "court_district_id": city,
        "status": "Revoked",
        "violence_types": ["Physical"],
        "relationship_to_victim": "Spouse",
        "assaults_children": false,
        "was_drunk_during_assault": false,
        "victim_id": victim_id,
        "offender_id": offender_id,
    });

    let update_req = test_helpers::with_auth_headers(
        test::TestRequest::put()
            .uri(&format!("/api/v1/protective-measures/{}", measure_id))
            .set_json(&update_payload),
        &config,
        &admin_token,
    )
    .to_request();

    let update_resp = test::call_service(&app, update_req).await;
    assert_eq!(update_resp.status(), StatusCode::OK);

    let update_body: serde_json::Value = test::read_body_json(update_resp).await;
    assert_eq!(
        update_body["data"]["process_number"].as_str().unwrap(),
        "99999-99.2025.8.26.0000"
    );
    assert_eq!(
        update_body["data"]["judicial_authority"].as_str().unwrap(),
        "Juiz B Atualizado"
    );
    assert_eq!(update_body["data"]["status"].as_str().unwrap(), "Revoked");
}

#[actix_rt::test]
async fn cannot_update_to_active_when_victim_already_has_active_measure() {
    let pool = test_helpers::setup_test_db().await;
    test_helpers::clean_database(&pool).await;

    let config = test_helpers::build_test_config();
    let app = test_helpers::create_full_test_app(pool.clone(), config.clone()).await;

    let city = db_fixtures::insert_city(&pool, "Cidade Conflict").await;
    let victim_id = db_fixtures::insert_victim(&pool, "Vitima", city).await;
    let offender_id = db_fixtures::insert_offender(&pool, "Agressor", city).await;

    let admin_claims = test_helpers::build_city_admin_claims(city);
    let admin_token = test_helpers::generate_jwt(&admin_claims, &config.jwt_secret);

    // Create first measure (inactive)
    let payload1 = build_measure_payload(victim_id, city, offender_id, "Revoked");
    let create_req1 = test_helpers::with_auth_headers(
        test::TestRequest::post()
            .uri("/api/v1/protective-measures")
            .set_json(&payload1),
        &config,
        &admin_token,
    )
    .to_request();
    let create_resp1 = test::call_service(&app, create_req1).await;
    assert_eq!(create_resp1.status(), StatusCode::CREATED);
    let body1: serde_json::Value = test::read_body_json(create_resp1).await;
    let measure_id_1 = body1["data"]["id"].as_str().unwrap();

    // Create second measure (active)
    let payload2 = build_measure_payload(victim_id, city, offender_id, "Valid");
    let create_req2 = test_helpers::with_auth_headers(
        test::TestRequest::post()
            .uri("/api/v1/protective-measures")
            .set_json(&payload2),
        &config,
        &admin_token,
    )
    .to_request();
    let create_resp2 = test::call_service(&app, create_req2).await;
    assert_eq!(create_resp2.status(), StatusCode::CREATED);

    // Try to update first measure to active (should fail)
    let update_payload = serde_json::json!({
        "process_number": "12345-67.2025.8.26.0000",
        "issued_at": NaiveDate::from_ymd_opt(2025, 1, 1).unwrap(),
        "judicial_authority": "Juiz A",
        "court_district_id": city,
        "status": "Valid",
        "violence_types": ["Physical"],
        "relationship_to_victim": "Spouse",
        "assaults_children": false,
        "was_drunk_during_assault": false,
        "victim_id": victim_id,
        "offender_id": offender_id,
    });

    let update_req = test_helpers::with_auth_headers(
        test::TestRequest::put()
            .uri(&format!("/api/v1/protective-measures/{}", measure_id_1))
            .set_json(&update_payload),
        &config,
        &admin_token,
    )
    .to_request();

    let update_resp = test::call_service(&app, update_req).await;
    assert_eq!(update_resp.status(), StatusCode::CONFLICT);
    let body: serde_json::Value = test::read_body_json(update_resp).await;
    assert!(
        body["message"]
            .as_str()
            .unwrap()
            .contains("active protective measure")
    );
}

#[actix_rt::test]
async fn update_measure_nonexistent_id_returns_404() {
    let pool = test_helpers::setup_test_db().await;
    test_helpers::clean_database(&pool).await;

    let config = test_helpers::build_test_config();
    let app = test_helpers::create_full_test_app(pool.clone(), config.clone()).await;

    let city = db_fixtures::insert_city(&pool, "Cidade NF").await;
    let victim_id = db_fixtures::insert_victim(&pool, "Vitima", city).await;
    let offender_id = db_fixtures::insert_offender(&pool, "Agressor", city).await;

    let admin_claims = test_helpers::build_city_admin_claims(city);
    let admin_token = test_helpers::generate_jwt(&admin_claims, &config.jwt_secret);

    let random_id = Uuid::new_v4();
    let update_payload = build_measure_payload(victim_id, city, offender_id, "Revoked");

    let update_req = test_helpers::with_auth_headers(
        test::TestRequest::put()
            .uri(&format!("/api/v1/protective-measures/{}", random_id))
            .set_json(&update_payload),
        &config,
        &admin_token,
    )
    .to_request();

    let update_resp = test::call_service(&app, update_req).await;
    assert_eq!(update_resp.status(), StatusCode::NOT_FOUND);
}

#[actix_rt::test]
async fn update_measure_change_victim_requires_permission_on_both() {
    let pool = test_helpers::setup_test_db().await;
    test_helpers::clean_database(&pool).await;

    let config = test_helpers::build_test_config();
    let app = test_helpers::create_full_test_app(pool.clone(), config.clone()).await;

    let city_a = db_fixtures::insert_city(&pool, "Cidade A").await;
    let city_b = db_fixtures::insert_city(&pool, "Cidade B").await;
    let victim_a = db_fixtures::insert_victim(&pool, "Vitima A", city_a).await;
    let offender_a = db_fixtures::insert_offender(&pool, "Agressor", city_a).await;
    let victim_b = db_fixtures::insert_victim(&pool, "Vitima B", city_b).await;
    let offender_b = db_fixtures::insert_offender(&pool, "Agressor", city_b).await;

    let root_claims = test_helpers::build_root_claims();
    let root_token = test_helpers::generate_jwt(&root_claims, &config.jwt_secret);

    // Create measure for victim A
    let payload = build_measure_payload(victim_a, city_a, offender_a, "Revoked");
    let create_req = test_helpers::with_auth_headers(
        test::TestRequest::post()
            .uri("/api/v1/protective-measures")
            .set_json(&payload),
        &config,
        &root_token,
    )
    .to_request();
    let create_resp = test::call_service(&app, create_req).await;
    assert_eq!(create_resp.status(), StatusCode::CREATED);
    let body: serde_json::Value = test::read_body_json(create_resp).await;
    let measure_id = body["data"]["id"].as_str().unwrap();

    // CITY_ADMIN A tries to change victim to B (different city)
    let admin_a_claims = test_helpers::build_city_admin_claims(city_a);
    let admin_a_token = test_helpers::generate_jwt(&admin_a_claims, &config.jwt_secret);

    let update_payload = build_measure_payload(victim_b, city_b, offender_b, "Revoked");
    let update_req = test_helpers::with_auth_headers(
        test::TestRequest::put()
            .uri(&format!("/api/v1/protective-measures/{}", measure_id))
            .set_json(&update_payload),
        &config,
        &admin_a_token,
    )
    .to_request();

    let update_resp = test::call_service(&app, update_req).await;
    assert_eq!(update_resp.status(), StatusCode::FORBIDDEN);
}

#[actix_rt::test]
async fn update_measure_with_empty_process_number_fails() {
    let pool = test_helpers::setup_test_db().await;
    test_helpers::clean_database(&pool).await;

    let config = test_helpers::build_test_config();
    let app = test_helpers::create_full_test_app(pool.clone(), config.clone()).await;

    let city = db_fixtures::insert_city(&pool, "Cidade Val").await;
    let victim_id = db_fixtures::insert_victim(&pool, "Vitima", city).await;
    let offender_id = db_fixtures::insert_offender(&pool, "Agressor", city).await;

    let admin_claims = test_helpers::build_city_admin_claims(city);
    let admin_token = test_helpers::generate_jwt(&admin_claims, &config.jwt_secret);

    // Create measure
    let payload = build_measure_payload(victim_id, city, offender_id, "Revoked");
    let create_req = test_helpers::with_auth_headers(
        test::TestRequest::post()
            .uri("/api/v1/protective-measures")
            .set_json(&payload),
        &config,
        &admin_token,
    )
    .to_request();
    let create_resp = test::call_service(&app, create_req).await;
    let body: serde_json::Value = test::read_body_json(create_resp).await;
    let measure_id = body["data"]["id"].as_str().unwrap();

    // Try to update with empty process_number
    let update_payload = serde_json::json!({
        "process_number": "",
        "issued_at": NaiveDate::from_ymd_opt(2025, 1, 1).unwrap(),
        "judicial_authority": "Juiz A",
        "court_district_id": city,
        "status": "Revoked",
        "violence_types": ["Physical"],
        "relationship_to_victim": "Spouse",
        "assaults_children": false,
        "was_drunk_during_assault": false,
        "victim_id": victim_id,
        "offender_id": offender_id,
    });

    let update_req = test_helpers::with_auth_headers(
        test::TestRequest::put()
            .uri(&format!("/api/v1/protective-measures/{}", measure_id))
            .set_json(&update_payload),
        &config,
        &admin_token,
    )
    .to_request();

    let update_resp = test::call_service(&app, update_req).await;
    assert_eq!(update_resp.status(), StatusCode::BAD_REQUEST);
    let body: serde_json::Value = test::read_body_json(update_resp).await;
    assert!(body["message"].as_str().unwrap().contains("process_number"));
}

#[actix_rt::test]
async fn update_measure_with_empty_judicial_authority_fails() {
    let pool = test_helpers::setup_test_db().await;
    test_helpers::clean_database(&pool).await;

    let config = test_helpers::build_test_config();
    let app = test_helpers::create_full_test_app(pool.clone(), config.clone()).await;

    let city = db_fixtures::insert_city(&pool, "Cidade Val2").await;
    let victim_id = db_fixtures::insert_victim(&pool, "Vitima", city).await;
    let offender_id = db_fixtures::insert_offender(&pool, "Agressor", city).await;

    let admin_claims = test_helpers::build_city_admin_claims(city);
    let admin_token = test_helpers::generate_jwt(&admin_claims, &config.jwt_secret);

    // Create measure
    let payload = build_measure_payload(victim_id, city, offender_id, "Revoked");
    let create_req = test_helpers::with_auth_headers(
        test::TestRequest::post()
            .uri("/api/v1/protective-measures")
            .set_json(&payload),
        &config,
        &admin_token,
    )
    .to_request();
    let create_resp = test::call_service(&app, create_req).await;
    let body: serde_json::Value = test::read_body_json(create_resp).await;
    let measure_id = body["data"]["id"].as_str().unwrap();

    // Try to update with empty judicial_authority
    let update_payload = serde_json::json!({
        "process_number": "12345-67.2025.8.26.0000",
        "issued_at": NaiveDate::from_ymd_opt(2025, 1, 1).unwrap(),
        "judicial_authority": "",
        "court_district_id": city,
        "status": "Revoked",
        "violence_types": ["Physical"],
        "relationship_to_victim": "Spouse",
        "assaults_children": false,
        "was_drunk_during_assault": false,
        "victim_id": victim_id,
        "offender_id": offender_id,
    });

    let update_req = test_helpers::with_auth_headers(
        test::TestRequest::put()
            .uri(&format!("/api/v1/protective-measures/{}", measure_id))
            .set_json(&update_payload),
        &config,
        &admin_token,
    )
    .to_request();

    let update_resp = test::call_service(&app, update_req).await;
    assert_eq!(update_resp.status(), StatusCode::BAD_REQUEST);
    let body: serde_json::Value = test::read_body_json(update_resp).await;
    assert!(
        body["message"]
            .as_str()
            .unwrap()
            .contains("judicial_authority")
    );
}

#[actix_rt::test]
async fn update_measure_to_nonexistent_victim_returns_404() {
    let pool = test_helpers::setup_test_db().await;
    test_helpers::clean_database(&pool).await;

    let config = test_helpers::build_test_config();
    let app = test_helpers::create_full_test_app(pool.clone(), config.clone()).await;

    let city = db_fixtures::insert_city(&pool, "Cidade NF2").await;
    let victim_id = db_fixtures::insert_victim(&pool, "Vitima", city).await;
    let offender_id = db_fixtures::insert_offender(&pool, "Agressor", city).await;

    let admin_claims = test_helpers::build_city_admin_claims(city);
    let admin_token = test_helpers::generate_jwt(&admin_claims, &config.jwt_secret);

    // Create measure
    let payload = build_measure_payload(victim_id, city, offender_id, "Revoked");
    let create_req = test_helpers::with_auth_headers(
        test::TestRequest::post()
            .uri("/api/v1/protective-measures")
            .set_json(&payload),
        &config,
        &admin_token,
    )
    .to_request();
    let create_resp = test::call_service(&app, create_req).await;
    let body: serde_json::Value = test::read_body_json(create_resp).await;
    let measure_id = body["data"]["id"].as_str().unwrap();

    // Try to update with nonexistent victim
    let random_victim_id = Uuid::new_v4();
    let update_payload = build_measure_payload(random_victim_id, city, offender_id, "Revoked");

    let update_req = test_helpers::with_auth_headers(
        test::TestRequest::put()
            .uri(&format!("/api/v1/protective-measures/{}", measure_id))
            .set_json(&update_payload),
        &config,
        &admin_token,
    )
    .to_request();

    let update_resp = test::call_service(&app, update_req).await;
    assert_eq!(update_resp.status(), StatusCode::NOT_FOUND);
}

#[actix_rt::test]
async fn update_measure_rejects_unknown_extensions_field() {
    let pool = test_helpers::setup_test_db().await;
    test_helpers::clean_database(&pool).await;

    let config = test_helpers::build_test_config();
    let app = test_helpers::create_full_test_app(pool.clone(), config.clone()).await;

    let city = db_fixtures::insert_city(&pool, "Cidade Extensao Update").await;
    let victim_id = db_fixtures::insert_victim(&pool, "Vitima", city).await;
    let offender_id = db_fixtures::insert_offender(&pool, "Agressor", city).await;

    let admin_claims = test_helpers::build_city_admin_claims(city);
    let admin_token = test_helpers::generate_jwt(&admin_claims, &config.jwt_secret);

    let payload_a = build_measure_payload(victim_id, city, offender_id, "Revoked");
    let create_req_a = test_helpers::with_auth_headers(
        test::TestRequest::post()
            .uri("/api/v1/protective-measures")
            .set_json(&payload_a),
        &config,
        &admin_token,
    )
    .to_request();
    let create_resp_a = test::call_service(&app, create_req_a).await;
    assert_eq!(create_resp_a.status(), StatusCode::CREATED);
    let create_body_a: serde_json::Value = test::read_body_json(create_resp_a).await;
    let measure_a_id = create_body_a["data"]["id"].as_str().unwrap();

    let update_payload = serde_json::json!({
        "process_number": "12345-67.2025.8.26.0000",
        "issued_at": NaiveDate::from_ymd_opt(2025, 1, 1).unwrap(),
        "judicial_authority": "Juiz A",
        "court_district_id": city,
        "status": "Revoked",
        "violence_types": ["Physical"],
        "relationship_to_victim": "Spouse",
        "assaults_children": false,
        "was_drunk_during_assault": false,
        "victim_id": victim_id,
        "offender_id": offender_id,
        "extensions": [
            {
                "extension_number": 1,
                "extension_date": NaiveDate::from_ymd_opt(2025, 3, 1).unwrap(),
                "notes": "campo removido do contrato"
            }
        ]
    });

    let update_req = test_helpers::with_auth_headers(
        test::TestRequest::put()
            .uri(&format!("/api/v1/protective-measures/{}", measure_a_id))
            .set_json(&update_payload),
        &config,
        &admin_token,
    )
    .to_request();

    let update_resp = test::call_service(&app, update_req).await;
    assert_eq!(update_resp.status(), StatusCode::UNPROCESSABLE_ENTITY);
    let body: serde_json::Value = test::read_body_json(update_resp).await;
    assert_eq!(body["field"].as_str().unwrap(), "extensions");
}
