use actix_web::{http::StatusCode, test};

use crate::common::{test_helpers, db_fixtures};

/// VULNERABILITY TEST: User should NOT be able to change another user's password
/// even if they know the current password
#[actix_rt::test]
async fn city_admin_cannot_change_other_city_admin_password() {
    let pool = test_helpers::setup_test_db().await;
    test_helpers::clean_database(&pool).await;

    let config = test_helpers::build_test_config();
    let app = test_helpers::create_full_test_app(pool.clone(), config.clone()).await;

    let city_a = db_fixtures::insert_city(&pool, "Cidade A").await;
    let city_b = db_fixtures::insert_city(&pool, "Cidade B").await;

    let root_claims = test_helpers::build_root_claims();
    let root_token = test_helpers::generate_jwt(&root_claims, &config.jwt_secret);

    // Create CITY_ADMIN A
    let admin_a_payload = serde_json::json!({
        "rank": "CAP PM",
        "registration": "100012345",
        "full_name": "Admin A",
        "profile": "CITY_ADMIN",
        "email": "admin.a@test.com",
        "password": "password123",
        "city_id": city_a,
        "permission_policies": null
    });

    let create_a_req = test_helpers::with_auth_headers(
        test::TestRequest::post()
            .uri("/api/v1/users")
            .set_json(&admin_a_payload),
        &config,
        &root_token,
    )
    .to_request();

    let create_a_resp = test::call_service(&app, create_a_req).await;
    assert_eq!(create_a_resp.status(), StatusCode::CREATED);
    let body_a: serde_json::Value = test::read_body_json(create_a_resp).await;
    let _admin_a_id = body_a["data"]["id"].as_str().unwrap();

    // Create CITY_ADMIN B with known password
    let admin_b_payload = serde_json::json!({
        "rank": "CAP PM",
        "registration": "100012346",
        "full_name": "Admin B",
        "profile": "CITY_ADMIN",
        "email": "admin.b@test.com",
        "password": "knownPassword456",
        "city_id": city_b,
        "permission_policies": null
    });

    let create_b_req = test_helpers::with_auth_headers(
        test::TestRequest::post()
            .uri("/api/v1/users")
            .set_json(&admin_b_payload),
        &config,
        &root_token,
    )
    .to_request();

    let create_b_resp = test::call_service(&app, create_b_req).await;
    assert_eq!(create_b_resp.status(), StatusCode::CREATED);
    let body_b: serde_json::Value = test::read_body_json(create_b_resp).await;
    let admin_b_id = body_b["data"]["id"].as_str().unwrap();

    // Admin A tries to login to get their token
    let login_payload_a = serde_json::json!({
        "email": "admin.a@test.com",
        "password": "password123"
    });

    let login_req_a = test::TestRequest::post()
        .uri("/api/v1/auth/login")
        .insert_header(("api_key", config.api_key.clone()))
        .set_json(&login_payload_a)
        .to_request();

    let login_resp_a = test::call_service(&app, login_req_a).await;
    let login_body_a: serde_json::Value = test::read_body_json(login_resp_a).await;
    let admin_a_token = login_body_a["data"]["token"].as_str().unwrap();

    // VULNERABILITY: Admin A tries to change Admin B's password
    // (knowing Admin B's current password)
    let change_password_payload = serde_json::json!({
        "current_password": "knownPassword456",  // Admin B's current password
        "new_password": "hacked123"
    });

    let change_req = test_helpers::with_auth_headers(
        test::TestRequest::patch()
            .uri(&format!("/api/v1/users/{}/password", admin_b_id))
            .set_json(&change_password_payload),
        &config,
        admin_a_token,
    )
    .to_request();

    let change_resp = test::call_service(&app, change_req).await;

    // EXPECTED: Should return 403 FORBIDDEN (user can only change their own password)
    // ACTUAL (BUG): Currently returns 200 OK and changes the password!
    println!("Response status: {:?}", change_resp.status());

    // This test will FAIL until the vulnerability is fixed
    assert_eq!(
        change_resp.status(),
        StatusCode::FORBIDDEN,
        "SECURITY BUG: User should NOT be able to change another user's password!"
    );
}

#[actix_rt::test]
async fn city_user_cannot_change_other_city_user_password() {
    let pool = test_helpers::setup_test_db().await;
    test_helpers::clean_database(&pool).await;

    let config = test_helpers::build_test_config();
    let app = test_helpers::create_full_test_app(pool.clone(), config.clone()).await;

    let city = db_fixtures::insert_city(&pool, "Test City").await;

    let root_claims = test_helpers::build_root_claims();
    let root_token = test_helpers::generate_jwt(&root_claims, &config.jwt_secret);

    // Create CITY_USER 1
    let user1_payload = serde_json::json!({
        "rank": "SD PM",
        "registration": "100012345",
        "full_name": "User 1",
        "profile": "CITY_USER",
        "email": "user1@test.com",
        "password": "password123",
        "city_id": city,
        "permission_policies": null
    });

    let create_u1_req = test_helpers::with_auth_headers(
        test::TestRequest::post()
            .uri("/api/v1/users")
            .set_json(&user1_payload),
        &config,
        &root_token,
    )
    .to_request();

    let create_u1_resp = test::call_service(&app, create_u1_req).await;
    let body_u1: serde_json::Value = test::read_body_json(create_u1_resp).await;
    let _user1_id = body_u1["data"]["id"].as_str().unwrap();

    // Create CITY_USER 2
    let user2_payload = serde_json::json!({
        "rank": "SD PM",
        "registration": "100012346",
        "full_name": "User 2",
        "profile": "CITY_USER",
        "email": "user2@test.com",
        "password": "knownPassword456",
        "city_id": city,
        "permission_policies": null
    });

    let create_u2_req = test_helpers::with_auth_headers(
        test::TestRequest::post()
            .uri("/api/v1/users")
            .set_json(&user2_payload),
        &config,
        &root_token,
    )
    .to_request();

    let create_u2_resp = test::call_service(&app, create_u2_req).await;
    let body_u2: serde_json::Value = test::read_body_json(create_u2_resp).await;
    let user2_id = body_u2["data"]["id"].as_str().unwrap();

    // User 1 logs in
    let login_payload = serde_json::json!({
        "email": "user1@test.com",
        "password": "password123"
    });

    let login_req = test::TestRequest::post()
        .uri("/api/v1/auth/login")
        .insert_header(("api_key", config.api_key.clone()))
        .set_json(&login_payload)
        .to_request();

    let login_resp = test::call_service(&app, login_req).await;
    let login_body: serde_json::Value = test::read_body_json(login_resp).await;
    let user1_token = login_body["data"]["token"].as_str().unwrap();

    // VULNERABILITY: User 1 tries to change User 2's password
    let change_password_payload = serde_json::json!({
        "current_password": "knownPassword456",
        "new_password": "hacked123"
    });

    let change_req = test_helpers::with_auth_headers(
        test::TestRequest::patch()
            .uri(&format!("/api/v1/users/{}/password", user2_id))
            .set_json(&change_password_payload),
        &config,
        user1_token,
    )
    .to_request();

    let change_resp = test::call_service(&app, change_req).await;

    // This should be FORBIDDEN
    assert_eq!(
        change_resp.status(),
        StatusCode::FORBIDDEN,
        "SECURITY BUG: User should only be able to change their own password!"
    );
}

#[actix_rt::test]
async fn user_can_change_own_password() {
    let pool = test_helpers::setup_test_db().await;
    test_helpers::clean_database(&pool).await;

    let config = test_helpers::build_test_config();
    let app = test_helpers::create_full_test_app(pool.clone(), config.clone()).await;

    let city = db_fixtures::insert_city(&pool, "Test City").await;

    let root_claims = test_helpers::build_root_claims();
    let root_token = test_helpers::generate_jwt(&root_claims, &config.jwt_secret);

    // Create user
    let user_payload = serde_json::json!({
        "rank": "SD PM",
        "registration": "100012345",
        "full_name": "Test User",
        "profile": "CITY_USER",
        "email": "testuser@test.com",
        "password": "oldPassword123",
        "city_id": city,
        "permission_policies": null
    });

    let create_req = test_helpers::with_auth_headers(
        test::TestRequest::post()
            .uri("/api/v1/users")
            .set_json(&user_payload),
        &config,
        &root_token,
    )
    .to_request();

    let create_resp = test::call_service(&app, create_req).await;
    let body: serde_json::Value = test::read_body_json(create_resp).await;
    let user_id = body["data"]["id"].as_str().unwrap();

    // User logs in
    let login_payload = serde_json::json!({
        "email": "testuser@test.com",
        "password": "oldPassword123"
    });

    let login_req = test::TestRequest::post()
        .uri("/api/v1/auth/login")
        .insert_header(("api_key", config.api_key.clone()))
        .set_json(&login_payload)
        .to_request();

    let login_resp = test::call_service(&app, login_req).await;
    let login_body: serde_json::Value = test::read_body_json(login_resp).await;
    let user_token = login_body["data"]["token"].as_str().unwrap();

    // User changes their OWN password (should work)
    let change_password_payload = serde_json::json!({
        "current_password": "oldPassword123",
        "new_password": "newPassword456"
    });

    let change_req = test_helpers::with_auth_headers(
        test::TestRequest::patch()
            .uri(&format!("/api/v1/users/{}/password", user_id))
            .set_json(&change_password_payload),
        &config,
        user_token,
    )
    .to_request();

    let change_resp = test::call_service(&app, change_req).await;

    // This should succeed
    assert_eq!(change_resp.status(), StatusCode::OK);

    // Verify new password works
    let login2_payload = serde_json::json!({
        "email": "testuser@test.com",
        "password": "newPassword456"
    });

    let login2_req = test::TestRequest::post()
        .uri("/api/v1/auth/login")
        .insert_header(("api_key", config.api_key.clone()))
        .set_json(&login2_payload)
        .to_request();

    let login2_resp = test::call_service(&app, login2_req).await;
    assert_eq!(login2_resp.status(), StatusCode::OK);
}

#[actix_rt::test]
async fn root_can_change_any_user_password() {
    let pool = test_helpers::setup_test_db().await;
    test_helpers::clean_database(&pool).await;

    let config = test_helpers::build_test_config();
    let app = test_helpers::create_full_test_app(pool.clone(), config.clone()).await;

    let city = db_fixtures::insert_city(&pool, "Test City").await;

    let root_claims = test_helpers::build_root_claims();
    let root_token = test_helpers::generate_jwt(&root_claims, &config.jwt_secret);

    // Create CITY_USER
    let user_payload = serde_json::json!({
        "rank": "SD PM",
        "registration": "100012345",
        "full_name": "City User",
        "profile": "CITY_USER",
        "email": "cityuser@test.com",
        "password": "userPassword123",
        "city_id": city,
        "permission_policies": null
    });

    let create_req = test_helpers::with_auth_headers(
        test::TestRequest::post()
            .uri("/api/v1/users")
            .set_json(&user_payload),
        &config,
        &root_token,
    )
    .to_request();

    let create_resp = test::call_service(&app, create_req).await;
    let body: serde_json::Value = test::read_body_json(create_resp).await;
    let user_id = body["data"]["id"].as_str().unwrap();

    // ROOT changes the user's password (should work)
    let change_password_payload = serde_json::json!({
        "current_password": "userPassword123",
        "new_password": "newPasswordByRoot"
    });

    let change_req = test_helpers::with_auth_headers(
        test::TestRequest::patch()
            .uri(&format!("/api/v1/users/{}/password", user_id))
            .set_json(&change_password_payload),
        &config,
        &root_token,
    )
    .to_request();

    let change_resp = test::call_service(&app, change_req).await;

    // ROOT should be able to change any user's password
    assert_eq!(change_resp.status(), StatusCode::OK);
}
