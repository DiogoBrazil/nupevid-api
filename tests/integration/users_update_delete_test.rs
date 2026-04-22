use actix_web::{http::StatusCode, test};
use uuid::Uuid;

use crate::common::{fixtures, test_helpers};
use nupevid_api::core::entities::auth::UserClaims;
use nupevid_api::core::value_objects::profiles::Profile;
use nupevid_api::core::value_objects::ranks::Rank;
use std::time::{SystemTime, UNIX_EPOCH};

#[actix_rt::test]
async fn test_update_user_success() {
    let pool = test_helpers::setup_test_db().await;
    test_helpers::clean_database(&pool).await;
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

    // Update user
    let update_data = fixtures::valid_update_user();
    let update_req = test_helpers::with_auth_headers(
        test::TestRequest::put()
            .uri(&format!("/api/v1/users/{}", user_id))
            .set_json(&update_data),
        &config,
        &token,
    )
    .to_request();

    let update_resp = test::call_service(&app, update_req).await;
    assert_eq!(update_resp.status(), StatusCode::OK);

    let update_body: serde_json::Value = test::read_body_json(update_resp).await;
    assert_eq!(update_body["status"].as_u64().unwrap(), 200);
    assert_eq!(
        update_body["data"]["full_name"].as_str().unwrap(),
        update_data.full_name
    );
    assert_eq!(
        update_body["data"]["email"].as_str().unwrap(),
        update_data.email
    );
}

#[actix_rt::test]
async fn non_root_cannot_access_or_modify_root_user() {
    let pool = test_helpers::setup_test_db().await;
    test_helpers::clean_database(&pool).await;
    let config = test_helpers::build_test_config();
    let app = test_helpers::create_full_test_app(pool.clone(), config.clone()).await;

    // Create ROOT user in DB
    let root_token =
        test_helpers::generate_jwt(&test_helpers::build_root_claims(), &config.jwt_secret);
    let root_user_payload = serde_json::json!({
        "rank": "CEL PM",
        "registration": "100000888",
        "full_name": "Root Two",
        "profile": "ROOT",
        "email": "root.two@test.com",
        "password": "Secret123!"
    });
    let root_create = test_helpers::with_auth_headers(
        test::TestRequest::post()
            .uri("/api/v1/users")
            .set_json(&root_user_payload),
        &config,
        &root_token,
    )
    .to_request();
    let root_resp = test::call_service(&app, root_create).await;
    let root_body: serde_json::Value = test::read_body_json(root_resp).await;
    let root_id: Uuid = root_body["data"]["id"].as_str().unwrap().parse().unwrap();

    // Non-root token
    let claims_user = UserClaims {
        id: Uuid::new_v4().to_string(),
        exp: (SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs() as usize)
            + 3600,
        iss: "nupevid-api".to_string(),
        aud: "nupevid-api".to_string(),
        rank: Rank::CapPm,
        registration: "100009990".to_string(),
        full_name: "City Admin".to_string(),
        profile: Profile::CityAdmin,
        email: "city.admin@test.com".to_string(),
        city_id: None,
    };
    let non_root_token = test_helpers::generate_jwt(&claims_user, &config.jwt_secret);

    // Get by id -> FORBIDDEN
    let get_req = test_helpers::with_auth_headers(
        test::TestRequest::get().uri(&format!("/api/v1/users/{}", root_id)),
        &config,
        &non_root_token,
    )
    .to_request();
    let get_resp = test::call_service(&app, get_req).await;
    assert_eq!(get_resp.status(), actix_web::http::StatusCode::FORBIDDEN);

    // Update -> FORBIDDEN
    let upd_payload = serde_json::json!({
        "rank": "CEL PM",
        "registration": "100000888",
        "full_name": "Root Two",
        "profile": "ROOT",
        "email": "root.two@test.com"
    });
    let upd_req = test_helpers::with_auth_headers(
        test::TestRequest::put()
            .uri(&format!("/api/v1/users/{}", root_id))
            .set_json(&upd_payload),
        &config,
        &non_root_token,
    )
    .to_request();
    let upd_resp = test::call_service(&app, upd_req).await;
    assert_eq!(upd_resp.status(), actix_web::http::StatusCode::FORBIDDEN);

    // Delete -> FORBIDDEN
    let del_req = test_helpers::with_auth_headers(
        test::TestRequest::delete().uri(&format!("/api/v1/users/{}", root_id)),
        &config,
        &non_root_token,
    )
    .to_request();
    let del_resp = test::call_service(&app, del_req).await;
    assert_eq!(del_resp.status(), actix_web::http::StatusCode::FORBIDDEN);
}

#[actix_rt::test]
async fn test_update_user_not_found() {
    let pool = test_helpers::setup_test_db().await;
    test_helpers::clean_database(&pool).await;
    let config = test_helpers::build_test_config();
    let app = test_helpers::create_full_test_app(pool.clone(), config.clone()).await;

    let claims = test_helpers::build_root_claims();
    let token = test_helpers::generate_jwt(&claims, &config.jwt_secret);

    let random_uuid = Uuid::new_v4();
    let update_data = fixtures::valid_update_user();
    let req = test_helpers::with_auth_headers(
        test::TestRequest::put()
            .uri(&format!("/api/v1/users/{}", random_uuid))
            .set_json(&update_data),
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

#[actix_rt::test]
async fn test_update_user_duplicate_email() {
    let pool = test_helpers::setup_test_db().await;
    test_helpers::clean_database(&pool).await;
    let config = test_helpers::build_test_config();
    let app = test_helpers::create_full_test_app(pool.clone(), config.clone()).await;

    let claims = test_helpers::build_root_claims();
    let token = test_helpers::generate_jwt(&claims, &config.jwt_secret);

    // Create two users
    let user1 = fixtures::valid_create_user();
    let user2 = fixtures::valid_create_user_2();

    let create_req1 = test_helpers::with_auth_headers(
        test::TestRequest::post()
            .uri("/api/v1/users")
            .set_json(&user1),
        &config,
        &token,
    )
    .to_request();
    let create_resp1 = test::call_service(&app, create_req1).await;
    let create_body1: serde_json::Value = test::read_body_json(create_resp1).await;
    let user1_id = create_body1["data"]["id"].as_str().unwrap();

    let create_req2 = test_helpers::with_auth_headers(
        test::TestRequest::post()
            .uri("/api/v1/users")
            .set_json(&user2),
        &config,
        &token,
    )
    .to_request();
    test::call_service(&app, create_req2).await;

    // Try to update user1 with user2's email
    let mut update_data = fixtures::valid_update_user();
    update_data.email = user2.email.clone();

    let update_req = test_helpers::with_auth_headers(
        test::TestRequest::put()
            .uri(&format!("/api/v1/users/{}", user1_id))
            .set_json(&update_data),
        &config,
        &token,
    )
    .to_request();

    let update_resp = test::call_service(&app, update_req).await;
    assert_eq!(update_resp.status(), StatusCode::BAD_REQUEST);

    let update_body: serde_json::Value = test::read_body_json(update_resp).await;
    assert_eq!(update_body["status_code"].as_u64().unwrap(), 400);
    assert!(
        update_body["message"]
            .as_str()
            .unwrap()
            .contains("already in use")
    );
}

#[actix_rt::test]
async fn test_update_user_duplicate_registration() {
    let pool = test_helpers::setup_test_db().await;
    test_helpers::clean_database(&pool).await;
    let config = test_helpers::build_test_config();
    let app = test_helpers::create_full_test_app(pool.clone(), config.clone()).await;

    let claims = test_helpers::build_root_claims();
    let token = test_helpers::generate_jwt(&claims, &config.jwt_secret);

    // Create two users
    let user1 = fixtures::valid_create_user();
    let user2 = fixtures::valid_create_user_2();

    let create_req1 = test_helpers::with_auth_headers(
        test::TestRequest::post()
            .uri("/api/v1/users")
            .set_json(&user1),
        &config,
        &token,
    )
    .to_request();
    let create_resp1 = test::call_service(&app, create_req1).await;
    let create_body1: serde_json::Value = test::read_body_json(create_resp1).await;
    let user1_id = create_body1["data"]["id"].as_str().unwrap();

    let create_req2 = test_helpers::with_auth_headers(
        test::TestRequest::post()
            .uri("/api/v1/users")
            .set_json(&user2),
        &config,
        &token,
    )
    .to_request();
    test::call_service(&app, create_req2).await;

    // Try to update user1 with user2's registration
    let mut update_data = fixtures::valid_update_user();
    update_data.registration = user2.registration.clone();

    let update_req = test_helpers::with_auth_headers(
        test::TestRequest::put()
            .uri(&format!("/api/v1/users/{}", user1_id))
            .set_json(&update_data),
        &config,
        &token,
    )
    .to_request();

    let update_resp = test::call_service(&app, update_req).await;
    assert_eq!(update_resp.status(), StatusCode::BAD_REQUEST);

    let update_body: serde_json::Value = test::read_body_json(update_resp).await;
    assert_eq!(update_body["status_code"].as_u64().unwrap(), 400);
    assert!(
        update_body["message"]
            .as_str()
            .unwrap()
            .contains("registration")
    );
    assert!(
        update_body["message"]
            .as_str()
            .unwrap()
            .contains("already exists")
    );
}

#[actix_rt::test]
async fn test_delete_user_success() {
    let pool = test_helpers::setup_test_db().await;
    test_helpers::clean_database(&pool).await;
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

    // Delete user
    let delete_req = test_helpers::with_auth_headers(
        test::TestRequest::delete().uri(&format!("/api/v1/users/{}", user_id)),
        &config,
        &token,
    )
    .to_request();

    let delete_resp = test::call_service(&app, delete_req).await;
    assert_eq!(delete_resp.status(), StatusCode::OK);

    let delete_body: serde_json::Value = test::read_body_json(delete_resp).await;
    assert_eq!(delete_body["status"].as_u64().unwrap(), 200);
    assert_eq!(delete_body["data"]["id"].as_str().unwrap(), user_id);

    // Verify user is deleted
    let get_req = test_helpers::with_auth_headers(
        test::TestRequest::get().uri(&format!("/api/v1/users/{}", user_id)),
        &config,
        &token,
    )
    .to_request();
    let get_resp = test::call_service(&app, get_req).await;
    assert_eq!(get_resp.status(), StatusCode::NOT_FOUND);
}

#[actix_rt::test]
async fn test_delete_user_not_found() {
    let pool = test_helpers::setup_test_db().await;
    test_helpers::clean_database(&pool).await;
    let config = test_helpers::build_test_config();
    let app = test_helpers::create_full_test_app(pool.clone(), config.clone()).await;

    let claims = test_helpers::build_root_claims();
    let token = test_helpers::generate_jwt(&claims, &config.jwt_secret);

    let random_uuid = Uuid::new_v4();
    let req = test_helpers::with_auth_headers(
        test::TestRequest::delete().uri(&format!("/api/v1/users/{}", random_uuid)),
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
