use actix_web::{http::StatusCode, test};
use serde_json::Value;

use crate::common::{db_fixtures, test_helpers};

#[actix_rt::test]
async fn test_create_offender_success() {
    let pool = test_helpers::setup_test_db().await;
    test_helpers::clean_database(&pool).await;

    let config = test_helpers::build_test_config();
    let app = test_helpers::create_full_test_app(pool.clone(), config.clone()).await;

    let city_id = db_fixtures::insert_city(&pool, "PORTO VELHO").await;

    let claims = test_helpers::build_root_claims();
    let token = test_helpers::generate_jwt(&claims, &config.jwt_secret);

    let payload = serde_json::json!({
        "full_name": "Test Offender",
        "city_id": city_id.to_string(),
        "imprisoned": false,
        "security_force": "Civil Police",
        "uses_alcohol": false,
        "uses_drugs": false,
        "has_psychiatric_issues": false,
        "education_level": "High School",
        "observation": null
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
    assert_eq!(body["data"]["is_public_security_agent"], true);
}

#[actix_rt::test]
async fn create_offender_with_valid_cpf_masked_success() {
    let pool = test_helpers::setup_test_db().await;
    test_helpers::clean_database(&pool).await;

    let config = test_helpers::build_test_config();
    let app = test_helpers::create_full_test_app(pool.clone(), config.clone()).await;

    let city_id = db_fixtures::insert_city(&pool, "PORTO VELHO").await;

    let claims = test_helpers::build_root_claims();
    let token = test_helpers::generate_jwt(&claims, &config.jwt_secret);

    let payload = serde_json::json!({
        "full_name": "Offender With CPF",
        "cpf": "529.982.247-25",
        "city_id": city_id.to_string(),
        "imprisoned": false,
        "uses_alcohol": false,
        "uses_drugs": false,
        "has_psychiatric_issues": false,
        "education_level": "High School"
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
    assert_eq!(body["data"]["cpf"].as_str().unwrap(), "529.982.247-25");
}

#[actix_rt::test]
async fn search_offenders_by_name_returns_matches() {
    let pool = test_helpers::setup_test_db().await;
    test_helpers::clean_database(&pool).await;

    let config = test_helpers::build_test_config();
    let app = test_helpers::create_full_test_app(pool.clone(), config.clone()).await;

    let city_id = db_fixtures::insert_city(&pool, "PORTO VELHO").await;

    let claims = test_helpers::build_root_claims();
    let token = test_helpers::generate_jwt(&claims, &config.jwt_secret);

    for name in ["Carlos Silva", "Pedro Lima"] {
        let payload = serde_json::json!({
            "full_name": name,
            "cpf": serde_json::Value::Null,
            "city_id": city_id.to_string(),
            "imprisoned": false,
            "uses_alcohol": false,
            "uses_drugs": false,
            "has_psychiatric_issues": false,
            "education_level": "High School"
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
    }

    let search_req = test_helpers::with_auth_headers(
        test::TestRequest::get().uri("/api/v1/offenders/search?name=carlos"),
        &config,
        &token,
    )
    .to_request();

    let search_resp = test::call_service(&app, search_req).await;
    assert_eq!(search_resp.status(), StatusCode::OK);
    let body: Value = test::read_body_json(search_resp).await;
    let results = body["data"].as_array().unwrap();
    assert_eq!(results.len(), 1);
    assert_eq!(results[0]["full_name"].as_str().unwrap(), "Carlos Silva");
}

#[actix_rt::test]
async fn search_offenders_by_cpf_returns_match() {
    let pool = test_helpers::setup_test_db().await;
    test_helpers::clean_database(&pool).await;

    let config = test_helpers::build_test_config();
    let app = test_helpers::create_full_test_app(pool.clone(), config.clone()).await;

    let city_id = db_fixtures::insert_city(&pool, "PORTO VELHO").await;

    let claims = test_helpers::build_root_claims();
    let token = test_helpers::generate_jwt(&claims, &config.jwt_secret);

    let payload = serde_json::json!({
        "full_name": "Offender CPF Search",
        "cpf": "529.982.247-25",
        "city_id": city_id.to_string(),
        "imprisoned": false,
        "uses_alcohol": false,
        "uses_drugs": false,
        "has_psychiatric_issues": false,
        "education_level": "High School"
    });

    let create_req = test_helpers::with_auth_headers(
        test::TestRequest::post()
            .uri("/api/v1/offenders")
            .set_json(&payload),
        &config,
        &token,
    )
    .to_request();

    let create_resp = test::call_service(&app, create_req).await;
    assert_eq!(create_resp.status(), StatusCode::CREATED);

    let search_req = test_helpers::with_auth_headers(
        test::TestRequest::get().uri("/api/v1/offenders/search?cpf=529.982.247-25"),
        &config,
        &token,
    )
    .to_request();

    let search_resp = test::call_service(&app, search_req).await;
    assert_eq!(search_resp.status(), StatusCode::OK);
    let body: Value = test::read_body_json(search_resp).await;
    let results = body["data"].as_array().unwrap();
    assert_eq!(results.len(), 1);
    assert_eq!(results[0]["cpf"].as_str().unwrap(), "529.982.247-25");
}

#[actix_rt::test]
async fn create_offender_with_invalid_cpf_format_fails() {
    let pool = test_helpers::setup_test_db().await;
    test_helpers::clean_database(&pool).await;

    let config = test_helpers::build_test_config();
    let app = test_helpers::create_full_test_app(pool.clone(), config.clone()).await;

    let city_id = db_fixtures::insert_city(&pool, "PORTO VELHO").await;

    let claims = test_helpers::build_root_claims();
    let token = test_helpers::generate_jwt(&claims, &config.jwt_secret);

    let payload = serde_json::json!({
        "full_name": "Offender Invalid CPF",
        "cpf": "123-456-789-00",
        "city_id": city_id.to_string(),
        "imprisoned": false,
        "uses_alcohol": false,
        "uses_drugs": false,
        "has_psychiatric_issues": false,
        "education_level": "High School"
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
    assert_eq!(resp.status(), StatusCode::BAD_REQUEST);

    let body: Value = test::read_body_json(resp).await;
    assert!(body["message"].as_str().unwrap().contains("cpf must match the format"));
}

#[actix_rt::test]
async fn create_offender_with_invalid_cpf_digits_fails() {
    let pool = test_helpers::setup_test_db().await;
    test_helpers::clean_database(&pool).await;

    let config = test_helpers::build_test_config();
    let app = test_helpers::create_full_test_app(pool.clone(), config.clone()).await;

    let city_id = db_fixtures::insert_city(&pool, "PORTO VELHO").await;

    let claims = test_helpers::build_root_claims();
    let token = test_helpers::generate_jwt(&claims, &config.jwt_secret);

    let payload = serde_json::json!({
        "full_name": "Offender Invalid CPF Digits",
        "cpf": "111.111.111-11",
        "city_id": city_id.to_string(),
        "imprisoned": false,
        "uses_alcohol": false,
        "uses_drugs": false,
        "has_psychiatric_issues": false,
        "education_level": "High School"
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
    assert_eq!(resp.status(), StatusCode::BAD_REQUEST);

    let body: Value = test::read_body_json(resp).await;
    assert!(body["message"].as_str().unwrap().contains("cpf has invalid check digits"));
}

#[actix_rt::test]
async fn test_get_offender_by_id() {
    let pool = test_helpers::setup_test_db().await;
    test_helpers::clean_database(&pool).await;

    let config = test_helpers::build_test_config();
    let app = test_helpers::create_full_test_app(pool.clone(), config.clone()).await;

    let city_id = db_fixtures::insert_city(&pool, "PORTO VELHO").await;
    let offender_id = db_fixtures::insert_offender(&pool, "John Doe", city_id).await;

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
    db_fixtures::insert_offender(&pool, "Offender 1", city_id).await;
    db_fixtures::insert_offender(&pool, "Offender 2", city_id).await;

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
    let offender_id = db_fixtures::insert_offender(&pool, "Old Name", city_id).await;

    let claims = test_helpers::build_root_claims();
    let token = test_helpers::generate_jwt(&claims, &config.jwt_secret);

    let payload = serde_json::json!({
        "full_name": "Updated Name",
        "city_id": city_id.to_string(),
        "imprisoned": true,
        "security_force": "Military Police",
        "uses_alcohol": true,
        "uses_drugs": false,
        "has_psychiatric_issues": false,
        "education_level": "College",
        "observation": null
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
    assert_eq!(body["data"]["is_public_security_agent"], true);
}

#[actix_rt::test]
async fn test_delete_offender() {
    let pool = test_helpers::setup_test_db().await;
    test_helpers::clean_database(&pool).await;

    let config = test_helpers::build_test_config();
    let app = test_helpers::create_full_test_app(pool.clone(), config.clone()).await;

    let city_id = db_fixtures::insert_city(&pool, "PORTO VELHO").await;
    let offender_id = db_fixtures::insert_offender(&pool, "To Delete", city_id).await;

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

    let offender1_id = db_fixtures::insert_offender(&pool, "Offender 1", city_id).await;
    let offender2_id = db_fixtures::insert_offender(&pool, "Offender 2", city_id).await;
    let offender3_id = db_fixtures::insert_offender(&pool, "Offender 3", city_id).await;

    db_fixtures::insert_protective_measure(&pool, victim1_id, offender1_id, city_id, "Valid").await;
    db_fixtures::insert_protective_measure(&pool, victim1_id, offender2_id, city_id, "Revoked").await;
    db_fixtures::insert_protective_measure(&pool, victim2_id, offender3_id, city_id, "Valid").await;

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

#[actix_rt::test]
async fn create_offender_without_city_id_uses_residential_address_city() {
    let pool = test_helpers::setup_test_db().await;
    test_helpers::clean_database(&pool).await;

    let config = test_helpers::build_test_config();
    let app = test_helpers::create_full_test_app(pool.clone(), config.clone()).await;

    let city_id = db_fixtures::insert_city(&pool, "City Residential").await;

    let claims = test_helpers::build_root_claims();
    let token = test_helpers::generate_jwt(&claims, &config.jwt_secret);

    let payload = serde_json::json!({
        "full_name": "Offender Residential",
        "city_id": null,
        "imprisoned": false,
        "uses_alcohol": false,
        "uses_drugs": false,
        "has_psychiatric_issues": false,
        "education_level": "High School",
        "addresses": [{
            "street": "Rua A",
            "number": "10",
            "district": "Centro",
            "city_id": city_id.to_string(),
            "zip_code": "12345-000",
            "complement": null,
            "address_type": "Residential"
        }]
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
    assert_eq!(body["data"]["city_id"], city_id.to_string());
}

#[actix_rt::test]
async fn create_offender_without_city_id_uses_work_address_city() {
    let pool = test_helpers::setup_test_db().await;
    test_helpers::clean_database(&pool).await;

    let config = test_helpers::build_test_config();
    let app = test_helpers::create_full_test_app(pool.clone(), config.clone()).await;

    let city_id = db_fixtures::insert_city(&pool, "City Work").await;

    let claims = test_helpers::build_root_claims();
    let token = test_helpers::generate_jwt(&claims, &config.jwt_secret);

    let payload = serde_json::json!({
        "full_name": "Offender Work",
        "city_id": null,
        "imprisoned": false,
        "uses_alcohol": false,
        "uses_drugs": false,
        "has_psychiatric_issues": false,
        "education_level": "High School",
        "addresses": [{
            "street": "Rua B",
            "number": "20",
            "district": "Bairro",
            "city_id": city_id.to_string(),
            "zip_code": "54321-000",
            "complement": null,
            "address_type": "Work"
        }]
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
    assert_eq!(body["data"]["city_id"], city_id.to_string());
}

#[actix_rt::test]
async fn create_offender_without_city_id_and_without_residential_or_work_address_fails() {
    let pool = test_helpers::setup_test_db().await;
    test_helpers::clean_database(&pool).await;

    let config = test_helpers::build_test_config();
    let app = test_helpers::create_full_test_app(pool.clone(), config.clone()).await;

    let city_id = db_fixtures::insert_city(&pool, "City Other").await;

    let claims = test_helpers::build_root_claims();
    let token = test_helpers::generate_jwt(&claims, &config.jwt_secret);

    let payload = serde_json::json!({
        "full_name": "Offender Other",
        "city_id": null,
        "imprisoned": false,
        "uses_alcohol": false,
        "uses_drugs": false,
        "has_psychiatric_issues": false,
        "education_level": "High School",
        "addresses": [{
            "street": "Rua C",
            "number": "30",
            "district": "Centro",
            "city_id": city_id.to_string(),
            "zip_code": "00000-000",
            "complement": null,
            "address_type": "Other"
        }]
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
    assert_eq!(resp.status(), StatusCode::BAD_REQUEST);
    let body: Value = test::read_body_json(resp).await;
    assert!(body["message"].as_str().unwrap().contains("no Residential or Work address provided"));
}
