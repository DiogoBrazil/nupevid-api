use actix_web::{http::StatusCode, test};

use crate::common::{fixtures, test_helpers};
use nupevid_api::core::value_objects::profiles::Profile;
use nupevid_api::core::value_objects::ranks::Rank;

#[actix_rt::test]
async fn test_create_user_success() {
    let pool = test_helpers::setup_test_db().await;
    test_helpers::clean_database(&pool).await;
    let config = test_helpers::build_test_config();
    let app = test_helpers::create_full_test_app(pool.clone(), config.clone()).await;

    // Generate ROOT token for authentication
    let claims = test_helpers::build_root_claims();
    let token = test_helpers::generate_jwt(&claims, &config.jwt_secret);

    let user = fixtures::valid_create_user();
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
    assert_eq!(body["data"]["email"].as_str().unwrap(), user.email);
    assert_eq!(body["data"]["full_name"].as_str().unwrap(), user.full_name);
    assert!(body["data"]["password"].is_null()); // Password should not be returned
}

#[actix_rt::test]
async fn test_create_user_duplicate_email() {
    let pool = test_helpers::setup_test_db().await;
    test_helpers::clean_database(&pool).await;
    let config = test_helpers::build_test_config();
    let app = test_helpers::create_full_test_app(pool.clone(), config.clone()).await;

    let claims = test_helpers::build_root_claims();
    let token = test_helpers::generate_jwt(&claims, &config.jwt_secret);

    let user = fixtures::valid_create_user();

    // Create first user
    let req1 = test_helpers::with_auth_headers(
        test::TestRequest::post()
            .uri("/api/v1/users")
            .set_json(&user),
        &config,
        &token,
    )
    .to_request();
    let resp1 = test::call_service(&app, req1).await;
    assert_eq!(resp1.status(), StatusCode::CREATED);

    // Try to create user with same email
    let req2 = test_helpers::with_auth_headers(
        test::TestRequest::post()
            .uri("/api/v1/users")
            .set_json(&user),
        &config,
        &token,
    )
    .to_request();
    let resp2 = test::call_service(&app, req2).await;
    assert_eq!(resp2.status(), StatusCode::BAD_REQUEST);

    let body: serde_json::Value = test::read_body_json(resp2).await;
    assert_eq!(body["status_code"].as_u64().unwrap(), 400);
    assert!(body["message"].as_str().unwrap().contains("email"));
    assert!(body["message"].as_str().unwrap().contains("already exists"));
}

#[actix_rt::test]
async fn test_create_user_duplicate_registration() {
    let pool = test_helpers::setup_test_db().await;
    test_helpers::clean_database(&pool).await;
    let config = test_helpers::build_test_config();
    let app = test_helpers::create_full_test_app(pool.clone(), config.clone()).await;

    let claims = test_helpers::build_root_claims();
    let token = test_helpers::generate_jwt(&claims, &config.jwt_secret);

    let user1 = fixtures::valid_create_user();
    let mut user2 = fixtures::valid_create_user();
    user2.email = "different@test.com".to_string(); // Different email, same registration

    // Create first user
    let req1 = test_helpers::with_auth_headers(
        test::TestRequest::post()
            .uri("/api/v1/users")
            .set_json(&user1),
        &config,
        &token,
    )
    .to_request();
    let resp1 = test::call_service(&app, req1).await;
    assert_eq!(resp1.status(), StatusCode::CREATED);

    // Try to create user with same registration
    let req2 = test_helpers::with_auth_headers(
        test::TestRequest::post()
            .uri("/api/v1/users")
            .set_json(&user2),
        &config,
        &token,
    )
    .to_request();
    let resp2 = test::call_service(&app, req2).await;
    assert_eq!(resp2.status(), StatusCode::BAD_REQUEST);

    let body: serde_json::Value = test::read_body_json(resp2).await;
    assert_eq!(body["status_code"].as_u64().unwrap(), 400);
    assert!(body["message"].as_str().unwrap().contains("registration"));
    assert!(body["message"].as_str().unwrap().contains("already exists"));
}

#[actix_rt::test]
async fn test_create_user_invalid_email() {
    let pool = test_helpers::setup_test_db().await;
    test_helpers::clean_database(&pool).await;
    let config = test_helpers::build_test_config();
    let app = test_helpers::create_full_test_app(pool.clone(), config.clone()).await;

    let claims = test_helpers::build_root_claims();
    let token = test_helpers::generate_jwt(&claims, &config.jwt_secret);

    let user = fixtures::create_user_with_invalid_email();
    let req = test_helpers::with_auth_headers(
        test::TestRequest::post()
            .uri("/api/v1/users")
            .set_json(&user),
        &config,
        &token,
    )
    .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), StatusCode::BAD_REQUEST);

    let body: serde_json::Value = test::read_body_json(resp).await;
    assert_eq!(body["status_code"].as_u64().unwrap(), 400);
    assert!(
        body["message"]
            .as_str()
            .unwrap()
            .contains("not a valid email")
    );
}

#[actix_rt::test]
async fn test_create_user_invalid_registration() {
    let pool = test_helpers::setup_test_db().await;
    test_helpers::clean_database(&pool).await;
    let config = test_helpers::build_test_config();
    let app = test_helpers::create_full_test_app(pool.clone(), config.clone()).await;

    let claims = test_helpers::build_root_claims();
    let token = test_helpers::generate_jwt(&claims, &config.jwt_secret);

    let user = fixtures::create_user_with_invalid_registration();
    let req = test_helpers::with_auth_headers(
        test::TestRequest::post()
            .uri("/api/v1/users")
            .set_json(&user),
        &config,
        &token,
    )
    .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), StatusCode::BAD_REQUEST);

    let body: serde_json::Value = test::read_body_json(resp).await;
    assert_eq!(body["status_code"].as_u64().unwrap(), 400);
    assert!(
        body["message"]
            .as_str()
            .unwrap()
            .contains("invalid registration")
    );
    assert!(body["message"].as_str().unwrap().contains("1000"));
}

#[actix_rt::test]
async fn test_create_user_empty_fields() {
    let pool = test_helpers::setup_test_db().await;
    test_helpers::clean_database(&pool).await;
    let config = test_helpers::build_test_config();
    let app = test_helpers::create_full_test_app(pool.clone(), config.clone()).await;

    let claims = test_helpers::build_root_claims();
    let token = test_helpers::generate_jwt(&claims, &config.jwt_secret);

    let user = fixtures::create_user_with_empty_fields();
    let req = test_helpers::with_auth_headers(
        test::TestRequest::post()
            .uri("/api/v1/users")
            .set_json(&user),
        &config,
        &token,
    )
    .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), StatusCode::BAD_REQUEST);

    let body: serde_json::Value = test::read_body_json(resp).await;
    assert_eq!(body["status_code"].as_u64().unwrap(), 400);
    assert!(
        body["message"]
            .as_str()
            .unwrap()
            .contains("cannot be empty")
    );
}

#[actix_rt::test]
async fn test_create_user_unauthorized_without_token() {
    let pool = test_helpers::setup_test_db().await;
    test_helpers::clean_database(&pool).await;
    let config = test_helpers::build_test_config();
    let app = test_helpers::create_full_test_app(pool.clone(), config.clone()).await;

    let user = fixtures::valid_create_user();
    let req = test::TestRequest::post()
        .uri("/api/v1/users")
        .insert_header(("api_key", config.api_key.clone()))
        .set_json(&user)
        .to_request();

    // The middleware returns an error, so we use try_call_service
    let result = test::try_call_service(&app, req).await;
    assert!(result.is_err(), "Expected unauthorized error");
}

#[actix_rt::test]
async fn test_city_user_cannot_create_users() {
    let pool = test_helpers::setup_test_db().await;
    test_helpers::clean_database(&pool).await;
    let config = test_helpers::build_test_config();
    let app = test_helpers::create_full_test_app(pool.clone(), config.clone()).await;

    // First create a city
    let root_claims = test_helpers::build_root_claims();
    let root_token = test_helpers::generate_jwt(&root_claims, &config.jwt_secret);

    let city_payload = serde_json::json!({
        "name": "PORTO VELHO",
        "state": "RO",
        "battalion": "1ºBPM"
    });
    let create_city_req = test_helpers::with_auth_headers(
        test::TestRequest::post()
            .uri("/api/v1/cities")
            .set_json(&city_payload),
        &config,
        &root_token,
    )
    .to_request();
    let city_resp = test::call_service(&app, create_city_req).await;
    let city_body: serde_json::Value = test::read_body_json(city_resp).await;
    let city_id: uuid::Uuid = city_body["data"]["id"].as_str().unwrap().parse().unwrap();

    // Create CITY_USER claims
    let city_user_claims = nupevid_api::core::entities::auth::ClaimsToUserToken {
        id: uuid::Uuid::new_v4().to_string(),
        exp: std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs() as usize
            + 3600,
        rank: Rank::SdPm,
        registration: "100022000".to_string(),
        full_name: "City User".to_string(),
        profile: Profile::CityUser,
        email: "city.user@test.com".to_string(),
        city_id: Some(city_id.to_string()),
    };
    let city_user_token = test_helpers::generate_jwt(&city_user_claims, &config.jwt_secret);

    // Try to create a user as CITY_USER
    let new_user = serde_json::json!({
        "rank": "SD PM",
        "registration": "100099999",
        "full_name": "New User",
        "profile": "CITY_USER",
        "email": "new.user@test.com",
        "password": "senha123",
        "city_id": city_id
    });

    let req = test_helpers::with_auth_headers(
        test::TestRequest::post()
            .uri("/api/v1/users")
            .set_json(&new_user),
        &config,
        &city_user_token,
    )
    .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), StatusCode::FORBIDDEN);

    let body: serde_json::Value = test::read_body_json(resp).await;
    assert!(body["message"].as_str().unwrap().contains("CITY_USER"));
}

#[actix_rt::test]
async fn test_city_admin_cannot_create_root() {
    let pool = test_helpers::setup_test_db().await;
    test_helpers::clean_database(&pool).await;
    let config = test_helpers::build_test_config();
    let app = test_helpers::create_full_test_app(pool.clone(), config.clone()).await;

    // First create a city
    let root_claims = test_helpers::build_root_claims();
    let root_token = test_helpers::generate_jwt(&root_claims, &config.jwt_secret);

    let city_payload = serde_json::json!({
        "name": "ARIQUEMES",
        "state": "RO",
        "battalion": "1ºBPM"
    });
    let create_city_req = test_helpers::with_auth_headers(
        test::TestRequest::post()
            .uri("/api/v1/cities")
            .set_json(&city_payload),
        &config,
        &root_token,
    )
    .to_request();
    let city_resp = test::call_service(&app, create_city_req).await;
    let city_body: serde_json::Value = test::read_body_json(city_resp).await;
    let city_id: uuid::Uuid = city_body["data"]["id"].as_str().unwrap().parse().unwrap();

    // Create CITY_ADMIN claims
    let city_admin_claims = test_helpers::build_city_admin_claims(city_id);
    let city_admin_token = test_helpers::generate_jwt(&city_admin_claims, &config.jwt_secret);

    // Try to create a ROOT user as CITY_ADMIN
    let new_root_user = serde_json::json!({
        "rank": "CAP PM",
        "registration": "100088888",
        "full_name": "New Root",
        "profile": "ROOT",
        "email": "new.root@test.com",
        "password": "senha123"
    });

    let req = test_helpers::with_auth_headers(
        test::TestRequest::post()
            .uri("/api/v1/users")
            .set_json(&new_root_user),
        &config,
        &city_admin_token,
    )
    .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), StatusCode::FORBIDDEN);

    let body: serde_json::Value = test::read_body_json(resp).await;
    assert!(body["message"].as_str().unwrap().contains("ROOT"));
}

#[actix_rt::test]
async fn test_city_admin_cannot_create_city_admin() {
    let pool = test_helpers::setup_test_db().await;
    test_helpers::clean_database(&pool).await;
    let config = test_helpers::build_test_config();
    let app = test_helpers::create_full_test_app(pool.clone(), config.clone()).await;

    // First create a city
    let root_claims = test_helpers::build_root_claims();
    let root_token = test_helpers::generate_jwt(&root_claims, &config.jwt_secret);

    let city_payload = serde_json::json!({
        "name": "CACOAL",
        "state": "RO",
        "battalion": "1ºBPM"
    });
    let create_city_req = test_helpers::with_auth_headers(
        test::TestRequest::post()
            .uri("/api/v1/cities")
            .set_json(&city_payload),
        &config,
        &root_token,
    )
    .to_request();
    let city_resp = test::call_service(&app, create_city_req).await;
    let city_body: serde_json::Value = test::read_body_json(city_resp).await;
    let city_id: uuid::Uuid = city_body["data"]["id"].as_str().unwrap().parse().unwrap();

    // Create CITY_ADMIN claims
    let city_admin_claims = test_helpers::build_city_admin_claims(city_id);
    let city_admin_token = test_helpers::generate_jwt(&city_admin_claims, &config.jwt_secret);

    // Try to create another CITY_ADMIN as CITY_ADMIN
    let new_city_admin = serde_json::json!({
        "rank": "CAP PM",
        "registration": "100077777",
        "full_name": "New City Admin",
        "profile": "CITY_ADMIN",
        "email": "new.city.admin@test.com",
        "password": "senha123",
        "city_id": city_id
    });

    let req = test_helpers::with_auth_headers(
        test::TestRequest::post()
            .uri("/api/v1/users")
            .set_json(&new_city_admin),
        &config,
        &city_admin_token,
    )
    .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), StatusCode::FORBIDDEN);

    let body: serde_json::Value = test::read_body_json(resp).await;
    assert!(body["message"].as_str().unwrap().contains("CITY_ADMIN"));
}

#[actix_rt::test]
async fn test_city_admin_city_id_in_body_is_ignored() {
    let pool = test_helpers::setup_test_db().await;
    test_helpers::clean_database(&pool).await;
    let config = test_helpers::build_test_config();
    let app = test_helpers::create_full_test_app(pool.clone(), config.clone()).await;

    // Create two cities as ROOT
    let root_claims = test_helpers::build_root_claims();
    let root_token = test_helpers::generate_jwt(&root_claims, &config.jwt_secret);

    let city1_payload =
        serde_json::json!({ "name": "VILHENA", "state": "RO", "battalion": "1ºBPM" });
    let create_city1_req = test_helpers::with_auth_headers(
        test::TestRequest::post()
            .uri("/api/v1/cities")
            .set_json(&city1_payload),
        &config,
        &root_token,
    )
    .to_request();
    let city1_resp = test::call_service(&app, create_city1_req).await;
    let city1_body: serde_json::Value = test::read_body_json(city1_resp).await;
    let city1_id: uuid::Uuid = city1_body["data"]["id"].as_str().unwrap().parse().unwrap();

    let city2_payload = serde_json::json!({ "name": "JARU", "state": "RO", "battalion": "2ºBPM" });
    let create_city2_req = test_helpers::with_auth_headers(
        test::TestRequest::post()
            .uri("/api/v1/cities")
            .set_json(&city2_payload),
        &config,
        &root_token,
    )
    .to_request();
    let city2_resp = test::call_service(&app, create_city2_req).await;
    let city2_body: serde_json::Value = test::read_body_json(city2_resp).await;
    let city2_id: uuid::Uuid = city2_body["data"]["id"].as_str().unwrap().parse().unwrap();

    // CITY_ADMIN of city1 tries to send city2_id in body - should be ignored
    let city_admin_claims = test_helpers::build_city_admin_claims(city1_id);
    let city_admin_token = test_helpers::generate_jwt(&city_admin_claims, &config.jwt_secret);

    let new_user = serde_json::json!({
        "rank": "SD PM",
        "registration": "100066666",
        "full_name": "User With Wrong City",
        "profile": "CITY_USER",
        "email": "wrong.city.user@test.com",
        "password": "senha123",
        "city_id": city2_id  // This should be IGNORED - user should be created in city1
    });

    let req = test_helpers::with_auth_headers(
        test::TestRequest::post()
            .uri("/api/v1/users")
            .set_json(&new_user),
        &config,
        &city_admin_token,
    )
    .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), StatusCode::CREATED);

    let body: serde_json::Value = test::read_body_json(resp).await;
    // Verify user was created in CITY_ADMIN's city (city1), NOT in city2 from body
    assert_eq!(
        body["data"]["city_id"].as_str().unwrap(),
        city1_id.to_string()
    );
    assert_ne!(
        body["data"]["city_id"].as_str().unwrap(),
        city2_id.to_string()
    );
}

#[actix_rt::test]
async fn test_city_admin_can_create_city_user_in_own_city() {
    let pool = test_helpers::setup_test_db().await;
    test_helpers::clean_database(&pool).await;
    let config = test_helpers::build_test_config();
    let app = test_helpers::create_full_test_app(pool.clone(), config.clone()).await;

    // Create a city as ROOT
    let root_claims = test_helpers::build_root_claims();
    let root_token = test_helpers::generate_jwt(&root_claims, &config.jwt_secret);

    let city_payload =
        serde_json::json!({ "name": "GUAJARÁ-MIRIM", "state": "RO", "battalion": "1ºBPM" });
    let create_city_req = test_helpers::with_auth_headers(
        test::TestRequest::post()
            .uri("/api/v1/cities")
            .set_json(&city_payload),
        &config,
        &root_token,
    )
    .to_request();
    let city_resp = test::call_service(&app, create_city_req).await;
    let city_body: serde_json::Value = test::read_body_json(city_resp).await;
    let city_id: uuid::Uuid = city_body["data"]["id"].as_str().unwrap().parse().unwrap();

    // CITY_ADMIN creates CITY_USER - city_id NOT provided in body (should be auto-filled from token)
    let city_admin_claims = test_helpers::build_city_admin_claims(city_id);
    let city_admin_token = test_helpers::generate_jwt(&city_admin_claims, &config.jwt_secret);

    let new_user = serde_json::json!({
        "rank": "SD PM",
        "registration": "100055555",
        "full_name": "New City User",
        "profile": "CITY_USER",
        "email": "new.city.user@test.com",
        "password": "senha123"
        // city_id is NOT provided - should be auto-filled from CITY_ADMIN's token
    });

    let req = test_helpers::with_auth_headers(
        test::TestRequest::post()
            .uri("/api/v1/users")
            .set_json(&new_user),
        &config,
        &city_admin_token,
    )
    .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), StatusCode::CREATED);

    let body: serde_json::Value = test::read_body_json(resp).await;
    assert_eq!(
        body["data"]["email"].as_str().unwrap(),
        "new.city.user@test.com"
    );
    assert_eq!(body["data"]["profile"].as_str().unwrap(), "CITY_USER");
    // Verify city_id was auto-filled from CITY_ADMIN's token
    assert_eq!(
        body["data"]["city_id"].as_str().unwrap(),
        city_id.to_string()
    );
}
