use actix_web::{http::StatusCode, test};

use crate::common::{db_fixtures, test_helpers};

fn assert_user_complement(user: &serde_json::Value, expected_user_id: Option<String>) {
    assert!(user.is_object());
    if let Some(expected_id) = expected_user_id {
        assert_eq!(user["id"].as_str().unwrap(), expected_id);
    }
    assert!(user["full_name"].as_str().is_some());
    assert!(user["rank"].as_str().is_some());
    assert!(user["registration"].as_str().is_some());
    assert!(user["profile"].as_str().is_some());
    assert!(user["email"].as_str().is_some());
    assert!(user.get("permission_policies").is_some());
    assert!(user.get("created_at").is_none());
    assert!(user.get("updated_at").is_none());
    assert!(user.get("password").is_none());
}

#[actix_rt::test]
async fn create_work_session_with_include_related_returns_user_details() {
    let pool = test_helpers::setup_test_db().await;
    test_helpers::clean_database(&pool).await;

    let config = test_helpers::build_test_config();
    let app = test_helpers::create_full_test_app(pool.clone(), config.clone()).await;

    let city_id = db_fixtures::insert_city(&pool, "Cidade WS").await;
    let user_id = db_fixtures::insert_user(
        &pool,
        "200001",
        "ws.user@test.com",
        "CITY_USER",
        Some(city_id),
    )
    .await;
    let member_id = db_fixtures::insert_user(
        &pool,
        "200002",
        "ws.member@test.com",
        "CITY_USER",
        Some(city_id),
    )
    .await;

    let mut claims = test_helpers::build_city_user_claims(city_id);
    claims.id = user_id.to_string();
    let token = test_helpers::generate_jwt(&claims, &config.jwt_secret);

    let payload = serde_json::json!({
        "description": "Sessao com detalhes",
        "members": [
            { "user_id": user_id, "function": "Commander" },
            { "user_id": member_id, "function": "Driver" }
        ]
    });

    let req = test_helpers::with_auth_headers(
        test::TestRequest::post()
            .uri("/api/v1/work-sessions?include_related_entities=true")
            .set_json(&payload),
        &config,
        &token,
    )
    .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), StatusCode::CREATED);
    let body: serde_json::Value = test::read_body_json(resp).await;

    let members = body["data"]["members"].as_array().unwrap();
    assert_eq!(members.len(), 2);
    for member in members {
        assert_user_complement(&member["user"], None);
    }
}

#[actix_rt::test]
async fn get_active_work_session_with_include_related_returns_user_details() {
    let pool = test_helpers::setup_test_db().await;
    test_helpers::clean_database(&pool).await;

    let config = test_helpers::build_test_config();
    let app = test_helpers::create_full_test_app(pool.clone(), config.clone()).await;

    let city_id = db_fixtures::insert_city(&pool, "Cidade WS Ativa").await;
    let user_id = db_fixtures::insert_user(
        &pool,
        "200010",
        "ws.active@test.com",
        "CITY_USER",
        Some(city_id),
    )
    .await;
    test_helpers::create_work_session_for_user(&pool, user_id).await;

    let mut claims = test_helpers::build_city_user_claims(city_id);
    claims.id = user_id.to_string();
    let token = test_helpers::generate_jwt(&claims, &config.jwt_secret);

    let req = test_helpers::with_auth_headers(
        test::TestRequest::get()
            .uri("/api/v1/work-sessions/active?include_related_entities=true"),
        &config,
        &token,
    )
    .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), StatusCode::OK);
    let body: serde_json::Value = test::read_body_json(resp).await;
    let members = body["data"]["members"].as_array().unwrap();
    assert_eq!(members.len(), 1);
    assert_user_complement(&members[0]["user"], Some(user_id.to_string()));
}

#[actix_rt::test]
async fn list_work_sessions_with_include_related_returns_user_details() {
    let pool = test_helpers::setup_test_db().await;
    test_helpers::clean_database(&pool).await;

    let config = test_helpers::build_test_config();
    let app = test_helpers::create_full_test_app(pool.clone(), config.clone()).await;

    let city_id = db_fixtures::insert_city(&pool, "Cidade WS Lista").await;
    let user_id = db_fixtures::insert_user(
        &pool,
        "200020",
        "ws.list@test.com",
        "CITY_USER",
        Some(city_id),
    )
    .await;
    test_helpers::create_work_session_for_user(&pool, user_id).await;

    let mut claims = test_helpers::build_city_user_claims(city_id);
    claims.id = user_id.to_string();
    let token = test_helpers::generate_jwt(&claims, &config.jwt_secret);

    let req = test_helpers::with_auth_headers(
        test::TestRequest::get().uri("/api/v1/work-sessions?include_related_entities=true"),
        &config,
        &token,
    )
    .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), StatusCode::OK);
    let body: serde_json::Value = test::read_body_json(resp).await;
    let sessions = body["data"].as_array().unwrap();
    assert!(!sessions.is_empty());
    let members = sessions[0]["members"].as_array().unwrap();
    assert!(!members.is_empty());
    assert_user_complement(&members[0]["user"], Some(user_id.to_string()));
}
