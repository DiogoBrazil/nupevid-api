use actix_web::{http::StatusCode, test};
use sqlx::PgPool;
use uuid::Uuid;

use crate::common::{fixtures, test_helpers};
use nupevid_api::core::entities::auth::UserClaims;
use nupevid_api::core::value_objects::profiles::Profile;
use nupevid_api::core::value_objects::ranks::Rank;
use std::time::{SystemTime, UNIX_EPOCH};

#[sqlx::test]
async fn test_get_all_users_success(pool: PgPool) {
    let config = test_helpers::build_test_config();
    let app = test_helpers::create_full_test_app(pool.clone(), config.clone()).await;

    let claims = test_helpers::build_root_claims();
    let token = test_helpers::generate_jwt(&claims, &config.jwt_secret);

    // Create two users
    let user1 = fixtures::valid_create_user();
    let user2 = fixtures::valid_create_user_2();

    for user in [user1, user2] {
        let req = test_helpers::with_auth_headers(
            test::TestRequest::post()
                .uri("/api/v1/users")
                .set_json(&user),
            &config,
            &token,
        )
        .to_request();
        test::call_service(&app, req).await;
    }

    // Get all users
    let req = test_helpers::with_auth_headers(
        test::TestRequest::get().uri("/api/v1/users"),
        &config,
        &token,
    )
    .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), StatusCode::OK);

    let body: serde_json::Value = test::read_body_json(resp).await;
    assert_eq!(body["status"].as_u64().unwrap(), 200);
    assert_eq!(body["data"].as_array().unwrap().len(), 2);
}

#[sqlx::test]
async fn test_get_all_users_empty(pool: PgPool) {
    let config = test_helpers::build_test_config();
    let app = test_helpers::create_full_test_app(pool.clone(), config.clone()).await;

    let claims = test_helpers::build_root_claims();
    let token = test_helpers::generate_jwt(&claims, &config.jwt_secret);

    let req = test_helpers::with_auth_headers(
        test::TestRequest::get().uri("/api/v1/users"),
        &config,
        &token,
    )
    .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), StatusCode::OK);

    let body: serde_json::Value = test::read_body_json(resp).await;
    assert_eq!(body["status"].as_u64().unwrap(), 200);
    assert_eq!(body["data"].as_array().unwrap().len(), 0);
}

#[sqlx::test]
async fn non_root_list_users_should_not_include_root(pool: PgPool) {
    let config = test_helpers::build_test_config();
    let app = test_helpers::create_full_test_app(pool.clone(), config.clone()).await;

    // Create a ROOT user in DB and a CITY_USER
    let root_token =
        test_helpers::generate_jwt(&test_helpers::build_root_claims(), &config.jwt_secret);
    let root_user_payload = serde_json::json!({
        "rank": "CEL PM",
        "registration": "100000777",
        "full_name": "Root DB User",
        "profile": "ROOT",
        "email": "root.db@test.com",
        "password": "Secret123!"
    });
    let create_root_req = test_helpers::with_auth_headers(
        test::TestRequest::post()
            .uri("/api/v1/users")
            .set_json(&root_user_payload),
        &config,
        &root_token,
    )
    .to_request();
    let _ = test::call_service(&app, create_root_req).await;

    // Create a city to bind CITY_USER
    let city_payload =
        serde_json::json!({"name": "PORTO VELHO", "state": "RO", "battalion": "1ºBPM"});
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
    let city_id: Uuid = city_body["data"]["id"].as_str().unwrap().parse().unwrap();

    // Create a CITY_USER
    let city_user_payload = fixtures::valid_create_user();
    let req = test_helpers::with_auth_headers(
        test::TestRequest::post()
            .uri("/api/v1/users")
            .set_json(&serde_json::json!({
                "rank": city_user_payload.rank,
                "registration": city_user_payload.registration,
                "full_name": city_user_payload.full_name,
                "profile": "CITY_USER",
                "email": city_user_payload.email,
                "password": city_user_payload.password,
                "city_id": city_id
            })),
        &config,
        &root_token,
    )
    .to_request();
    let _ = test::call_service(&app, req).await;

    // Non-root token (CITY_USER)
    let claims_user = UserClaims {
        id: Uuid::new_v4().to_string(),
        exp: (SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs() as usize)
            + 3600,
        iss: "nupevid-api".to_string(),
        aud: "nupevid-api".to_string(),
        rank: Rank::CbPm,
        registration: "100009991".to_string(),
        full_name: "Any User".to_string(),
        profile: Profile::CityUser,
        email: "any.user@test.com".to_string(),
        city_id: Some(city_id.to_string()),
    };
    let non_root_token = test_helpers::generate_jwt(&claims_user, &config.jwt_secret);

    // List users as non-root -> must not include ROOT
    let list_req = test_helpers::with_auth_headers(
        test::TestRequest::get().uri("/api/v1/users"),
        &config,
        &non_root_token,
    )
    .to_request();
    let list_resp = test::call_service(&app, list_req).await;
    assert_eq!(list_resp.status(), StatusCode::OK);
    let list_body: serde_json::Value = test::read_body_json(list_resp).await;
    let arr = list_body["data"].as_array().unwrap();
    assert!(arr.iter().all(|u| u["profile"].as_str().unwrap() != "ROOT"));
}

#[sqlx::test]
async fn test_get_user_by_id_success(pool: PgPool) {
    let config = test_helpers::build_test_config();
    let app = test_helpers::create_full_test_app(pool.clone(), config.clone()).await;

    let claims = test_helpers::build_root_claims();
    let token = test_helpers::generate_jwt(&claims, &config.jwt_secret);

    // Create a user
    let user = fixtures::valid_create_user();
    let create_req = test_helpers::with_auth_headers(
        test::TestRequest::post()
            .uri("/api/v1/users")
            .set_json(&user),
        &config,
        &token,
    )
    .to_request();
    let create_resp = test::call_service(&app, create_req).await;
    let create_body: serde_json::Value = test::read_body_json(create_resp).await;
    let user_id = create_body["data"]["id"].as_str().unwrap();

    // Get user by ID
    let get_req = test_helpers::with_auth_headers(
        test::TestRequest::get().uri(&format!("/api/v1/users/{}", user_id)),
        &config,
        &token,
    )
    .to_request();

    let get_resp = test::call_service(&app, get_req).await;
    assert_eq!(get_resp.status(), StatusCode::OK);

    let get_body: serde_json::Value = test::read_body_json(get_resp).await;
    assert_eq!(get_body["status"].as_u64().unwrap(), 200);
    assert_eq!(get_body["data"]["id"].as_str().unwrap(), user_id);
    assert_eq!(get_body["data"]["email"].as_str().unwrap(), user.email);
}

#[sqlx::test]
async fn test_get_user_by_id_not_found(pool: PgPool) {
    let config = test_helpers::build_test_config();
    let app = test_helpers::create_full_test_app(pool.clone(), config.clone()).await;

    let claims = test_helpers::build_root_claims();
    let token = test_helpers::generate_jwt(&claims, &config.jwt_secret);

    let random_uuid = Uuid::new_v4();
    let req = test_helpers::with_auth_headers(
        test::TestRequest::get().uri(&format!("/api/v1/users/{}", random_uuid)),
        &config,
        &token,
    )
    .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), StatusCode::NOT_FOUND);

    let body: serde_json::Value = test::read_body_json(resp).await;
    assert_eq!(body["status_code"].as_u64().unwrap(), 404);
    assert!(body["message"].as_str().unwrap().contains("not found"));
}

#[sqlx::test]
async fn test_get_user_by_invalid_uuid(pool: PgPool) {
    let config = test_helpers::build_test_config();
    let app = test_helpers::create_full_test_app(pool.clone(), config.clone()).await;

    let claims = test_helpers::build_root_claims();
    let token = test_helpers::generate_jwt(&claims, &config.jwt_secret);

    let req = test_helpers::with_auth_headers(
        test::TestRequest::get().uri("/api/v1/users/invalid-uuid-format"),
        &config,
        &token,
    )
    .to_request();

    let resp = test::call_service(&app, req).await;
    // Actix will return BAD_REQUEST for invalid UUID in path
    assert_eq!(resp.status(), StatusCode::NOT_FOUND); // Route not matched
}

#[sqlx::test]
async fn city_admin_only_sees_users_from_permitted_cities(pool: PgPool) {
    let config = test_helpers::build_test_config();
    let app = test_helpers::create_full_test_app(pool.clone(), config.clone()).await;

    let root_claims = test_helpers::build_root_claims();
    let root_token = test_helpers::generate_jwt(&root_claims, &config.jwt_secret);

    // Create two cities
    let city1_payload =
        serde_json::json!({"name": "PORTO VELHO", "state": "RO", "battalion": "1ºBPM"});
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
    let city1_id: Uuid = city1_body["data"]["id"].as_str().unwrap().parse().unwrap();

    let city2_payload =
        serde_json::json!({"name": "JI-PARANÁ", "state": "RO", "battalion": "2ºBPM"});
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
    let city2_id: Uuid = city2_body["data"]["id"].as_str().unwrap().parse().unwrap();

    // Create CITY_ADMIN for city1
    let city_admin_payload = serde_json::json!({
        "rank": "MAJ PM",
        "registration": "100000111",
        "full_name": "Admin City 1",
        "profile": "CITY_ADMIN",
        "email": "admin.city1@test.com",
        "password": "Secret123!",
        "city_id": city1_id
    });
    let create_admin_req = test_helpers::with_auth_headers(
        test::TestRequest::post()
            .uri("/api/v1/users")
            .set_json(&city_admin_payload),
        &config,
        &root_token,
    )
    .to_request();
    let admin_resp = test::call_service(&app, create_admin_req).await;
    let admin_body: serde_json::Value = test::read_body_json(admin_resp).await;
    let admin_id: Uuid = admin_body["data"]["id"].as_str().unwrap().parse().unwrap();

    // Create CITY_USER in city1 (should be visible to CITY_ADMIN)
    let user_city1_payload = serde_json::json!({
        "rank": "SD PM",
        "registration": "100000222",
        "full_name": "User City 1",
        "profile": "CITY_USER",
        "email": "user.city1@test.com",
        "password": "Secret123!",
        "city_id": city1_id
    });
    let create_user1_req = test_helpers::with_auth_headers(
        test::TestRequest::post()
            .uri("/api/v1/users")
            .set_json(&user_city1_payload),
        &config,
        &root_token,
    )
    .to_request();
    test::call_service(&app, create_user1_req).await;

    // Create CITY_USER in city2 (should NOT be visible to CITY_ADMIN from city1)
    let user_city2_payload = serde_json::json!({
        "rank": "SD PM",
        "registration": "100000333",
        "full_name": "User City 2",
        "profile": "CITY_USER",
        "email": "user.city2@test.com",
        "password": "Secret123!",
        "city_id": city2_id
    });
    let create_user2_req = test_helpers::with_auth_headers(
        test::TestRequest::post()
            .uri("/api/v1/users")
            .set_json(&user_city2_payload),
        &config,
        &root_token,
    )
    .to_request();
    test::call_service(&app, create_user2_req).await;

    // Create token for CITY_ADMIN with read_users permission only for city1
    let admin_claims = UserClaims {
        id: admin_id.to_string(),
        exp: (SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs() as usize)
            + 3600,
        iss: "nupevid-api".to_string(),
        aud: "nupevid-api".to_string(),
        rank: Rank::MajPm,
        registration: "100000111".to_string(),
        full_name: "Admin City 1".to_string(),
        profile: Profile::CityAdmin,
        email: "admin.city1@test.com".to_string(),
        city_id: Some(city1_id.to_string()),
    };
    let admin_token = test_helpers::generate_jwt(&admin_claims, &config.jwt_secret);

    // Get all users as CITY_ADMIN
    let list_req = test_helpers::with_auth_headers(
        test::TestRequest::get().uri("/api/v1/users"),
        &config,
        &admin_token,
    )
    .to_request();
    let list_resp = test::call_service(&app, list_req).await;
    assert_eq!(list_resp.status(), StatusCode::OK);

    let list_body: serde_json::Value = test::read_body_json(list_resp).await;
    let users = list_body["data"].as_array().unwrap();

    // CITY_ADMIN should only see users from city1
    // Expected: 2 users (the admin itself + user from city1)
    assert_eq!(
        users.len(),
        2,
        "CITY_ADMIN should only see users from permitted cities"
    );

    // Verify all returned users belong to city1
    for user in users {
        let user_city_id: Uuid = user["city_id"].as_str().unwrap().parse().unwrap();
        assert_eq!(
            user_city_id, city1_id,
            "All returned users should belong to city1"
        );
        assert_ne!(
            user["email"].as_str().unwrap(),
            "user.city2@test.com",
            "User from city2 should not be visible"
        );
    }
}
