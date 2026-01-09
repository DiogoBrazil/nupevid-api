use actix_web::{http::StatusCode, test};
use uuid::Uuid;

use crate::common::{test_helpers, db_fixtures};

#[actix_rt::test]
async fn create_victim_with_empty_full_name_fails() {
    let pool = test_helpers::setup_test_db().await;
    test_helpers::clean_database(&pool).await;

    let config = test_helpers::build_test_config();
    let app = test_helpers::create_full_test_app(pool.clone(), config.clone()).await;

    let city = db_fixtures::insert_city(&pool, "Test City").await;

    let root_claims = test_helpers::build_root_claims();
    let root_token = test_helpers::generate_jwt(&root_claims, &config.jwt_secret);

    let payload = serde_json::json!({
        "full_name": "",
        "cpf": null,
        "birth_date": null,
        "city_id": city,
        "phones": null,
        "addresses": null,
        "education_level": null,
        "occupation": null,
        "has_children": "No",
        "children_count": null,
        "special_needs_type": null,
        "uses_alcohol": false,
        "uses_drugs": false,
        "psychiatric_issues_type": null,
    });

    let req = test_helpers::with_auth_headers(
        test::TestRequest::post()
            .uri("/api/v1/victims")
            .set_json(&payload),
        &config,
        &root_token,
    )
    .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), StatusCode::BAD_REQUEST);
    let body: serde_json::Value = test::read_body_json(resp).await;
    assert!(body["message"].as_str().unwrap().contains("full_name"));
}

#[actix_rt::test]
async fn create_victim_with_invalid_cpf_format_fails() {
    let pool = test_helpers::setup_test_db().await;
    test_helpers::clean_database(&pool).await;

    let config = test_helpers::build_test_config();
    let app = test_helpers::create_full_test_app(pool.clone(), config.clone()).await;

    let city = db_fixtures::insert_city(&pool, "Test City").await;

    let root_claims = test_helpers::build_root_claims();
    let root_token = test_helpers::generate_jwt(&root_claims, &config.jwt_secret);

    let payload = serde_json::json!({
        "full_name": "Victim Invalid CPF",
        "cpf": "123-456-789-00",
        "birth_date": null,
        "city_id": city,
        "phones": null,
        "addresses": null,
        "education_level": null,
        "occupation": null,
        "has_children": "No",
        "children_count": null,
        "special_needs_type": null,
        "uses_alcohol": false,
        "uses_drugs": false,
        "psychiatric_issues_type": null,
    });

    let req = test_helpers::with_auth_headers(
        test::TestRequest::post()
            .uri("/api/v1/victims")
            .set_json(&payload),
        &config,
        &root_token,
    )
    .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), StatusCode::BAD_REQUEST);
    let body: serde_json::Value = test::read_body_json(resp).await;
    assert!(body["message"].as_str().unwrap().contains("cpf must match the format"));
}

#[actix_rt::test]
async fn create_victim_with_invalid_cpf_digits_fails() {
    let pool = test_helpers::setup_test_db().await;
    test_helpers::clean_database(&pool).await;

    let config = test_helpers::build_test_config();
    let app = test_helpers::create_full_test_app(pool.clone(), config.clone()).await;

    let city = db_fixtures::insert_city(&pool, "Test City").await;

    let root_claims = test_helpers::build_root_claims();
    let root_token = test_helpers::generate_jwt(&root_claims, &config.jwt_secret);

    let payload = serde_json::json!({
        "full_name": "Victim Invalid CPF Digits",
        "cpf": "111.111.111-11",
        "birth_date": null,
        "city_id": city,
        "phones": null,
        "addresses": null,
        "education_level": null,
        "occupation": null,
        "has_children": "No",
        "children_count": null,
        "special_needs_type": null,
        "uses_alcohol": false,
        "uses_drugs": false,
        "psychiatric_issues_type": null,
    });

    let req = test_helpers::with_auth_headers(
        test::TestRequest::post()
            .uri("/api/v1/victims")
            .set_json(&payload),
        &config,
        &root_token,
    )
    .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), StatusCode::BAD_REQUEST);
    let body: serde_json::Value = test::read_body_json(resp).await;
    assert!(body["message"].as_str().unwrap().contains("cpf has invalid check digits"));
}

#[actix_rt::test]
async fn create_victim_with_duplicate_cpf_fails() {
    let pool = test_helpers::setup_test_db().await;
    test_helpers::clean_database(&pool).await;

    let config = test_helpers::build_test_config();
    let app = test_helpers::create_full_test_app(pool.clone(), config.clone()).await;

    let city = db_fixtures::insert_city(&pool, "Test City").await;

    let root_claims = test_helpers::build_root_claims();
    let root_token = test_helpers::generate_jwt(&root_claims, &config.jwt_secret);

    let payload1 = serde_json::json!({
        "full_name": "Victim One",
        "cpf": "529.982.247-25",
        "birth_date": null,
        "city_id": city,
        "phones": null,
        "addresses": null,
        "education_level": null,
        "occupation": null,
        "has_children": "No",
        "children_count": null,
        "special_needs_type": null,
        "uses_alcohol": false,
        "uses_drugs": false,
        "psychiatric_issues_type": null,
    });

    let req1 = test_helpers::with_auth_headers(
        test::TestRequest::post()
            .uri("/api/v1/victims")
            .set_json(&payload1),
        &config,
        &root_token,
    )
    .to_request();

    let resp1 = test::call_service(&app, req1).await;
    assert_eq!(resp1.status(), StatusCode::CREATED);

    // Attempt to create another victim with the same CPF
    let payload2 = serde_json::json!({
        "full_name": "Victim Two",
        "cpf": "529.982.247-25",
        "birth_date": null,
        "city_id": city,
        "phones": null,
        "addresses": null,
        "education_level": null,
        "occupation": null,
        "has_children": "No",
        "children_count": null,
        "special_needs_type": null,
        "uses_alcohol": false,
        "uses_drugs": false,
        "psychiatric_issues_type": null,
    });

    let req2 = test_helpers::with_auth_headers(
        test::TestRequest::post()
            .uri("/api/v1/victims")
            .set_json(&payload2),
        &config,
        &root_token,
    )
    .to_request();

    let resp2 = test::call_service(&app, req2).await;
    assert_eq!(resp2.status(), StatusCode::CONFLICT);
    let body: serde_json::Value = test::read_body_json(resp2).await;
    assert!(body["message"].as_str().unwrap().contains("A victim with this CPF already exists"));
}

#[actix_rt::test]
async fn create_victim_without_city_id_uses_residential_address_city() {
    let pool = test_helpers::setup_test_db().await;
    test_helpers::clean_database(&pool).await;

    let config = test_helpers::build_test_config();
    let app = test_helpers::create_full_test_app(pool.clone(), config.clone()).await;

    let city = db_fixtures::insert_city(&pool, "City Residential").await;

    let root_claims = test_helpers::build_root_claims();
    let root_token = test_helpers::generate_jwt(&root_claims, &config.jwt_secret);

    let payload = serde_json::json!({
        "full_name": "Victim Residential",
        "cpf": null,
        "birth_date": null,
        "city_id": null,
        "phones": null,
        "addresses": [{
            "street": "Rua A",
            "number": "10",
            "district": "Centro",
            "city_id": city,
            "zip_code": "12345-000",
            "complement": null,
            "address_type": "Residential"
        }],
        "education_level": null,
        "occupation": null,
        "has_children": "No",
        "children_count": null,
        "special_needs_type": null,
        "uses_alcohol": false,
        "uses_drugs": false,
        "psychiatric_issues_type": null,
    });

    let req = test_helpers::with_auth_headers(
        test::TestRequest::post()
            .uri("/api/v1/victims")
            .set_json(&payload),
        &config,
        &root_token,
    )
    .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), StatusCode::CREATED);
    let body: serde_json::Value = test::read_body_json(resp).await;
    assert_eq!(body["data"]["city_id"].as_str().unwrap(), city.to_string());
}

#[actix_rt::test]
async fn create_victim_without_city_id_uses_work_address_city() {
    let pool = test_helpers::setup_test_db().await;
    test_helpers::clean_database(&pool).await;

    let config = test_helpers::build_test_config();
    let app = test_helpers::create_full_test_app(pool.clone(), config.clone()).await;

    let city = db_fixtures::insert_city(&pool, "City Work").await;

    let root_claims = test_helpers::build_root_claims();
    let root_token = test_helpers::generate_jwt(&root_claims, &config.jwt_secret);

    let payload = serde_json::json!({
        "full_name": "Victim Work",
        "cpf": null,
        "birth_date": null,
        "city_id": null,
        "phones": null,
        "addresses": [{
            "street": "Rua B",
            "number": "20",
            "district": "Bairro",
            "city_id": city,
            "zip_code": "54321-000",
            "complement": null,
            "address_type": "Work"
        }],
        "education_level": null,
        "occupation": null,
        "has_children": "No",
        "children_count": null,
        "special_needs_type": null,
        "uses_alcohol": false,
        "uses_drugs": false,
        "psychiatric_issues_type": null,
    });

    let req = test_helpers::with_auth_headers(
        test::TestRequest::post()
            .uri("/api/v1/victims")
            .set_json(&payload),
        &config,
        &root_token,
    )
    .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), StatusCode::CREATED);
    let body: serde_json::Value = test::read_body_json(resp).await;
    assert_eq!(body["data"]["city_id"].as_str().unwrap(), city.to_string());
}

#[actix_rt::test]
async fn create_victim_without_city_id_and_without_residential_or_work_address_fails() {
    let pool = test_helpers::setup_test_db().await;
    test_helpers::clean_database(&pool).await;

    let config = test_helpers::build_test_config();
    let app = test_helpers::create_full_test_app(pool.clone(), config.clone()).await;

    let city = db_fixtures::insert_city(&pool, "City Other").await;

    let root_claims = test_helpers::build_root_claims();
    let root_token = test_helpers::generate_jwt(&root_claims, &config.jwt_secret);

    let payload = serde_json::json!({
        "full_name": "Victim Other",
        "cpf": null,
        "birth_date": null,
        "city_id": null,
        "phones": null,
        "addresses": [{
            "street": "Rua C",
            "number": "30",
            "district": "Centro",
            "city_id": city,
            "zip_code": "00000-000",
            "complement": null,
            "address_type": "Other"
        }],
        "education_level": null,
        "occupation": null,
        "has_children": "No",
        "children_count": null,
        "special_needs_type": null,
        "uses_alcohol": false,
        "uses_drugs": false,
        "psychiatric_issues_type": null,
    });

    let req = test_helpers::with_auth_headers(
        test::TestRequest::post()
            .uri("/api/v1/victims")
            .set_json(&payload),
        &config,
        &root_token,
    )
    .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), StatusCode::BAD_REQUEST);
    let body: serde_json::Value = test::read_body_json(resp).await;
    assert!(body["message"].as_str().unwrap().contains("no Residential or Work address provided"));
}

#[actix_rt::test]
async fn update_victim_with_empty_full_name_fails() {
    let pool = test_helpers::setup_test_db().await;
    test_helpers::clean_database(&pool).await;

    let config = test_helpers::build_test_config();
    let app = test_helpers::create_full_test_app(pool.clone(), config.clone()).await;

    let city = db_fixtures::insert_city(&pool, "Test City").await;
    let victim_id = db_fixtures::insert_victim(&pool, "Original Name", city).await;

    let root_claims = test_helpers::build_root_claims();
    let root_token = test_helpers::generate_jwt(&root_claims, &config.jwt_secret);

    let update_payload = serde_json::json!({
        "full_name": "",
        "cpf": null,
        "birth_date": null,
        "city_id": city,
        "phones": null,
        "addresses": null,
        "education_level": null,
        "occupation": null,
        "has_children": "No",
        "children_count": null,
        "special_needs_type": null,
        "uses_alcohol": false,
        "uses_drugs": false,
        "psychiatric_issues_type": null,
    });

    let req = test_helpers::with_auth_headers(
        test::TestRequest::put()
            .uri(&format!("/api/v1/victims/{}", victim_id))
            .set_json(&update_payload),
        &config,
        &root_token,
    )
    .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), StatusCode::BAD_REQUEST);
    let body: serde_json::Value = test::read_body_json(resp).await;
    assert!(body["message"].as_str().unwrap().contains("full_name"));
}

#[actix_rt::test]
async fn update_victim_change_city_requires_permission_on_both_cities() {
    let pool = test_helpers::setup_test_db().await;
    test_helpers::clean_database(&pool).await;

    let config = test_helpers::build_test_config();
    let app = test_helpers::create_full_test_app(pool.clone(), config.clone()).await;

    let city_a = db_fixtures::insert_city(&pool, "Cidade A").await;
    let city_b = db_fixtures::insert_city(&pool, "Cidade B").await;
    let victim_id = db_fixtures::insert_victim(&pool, "Vitima A", city_a).await;

    // CITY_ADMIN for city_a only
    let admin_a_claims = test_helpers::build_city_admin_claims(city_a);
    let admin_a_token = test_helpers::generate_jwt(&admin_a_claims, &config.jwt_secret);

    // Try to update victim to city_b (admin doesn't have permission there)
    let update_payload = serde_json::json!({
        "full_name": "Vitima Updated",
        "cpf": null,
        "birth_date": null,
        "city_id": city_b,
        "phones": null,
        "addresses": null,
        "education_level": null,
        "occupation": null,
        "has_children": "No",
        "children_count": null,
        "special_needs_type": null,
        "uses_alcohol": false,
        "uses_drugs": false,
        "psychiatric_issues_type": null,
    });

    let req = test_helpers::with_auth_headers(
        test::TestRequest::put()
            .uri(&format!("/api/v1/victims/{}", victim_id))
            .set_json(&update_payload),
        &config,
        &admin_a_token,
    )
    .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), StatusCode::FORBIDDEN);
}

#[actix_rt::test]
async fn update_nonexistent_victim_returns_404() {
    let pool = test_helpers::setup_test_db().await;
    test_helpers::clean_database(&pool).await;

    let config = test_helpers::build_test_config();
    let app = test_helpers::create_full_test_app(pool.clone(), config.clone()).await;

    let city = db_fixtures::insert_city(&pool, "Test City").await;

    let root_claims = test_helpers::build_root_claims();
    let root_token = test_helpers::generate_jwt(&root_claims, &config.jwt_secret);

    let random_id = Uuid::new_v4();

    let update_payload = serde_json::json!({
        "full_name": "Updated Name",
        "cpf": null,
        "birth_date": null,
        "city_id": city,
        "phones": null,
        "addresses": null,
        "education_level": null,
        "occupation": null,
        "has_children": "No",
        "children_count": null,
        "special_needs_type": null,
        "uses_alcohol": false,
        "uses_drugs": false,
        "psychiatric_issues_type": null,
    });

    let req = test_helpers::with_auth_headers(
        test::TestRequest::put()
            .uri(&format!("/api/v1/victims/{}", random_id))
            .set_json(&update_payload),
        &config,
        &root_token,
    )
    .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), StatusCode::NOT_FOUND);
}
