use actix_web::{http::StatusCode, test};

use crate::common::{db_fixtures, test_helpers};

// ==================== UPDATE MEMBER FUNCTION TESTS ====================

#[actix_rt::test]
async fn update_member_function_success() {
    let pool = test_helpers::setup_test_db().await;
    test_helpers::clean_database(&pool).await;

    let config = test_helpers::build_test_config();
    let app = test_helpers::create_full_test_app(pool.clone(), config.clone()).await;

    let city_id = db_fixtures::insert_city(&pool, "Test City").await;
    let creator_id = db_fixtures::insert_user(
        &pool,
        "100001",
        "creator@test.com",
        "CITY_USER",
        Some(city_id),
    )
    .await;
    let member_id = db_fixtures::insert_user(
        &pool,
        "100002",
        "member@test.com",
        "CITY_USER",
        Some(city_id),
    )
    .await;

    let mut claims = test_helpers::build_city_user_claims(city_id);
    claims.id = creator_id.to_string();
    let token = test_helpers::generate_jwt(&claims, &config.jwt_secret);

    // Create session with member as Patroller
    let create_payload = serde_json::json!({
        "description": "Test session",
        "members": [
            {
                "user_id": creator_id,
                "function": "Commander"
            },
            {
                "user_id": member_id,
                "function": "Patroller"
            }
        ]
    });

    let create_req = test_helpers::with_auth_headers(
        test::TestRequest::post()
            .uri("/api/v1/work-sessions")
            .set_json(&create_payload),
        &config,
        &token,
    )
    .to_request();

    let create_resp = test::call_service(&app, create_req).await;
    assert_eq!(create_resp.status(), StatusCode::CREATED);
    let create_body: serde_json::Value = test::read_body_json(create_resp).await;
    let session_id = create_body["data"]["id"].as_str().unwrap();

    // Update member function to Driver
    let update_payload = serde_json::json!({
        "function": "Driver"
    });

    let update_req = test_helpers::with_auth_headers(
        test::TestRequest::put()
            .uri(&format!(
                "/api/v1/work-sessions/{}/members/{}/function",
                session_id, member_id
            ))
            .set_json(&update_payload),
        &config,
        &token,
    )
    .to_request();

    let update_resp = test::call_service(&app, update_req).await;
    assert_eq!(update_resp.status(), StatusCode::OK);

    // Verify function was updated
    let get_req = test_helpers::with_auth_headers(
        test::TestRequest::get().uri(&format!("/api/v1/work-sessions/{}", session_id)),
        &config,
        &token,
    )
    .to_request();

    let get_resp = test::call_service(&app, get_req).await;
    let get_body: serde_json::Value = test::read_body_json(get_resp).await;
    let members = get_body["data"]["members"].as_array().unwrap();

    // Find the member that was updated
    let updated_member = members
        .iter()
        .find(|m| m["user_id"].as_str().unwrap() == member_id.to_string())
        .unwrap();

    assert_eq!(updated_member["function"].as_str().unwrap(), "Driver");
}

#[actix_rt::test]
async fn update_member_function_to_commander_fails_when_already_has_commander() {
    let pool = test_helpers::setup_test_db().await;
    test_helpers::clean_database(&pool).await;

    let config = test_helpers::build_test_config();
    let app = test_helpers::create_full_test_app(pool.clone(), config.clone()).await;

    let city_id = db_fixtures::insert_city(&pool, "Test City").await;
    let creator_id = db_fixtures::insert_user(
        &pool,
        "100003",
        "creator@test.com",
        "CITY_USER",
        Some(city_id),
    )
    .await;
    let member_id = db_fixtures::insert_user(
        &pool,
        "100004",
        "member@test.com",
        "CITY_USER",
        Some(city_id),
    )
    .await;

    let mut claims = test_helpers::build_city_user_claims(city_id);
    claims.id = creator_id.to_string();
    let token = test_helpers::generate_jwt(&claims, &config.jwt_secret);

    // Create session
    let create_payload = serde_json::json!({
        "description": "Test session",
        "members": [
            {
                "user_id": creator_id,
                "function": "Commander"
            },
            {
                "user_id": member_id,
                "function": "Driver"
            }
        ]
    });

    let create_req = test_helpers::with_auth_headers(
        test::TestRequest::post()
            .uri("/api/v1/work-sessions")
            .set_json(&create_payload),
        &config,
        &token,
    )
    .to_request();

    let create_resp = test::call_service(&app, create_req).await;
    let create_body: serde_json::Value = test::read_body_json(create_resp).await;
    let session_id = create_body["data"]["id"].as_str().unwrap();

    // Try to update member function to Commander (should fail - creator is already Commander)
    let update_payload = serde_json::json!({
        "function": "Commander"
    });

    let update_req = test_helpers::with_auth_headers(
        test::TestRequest::put()
            .uri(&format!(
                "/api/v1/work-sessions/{}/members/{}/function",
                session_id, member_id
            ))
            .set_json(&update_payload),
        &config,
        &token,
    )
    .to_request();

    let update_resp = test::call_service(&app, update_req).await;
    assert_eq!(update_resp.status(), StatusCode::BAD_REQUEST);
}

#[actix_rt::test]
async fn update_member_function_non_creator_fails() {
    let pool = test_helpers::setup_test_db().await;
    test_helpers::clean_database(&pool).await;

    let config = test_helpers::build_test_config();
    let app = test_helpers::create_full_test_app(pool.clone(), config.clone()).await;

    let city_id = db_fixtures::insert_city(&pool, "Test City").await;
    let creator_id = db_fixtures::insert_user(
        &pool,
        "100005",
        "creator@test.com",
        "CITY_USER",
        Some(city_id),
    )
    .await;
    let member_id = db_fixtures::insert_user(
        &pool,
        "100006",
        "member@test.com",
        "CITY_USER",
        Some(city_id),
    )
    .await;
    let other_user_id = db_fixtures::insert_user(
        &pool,
        "100007",
        "other@test.com",
        "CITY_USER",
        Some(city_id),
    )
    .await;

    let mut creator_claims = test_helpers::build_city_user_claims(city_id);
    creator_claims.id = creator_id.to_string();
    let creator_token = test_helpers::generate_jwt(&creator_claims, &config.jwt_secret);

    // Create session as creator
    let create_payload = serde_json::json!({
        "description": "Test session",
        "members": [
            {
                "user_id": creator_id,
                "function": "Commander"
            },
            {
                "user_id": member_id,
                "function": "Driver"
            },
            {
                "user_id": other_user_id,
                "function": "Patroller"
            }
        ]
    });

    let create_req = test_helpers::with_auth_headers(
        test::TestRequest::post()
            .uri("/api/v1/work-sessions")
            .set_json(&create_payload),
        &config,
        &creator_token,
    )
    .to_request();

    let create_resp = test::call_service(&app, create_req).await;
    let create_body: serde_json::Value = test::read_body_json(create_resp).await;
    let session_id = create_body["data"]["id"].as_str().unwrap();

    // Try to update as different user (not creator)
    let mut other_claims = test_helpers::build_city_user_claims(city_id);
    other_claims.id = other_user_id.to_string();
    let other_token = test_helpers::generate_jwt(&other_claims, &config.jwt_secret);

    let update_payload = serde_json::json!({
        "function": "Patroller"
    });

    let update_req = test_helpers::with_auth_headers(
        test::TestRequest::put()
            .uri(&format!(
                "/api/v1/work-sessions/{}/members/{}/function",
                session_id, member_id
            ))
            .set_json(&update_payload),
        &config,
        &other_token,
    )
    .to_request();

    let update_resp = test::call_service(&app, update_req).await;
    assert_eq!(update_resp.status(), StatusCode::FORBIDDEN);
}

#[actix_rt::test]
async fn update_member_function_nonexistent_member_fails() {
    let pool = test_helpers::setup_test_db().await;
    test_helpers::clean_database(&pool).await;

    let config = test_helpers::build_test_config();
    let app = test_helpers::create_full_test_app(pool.clone(), config.clone()).await;

    let city_id = db_fixtures::insert_city(&pool, "Test City").await;
    let creator_id = db_fixtures::insert_user(
        &pool,
        "100008",
        "creator@test.com",
        "CITY_USER",
        Some(city_id),
    )
    .await;
    let nonexistent_user_id = uuid::Uuid::new_v4();

    let mut claims = test_helpers::build_city_user_claims(city_id);
    claims.id = creator_id.to_string();
    let token = test_helpers::generate_jwt(&claims, &config.jwt_secret);

    // Create session
    let create_payload = serde_json::json!({
        "description": "Test session",
        "members": [
            {
                "user_id": creator_id,
                "function": "Commander"
            }
        ]
    });

    let create_req = test_helpers::with_auth_headers(
        test::TestRequest::post()
            .uri("/api/v1/work-sessions")
            .set_json(&create_payload),
        &config,
        &token,
    )
    .to_request();

    let create_resp = test::call_service(&app, create_req).await;
    let create_body: serde_json::Value = test::read_body_json(create_resp).await;
    let session_id = create_body["data"]["id"].as_str().unwrap();

    // Try to update function of user that's not in the session
    let update_payload = serde_json::json!({
        "function": "Driver"
    });

    let update_req = test_helpers::with_auth_headers(
        test::TestRequest::put()
            .uri(&format!(
                "/api/v1/work-sessions/{}/members/{}/function",
                session_id, nonexistent_user_id
            ))
            .set_json(&update_payload),
        &config,
        &token,
    )
    .to_request();

    let update_resp = test::call_service(&app, update_req).await;
    assert_eq!(update_resp.status(), StatusCode::NOT_FOUND);
}

// ==================== LIST SESSIONS TESTS ====================

#[actix_rt::test]
async fn list_sessions_returns_own_sessions_for_regular_user() {
    let pool = test_helpers::setup_test_db().await;
    test_helpers::clean_database(&pool).await;

    let config = test_helpers::build_test_config();
    let app = test_helpers::create_full_test_app(pool.clone(), config.clone()).await;

    let city_id = db_fixtures::insert_city(&pool, "Test City").await;
    let user1_id = db_fixtures::insert_user(
        &pool,
        "100009",
        "user1@test.com",
        "CITY_USER",
        Some(city_id),
    )
    .await;
    let user2_id = db_fixtures::insert_user(
        &pool,
        "100010",
        "user2@test.com",
        "CITY_USER",
        Some(city_id),
    )
    .await;

    // Create session as user1
    let mut user1_claims = test_helpers::build_city_user_claims(city_id);
    user1_claims.id = user1_id.to_string();
    let user1_token = test_helpers::generate_jwt(&user1_claims, &config.jwt_secret);

    let create_payload = serde_json::json!({
        "description": "User1 session",
        "members": [
            {
                "user_id": user1_id,
                "function": "Commander"
            }
        ]
    });

    let create_req = test_helpers::with_auth_headers(
        test::TestRequest::post()
            .uri("/api/v1/work-sessions")
            .set_json(&create_payload),
        &config,
        &user1_token,
    )
    .to_request();

    test::call_service(&app, create_req).await;

    // Create session as user2
    let mut user2_claims = test_helpers::build_city_user_claims(city_id);
    user2_claims.id = user2_id.to_string();
    let user2_token = test_helpers::generate_jwt(&user2_claims, &config.jwt_secret);

    let create_payload2 = serde_json::json!({
        "description": "User2 session",
        "members": [
            {
                "user_id": user2_id,
                "function": "Commander"
            }
        ]
    });

    let create_req2 = test_helpers::with_auth_headers(
        test::TestRequest::post()
            .uri("/api/v1/work-sessions")
            .set_json(&create_payload2),
        &config,
        &user2_token,
    )
    .to_request();

    test::call_service(&app, create_req2).await;

    // List sessions as user1 (should only see own session)
    let list_req = test_helpers::with_auth_headers(
        test::TestRequest::get().uri("/api/v1/work-sessions"),
        &config,
        &user1_token,
    )
    .to_request();

    let list_resp = test::call_service(&app, list_req).await;
    assert_eq!(list_resp.status(), StatusCode::OK);
    let list_body: serde_json::Value = test::read_body_json(list_resp).await;
    let sessions = list_body["data"].as_array().unwrap();

    assert_eq!(sessions.len(), 1);
    assert_eq!(
        sessions[0]["description"].as_str().unwrap(),
        "User1 session"
    );
}

#[actix_rt::test]
async fn list_sessions_city_admin_sees_all_city_sessions() {
    let pool = test_helpers::setup_test_db().await;
    test_helpers::clean_database(&pool).await;

    let config = test_helpers::build_test_config();
    let app = test_helpers::create_full_test_app(pool.clone(), config.clone()).await;

    let city_id = db_fixtures::insert_city(&pool, "Test City").await;
    let admin_id = db_fixtures::insert_user(
        &pool,
        "100011",
        "admin@test.com",
        "CITY_ADMIN",
        Some(city_id),
    )
    .await;
    let user_id =
        db_fixtures::insert_user(&pool, "100012", "user@test.com", "CITY_USER", Some(city_id))
            .await;

    // Create session as regular user
    let mut user_claims = test_helpers::build_city_user_claims(city_id);
    user_claims.id = user_id.to_string();
    let user_token = test_helpers::generate_jwt(&user_claims, &config.jwt_secret);

    let create_payload = serde_json::json!({
        "description": "User session",
        "members": [
            {
                "user_id": user_id,
                "function": "Commander"
            }
        ]
    });

    let create_req = test_helpers::with_auth_headers(
        test::TestRequest::post()
            .uri("/api/v1/work-sessions")
            .set_json(&create_payload),
        &config,
        &user_token,
    )
    .to_request();

    test::call_service(&app, create_req).await;

    // Create session as admin
    let mut admin_claims = test_helpers::build_city_admin_claims(city_id);
    admin_claims.id = admin_id.to_string();
    let admin_token = test_helpers::generate_jwt(&admin_claims, &config.jwt_secret);

    let create_payload2 = serde_json::json!({
        "description": "Admin session",
        "members": [
            {
                "user_id": admin_id,
                "function": "Commander"
            }
        ]
    });

    let create_req2 = test_helpers::with_auth_headers(
        test::TestRequest::post()
            .uri("/api/v1/work-sessions")
            .set_json(&create_payload2),
        &config,
        &admin_token,
    )
    .to_request();

    test::call_service(&app, create_req2).await;

    // List sessions as admin (should see both)
    let list_req = test_helpers::with_auth_headers(
        test::TestRequest::get().uri("/api/v1/work-sessions"),
        &config,
        &admin_token,
    )
    .to_request();

    let list_resp = test::call_service(&app, list_req).await;
    assert_eq!(list_resp.status(), StatusCode::OK);
    let list_body: serde_json::Value = test::read_body_json(list_resp).await;
    let sessions = list_body["data"].as_array().unwrap();

    assert_eq!(sessions.len(), 2);
}

#[actix_rt::test]
async fn list_sessions_root_sees_all_sessions() {
    let pool = test_helpers::setup_test_db().await;
    test_helpers::clean_database(&pool).await;

    let config = test_helpers::build_test_config();
    let app = test_helpers::create_full_test_app(pool.clone(), config.clone()).await;

    let city1_id = db_fixtures::insert_city(&pool, "City 1").await;
    let city2_id = db_fixtures::insert_city(&pool, "City 2").await;
    let user1_id = db_fixtures::insert_user(
        &pool,
        "100013",
        "user1@test.com",
        "CITY_USER",
        Some(city1_id),
    )
    .await;
    let user2_id = db_fixtures::insert_user(
        &pool,
        "100014",
        "user2@test.com",
        "CITY_USER",
        Some(city2_id),
    )
    .await;

    // Create session in city1
    let mut user1_claims = test_helpers::build_city_user_claims(city1_id);
    user1_claims.id = user1_id.to_string();
    let user1_token = test_helpers::generate_jwt(&user1_claims, &config.jwt_secret);

    let create_payload1 = serde_json::json!({
        "description": "City1 session",
        "members": [
            {
                "user_id": user1_id,
                "function": "Commander"
            }
        ]
    });

    let create_req1 = test_helpers::with_auth_headers(
        test::TestRequest::post()
            .uri("/api/v1/work-sessions")
            .set_json(&create_payload1),
        &config,
        &user1_token,
    )
    .to_request();

    test::call_service(&app, create_req1).await;

    // Create session in city2
    let mut user2_claims = test_helpers::build_city_user_claims(city2_id);
    user2_claims.id = user2_id.to_string();
    let user2_token = test_helpers::generate_jwt(&user2_claims, &config.jwt_secret);

    let create_payload2 = serde_json::json!({
        "description": "City2 session",
        "members": [
            {
                "user_id": user2_id,
                "function": "Commander"
            }
        ]
    });

    let create_req2 = test_helpers::with_auth_headers(
        test::TestRequest::post()
            .uri("/api/v1/work-sessions")
            .set_json(&create_payload2),
        &config,
        &user2_token,
    )
    .to_request();

    test::call_service(&app, create_req2).await;

    // List sessions as ROOT (should see both cities)
    let root_claims = test_helpers::build_root_claims();
    let root_token = test_helpers::generate_jwt(&root_claims, &config.jwt_secret);

    let list_req = test_helpers::with_auth_headers(
        test::TestRequest::get().uri("/api/v1/work-sessions"),
        &config,
        &root_token,
    )
    .to_request();

    let list_resp = test::call_service(&app, list_req).await;
    assert_eq!(list_resp.status(), StatusCode::OK);
    let list_body: serde_json::Value = test::read_body_json(list_resp).await;
    let sessions = list_body["data"].as_array().unwrap();

    assert_eq!(sessions.len(), 2);
}

#[actix_rt::test]
async fn list_sessions_with_date_filters() {
    let pool = test_helpers::setup_test_db().await;
    test_helpers::clean_database(&pool).await;

    let config = test_helpers::build_test_config();
    let app = test_helpers::create_full_test_app(pool.clone(), config.clone()).await;

    let city_id = db_fixtures::insert_city(&pool, "Test City").await;
    let user_id =
        db_fixtures::insert_user(&pool, "100015", "user@test.com", "CITY_USER", Some(city_id))
            .await;

    let mut claims = test_helpers::build_city_user_claims(city_id);
    claims.id = user_id.to_string();
    let token = test_helpers::generate_jwt(&claims, &config.jwt_secret);

    // Create session
    let create_payload = serde_json::json!({
        "description": "Test session",
        "members": [
            {
                "user_id": user_id,
                "function": "Commander"
            }
        ]
    });

    let create_req = test_helpers::with_auth_headers(
        test::TestRequest::post()
            .uri("/api/v1/work-sessions")
            .set_json(&create_payload),
        &config,
        &token,
    )
    .to_request();

    test::call_service(&app, create_req).await;

    // List with future start date (should return empty)
    let list_req = test_helpers::with_auth_headers(
        test::TestRequest::get().uri("/api/v1/work-sessions?start_date=2099-01-01"),
        &config,
        &token,
    )
    .to_request();

    let list_resp = test::call_service(&app, list_req).await;
    let list_body: serde_json::Value = test::read_body_json(list_resp).await;
    let sessions = list_body["data"].as_array().unwrap();

    assert_eq!(sessions.len(), 0);
}
