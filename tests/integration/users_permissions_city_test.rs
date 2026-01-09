use actix_web::{http::StatusCode, test};
use std::time::{SystemTime, UNIX_EPOCH};
use uuid::Uuid;

use crate::common::test_helpers;
use nupevid_api::core::entities::auth::ClaimsToUserToken;

#[actix_rt::test]
async fn city_admin_cannot_get_user_from_other_city() {
    let pool = test_helpers::setup_test_db().await;
    test_helpers::clean_database(&pool).await;
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

    // Create CITY_USER in city2 (should NOT be accessible to CITY_ADMIN from city1)
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
    let user2_resp = test::call_service(&app, create_user2_req).await;
    let user2_body: serde_json::Value = test::read_body_json(user2_resp).await;
    let user2_id: Uuid = user2_body["data"]["id"].as_str().unwrap().parse().unwrap();

    // Create token for CITY_ADMIN with read_users permission only for city1
    let admin_claims = ClaimsToUserToken {
        id: admin_id.to_string(),
        exp: (SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs() as usize)
            + 3600,
        rank: "MAJ PM".to_string(),
        registration: "100000111".to_string(),
        full_name: "Admin City 1".to_string(),
        profile: "CITY_ADMIN".to_string(),
        email: "admin.city1@test.com".to_string(),
        city_id: Some(city1_id.to_string()),
    };
    let admin_token = test_helpers::generate_jwt(&admin_claims, &config.jwt_secret);

    // Try to get user from city2 as CITY_ADMIN from city1 - should fail
    let get_user_req = test_helpers::with_auth_headers(
        test::TestRequest::get().uri(&format!("/api/v1/users/{}", user2_id)),
        &config,
        &admin_token,
    )
    .to_request();
    let get_user_resp = test::call_service(&app, get_user_req).await;

    assert_eq!(
        get_user_resp.status(),
        StatusCode::FORBIDDEN,
        "CITY_ADMIN should not access users from other cities"
    );
}

#[actix_rt::test]
async fn city_admin_cannot_update_user_from_other_city() {
    let pool = test_helpers::setup_test_db().await;
    test_helpers::clean_database(&pool).await;
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

    // Create CITY_USER in city2
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
    let user2_resp = test::call_service(&app, create_user2_req).await;
    let user2_body: serde_json::Value = test::read_body_json(user2_resp).await;
    let user2_id: Uuid = user2_body["data"]["id"].as_str().unwrap().parse().unwrap();

    // Create token for CITY_ADMIN
    let admin_claims = ClaimsToUserToken {
        id: admin_id.to_string(),
        exp: (SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs() as usize)
            + 3600,
        rank: "MAJ PM".to_string(),
        registration: "100000111".to_string(),
        full_name: "Admin City 1".to_string(),
        profile: "CITY_ADMIN".to_string(),
        email: "admin.city1@test.com".to_string(),
        city_id: Some(city1_id.to_string()),
    };
    let admin_token = test_helpers::generate_jwt(&admin_claims, &config.jwt_secret);

    // Try to update user from city2 - should fail
    let update_payload = serde_json::json!({
        "rank": "CB PM",
        "registration": "100000333",
        "full_name": "User City 2 Updated",
        "profile": "CITY_USER",
        "email": "user.city2@test.com",
        "city_id": city2_id
    });
    let update_req = test_helpers::with_auth_headers(
        test::TestRequest::put()
            .uri(&format!("/api/v1/users/{}", user2_id))
            .set_json(&update_payload),
        &config,
        &admin_token,
    )
    .to_request();
    let update_resp = test::call_service(&app, update_req).await;

    assert_eq!(
        update_resp.status(),
        StatusCode::FORBIDDEN,
        "CITY_ADMIN should not update users from other cities"
    );
}

#[actix_rt::test]
async fn city_admin_cannot_delete_user_from_other_city() {
    let pool = test_helpers::setup_test_db().await;
    test_helpers::clean_database(&pool).await;
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

    // Create CITY_USER in city2
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
    let user2_resp = test::call_service(&app, create_user2_req).await;
    let user2_body: serde_json::Value = test::read_body_json(user2_resp).await;
    let user2_id: Uuid = user2_body["data"]["id"].as_str().unwrap().parse().unwrap();

    // Create token for CITY_ADMIN
    let admin_claims = ClaimsToUserToken {
        id: admin_id.to_string(),
        exp: (SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs() as usize)
            + 3600,
        rank: "MAJ PM".to_string(),
        registration: "100000111".to_string(),
        full_name: "Admin City 1".to_string(),
        profile: "CITY_ADMIN".to_string(),
        email: "admin.city1@test.com".to_string(),
        city_id: Some(city1_id.to_string()),
    };
    let admin_token = test_helpers::generate_jwt(&admin_claims, &config.jwt_secret);

    // Try to delete user from city2 - should fail
    let delete_req = test_helpers::with_auth_headers(
        test::TestRequest::delete().uri(&format!("/api/v1/users/{}", user2_id)),
        &config,
        &admin_token,
    )
    .to_request();
    let delete_resp = test::call_service(&app, delete_req).await;

    assert_eq!(
        delete_resp.status(),
        StatusCode::FORBIDDEN,
        "CITY_ADMIN should not delete users from other cities"
    );
}
