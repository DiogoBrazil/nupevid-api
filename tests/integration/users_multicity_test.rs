use actix_web::{http::StatusCode, test};

use crate::common::{db_fixtures, fixtures, test_helpers};
use nupevid_api::core::value_objects::profiles::Profile;

#[actix_rt::test]
async fn create_city_admin_success_with_city_id() {
    let pool = test_helpers::setup_test_db().await;
    test_helpers::clean_database(&pool).await;
    let config = test_helpers::build_test_config();
    let app = test_helpers::create_full_test_app(pool.clone(), config.clone()).await;

    let claims = test_helpers::build_root_claims();
    let token = test_helpers::generate_jwt(&claims, &config.jwt_secret);

    let city_id = db_fixtures::insert_city(&pool, "Cidade Admin").await;

    let mut user = fixtures::valid_create_user();
    user.profile = Profile::CityAdmin;
    user.city_id = Some(city_id);

    let req = test_helpers::with_auth_headers(
        test::TestRequest::post()
            .uri("/api/v1/users")
            .set_json(&user),
        &config,
        &token,
    )
    .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), StatusCode::CREATED);

    let body: serde_json::Value = test::read_body_json(resp).await;
    assert_eq!(body["status"].as_u64().unwrap(), 201);
    assert_eq!(body["data"]["profile"].as_str().unwrap(), "CITY_ADMIN");
    assert_eq!(
        body["data"]["city_id"].as_str().unwrap(),
        city_id.to_string()
    );
}

#[actix_rt::test]
async fn create_second_city_admin_same_city_should_fail() {
    let pool = test_helpers::setup_test_db().await;
    test_helpers::clean_database(&pool).await;
    let config = test_helpers::build_test_config();
    let app = test_helpers::create_full_test_app(pool.clone(), config.clone()).await;

    let claims = test_helpers::build_root_claims();
    let token = test_helpers::generate_jwt(&claims, &config.jwt_secret);

    let city_id = db_fixtures::insert_city(&pool, "Cidade Unica").await;

    let mut admin1 = fixtures::valid_create_user();
    admin1.profile = Profile::CityAdmin;
    admin1.city_id = Some(city_id);

    let mut admin2 = fixtures::valid_create_user_2();
    admin2.profile = Profile::CityAdmin;
    admin2.city_id = Some(city_id);

    // First admin should succeed
    let req1 = test_helpers::with_auth_headers(
        test::TestRequest::post()
            .uri("/api/v1/users")
            .set_json(&admin1),
        &config,
        &token,
    )
    .to_request();
    let resp1 = test::call_service(&app, req1).await;
    assert_eq!(resp1.status(), StatusCode::CREATED);

    // Second admin for same city must fail with 400
    let req2 = test_helpers::with_auth_headers(
        test::TestRequest::post()
            .uri("/api/v1/users")
            .set_json(&admin2),
        &config,
        &token,
    )
    .to_request();
    let resp2 = test::call_service(&app, req2).await;
    assert_eq!(resp2.status(), StatusCode::BAD_REQUEST);

    let body: serde_json::Value = test::read_body_json(resp2).await;
    assert_eq!(body["status_code"].as_u64().unwrap(), 400);
    assert!(
        body["message"]
            .as_str()
            .unwrap()
            .contains("CITY_ADMIN already exists")
    );
}

#[actix_rt::test]
async fn update_user_to_city_admin_without_city_id_should_fail() {
    let pool = test_helpers::setup_test_db().await;
    test_helpers::clean_database(&pool).await;
    let config = test_helpers::build_test_config();
    let app = test_helpers::create_full_test_app(pool.clone(), config.clone()).await;

    let claims = test_helpers::build_root_claims();
    let token = test_helpers::generate_jwt(&claims, &config.jwt_secret);

    // Create a normal user first
    let create_user = fixtures::valid_create_user();
    let create_req = test_helpers::with_auth_headers(
        test::TestRequest::post()
            .uri("/api/v1/users")
            .set_json(&create_user),
        &config,
        &token,
    )
    .to_request();
    let create_resp = test::call_service(&app, create_req).await;
    assert_eq!(create_resp.status(), StatusCode::CREATED);
    let create_body: serde_json::Value = test::read_body_json(create_resp).await;
    let user_id = create_body["data"]["id"].as_str().unwrap();

    // Try to update to CITY_ADMIN without city_id
    let mut update_data = fixtures::valid_update_user();
    update_data.profile = Profile::CityAdmin;
    update_data.city_id = None;

    let update_req = test_helpers::with_auth_headers(
        test::TestRequest::put()
            .uri(&format!("/api/v1/users/{}", user_id))
            .set_json(&update_data),
        &config,
        &token,
    )
    .to_request();

    let update_resp = test::call_service(&app, update_req).await;
    assert_eq!(update_resp.status(), StatusCode::BAD_REQUEST);

    let body: serde_json::Value = test::read_body_json(update_resp).await;
    assert_eq!(body["status_code"].as_u64().unwrap(), 400);
    assert!(
        body["message"]
            .as_str()
            .unwrap()
            .contains("city_id is required")
    );
}

#[actix_rt::test]
async fn update_user_creating_duplicate_city_admin_should_fail() {
    let pool = test_helpers::setup_test_db().await;
    test_helpers::clean_database(&pool).await;
    let config = test_helpers::build_test_config();
    let app = test_helpers::create_full_test_app(pool.clone(), config.clone()).await;

    let claims = test_helpers::build_root_claims();
    let token = test_helpers::generate_jwt(&claims, &config.jwt_secret);

    let city_id = db_fixtures::insert_city(&pool, "Cidade Atualiza").await;

    // First CITY_ADMIN
    let mut admin1 = fixtures::valid_create_user();
    admin1.profile = Profile::CityAdmin;
    admin1.city_id = Some(city_id);
    let req1 = test_helpers::with_auth_headers(
        test::TestRequest::post()
            .uri("/api/v1/users")
            .set_json(&admin1),
        &config,
        &token,
    )
    .to_request();
    let resp1 = test::call_service(&app, req1).await;
    assert_eq!(resp1.status(), StatusCode::CREATED);

    // Second user (non-admin) in same city
    let mut user2 = fixtures::valid_create_user_2();
    user2.city_id = Some(city_id);
    let req2 = test_helpers::with_auth_headers(
        test::TestRequest::post()
            .uri("/api/v1/users")
            .set_json(&user2),
        &config,
        &token,
    )
    .to_request();
    let resp2 = test::call_service(&app, req2).await;
    assert_eq!(resp2.status(), StatusCode::CREATED);
    let body2: serde_json::Value = test::read_body_json(resp2).await;
    let user2_id = body2["data"]["id"].as_str().unwrap();

    // Try to promote user2 to CITY_ADMIN for same city
    let mut update_data = fixtures::valid_update_user();
    update_data.profile = Profile::CityAdmin;
    update_data.city_id = Some(city_id);

    let update_req = test_helpers::with_auth_headers(
        test::TestRequest::put()
            .uri(&format!("/api/v1/users/{}", user2_id))
            .set_json(&update_data),
        &config,
        &token,
    )
    .to_request();

    let update_resp = test::call_service(&app, update_req).await;
    assert_eq!(update_resp.status(), StatusCode::BAD_REQUEST);

    let body: serde_json::Value = test::read_body_json(update_resp).await;
    assert_eq!(body["status_code"].as_u64().unwrap(), 400);
    assert!(
        body["message"]
            .as_str()
            .unwrap()
            .contains("CITY_ADMIN already exists")
    );
}
