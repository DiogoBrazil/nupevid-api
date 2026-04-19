use actix_web::{http::StatusCode, test};
use uuid::Uuid;

use crate::common::test_helpers;
use nupevid_api::core::entities::auth::UserClaims;
use nupevid_api::core::value_objects::profiles::Profile;
use nupevid_api::core::value_objects::ranks::Rank;
use std::time::{SystemTime, UNIX_EPOCH};

fn build_user_payload(full_name: &str, registration: &str, email: &str) -> serde_json::Value {
    serde_json::json!({
        "rank": "CAP PM",
        "registration": registration,
        "full_name": full_name,
        "profile": "ROOT",
        "email": email,
        "password": "Senha123!"
    })
}

#[actix_rt::test]
async fn search_users_by_name_returns_matches() {
    let pool = test_helpers::setup_test_db().await;
    test_helpers::clean_database(&pool).await;

    let config = test_helpers::build_test_config();
    let app = test_helpers::create_full_test_app(pool.clone(), config.clone()).await;

    let root_claims = test_helpers::build_root_claims();
    let root_token = test_helpers::generate_jwt(&root_claims, &config.jwt_secret);

    for (name, registration, email) in [
        ("Maria Silva", "100012345", "maria.silva@test.com"),
        ("Carlos Souza", "100012346", "carlos.souza@test.com"),
    ] {
        let payload = build_user_payload(name, registration, email);
        let req = test_helpers::with_auth_headers(
            test::TestRequest::post()
                .uri("/api/v1/users")
                .set_json(&payload),
            &config,
            &root_token,
        )
        .to_request();

        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status(), StatusCode::CREATED);
    }

    let search_req = test_helpers::with_auth_headers(
        test::TestRequest::get().uri("/api/v1/users/search?name=maria"),
        &config,
        &root_token,
    )
    .to_request();

    let search_resp = test::call_service(&app, search_req).await;
    assert_eq!(search_resp.status(), StatusCode::OK);
    let body: serde_json::Value = test::read_body_json(search_resp).await;
    let results = body["data"].as_array().unwrap();
    assert_eq!(results.len(), 1);
    assert_eq!(results[0]["full_name"].as_str().unwrap(), "Maria Silva");
}

#[actix_rt::test]
async fn search_users_by_registration_returns_match() {
    let pool = test_helpers::setup_test_db().await;
    test_helpers::clean_database(&pool).await;

    let config = test_helpers::build_test_config();
    let app = test_helpers::create_full_test_app(pool.clone(), config.clone()).await;

    let root_claims = test_helpers::build_root_claims();
    let root_token = test_helpers::generate_jwt(&root_claims, &config.jwt_secret);

    let payload = build_user_payload("Ana Lima", "100099999", "ana.lima@test.com");
    let create_req = test_helpers::with_auth_headers(
        test::TestRequest::post()
            .uri("/api/v1/users")
            .set_json(&payload),
        &config,
        &root_token,
    )
    .to_request();

    let create_resp = test::call_service(&app, create_req).await;
    assert_eq!(create_resp.status(), StatusCode::CREATED);

    let search_req = test_helpers::with_auth_headers(
        test::TestRequest::get().uri("/api/v1/users/search?registration=100099999"),
        &config,
        &root_token,
    )
    .to_request();

    let search_resp = test::call_service(&app, search_req).await;
    assert_eq!(search_resp.status(), StatusCode::OK);
    let body: serde_json::Value = test::read_body_json(search_resp).await;
    let results = body["data"].as_array().unwrap();
    assert_eq!(results.len(), 1);
    assert_eq!(results[0]["registration"].as_str().unwrap(), "100099999");
}

#[actix_rt::test]
async fn search_users_city_admin_filters_by_city_and_excludes_root() {
    let pool = test_helpers::setup_test_db().await;
    test_helpers::clean_database(&pool).await;

    let config = test_helpers::build_test_config();
    let app = test_helpers::create_full_test_app(pool.clone(), config.clone()).await;

    let root_claims = test_helpers::build_root_claims();
    let root_token = test_helpers::generate_jwt(&root_claims, &config.jwt_secret);

    let city_a_req = test_helpers::with_auth_headers(
        test::TestRequest::post()
            .uri("/api/v1/cities")
            .set_json(&serde_json::json!({
                "name": "PORTO VELHO",
                "state": "RO",
                "battalion": "1ºBPM"
            })),
        &config,
        &root_token,
    )
    .to_request();
    let city_a_resp = test::call_service(&app, city_a_req).await;
    assert_eq!(city_a_resp.status(), StatusCode::CREATED);
    let city_a_body: serde_json::Value = test::read_body_json(city_a_resp).await;
    let city_a_id: Uuid = city_a_body["data"]["id"].as_str().unwrap().parse().unwrap();

    let city_b_req = test_helpers::with_auth_headers(
        test::TestRequest::post()
            .uri("/api/v1/cities")
            .set_json(&serde_json::json!({
                "name": "JI-PARANÁ",
                "state": "RO",
                "battalion": "2ºBPM"
            })),
        &config,
        &root_token,
    )
    .to_request();
    let city_b_resp = test::call_service(&app, city_b_req).await;
    assert_eq!(city_b_resp.status(), StatusCode::CREATED);
    let city_b_body: serde_json::Value = test::read_body_json(city_b_resp).await;
    let city_b_id: Uuid = city_b_body["data"]["id"].as_str().unwrap().parse().unwrap();

    let user_a_payload = serde_json::json!({
        "rank": "CAP PM",
        "registration": "100077771",
        "full_name": "User City A",
        "profile": "CITY_USER",
        "email": "user.city.a@test.com",
        "password": "Senha123!",
        "city_id": city_a_id
    });
    let create_user_a_req = test_helpers::with_auth_headers(
        test::TestRequest::post()
            .uri("/api/v1/users")
            .set_json(&user_a_payload),
        &config,
        &root_token,
    )
    .to_request();
    let create_user_a_resp = test::call_service(&app, create_user_a_req).await;
    assert_eq!(create_user_a_resp.status(), StatusCode::CREATED);

    let user_b_payload = serde_json::json!({
        "rank": "CAP PM",
        "registration": "100077772",
        "full_name": "User City B",
        "profile": "CITY_USER",
        "email": "user.city.b@test.com",
        "password": "Senha123!",
        "city_id": city_b_id
    });
    let create_user_b_req = test_helpers::with_auth_headers(
        test::TestRequest::post()
            .uri("/api/v1/users")
            .set_json(&user_b_payload),
        &config,
        &root_token,
    )
    .to_request();
    let create_user_b_resp = test::call_service(&app, create_user_b_req).await;
    assert_eq!(create_user_b_resp.status(), StatusCode::CREATED);

    let root_user_payload = serde_json::json!({
        "rank": "CEL PM",
        "registration": "100077773",
        "full_name": "Root User",
        "profile": "ROOT",
        "email": "root.user@test.com",
        "password": "Senha123!"
    });
    let create_root_user_req = test_helpers::with_auth_headers(
        test::TestRequest::post()
            .uri("/api/v1/users")
            .set_json(&root_user_payload),
        &config,
        &root_token,
    )
    .to_request();
    let create_root_user_resp = test::call_service(&app, create_root_user_req).await;
    assert_eq!(create_root_user_resp.status(), StatusCode::CREATED);

    let city_admin_payload = serde_json::json!({
        "rank": "CAP PM",
        "registration": "100077774",
        "full_name": "City Admin",
        "profile": "CITY_ADMIN",
        "email": "city.admin@test.com",
        "password": "Senha123!",
        "city_id": city_a_id,
        "permission_policies": {
            "read_users": [city_a_id]
        }
    });
    let create_city_admin_req = test_helpers::with_auth_headers(
        test::TestRequest::post()
            .uri("/api/v1/users")
            .set_json(&city_admin_payload),
        &config,
        &root_token,
    )
    .to_request();
    let create_city_admin_resp = test::call_service(&app, create_city_admin_req).await;
    assert_eq!(create_city_admin_resp.status(), StatusCode::CREATED);
    let city_admin_body: serde_json::Value = test::read_body_json(create_city_admin_resp).await;
    let city_admin_id: Uuid = city_admin_body["data"]["id"]
        .as_str()
        .unwrap()
        .parse()
        .unwrap();

    let city_admin_claims = UserClaims {
        id: city_admin_id.to_string(),
        exp: (SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs() as usize)
            + 3600,
        iss: "nupevid-api".to_string(),
        aud: "nupevid-api".to_string(),
        rank: Rank::CapPm,
        registration: "100077774".to_string(),
        full_name: "City Admin".to_string(),
        profile: Profile::CityAdmin,
        email: "city.admin@test.com".to_string(),
        city_id: Some(city_a_id.to_string()),
    };
    let city_admin_token = test_helpers::generate_jwt(&city_admin_claims, &config.jwt_secret);

    let search_req = test_helpers::with_auth_headers(
        test::TestRequest::get().uri("/api/v1/users/search?name=user"),
        &config,
        &city_admin_token,
    )
    .to_request();
    let search_resp = test::call_service(&app, search_req).await;
    assert_eq!(search_resp.status(), StatusCode::OK);
    let body: serde_json::Value = test::read_body_json(search_resp).await;
    let results = body["data"].as_array().unwrap();
    assert_eq!(results.len(), 1);
    assert_eq!(results[0]["full_name"].as_str().unwrap(), "User City A");
    assert!(
        results
            .iter()
            .all(|u| u["profile"].as_str().unwrap() != "ROOT")
    );
}

#[actix_rt::test]
async fn search_users_rejects_invalid_registration() {
    let pool = test_helpers::setup_test_db().await;
    test_helpers::clean_database(&pool).await;

    let config = test_helpers::build_test_config();
    let app = test_helpers::create_full_test_app(pool.clone(), config.clone()).await;

    let root_claims = test_helpers::build_root_claims();
    let root_token = test_helpers::generate_jwt(&root_claims, &config.jwt_secret);

    let search_req = test_helpers::with_auth_headers(
        test::TestRequest::get().uri("/api/v1/users/search?registration=99999"),
        &config,
        &root_token,
    )
    .to_request();

    let search_resp = test::call_service(&app, search_req).await;
    assert_eq!(search_resp.status(), StatusCode::BAD_REQUEST);
}

#[actix_rt::test]
async fn search_users_rejects_missing_filters() {
    let pool = test_helpers::setup_test_db().await;
    test_helpers::clean_database(&pool).await;

    let config = test_helpers::build_test_config();
    let app = test_helpers::create_full_test_app(pool.clone(), config.clone()).await;

    let root_claims = test_helpers::build_root_claims();
    let root_token = test_helpers::generate_jwt(&root_claims, &config.jwt_secret);

    let search_req = test_helpers::with_auth_headers(
        test::TestRequest::get().uri("/api/v1/users/search"),
        &config,
        &root_token,
    )
    .to_request();

    let search_resp = test::call_service(&app, search_req).await;
    assert_eq!(search_resp.status(), StatusCode::BAD_REQUEST);
    let body: serde_json::Value = test::read_body_json(search_resp).await;
    assert!(
        body["message"]
            .as_str()
            .unwrap()
            .contains("query parameter 'name' or 'registration' is required")
    );
}

#[actix_rt::test]
async fn search_users_rejects_conflicting_filters() {
    let pool = test_helpers::setup_test_db().await;
    test_helpers::clean_database(&pool).await;

    let config = test_helpers::build_test_config();
    let app = test_helpers::create_full_test_app(pool.clone(), config.clone()).await;

    let root_claims = test_helpers::build_root_claims();
    let root_token = test_helpers::generate_jwt(&root_claims, &config.jwt_secret);

    let search_req = test_helpers::with_auth_headers(
        test::TestRequest::get().uri("/api/v1/users/search?name=maria&registration=100099999"),
        &config,
        &root_token,
    )
    .to_request();

    let search_resp = test::call_service(&app, search_req).await;
    assert_eq!(search_resp.status(), StatusCode::BAD_REQUEST);
    let body: serde_json::Value = test::read_body_json(search_resp).await;
    assert!(
        body["message"]
            .as_str()
            .unwrap()
            .contains("provide either 'name' or 'registration', not both")
    );
}

#[actix_rt::test]
async fn search_users_rejects_empty_name_filter() {
    let pool = test_helpers::setup_test_db().await;
    test_helpers::clean_database(&pool).await;

    let config = test_helpers::build_test_config();
    let app = test_helpers::create_full_test_app(pool.clone(), config.clone()).await;

    let root_claims = test_helpers::build_root_claims();
    let root_token = test_helpers::generate_jwt(&root_claims, &config.jwt_secret);

    let search_req = test_helpers::with_auth_headers(
        test::TestRequest::get().uri("/api/v1/users/search?name=%20%20"),
        &config,
        &root_token,
    )
    .to_request();

    let search_resp = test::call_service(&app, search_req).await;
    assert_eq!(search_resp.status(), StatusCode::BAD_REQUEST);
    let body: serde_json::Value = test::read_body_json(search_resp).await;
    assert!(
        body["message"]
            .as_str()
            .unwrap()
            .contains("query parameter 'name' cannot be empty")
    );
}
