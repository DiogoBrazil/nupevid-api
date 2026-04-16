use actix_web::{http::StatusCode, test};
use chrono::NaiveDate;
use uuid::Uuid;

use crate::common::{db_fixtures, test_helpers};

fn build_measure_payload(
    victim_id: Uuid,
    court_district_id: Uuid,
    offender_id: Uuid,
) -> serde_json::Value {
    serde_json::json!({
        "process_number": "45678-90.2025.8.26.0000",
        "issued_at": NaiveDate::from_ymd_opt(2025, 2, 1).unwrap(),
        "judicial_authority": "Juiz B",
        "court_district_id": court_district_id,
        "status": "Valid",
        "violence_types": ["Physical"],
        "relationship_to_victim": "Spouse",
        "assaults_children": false,
        "was_drunk_during_assault": false,
        "victim_id": victim_id,
        "offender_id": offender_id,
    })
}

fn assert_entity_without_timestamps(entity: &serde_json::Value) {
    assert!(entity.is_object());
    assert!(entity.get("created_at").is_none());
    assert!(entity.get("updated_at").is_none());
}

#[actix_rt::test]
async fn create_protective_measure_with_include_related_returns_entities() {
    let pool = test_helpers::setup_test_db().await;
    test_helpers::clean_database(&pool).await;

    let config = test_helpers::build_test_config();
    let app = test_helpers::create_full_test_app(pool.clone(), config.clone()).await;

    let city = db_fixtures::insert_city(&pool, "Cidade PM").await;
    let victim_id = db_fixtures::insert_victim(&pool, "Vitima PM", city).await;
    let offender_id = db_fixtures::insert_offender(&pool, "Agressor PM", city).await;

    let admin_claims = test_helpers::build_city_admin_claims(city);
    let admin_token = test_helpers::generate_jwt(&admin_claims, &config.jwt_secret);

    let payload = build_measure_payload(victim_id, city, offender_id);

    let req = test_helpers::with_auth_headers(
        test::TestRequest::post()
            .uri("/api/v1/protective-measures?include_related_entities=true")
            .set_json(&payload),
        &config,
        &admin_token,
    )
    .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), StatusCode::CREATED);
    let body: serde_json::Value = test::read_body_json(resp).await;
    let data = &body["data"];
    assert_eq!(
        data["victim"]["id"].as_str().unwrap(),
        victim_id.to_string()
    );
    assert_eq!(
        data["offender"]["id"].as_str().unwrap(),
        offender_id.to_string()
    );
    assert_eq!(
        data["court_district"]["id"].as_str().unwrap(),
        city.to_string()
    );
    assert_entity_without_timestamps(&data["victim"]);
    assert_entity_without_timestamps(&data["offender"]);
    assert_entity_without_timestamps(&data["court_district"]);
}

#[actix_rt::test]
async fn get_protective_measure_by_id_with_include_related_returns_entities() {
    let pool = test_helpers::setup_test_db().await;
    test_helpers::clean_database(&pool).await;

    let config = test_helpers::build_test_config();
    let app = test_helpers::create_full_test_app(pool.clone(), config.clone()).await;

    let city = db_fixtures::insert_city(&pool, "Cidade PM Get").await;
    let victim_id = db_fixtures::insert_victim(&pool, "Vitima PM Get", city).await;
    let offender_id = db_fixtures::insert_offender(&pool, "Agressor PM Get", city).await;
    let measure_id =
        db_fixtures::insert_protective_measure(&pool, victim_id, offender_id, city, "Valid").await;

    let admin_claims = test_helpers::build_city_admin_claims(city);
    let admin_token = test_helpers::generate_jwt(&admin_claims, &config.jwt_secret);

    let req = test_helpers::with_auth_headers(
        test::TestRequest::get().uri(&format!(
            "/api/v1/protective-measures/{}?include_related_entities=true",
            measure_id
        )),
        &config,
        &admin_token,
    )
    .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), StatusCode::OK);
    let body: serde_json::Value = test::read_body_json(resp).await;
    let data = &body["data"];
    assert_eq!(
        data["victim"]["id"].as_str().unwrap(),
        victim_id.to_string()
    );
    assert_eq!(
        data["offender"]["id"].as_str().unwrap(),
        offender_id.to_string()
    );
    assert_eq!(
        data["court_district"]["id"].as_str().unwrap(),
        city.to_string()
    );
    assert_entity_without_timestamps(&data["victim"]);
    assert_entity_without_timestamps(&data["offender"]);
    assert_entity_without_timestamps(&data["court_district"]);
}

#[actix_rt::test]
async fn list_protective_measures_with_include_related_returns_entities() {
    let pool = test_helpers::setup_test_db().await;
    test_helpers::clean_database(&pool).await;

    let config = test_helpers::build_test_config();
    let app = test_helpers::create_full_test_app(pool.clone(), config.clone()).await;

    let city = db_fixtures::insert_city(&pool, "Cidade PM Lista").await;
    let victim_id = db_fixtures::insert_victim(&pool, "Vitima PM Lista", city).await;
    let offender_id = db_fixtures::insert_offender(&pool, "Agressor PM Lista", city).await;
    db_fixtures::insert_protective_measure(&pool, victim_id, offender_id, city, "Valid").await;

    let admin_claims = test_helpers::build_city_admin_claims(city);
    let admin_token = test_helpers::generate_jwt(&admin_claims, &config.jwt_secret);

    let req = test_helpers::with_auth_headers(
        test::TestRequest::get().uri("/api/v1/protective-measures?include_related_entities=true"),
        &config,
        &admin_token,
    )
    .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), StatusCode::OK);
    let body: serde_json::Value = test::read_body_json(resp).await;
    let measures = body["data"].as_array().unwrap();
    assert!(!measures.is_empty());
    assert_entity_without_timestamps(&measures[0]["victim"]);
    assert_entity_without_timestamps(&measures[0]["offender"]);
    assert_entity_without_timestamps(&measures[0]["court_district"]);
}

#[actix_rt::test]
async fn get_protective_measures_by_victim_with_include_related_returns_entities() {
    let pool = test_helpers::setup_test_db().await;
    test_helpers::clean_database(&pool).await;

    let config = test_helpers::build_test_config();
    let app = test_helpers::create_full_test_app(pool.clone(), config.clone()).await;

    let city = db_fixtures::insert_city(&pool, "Cidade PM Victim").await;
    let victim_id = db_fixtures::insert_victim(&pool, "Vitima PM Victim", city).await;
    let offender_id = db_fixtures::insert_offender(&pool, "Agressor PM Victim", city).await;
    db_fixtures::insert_protective_measure(&pool, victim_id, offender_id, city, "Valid").await;

    let admin_claims = test_helpers::build_city_admin_claims(city);
    let admin_token = test_helpers::generate_jwt(&admin_claims, &config.jwt_secret);

    let req = test_helpers::with_auth_headers(
        test::TestRequest::get().uri(&format!(
            "/api/v1/protective-measures/victim/{}?include_related_entities=true",
            victim_id
        )),
        &config,
        &admin_token,
    )
    .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), StatusCode::OK);
    let body: serde_json::Value = test::read_body_json(resp).await;
    let measures = body["data"].as_array().unwrap();
    assert!(!measures.is_empty());
    assert_entity_without_timestamps(&measures[0]["victim"]);
    assert_entity_without_timestamps(&measures[0]["offender"]);
    assert_entity_without_timestamps(&measures[0]["court_district"]);
}
