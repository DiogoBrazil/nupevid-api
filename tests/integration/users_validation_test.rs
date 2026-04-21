use actix_web::{http::StatusCode, test};
use uuid::Uuid;

use crate::common::{db_fixtures, test_helpers};

#[actix_rt::test]
async fn create_user_with_registration_exceeding_max_length() {
    let pool = test_helpers::setup_test_db().await;
    test_helpers::clean_database(&pool).await;

    let config = test_helpers::build_test_config();
    let app = test_helpers::create_full_test_app(pool.clone(), config.clone()).await;

    let root_claims = test_helpers::build_root_claims();
    let root_token = test_helpers::generate_jwt(&root_claims, &config.jwt_secret);

    // Registration with 10 characters (max is 9)
    let user = serde_json::json!({
        "rank": "CAP PM",
        "registration": "1000123456", // 10 chars
        "full_name": "Test User",
        "profile": "ROOT",
        "email": "test@test.com",
        "password": "password123",
        "city_id": null,
        "permission_policies": null
    });

    let req = test_helpers::with_auth_headers(
        test::TestRequest::post()
            .uri("/api/v1/users")
            .set_json(&user),
        &config,
        &root_token,
    )
    .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), StatusCode::BAD_REQUEST);
    let body: serde_json::Value = test::read_body_json(resp).await;
    assert!(body["message"].as_str().unwrap().contains("registration"));
}

#[actix_rt::test]
async fn create_user_with_registration_containing_letters() {
    let pool = test_helpers::setup_test_db().await;
    test_helpers::clean_database(&pool).await;

    let config = test_helpers::build_test_config();
    let app = test_helpers::create_full_test_app(pool.clone(), config.clone()).await;

    let root_claims = test_helpers::build_root_claims();
    let root_token = test_helpers::generate_jwt(&root_claims, &config.jwt_secret);

    let user = serde_json::json!({
        "rank": "CAP PM",
        "registration": "1000ABC12",
        "full_name": "Test User",
        "profile": "ROOT",
        "email": "test@test.com",
        "password": "password123",
        "city_id": null,
        "permission_policies": null
    });

    let req = test_helpers::with_auth_headers(
        test::TestRequest::post()
            .uri("/api/v1/users")
            .set_json(&user),
        &config,
        &root_token,
    )
    .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), StatusCode::BAD_REQUEST);
    let body: serde_json::Value = test::read_body_json(resp).await;
    assert!(body["message"].as_str().unwrap().contains("registration"));
}

#[actix_rt::test]
async fn create_city_admin_without_city_id_fails() {
    let pool = test_helpers::setup_test_db().await;
    test_helpers::clean_database(&pool).await;

    let config = test_helpers::build_test_config();
    let app = test_helpers::create_full_test_app(pool.clone(), config.clone()).await;

    let root_claims = test_helpers::build_root_claims();
    let root_token = test_helpers::generate_jwt(&root_claims, &config.jwt_secret);

    let user = serde_json::json!({
        "rank": "CAP PM",
        "registration": "100012345",
        "full_name": "Admin Without City",
        "profile": "CITY_ADMIN",
        "email": "admin.nocity@test.com",
        "password": "password123",
        "city_id": null,
        "permission_policies": null
    });

    let req = test_helpers::with_auth_headers(
        test::TestRequest::post()
            .uri("/api/v1/users")
            .set_json(&user),
        &config,
        &root_token,
    )
    .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), StatusCode::BAD_REQUEST);
    let body: serde_json::Value = test::read_body_json(resp).await;
    assert!(
        body["message"]
            .as_str()
            .unwrap()
            .contains("city_id is required")
    );
}

#[actix_rt::test]
async fn create_city_user_without_city_id_fails() {
    let pool = test_helpers::setup_test_db().await;
    test_helpers::clean_database(&pool).await;

    let config = test_helpers::build_test_config();
    let app = test_helpers::create_full_test_app(pool.clone(), config.clone()).await;

    let root_claims = test_helpers::build_root_claims();
    let root_token = test_helpers::generate_jwt(&root_claims, &config.jwt_secret);

    let user = serde_json::json!({
        "rank": "SD PM",
        "registration": "100067890",
        "full_name": "User Without City",
        "profile": "CITY_USER",
        "email": "user.nocity@test.com",
        "password": "password123",
        "city_id": null,
        "permission_policies": null
    });

    let req = test_helpers::with_auth_headers(
        test::TestRequest::post()
            .uri("/api/v1/users")
            .set_json(&user),
        &config,
        &root_token,
    )
    .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), StatusCode::BAD_REQUEST);
    let body: serde_json::Value = test::read_body_json(resp).await;
    assert!(
        body["message"]
            .as_str()
            .unwrap()
            .contains("city_id is required")
    );
}

#[actix_rt::test]
async fn create_user_with_invalid_policy_name_fails() {
    let pool = test_helpers::setup_test_db().await;
    test_helpers::clean_database(&pool).await;

    let config = test_helpers::build_test_config();
    let app = test_helpers::create_full_test_app(pool.clone(), config.clone()).await;

    let city = db_fixtures::insert_city(&pool, "Test City").await;

    let root_claims = test_helpers::build_root_claims();
    let root_token = test_helpers::generate_jwt(&root_claims, &config.jwt_secret);

    let user = serde_json::json!({
        "rank": "SD PM",
        "registration": "100067890",
        "full_name": "User With Bad Policy",
        "profile": "CITY_USER",
        "email": "user.badpolicy@test.com",
        "password": "password123",
        "city_id": city,
        "permission_policies": {
            "invalid_policy_name": [city]
        }
    });

    let req = test_helpers::with_auth_headers(
        test::TestRequest::post()
            .uri("/api/v1/users")
            .set_json(&user),
        &config,
        &root_token,
    )
    .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), StatusCode::UNPROCESSABLE_ENTITY);
}

#[actix_rt::test]
async fn update_user_with_invalid_rank_fails() {
    let pool = test_helpers::setup_test_db().await;
    test_helpers::clean_database(&pool).await;

    let config = test_helpers::build_test_config();
    let app = test_helpers::create_full_test_app(pool.clone(), config.clone()).await;

    let root_claims = test_helpers::build_root_claims();
    let root_token = test_helpers::generate_jwt(&root_claims, &config.jwt_secret);

    // Create user first
    let user = serde_json::json!({
        "rank": "CAP PM",
        "registration": "100012345",
        "full_name": "Test User",
        "profile": "ROOT",
        "email": "test@test.com",
        "password": "password123",
        "city_id": null,
        "permission_policies": null
    });

    let create_req = test_helpers::with_auth_headers(
        test::TestRequest::post()
            .uri("/api/v1/users")
            .set_json(&user),
        &config,
        &root_token,
    )
    .to_request();

    let create_resp = test::call_service(&app, create_req).await;
    assert_eq!(create_resp.status(), StatusCode::CREATED);
    let body: serde_json::Value = test::read_body_json(create_resp).await;
    let user_id = body["data"]["id"].as_str().unwrap();

    // Try to update with invalid rank
    let update_payload = serde_json::json!({
        "rank": "INVALID RANK",
        "registration": "100012345",
        "full_name": "Test User Updated",
        "profile": "ROOT",
        "email": "test@test.com",
        "city_id": null,
        "permission_policies": null
    });

    let update_req = test_helpers::with_auth_headers(
        test::TestRequest::put()
            .uri(&format!("/api/v1/users/{}", user_id))
            .set_json(&update_payload),
        &config,
        &root_token,
    )
    .to_request();

    let update_resp = test::call_service(&app, update_req).await;
    assert_eq!(update_resp.status(), StatusCode::UNPROCESSABLE_ENTITY);
}

#[actix_rt::test]
async fn update_user_with_invalid_email_fails() {
    let pool = test_helpers::setup_test_db().await;
    test_helpers::clean_database(&pool).await;

    let config = test_helpers::build_test_config();
    let app = test_helpers::create_full_test_app(pool.clone(), config.clone()).await;

    let root_claims = test_helpers::build_root_claims();
    let root_token = test_helpers::generate_jwt(&root_claims, &config.jwt_secret);

    // Create user
    let user = serde_json::json!({
        "rank": "CAP PM",
        "registration": "100012345",
        "full_name": "Test User",
        "profile": "ROOT",
        "email": "test@test.com",
        "password": "password123",
        "city_id": null,
        "permission_policies": null
    });

    let create_req = test_helpers::with_auth_headers(
        test::TestRequest::post()
            .uri("/api/v1/users")
            .set_json(&user),
        &config,
        &root_token,
    )
    .to_request();

    let create_resp = test::call_service(&app, create_req).await;
    let body: serde_json::Value = test::read_body_json(create_resp).await;
    let user_id = body["data"]["id"].as_str().unwrap();

    // Update with invalid email
    let update_payload = serde_json::json!({
        "rank": "CAP PM",
        "registration": "100012345",
        "full_name": "Test User",
        "profile": "ROOT",
        "email": "not-a-valid-email",
        "city_id": null,
        "permission_policies": null
    });

    let update_req = test_helpers::with_auth_headers(
        test::TestRequest::put()
            .uri(&format!("/api/v1/users/{}", user_id))
            .set_json(&update_payload),
        &config,
        &root_token,
    )
    .to_request();

    let update_resp = test::call_service(&app, update_req).await;
    assert_eq!(update_resp.status(), StatusCode::BAD_REQUEST);
    let body: serde_json::Value = test::read_body_json(update_resp).await;
    assert!(
        body["message"]
            .as_str()
            .unwrap()
            .contains("not a valid email")
    );
}

#[actix_rt::test]
async fn update_city_admin_removing_city_id_fails() {
    let pool = test_helpers::setup_test_db().await;
    test_helpers::clean_database(&pool).await;

    let config = test_helpers::build_test_config();
    let app = test_helpers::create_full_test_app(pool.clone(), config.clone()).await;

    let city = db_fixtures::insert_city(&pool, "Test City").await;

    let root_claims = test_helpers::build_root_claims();
    let root_token = test_helpers::generate_jwt(&root_claims, &config.jwt_secret);

    // Create CITY_ADMIN
    let user = serde_json::json!({
        "rank": "CAP PM",
        "registration": "100012345",
        "full_name": "City Admin",
        "profile": "CITY_ADMIN",
        "email": "admin@test.com",
        "password": "password123",
        "city_id": city,
        "permission_policies": null
    });

    let create_req = test_helpers::with_auth_headers(
        test::TestRequest::post()
            .uri("/api/v1/users")
            .set_json(&user),
        &config,
        &root_token,
    )
    .to_request();

    let create_resp = test::call_service(&app, create_req).await;
    let body: serde_json::Value = test::read_body_json(create_resp).await;
    let user_id = body["data"]["id"].as_str().unwrap();

    // Try to remove city_id
    let update_payload = serde_json::json!({
        "rank": "CAP PM",
        "registration": "100012345",
        "full_name": "City Admin",
        "profile": "CITY_ADMIN",
        "email": "admin@test.com",
        "city_id": null,
        "permission_policies": null
    });

    let update_req = test_helpers::with_auth_headers(
        test::TestRequest::put()
            .uri(&format!("/api/v1/users/{}", user_id))
            .set_json(&update_payload),
        &config,
        &root_token,
    )
    .to_request();

    let update_resp = test::call_service(&app, update_req).await;
    assert_eq!(update_resp.status(), StatusCode::BAD_REQUEST);
    let body: serde_json::Value = test::read_body_json(update_resp).await;
    assert!(
        body["message"]
            .as_str()
            .unwrap()
            .contains("city_id is required")
    );
}

#[actix_rt::test]
async fn update_password_with_empty_current_password_fails() {
    let pool = test_helpers::setup_test_db().await;
    test_helpers::clean_database(&pool).await;

    let config = test_helpers::build_test_config();
    let app = test_helpers::create_full_test_app(pool.clone(), config.clone()).await;

    let city = db_fixtures::insert_city(&pool, "Test City").await;

    let root_claims = test_helpers::build_root_claims();
    let root_token = test_helpers::generate_jwt(&root_claims, &config.jwt_secret);

    // Create CITY_USER (non-ROOT users need current_password)
    let user = serde_json::json!({
        "rank": "SD PM",
        "registration": "100012345",
        "full_name": "Test User",
        "profile": "CITY_USER",
        "email": "test@test.com",
        "password": "password123",
        "city_id": city,
        "permission_policies": null
    });

    let create_req = test_helpers::with_auth_headers(
        test::TestRequest::post()
            .uri("/api/v1/users")
            .set_json(&user),
        &config,
        &root_token,
    )
    .to_request();

    let create_resp = test::call_service(&app, create_req).await;
    let body: serde_json::Value = test::read_body_json(create_resp).await;
    let _user_id = body["data"]["id"].as_str().unwrap();

    // User logs in to get their token
    let login_payload = serde_json::json!({
        "email": "test@test.com",
        "password": "password123"
    });

    let login_req = test::TestRequest::post()
        .uri("/api/v1/auth/login")
        .insert_header(("api_key", config.api_key.clone()))
        .set_json(&login_payload)
        .to_request();

    let login_resp = test::call_service(&app, login_req).await;
    let login_body: serde_json::Value = test::read_body_json(login_resp).await;
    let user_token = login_body["data"]["token"].as_str().unwrap();

    // Try to update password with empty current_password
    let password_payload = serde_json::json!({
        "current_password": "",
        "new_password": "newPassword456"
    });

    let password_req = test_helpers::with_auth_headers(
        test::TestRequest::patch()
            .uri("/api/v1/users/password")
            .set_json(&password_payload),
        &config,
        user_token,
    )
    .to_request();

    let password_resp = test::call_service(&app, password_req).await;
    assert_eq!(password_resp.status(), StatusCode::BAD_REQUEST);
    let body: serde_json::Value = test::read_body_json(password_resp).await;
    assert!(
        body["message"]
            .as_str()
            .unwrap()
            .contains("current_password is required")
    );
}

#[actix_rt::test]
async fn update_password_with_empty_new_password_fails() {
    let pool = test_helpers::setup_test_db().await;
    test_helpers::clean_database(&pool).await;

    let config = test_helpers::build_test_config();
    let app = test_helpers::create_full_test_app(pool.clone(), config.clone()).await;

    let root_claims = test_helpers::build_root_claims();
    let root_token = test_helpers::generate_jwt(&root_claims, &config.jwt_secret);

    // Create user
    let user = serde_json::json!({
        "rank": "CAP PM",
        "registration": "100012345",
        "full_name": "Test User",
        "profile": "ROOT",
        "email": "test@test.com",
        "password": "password123",
        "city_id": null,
        "permission_policies": null
    });

    let create_req = test_helpers::with_auth_headers(
        test::TestRequest::post()
            .uri("/api/v1/users")
            .set_json(&user),
        &config,
        &root_token,
    )
    .to_request();

    let create_resp = test::call_service(&app, create_req).await;
    let body: serde_json::Value = test::read_body_json(create_resp).await;
    let _user_id = body["data"]["id"].as_str().unwrap();

    // Try to update password with empty new_password
    let password_payload = serde_json::json!({
        "current_password": "password123",
        "new_password": ""
    });

    let login_payload = serde_json::json!({
        "email": "test@test.com",
        "password": "password123"
    });

    let login_req = test::TestRequest::post()
        .uri("/api/v1/auth/login")
        .insert_header(("api_key", config.api_key.clone()))
        .set_json(&login_payload)
        .to_request();

    let login_resp = test::call_service(&app, login_req).await;
    let login_body: serde_json::Value = test::read_body_json(login_resp).await;
    let user_token = login_body["data"]["token"].as_str().unwrap();

    let password_req = test_helpers::with_auth_headers(
        test::TestRequest::patch()
            .uri("/api/v1/users/password")
            .set_json(&password_payload),
        &config,
        user_token,
    )
    .to_request();

    let password_resp = test::call_service(&app, password_req).await;
    assert_eq!(password_resp.status(), StatusCode::BAD_REQUEST);
    let body: serde_json::Value = test::read_body_json(password_resp).await;
    assert!(
        body["message"]
            .as_str()
            .unwrap()
            .contains("new_password is required")
    );
}

#[actix_rt::test]
async fn append_invalid_policy_name_fails() {
    let pool = test_helpers::setup_test_db().await;
    test_helpers::clean_database(&pool).await;

    let config = test_helpers::build_test_config();
    let app = test_helpers::create_full_test_app(pool.clone(), config.clone()).await;

    let city = db_fixtures::insert_city(&pool, "Test City").await;

    let root_claims = test_helpers::build_root_claims();
    let root_token = test_helpers::generate_jwt(&root_claims, &config.jwt_secret);

    // Create CITY_USER
    let user = serde_json::json!({
        "rank": "SD PM",
        "registration": "100012345",
        "full_name": "City User",
        "profile": "CITY_USER",
        "email": "user@test.com",
        "password": "password123",
        "city_id": city,
        "permission_policies": null
    });

    let create_req = test_helpers::with_auth_headers(
        test::TestRequest::post()
            .uri("/api/v1/users")
            .set_json(&user),
        &config,
        &root_token,
    )
    .to_request();

    let create_resp = test::call_service(&app, create_req).await;
    let body: serde_json::Value = test::read_body_json(create_resp).await;
    let user_id = body["data"]["id"].as_str().unwrap();

    // Try to append invalid policy
    let append_payload = serde_json::json!({
        "city_ids": [city]
    });

    let append_req = test_helpers::with_auth_headers(
        test::TestRequest::post()
            .uri(&format!(
                "/api/v1/users/{}/policies/invalid_policy/cities",
                user_id
            ))
            .set_json(&append_payload),
        &config,
        &root_token,
    )
    .to_request();

    let append_resp = test::call_service(&app, append_req).await;
    assert_eq!(append_resp.status(), StatusCode::BAD_REQUEST);
    let body: serde_json::Value = test::read_body_json(append_resp).await;
    assert!(
        body["message"]
            .as_str()
            .unwrap()
            .contains("Invalid policy name")
    );
}

#[actix_rt::test]
async fn append_policy_to_nonexistent_user_returns_404() {
    let pool = test_helpers::setup_test_db().await;
    test_helpers::clean_database(&pool).await;

    let config = test_helpers::build_test_config();
    let app = test_helpers::create_full_test_app(pool.clone(), config.clone()).await;

    let city = db_fixtures::insert_city(&pool, "Test City").await;

    let root_claims = test_helpers::build_root_claims();
    let root_token = test_helpers::generate_jwt(&root_claims, &config.jwt_secret);

    let random_user_id = Uuid::new_v4();

    let append_payload = serde_json::json!({
        "city_ids": [city]
    });

    let append_req = test_helpers::with_auth_headers(
        test::TestRequest::post()
            .uri(&format!(
                "/api/v1/users/{}/policies/read_victims/cities",
                random_user_id
            ))
            .set_json(&append_payload),
        &config,
        &root_token,
    )
    .to_request();

    let append_resp = test::call_service(&app, append_req).await;
    assert_eq!(append_resp.status(), StatusCode::NOT_FOUND);
}

#[actix_rt::test]
async fn append_policy_with_empty_city_ids_array() {
    let pool = test_helpers::setup_test_db().await;
    test_helpers::clean_database(&pool).await;

    let config = test_helpers::build_test_config();
    let app = test_helpers::create_full_test_app(pool.clone(), config.clone()).await;

    let city = db_fixtures::insert_city(&pool, "Test City").await;

    let root_claims = test_helpers::build_root_claims();
    let root_token = test_helpers::generate_jwt(&root_claims, &config.jwt_secret);

    // Create CITY_USER
    let user = serde_json::json!({
        "rank": "SD PM",
        "registration": "100012345",
        "full_name": "City User",
        "profile": "CITY_USER",
        "email": "user@test.com",
        "password": "password123",
        "city_id": city,
        "permission_policies": null
    });

    let create_req = test_helpers::with_auth_headers(
        test::TestRequest::post()
            .uri("/api/v1/users")
            .set_json(&user),
        &config,
        &root_token,
    )
    .to_request();

    let create_resp = test::call_service(&app, create_req).await;
    assert_eq!(create_resp.status(), StatusCode::CREATED);
    let body: serde_json::Value = test::read_body_json(create_resp).await;
    let user_id = body["data"]["id"].as_str().unwrap();

    let get_before_req = test_helpers::with_auth_headers(
        test::TestRequest::get().uri(&format!("/api/v1/users/{}", user_id)),
        &config,
        &root_token,
    )
    .to_request();
    let get_before_resp = test::call_service(&app, get_before_req).await;
    assert_eq!(get_before_resp.status(), StatusCode::OK);
    let before_body: serde_json::Value = test::read_body_json(get_before_resp).await;
    let before_policies = before_body["data"]["permission_policies"].clone();

    // Try to append with empty city_ids (should succeed but not change anything)
    let append_payload = serde_json::json!({
        "city_ids": []
    });

    let append_req = test_helpers::with_auth_headers(
        test::TestRequest::post()
            .uri(&format!(
                "/api/v1/users/{}/policies/read_victims/cities",
                user_id
            ))
            .set_json(&append_payload),
        &config,
        &root_token,
    )
    .to_request();

    let append_resp = test::call_service(&app, append_req).await;
    // This should succeed (empty array is valid, just doesn't add anything)
    assert_eq!(append_resp.status(), StatusCode::OK);

    let get_after_req = test_helpers::with_auth_headers(
        test::TestRequest::get().uri(&format!("/api/v1/users/{}", user_id)),
        &config,
        &root_token,
    )
    .to_request();
    let get_after_resp = test::call_service(&app, get_after_req).await;
    assert_eq!(get_after_resp.status(), StatusCode::OK);
    let after_body: serde_json::Value = test::read_body_json(get_after_resp).await;
    assert_eq!(after_body["data"]["permission_policies"], before_policies);
}
