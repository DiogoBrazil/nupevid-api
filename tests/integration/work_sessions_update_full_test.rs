use actix_web::{http::StatusCode, test};

use crate::common::{db_fixtures, test_helpers};

#[actix_rt::test]
async fn update_work_session_replaces_members_and_description() {
    let pool = test_helpers::setup_test_db().await;
    test_helpers::clean_database(&pool).await;

    let config = test_helpers::build_test_config();
    let app = test_helpers::create_full_test_app(pool.clone(), config.clone()).await;

    let city_id = db_fixtures::insert_city(&pool, "Cidade Update").await;
    let user_id = db_fixtures::insert_user(
        &pool,
        "300001",
        "update.user@test.com",
        "CITY_USER",
        Some(city_id),
    )
    .await;
    let member_id = db_fixtures::insert_user(
        &pool,
        "300002",
        "update.member@test.com",
        "CITY_USER",
        Some(city_id),
    )
    .await;

    let mut claims = test_helpers::build_city_user_claims(city_id);
    claims.id = user_id.to_string();
    let token = test_helpers::generate_jwt(&claims, &config.jwt_secret);

    let create_payload = serde_json::json!({
        "description": "Sessao original",
        "members": [
            { "user_id": user_id, "function": "Commander" }
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

    let update_payload = serde_json::json!({
        "description": "Sessao atualizada",
        "members": [
            { "user_id": user_id, "function": "Commander" },
            { "user_id": member_id, "function": "Driver" }
        ]
    });

    let update_req = test_helpers::with_auth_headers(
        test::TestRequest::put()
            .uri(&format!(
                "/api/v1/work-sessions/{}?include_complement_for_entities=true",
                session_id
            ))
            .set_json(&update_payload),
        &config,
        &token,
    )
    .to_request();

    let update_resp = test::call_service(&app, update_req).await;
    assert_eq!(update_resp.status(), StatusCode::OK);
    let update_body: serde_json::Value = test::read_body_json(update_resp).await;
    assert_eq!(
        update_body["data"]["description"].as_str().unwrap(),
        "Sessao atualizada"
    );

    let members = update_body["data"]["members"].as_array().unwrap();
    assert_eq!(members.len(), 2);
    assert!(members.iter().all(|m| m.get("user").is_some()));
}

#[actix_rt::test]
async fn update_work_session_without_description_keeps_existing() {
    let pool = test_helpers::setup_test_db().await;
    test_helpers::clean_database(&pool).await;

    let config = test_helpers::build_test_config();
    let app = test_helpers::create_full_test_app(pool.clone(), config.clone()).await;

    let city_id = db_fixtures::insert_city(&pool, "Cidade Update 2").await;
    let user_id = db_fixtures::insert_user(
        &pool,
        "300010",
        "update.user2@test.com",
        "CITY_USER",
        Some(city_id),
    )
    .await;
    let member_id = db_fixtures::insert_user(
        &pool,
        "300011",
        "update.member2@test.com",
        "CITY_USER",
        Some(city_id),
    )
    .await;

    let mut claims = test_helpers::build_city_user_claims(city_id);
    claims.id = user_id.to_string();
    let token = test_helpers::generate_jwt(&claims, &config.jwt_secret);

    let create_payload = serde_json::json!({
        "description": "Descricao mantida",
        "members": [
            { "user_id": user_id, "function": "Commander" }
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

    let update_payload = serde_json::json!({
        "members": [
            { "user_id": user_id, "function": "Commander" },
            { "user_id": member_id, "function": "Patroller" }
        ]
    });

    let update_req = test_helpers::with_auth_headers(
        test::TestRequest::put()
            .uri(&format!("/api/v1/work-sessions/{}", session_id))
            .set_json(&update_payload),
        &config,
        &token,
    )
    .to_request();

    let update_resp = test::call_service(&app, update_req).await;
    assert_eq!(update_resp.status(), StatusCode::OK);
    let update_body: serde_json::Value = test::read_body_json(update_resp).await;
    assert_eq!(
        update_body["data"]["description"].as_str().unwrap(),
        "Descricao mantida"
    );
    let members = update_body["data"]["members"].as_array().unwrap();
    assert_eq!(members.len(), 2);
}
