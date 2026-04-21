use actix_web::{http::StatusCode, test};
use sqlx::PgPool;

use crate::common::{db_fixtures, test_helpers};

/// User should NOT be able to change another user's password
/// even if they know the current password
#[sqlx::test]
async fn city_admin_cannot_change_other_city_admin_password(pool: PgPool) {

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
    let _admin_b_id = body_b["data"]["id"].as_str().unwrap();

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

    // Admin A tries to change Admin B's password (knowing Admin B's current password)
    let change_password_payload = serde_json::json!({
        "current_password": "knownPassword456",  // Admin B's current password
        "new_password": "hacked123"
    });

    let change_req = test_helpers::with_auth_headers(
        test::TestRequest::patch()
            .uri("/api/v1/users/password")
            .set_json(&change_password_payload),
        &config,
        admin_a_token,
    )
    .to_request();

    let change_resp = test::call_service(&app, change_req).await;

    assert_eq!(
        change_resp.status(),
        StatusCode::BAD_REQUEST,
        "User should not be able to change another user's password"
    );

    let login_payload_b = serde_json::json!({
        "email": "admin.b@test.com",
        "password": "knownPassword456"
    });

    let login_req_b = test::TestRequest::post()
        .uri("/api/v1/auth/login")
        .insert_header(("api_key", config.api_key.clone()))
        .set_json(&login_payload_b)
        .to_request();

    let login_resp_b = test::call_service(&app, login_req_b).await;
    assert_eq!(login_resp_b.status(), StatusCode::OK);
}

#[sqlx::test]
async fn city_user_cannot_change_other_city_user_password(pool: PgPool) {

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
    let _user2_id = body_u2["data"]["id"].as_str().unwrap();

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

    // User 1 tries to change User 2's password
    let change_password_payload = serde_json::json!({
        "current_password": "knownPassword456",
        "new_password": "hacked123"
    });

    let change_req = test_helpers::with_auth_headers(
        test::TestRequest::patch()
            .uri("/api/v1/users/password")
            .set_json(&change_password_payload),
        &config,
        user1_token,
    )
    .to_request();

    let change_resp = test::call_service(&app, change_req).await;

    assert_eq!(
        change_resp.status(),
        StatusCode::BAD_REQUEST,
        "User should not be able to change another user's password"
    );

    let login_payload_b = serde_json::json!({
        "email": "user2@test.com",
        "password": "knownPassword456"
    });

    let login_req_b = test::TestRequest::post()
        .uri("/api/v1/auth/login")
        .insert_header(("api_key", config.api_key.clone()))
        .set_json(&login_payload_b)
        .to_request();

    let login_resp_b = test::call_service(&app, login_req_b).await;
    assert_eq!(login_resp_b.status(), StatusCode::OK);
}

#[sqlx::test]
async fn user_can_change_own_password(pool: PgPool) {

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
    let _user_id = body["data"]["id"].as_str().unwrap();

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
            .uri("/api/v1/users/password")
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

#[sqlx::test]
async fn root_can_change_any_user_password(pool: PgPool) {

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

    // ROOT resets the user's password
    let reset_req = test_helpers::with_auth_headers(
        test::TestRequest::post().uri(&format!("/api/v1/users/{}/password/reset", user_id)),
        &config,
        &root_token,
    )
    .to_request();

    let reset_resp = test::call_service(&app, reset_req).await;

    // ROOT should be able to reset any user's password
    assert_eq!(reset_resp.status(), StatusCode::OK);
    let reset_body: serde_json::Value = test::read_body_json(reset_resp).await;
    assert!(
        reset_body["data"]["temporary_password"]
            .as_str()
            .unwrap()
            .starts_with("prov")
    );
}

#[sqlx::test]
async fn root_reset_password_allows_login_with_temporary_password(pool: PgPool) {

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

    // ROOT resets password
    let reset_req = test_helpers::with_auth_headers(
        test::TestRequest::post().uri(&format!("/api/v1/users/{}/password/reset", user_id)),
        &config,
        &root_token,
    )
    .to_request();

    let reset_resp = test::call_service(&app, reset_req).await;
    assert_eq!(reset_resp.status(), StatusCode::OK);
    let reset_body: serde_json::Value = test::read_body_json(reset_resp).await;
    let temporary_password = reset_body["data"]["temporary_password"].as_str().unwrap();

    // Verify new password works
    let login_payload = serde_json::json!({
        "email": "cityuser@test.com",
        "password": temporary_password
    });

    let login_req = test::TestRequest::post()
        .uri("/api/v1/auth/login")
        .insert_header(("api_key", config.api_key.clone()))
        .set_json(&login_payload)
        .to_request();

    let login_resp = test::call_service(&app, login_req).await;
    assert_eq!(login_resp.status(), StatusCode::OK);
}

#[sqlx::test]
async fn city_admin_can_reset_password_for_user_in_same_city(pool: PgPool) {

    let config = test_helpers::build_test_config();
    let app = test_helpers::create_full_test_app(pool.clone(), config.clone()).await;

    let city = db_fixtures::insert_city(&pool, "Cidade Segura").await;

    let root_claims = test_helpers::build_root_claims();
    let root_token = test_helpers::generate_jwt(&root_claims, &config.jwt_secret);

    let admin_payload = serde_json::json!({
        "rank": "CAP PM",
        "registration": "100099991",
        "full_name": "Admin Mesmo Municipio",
        "profile": "CITY_ADMIN",
        "email": "same.city.admin@test.com",
        "password": "adminPassword123",
        "city_id": city,
        "permission_policies": null
    });
    let create_admin_req = test_helpers::with_auth_headers(
        test::TestRequest::post()
            .uri("/api/v1/users")
            .set_json(&admin_payload),
        &config,
        &root_token,
    )
    .to_request();
    let create_admin_resp = test::call_service(&app, create_admin_req).await;
    assert_eq!(create_admin_resp.status(), StatusCode::CREATED);
    let admin_body: serde_json::Value = test::read_body_json(create_admin_resp).await;
    let admin_id = admin_body["data"]["id"].as_str().unwrap();

    let user_payload = serde_json::json!({
        "rank": "SD PM",
        "registration": "100099992",
        "full_name": "Usuario Mesmo Municipio",
        "profile": "CITY_USER",
        "email": "same.city.user@test.com",
        "password": "userPassword123",
        "city_id": city,
        "permission_policies": null
    });
    let create_user_req = test_helpers::with_auth_headers(
        test::TestRequest::post()
            .uri("/api/v1/users")
            .set_json(&user_payload),
        &config,
        &root_token,
    )
    .to_request();
    let create_user_resp = test::call_service(&app, create_user_req).await;
    assert_eq!(create_user_resp.status(), StatusCode::CREATED);
    let user_body: serde_json::Value = test::read_body_json(create_user_resp).await;
    let user_id = user_body["data"]["id"].as_str().unwrap();

    let login_payload = serde_json::json!({
        "email": "same.city.admin@test.com",
        "password": "adminPassword123"
    });
    let login_req = test::TestRequest::post()
        .uri("/api/v1/auth/login")
        .insert_header(("api_key", config.api_key.clone()))
        .set_json(&login_payload)
        .to_request();
    let login_resp = test::call_service(&app, login_req).await;
    assert_eq!(login_resp.status(), StatusCode::OK);
    let login_body: serde_json::Value = test::read_body_json(login_resp).await;
    let admin_token = login_body["data"]["token"].as_str().unwrap();

    let reset_req = test_helpers::with_auth_headers(
        test::TestRequest::post().uri(&format!("/api/v1/users/{}/password/reset", user_id)),
        &config,
        admin_token,
    )
    .to_request();
    let reset_resp = test::call_service(&app, reset_req).await;
    assert_eq!(reset_resp.status(), StatusCode::OK);
    let reset_body: serde_json::Value = test::read_body_json(reset_resp).await;
    let temporary_password = reset_body["data"]["temporary_password"].as_str().unwrap();
    assert!(temporary_password.starts_with("prov"));

    let temp_login_payload = serde_json::json!({
        "email": "same.city.user@test.com",
        "password": temporary_password
    });
    let temp_login_req = test::TestRequest::post()
        .uri("/api/v1/auth/login")
        .insert_header(("api_key", config.api_key.clone()))
        .set_json(&temp_login_payload)
        .to_request();
    let temp_login_resp = test::call_service(&app, temp_login_req).await;
    assert_eq!(temp_login_resp.status(), StatusCode::OK);
    let temp_login_body: serde_json::Value = test::read_body_json(temp_login_resp).await;
    assert_eq!(temp_login_body["data"]["id"].as_str().unwrap(), user_id);
    assert!(
        !temp_login_body["data"]["token"]
            .as_str()
            .unwrap()
            .is_empty()
    );

    assert_ne!(admin_id, user_id);
}

#[sqlx::test]
async fn city_admin_cannot_reset_password_for_user_in_other_city(pool: PgPool) {

    let config = test_helpers::build_test_config();
    let app = test_helpers::create_full_test_app(pool.clone(), config.clone()).await;

    let city_a = db_fixtures::insert_city(&pool, "Cidade A").await;
    let city_b = db_fixtures::insert_city(&pool, "Cidade B").await;

    let root_claims = test_helpers::build_root_claims();
    let root_token = test_helpers::generate_jwt(&root_claims, &config.jwt_secret);

    let admin_payload = serde_json::json!({
        "rank": "CAP PM",
        "registration": "100099993",
        "full_name": "Admin Cidade A",
        "profile": "CITY_ADMIN",
        "email": "other.city.admin@test.com",
        "password": "adminPassword123",
        "city_id": city_a,
        "permission_policies": null
    });
    let create_admin_req = test_helpers::with_auth_headers(
        test::TestRequest::post()
            .uri("/api/v1/users")
            .set_json(&admin_payload),
        &config,
        &root_token,
    )
    .to_request();
    let create_admin_resp = test::call_service(&app, create_admin_req).await;
    assert_eq!(create_admin_resp.status(), StatusCode::CREATED);

    let user_payload = serde_json::json!({
        "rank": "SD PM",
        "registration": "100099994",
        "full_name": "Usuario Cidade B",
        "profile": "CITY_USER",
        "email": "other.city.user@test.com",
        "password": "userPassword123",
        "city_id": city_b,
        "permission_policies": null
    });
    let create_user_req = test_helpers::with_auth_headers(
        test::TestRequest::post()
            .uri("/api/v1/users")
            .set_json(&user_payload),
        &config,
        &root_token,
    )
    .to_request();
    let create_user_resp = test::call_service(&app, create_user_req).await;
    assert_eq!(create_user_resp.status(), StatusCode::CREATED);
    let user_body: serde_json::Value = test::read_body_json(create_user_resp).await;
    let user_id = user_body["data"]["id"].as_str().unwrap();

    let login_payload = serde_json::json!({
        "email": "other.city.admin@test.com",
        "password": "adminPassword123"
    });
    let login_req = test::TestRequest::post()
        .uri("/api/v1/auth/login")
        .insert_header(("api_key", config.api_key.clone()))
        .set_json(&login_payload)
        .to_request();
    let login_resp = test::call_service(&app, login_req).await;
    assert_eq!(login_resp.status(), StatusCode::OK);
    let login_body: serde_json::Value = test::read_body_json(login_resp).await;
    let admin_token = login_body["data"]["token"].as_str().unwrap();

    let reset_req = test_helpers::with_auth_headers(
        test::TestRequest::post().uri(&format!("/api/v1/users/{}/password/reset", user_id)),
        &config,
        admin_token,
    )
    .to_request();
    let reset_resp = test::call_service(&app, reset_req).await;
    assert_eq!(reset_resp.status(), StatusCode::FORBIDDEN);
    let reset_body: serde_json::Value = test::read_body_json(reset_resp).await;
    assert_eq!(
        reset_body["message"].as_str().unwrap(),
        "Forbidden: CITY_ADMIN can only reset passwords for users in the same city"
    );
}

#[sqlx::test]
async fn city_user_cannot_reset_passwords(pool: PgPool) {

    let config = test_helpers::build_test_config();
    let app = test_helpers::create_full_test_app(pool.clone(), config.clone()).await;

    let city = db_fixtures::insert_city(&pool, "Cidade Restrita").await;
    let root_claims = test_helpers::build_root_claims();
    let root_token = test_helpers::generate_jwt(&root_claims, &config.jwt_secret);

    let acting_user_payload = serde_json::json!({
        "rank": "SD PM",
        "registration": "100099995",
        "full_name": "Usuario Ator",
        "profile": "CITY_USER",
        "email": "actor.city.user@test.com",
        "password": "userPassword123",
        "city_id": city,
        "permission_policies": null
    });
    let create_actor_req = test_helpers::with_auth_headers(
        test::TestRequest::post()
            .uri("/api/v1/users")
            .set_json(&acting_user_payload),
        &config,
        &root_token,
    )
    .to_request();
    let create_actor_resp = test::call_service(&app, create_actor_req).await;
    assert_eq!(create_actor_resp.status(), StatusCode::CREATED);

    let target_user_payload = serde_json::json!({
        "rank": "SD PM",
        "registration": "100099996",
        "full_name": "Usuario Alvo",
        "profile": "CITY_USER",
        "email": "target.city.user@test.com",
        "password": "targetPassword123",
        "city_id": city,
        "permission_policies": null
    });
    let create_target_req = test_helpers::with_auth_headers(
        test::TestRequest::post()
            .uri("/api/v1/users")
            .set_json(&target_user_payload),
        &config,
        &root_token,
    )
    .to_request();
    let create_target_resp = test::call_service(&app, create_target_req).await;
    assert_eq!(create_target_resp.status(), StatusCode::CREATED);
    let target_body: serde_json::Value = test::read_body_json(create_target_resp).await;
    let target_user_id = target_body["data"]["id"].as_str().unwrap();

    let login_payload = serde_json::json!({
        "email": "actor.city.user@test.com",
        "password": "userPassword123"
    });
    let login_req = test::TestRequest::post()
        .uri("/api/v1/auth/login")
        .insert_header(("api_key", config.api_key.clone()))
        .set_json(&login_payload)
        .to_request();
    let login_resp = test::call_service(&app, login_req).await;
    assert_eq!(login_resp.status(), StatusCode::OK);
    let login_body: serde_json::Value = test::read_body_json(login_resp).await;
    let actor_token = login_body["data"]["token"].as_str().unwrap();

    let reset_req = test_helpers::with_auth_headers(
        test::TestRequest::post().uri(&format!("/api/v1/users/{}/password/reset", target_user_id)),
        &config,
        actor_token,
    )
    .to_request();
    let reset_resp = test::call_service(&app, reset_req).await;
    assert_eq!(reset_resp.status(), StatusCode::FORBIDDEN);
    let reset_body: serde_json::Value = test::read_body_json(reset_resp).await;
    assert_eq!(
        reset_body["message"].as_str().unwrap(),
        "Forbidden: Only ROOT or CITY_ADMIN can reset passwords"
    );
}
