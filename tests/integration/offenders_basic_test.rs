use actix_web::{http::StatusCode, test};
use serde_json::Value;

use crate::common::{db_fixtures, test_helpers};

#[actix_rt::test]
async fn test_create_offender_success() {
    let pool = test_helpers::setup_test_db().await;
    test_helpers::clean_database(&pool).await;

    let config = test_helpers::build_test_config();
    let app = test_helpers::create_full_test_app(pool.clone(), config.clone()).await;

    // Create city and victim first
    let city_id = db_fixtures::insert_city(&pool, "PORTO VELHO").await;
    let victim_id = db_fixtures::insert_victim(&pool, "Test Victim", city_id).await;

    let claims = test_helpers::build_root_claims();
    let token = test_helpers::generate_jwt(&claims, &config.jwt_secret);

    let payload = serde_json::json!({
        "full_name": "Test Offender",
        "city_id": city_id.to_string(),
        "victim_id": victim_id.to_string(),
        "imprisoned": false,
        "is_public_security_agent": false,
        "relationship_to_victim": "Spouse",
        "uses_alcohol": false,
        "uses_drugs": false,
        "has_psychiatric_issues": false,
        "was_drunk_during_assault": false
    });

    let req = test_helpers::with_auth_headers(
        test::TestRequest::post()
            .uri("/api/v1/offenders")
            .set_json(&payload),
        &config,
        &token,
    )
    .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), StatusCode::CREATED);

    let body: Value = test::read_body_json(resp).await;
    assert_eq!(body["data"]["full_name"], "Test Offender");
    assert_eq!(body["data"]["relationship_to_victim"], "Spouse");
}

#[actix_rt::test]
async fn test_get_offender_by_id() {
    let pool = test_helpers::setup_test_db().await;
    test_helpers::clean_database(&pool).await;

    let config = test_helpers::build_test_config();
    let app = test_helpers::create_full_test_app(pool.clone(), config.clone()).await;

    let city_id = db_fixtures::insert_city(&pool, "PORTO VELHO").await;
    let victim_id = db_fixtures::insert_victim(&pool, "Test Victim", city_id).await;
    let offender_id =
        db_fixtures::insert_offender(&pool, "John Doe", city_id, victim_id).await;

    let claims = test_helpers::build_root_claims();
    let token = test_helpers::generate_jwt(&claims, &config.jwt_secret);

    let req = test_helpers::with_auth_headers(
        test::TestRequest::get().uri(&format!("/api/v1/offenders/{}", offender_id)),
        &config,
        &token,
    )
    .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), StatusCode::OK);

    let body: Value = test::read_body_json(resp).await;
    assert_eq!(body["data"]["full_name"], "John Doe");
    assert_eq!(body["data"]["id"], offender_id.to_string());
}

#[actix_rt::test]
async fn test_get_all_offenders() {
    let pool = test_helpers::setup_test_db().await;
    test_helpers::clean_database(&pool).await;

    let config = test_helpers::build_test_config();
    let app = test_helpers::create_full_test_app(pool.clone(), config.clone()).await;

    let city_id = db_fixtures::insert_city(&pool, "PORTO VELHO").await;
    let victim_id = db_fixtures::insert_victim(&pool, "Test Victim", city_id).await;
    db_fixtures::insert_offender(&pool, "Offender 1", city_id, victim_id).await;
    db_fixtures::insert_offender(&pool, "Offender 2", city_id, victim_id).await;

    let claims = test_helpers::build_root_claims();
    let token = test_helpers::generate_jwt(&claims, &config.jwt_secret);

    let req = test_helpers::with_auth_headers(
        test::TestRequest::get().uri("/api/v1/offenders"),
        &config,
        &token,
    )
    .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), StatusCode::OK);

    let body: Value = test::read_body_json(resp).await;
    assert_eq!(body["data"].as_array().unwrap().len(), 2);
}

#[actix_rt::test]
async fn test_update_offender() {
    let pool = test_helpers::setup_test_db().await;
    test_helpers::clean_database(&pool).await;

    let config = test_helpers::build_test_config();
    let app = test_helpers::create_full_test_app(pool.clone(), config.clone()).await;

    let city_id = db_fixtures::insert_city(&pool, "PORTO VELHO").await;
    let victim_id = db_fixtures::insert_victim(&pool, "Test Victim", city_id).await;
    let offender_id =
        db_fixtures::insert_offender(&pool, "Old Name", city_id, victim_id).await;

    let claims = test_helpers::build_root_claims();
    let token = test_helpers::generate_jwt(&claims, &config.jwt_secret);

    let payload = serde_json::json!({
        "full_name": "Updated Name",
        "city_id": city_id.to_string(),
        "victim_id": victim_id.to_string(),
        "imprisoned": true,
        "is_public_security_agent": false,
        "relationship_to_victim": "Ex-Spouse",
        "uses_alcohol": true,
        "uses_drugs": false,
        "has_psychiatric_issues": false,
        "was_drunk_during_assault": true
    });

    let req = test_helpers::with_auth_headers(
        test::TestRequest::put()
            .uri(&format!("/api/v1/offenders/{}", offender_id))
            .set_json(&payload),
        &config,
        &token,
    )
    .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), StatusCode::OK);

    let body: Value = test::read_body_json(resp).await;
    assert_eq!(body["data"]["full_name"], "Updated Name");
    assert_eq!(body["data"]["imprisoned"], true);
    assert_eq!(body["data"]["relationship_to_victim"], "Ex-Spouse");
}

#[actix_rt::test]
async fn test_delete_offender() {
    let pool = test_helpers::setup_test_db().await;
    test_helpers::clean_database(&pool).await;

    let config = test_helpers::build_test_config();
    let app = test_helpers::create_full_test_app(pool.clone(), config.clone()).await;

    let city_id = db_fixtures::insert_city(&pool, "PORTO VELHO").await;
    let victim_id = db_fixtures::insert_victim(&pool, "Test Victim", city_id).await;
    let offender_id =
        db_fixtures::insert_offender(&pool, "To Delete", city_id, victim_id).await;

    let claims = test_helpers::build_root_claims();
    let token = test_helpers::generate_jwt(&claims, &config.jwt_secret);

    let req = test_helpers::with_auth_headers(
        test::TestRequest::delete().uri(&format!("/api/v1/offenders/{}", offender_id)),
        &config,
        &token,
    )
    .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), StatusCode::OK);

    // Verify offender is soft-deleted
    let req = test_helpers::with_auth_headers(
        test::TestRequest::get().uri(&format!("/api/v1/offenders/{}", offender_id)),
        &config,
        &token,
    )
    .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), StatusCode::NOT_FOUND);
}

#[actix_rt::test]
async fn test_get_offenders_by_victim_id() {
    let pool = test_helpers::setup_test_db().await;
    test_helpers::clean_database(&pool).await;

    let config = test_helpers::build_test_config();
    let app = test_helpers::create_full_test_app(pool.clone(), config.clone()).await;

    let city_id = db_fixtures::insert_city(&pool, "PORTO VELHO").await;
    let victim1_id = db_fixtures::insert_victim(&pool, "Victim 1", city_id).await;
    let victim2_id = db_fixtures::insert_victim(&pool, "Victim 2", city_id).await;

    db_fixtures::insert_offender(&pool, "Offender 1", city_id, victim1_id).await;
    db_fixtures::insert_offender(&pool, "Offender 2", city_id, victim1_id).await;
    db_fixtures::insert_offender(&pool, "Offender 3", city_id, victim2_id).await;

    let claims = test_helpers::build_root_claims();
    let token = test_helpers::generate_jwt(&claims, &config.jwt_secret);

    let req = test_helpers::with_auth_headers(
        test::TestRequest::get().uri(&format!("/api/v1/offenders/victim/{}", victim1_id)),
        &config,
        &token,
    )
    .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), StatusCode::OK);

    let body: Value = test::read_body_json(resp).await;
    assert_eq!(body["data"].as_array().unwrap().len(), 2);
}
