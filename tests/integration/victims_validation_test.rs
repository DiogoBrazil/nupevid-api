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
        "phone": null,
        "city_id": city,
        "address": null,
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
        "phone": null,
        "city_id": city,
        "address": null,
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
        "phone": null,
        "city_id": city_b,
        "address": null,
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
        "phone": null,
        "city_id": city,
        "address": null,
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
