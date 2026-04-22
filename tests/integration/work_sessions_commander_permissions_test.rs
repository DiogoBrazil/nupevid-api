use actix_web::{http::StatusCode, test};

use crate::common::{db_fixtures, test_helpers};

#[actix_rt::test]
async fn commander_non_creator_can_add_member() {
    let pool = test_helpers::setup_test_db().await;
    test_helpers::clean_database(&pool).await;

    let config = test_helpers::build_test_config();
    let app = test_helpers::create_full_test_app(pool.clone(), config.clone()).await;

    let city = db_fixtures::insert_city(&pool, "Test City").await;
    let creator_id =
        db_fixtures::insert_user(&pool, "900001", "creator@test.com", "CITY_USER", Some(city))
            .await;
    let commander_id = db_fixtures::insert_user(
        &pool,
        "900002",
        "commander@test.com",
        "CITY_USER",
        Some(city),
    )
    .await;
    let member_id =
        db_fixtures::insert_user(&pool, "900003", "member@test.com", "CITY_USER", Some(city)).await;
    let new_member_id = db_fixtures::insert_user(
        &pool,
        "900004",
        "new_member@test.com",
        "CITY_USER",
        Some(city),
    )
    .await;

    let mut creator_claims = test_helpers::build_city_user_claims(city);
    creator_claims.id = creator_id.to_string();
    let creator_token = test_helpers::generate_jwt(&creator_claims, &config.jwt_secret);

    let create_payload = serde_json::json!({
        "description": "Session with commander",
        "members": [
            {
                "user_id": creator_id,
                "function": "Patroller"
            },
            {
                "user_id": commander_id,
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
        &creator_token,
    )
    .to_request();

    let create_resp = test::call_service(&app, create_req).await;
    assert_eq!(create_resp.status(), StatusCode::CREATED);
    let create_body: serde_json::Value = test::read_body_json(create_resp).await;
    let session_id = create_body["data"]["id"].as_str().unwrap();

    let mut commander_claims = test_helpers::build_city_user_claims(city);
    commander_claims.id = commander_id.to_string();
    let commander_token = test_helpers::generate_jwt(&commander_claims, &config.jwt_secret);

    let add_payload = serde_json::json!({
        "user_id": new_member_id,
        "function": "Patroller"
    });

    let add_req = test_helpers::with_auth_headers(
        test::TestRequest::post()
            .uri(&format!("/api/v1/work-sessions/{}/members", session_id))
            .set_json(&add_payload),
        &config,
        &commander_token,
    )
    .to_request();

    let add_resp = test::call_service(&app, add_req).await;
    assert_eq!(add_resp.status(), StatusCode::OK);
}

#[actix_rt::test]
async fn commander_non_creator_can_remove_member() {
    let pool = test_helpers::setup_test_db().await;
    test_helpers::clean_database(&pool).await;

    let config = test_helpers::build_test_config();
    let app = test_helpers::create_full_test_app(pool.clone(), config.clone()).await;

    let city = db_fixtures::insert_city(&pool, "Test City").await;
    let creator_id =
        db_fixtures::insert_user(&pool, "900010", "creator@test.com", "CITY_USER", Some(city))
            .await;
    let commander_id = db_fixtures::insert_user(
        &pool,
        "900011",
        "commander@test.com",
        "CITY_USER",
        Some(city),
    )
    .await;
    let member_id =
        db_fixtures::insert_user(&pool, "900012", "member@test.com", "CITY_USER", Some(city)).await;

    let mut creator_claims = test_helpers::build_city_user_claims(city);
    creator_claims.id = creator_id.to_string();
    let creator_token = test_helpers::generate_jwt(&creator_claims, &config.jwt_secret);

    let create_payload = serde_json::json!({
        "description": "Session with commander",
        "members": [
            {
                "user_id": creator_id,
                "function": "Patroller"
            },
            {
                "user_id": commander_id,
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
        &creator_token,
    )
    .to_request();

    let create_resp = test::call_service(&app, create_req).await;
    assert_eq!(create_resp.status(), StatusCode::CREATED);
    let create_body: serde_json::Value = test::read_body_json(create_resp).await;
    let session_id = create_body["data"]["id"].as_str().unwrap();

    let mut commander_claims = test_helpers::build_city_user_claims(city);
    commander_claims.id = commander_id.to_string();
    let commander_token = test_helpers::generate_jwt(&commander_claims, &config.jwt_secret);

    let remove_req = test_helpers::with_auth_headers(
        test::TestRequest::delete().uri(&format!(
            "/api/v1/work-sessions/{}/members/{}",
            session_id, member_id
        )),
        &config,
        &commander_token,
    )
    .to_request();

    let remove_resp = test::call_service(&app, remove_req).await;
    assert_eq!(remove_resp.status(), StatusCode::OK);
}

#[actix_rt::test]
async fn commander_non_creator_can_update_members() {
    let pool = test_helpers::setup_test_db().await;
    test_helpers::clean_database(&pool).await;

    let config = test_helpers::build_test_config();
    let app = test_helpers::create_full_test_app(pool.clone(), config.clone()).await;

    let city = db_fixtures::insert_city(&pool, "Test City").await;
    let creator_id =
        db_fixtures::insert_user(&pool, "900020", "creator@test.com", "CITY_USER", Some(city))
            .await;
    let commander_id = db_fixtures::insert_user(
        &pool,
        "900021",
        "commander@test.com",
        "CITY_USER",
        Some(city),
    )
    .await;
    let member_id =
        db_fixtures::insert_user(&pool, "900022", "member@test.com", "CITY_USER", Some(city)).await;
    let new_member_id = db_fixtures::insert_user(
        &pool,
        "900023",
        "new_member@test.com",
        "CITY_USER",
        Some(city),
    )
    .await;

    let mut creator_claims = test_helpers::build_city_user_claims(city);
    creator_claims.id = creator_id.to_string();
    let creator_token = test_helpers::generate_jwt(&creator_claims, &config.jwt_secret);

    let create_payload = serde_json::json!({
        "description": "Session with commander",
        "members": [
            {
                "user_id": creator_id,
                "function": "Patroller"
            },
            {
                "user_id": commander_id,
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
        &creator_token,
    )
    .to_request();

    let create_resp = test::call_service(&app, create_req).await;
    assert_eq!(create_resp.status(), StatusCode::CREATED);
    let create_body: serde_json::Value = test::read_body_json(create_resp).await;
    let session_id = create_body["data"]["id"].as_str().unwrap();

    let mut commander_claims = test_helpers::build_city_user_claims(city);
    commander_claims.id = commander_id.to_string();
    let commander_token = test_helpers::generate_jwt(&commander_claims, &config.jwt_secret);

    let update_payload = serde_json::json!({
        "members": [
            {
                "user_id": creator_id,
                "function": "Patroller"
            },
            {
                "user_id": commander_id,
                "function": "Commander"
            },
            {
                "user_id": member_id,
                "function": "Patroller"
            },
            {
                "user_id": new_member_id,
                "function": "Driver"
            }
        ]
    });

    let update_req = test_helpers::with_auth_headers(
        test::TestRequest::put()
            .uri(&format!("/api/v1/work-sessions/{}/members", session_id))
            .set_json(&update_payload),
        &config,
        &commander_token,
    )
    .to_request();

    let update_resp = test::call_service(&app, update_req).await;
    assert_eq!(update_resp.status(), StatusCode::OK);
}

#[actix_rt::test]
async fn commander_non_creator_can_update_member_function() {
    let pool = test_helpers::setup_test_db().await;
    test_helpers::clean_database(&pool).await;

    let config = test_helpers::build_test_config();
    let app = test_helpers::create_full_test_app(pool.clone(), config.clone()).await;

    let city = db_fixtures::insert_city(&pool, "Test City").await;
    let creator_id =
        db_fixtures::insert_user(&pool, "900030", "creator@test.com", "CITY_USER", Some(city))
            .await;
    let commander_id = db_fixtures::insert_user(
        &pool,
        "900031",
        "commander@test.com",
        "CITY_USER",
        Some(city),
    )
    .await;
    let member_id =
        db_fixtures::insert_user(&pool, "900032", "member@test.com", "CITY_USER", Some(city)).await;

    let mut creator_claims = test_helpers::build_city_user_claims(city);
    creator_claims.id = creator_id.to_string();
    let creator_token = test_helpers::generate_jwt(&creator_claims, &config.jwt_secret);

    let create_payload = serde_json::json!({
        "description": "Session with commander",
        "members": [
            {
                "user_id": creator_id,
                "function": "Patroller"
            },
            {
                "user_id": commander_id,
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
        &creator_token,
    )
    .to_request();

    let create_resp = test::call_service(&app, create_req).await;
    assert_eq!(create_resp.status(), StatusCode::CREATED);
    let create_body: serde_json::Value = test::read_body_json(create_resp).await;
    let session_id = create_body["data"]["id"].as_str().unwrap();

    let mut commander_claims = test_helpers::build_city_user_claims(city);
    commander_claims.id = commander_id.to_string();
    let commander_token = test_helpers::generate_jwt(&commander_claims, &config.jwt_secret);

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
        &commander_token,
    )
    .to_request();

    let update_resp = test::call_service(&app, update_req).await;
    assert_eq!(update_resp.status(), StatusCode::OK);
}

#[actix_rt::test]
async fn commander_non_creator_can_end_session() {
    let pool = test_helpers::setup_test_db().await;
    test_helpers::clean_database(&pool).await;

    let config = test_helpers::build_test_config();
    let app = test_helpers::create_full_test_app(pool.clone(), config.clone()).await;

    let city = db_fixtures::insert_city(&pool, "Test City").await;
    let creator_id =
        db_fixtures::insert_user(&pool, "900040", "creator@test.com", "CITY_USER", Some(city))
            .await;
    let commander_id = db_fixtures::insert_user(
        &pool,
        "900041",
        "commander@test.com",
        "CITY_USER",
        Some(city),
    )
    .await;

    let mut creator_claims = test_helpers::build_city_user_claims(city);
    creator_claims.id = creator_id.to_string();
    let creator_token = test_helpers::generate_jwt(&creator_claims, &config.jwt_secret);

    let create_payload = serde_json::json!({
        "description": "Session with commander",
        "members": [
            {
                "user_id": creator_id,
                "function": "Patroller"
            },
            {
                "user_id": commander_id,
                "function": "Commander"
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
    assert_eq!(create_resp.status(), StatusCode::CREATED);
    let create_body: serde_json::Value = test::read_body_json(create_resp).await;
    let session_id = create_body["data"]["id"].as_str().unwrap();

    let mut commander_claims = test_helpers::build_city_user_claims(city);
    commander_claims.id = commander_id.to_string();
    let commander_token = test_helpers::generate_jwt(&commander_claims, &config.jwt_secret);

    let end_req = test_helpers::with_auth_headers(
        test::TestRequest::post().uri("/api/v1/work-sessions/end"),
        &config,
        &commander_token,
    )
    .to_request();

    let end_resp = test::call_service(&app, end_req).await;
    assert_eq!(end_resp.status(), StatusCode::OK);

    let is_active: bool = sqlx::query_scalar("SELECT is_active FROM work_sessions WHERE id = $1")
        .bind(uuid::Uuid::parse_str(session_id).unwrap())
        .fetch_one(&pool)
        .await
        .expect("Failed to check session");

    assert_eq!(is_active, false);
}

#[actix_rt::test]
async fn member_non_commander_cannot_remove_member() {
    let pool = test_helpers::setup_test_db().await;
    test_helpers::clean_database(&pool).await;

    let config = test_helpers::build_test_config();
    let app = test_helpers::create_full_test_app(pool.clone(), config.clone()).await;

    let city = db_fixtures::insert_city(&pool, "Test City").await;
    let creator_id =
        db_fixtures::insert_user(&pool, "900050", "creator@test.com", "CITY_USER", Some(city))
            .await;
    let commander_id = db_fixtures::insert_user(
        &pool,
        "900051",
        "commander@test.com",
        "CITY_USER",
        Some(city),
    )
    .await;
    let member_id =
        db_fixtures::insert_user(&pool, "900052", "member@test.com", "CITY_USER", Some(city)).await;

    let mut creator_claims = test_helpers::build_city_user_claims(city);
    creator_claims.id = creator_id.to_string();
    let creator_token = test_helpers::generate_jwt(&creator_claims, &config.jwt_secret);

    let create_payload = serde_json::json!({
        "description": "Session with commander",
        "members": [
            {
                "user_id": creator_id,
                "function": "Patroller"
            },
            {
                "user_id": commander_id,
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
        &creator_token,
    )
    .to_request();

    let create_resp = test::call_service(&app, create_req).await;
    assert_eq!(create_resp.status(), StatusCode::CREATED);
    let create_body: serde_json::Value = test::read_body_json(create_resp).await;
    let session_id = create_body["data"]["id"].as_str().unwrap();

    let mut member_claims = test_helpers::build_city_user_claims(city);
    member_claims.id = member_id.to_string();
    let member_token = test_helpers::generate_jwt(&member_claims, &config.jwt_secret);

    let remove_req = test_helpers::with_auth_headers(
        test::TestRequest::delete().uri(&format!(
            "/api/v1/work-sessions/{}/members/{}",
            session_id, commander_id
        )),
        &config,
        &member_token,
    )
    .to_request();

    let remove_resp = test::call_service(&app, remove_req).await;
    assert_eq!(remove_resp.status(), StatusCode::FORBIDDEN);
}

#[actix_rt::test]
async fn member_non_commander_cannot_update_members() {
    let pool = test_helpers::setup_test_db().await;
    test_helpers::clean_database(&pool).await;

    let config = test_helpers::build_test_config();
    let app = test_helpers::create_full_test_app(pool.clone(), config.clone()).await;

    let city = db_fixtures::insert_city(&pool, "Test City").await;
    let creator_id =
        db_fixtures::insert_user(&pool, "900060", "creator@test.com", "CITY_USER", Some(city))
            .await;
    let commander_id = db_fixtures::insert_user(
        &pool,
        "900061",
        "commander@test.com",
        "CITY_USER",
        Some(city),
    )
    .await;
    let member_id =
        db_fixtures::insert_user(&pool, "900062", "member@test.com", "CITY_USER", Some(city)).await;

    let mut creator_claims = test_helpers::build_city_user_claims(city);
    creator_claims.id = creator_id.to_string();
    let creator_token = test_helpers::generate_jwt(&creator_claims, &config.jwt_secret);

    let create_payload = serde_json::json!({
        "description": "Session with commander",
        "members": [
            {
                "user_id": creator_id,
                "function": "Patroller"
            },
            {
                "user_id": commander_id,
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
        &creator_token,
    )
    .to_request();

    let create_resp = test::call_service(&app, create_req).await;
    assert_eq!(create_resp.status(), StatusCode::CREATED);
    let create_body: serde_json::Value = test::read_body_json(create_resp).await;
    let session_id = create_body["data"]["id"].as_str().unwrap();

    let mut member_claims = test_helpers::build_city_user_claims(city);
    member_claims.id = member_id.to_string();
    let member_token = test_helpers::generate_jwt(&member_claims, &config.jwt_secret);

    let update_payload = serde_json::json!({
        "members": [
            {
                "user_id": commander_id,
                "function": "Commander"
            },
            {
                "user_id": member_id,
                "function": "Driver"
            }
        ]
    });

    let update_req = test_helpers::with_auth_headers(
        test::TestRequest::put()
            .uri(&format!("/api/v1/work-sessions/{}/members", session_id))
            .set_json(&update_payload),
        &config,
        &member_token,
    )
    .to_request();

    let update_resp = test::call_service(&app, update_req).await;
    assert_eq!(update_resp.status(), StatusCode::FORBIDDEN);
}

#[actix_rt::test]
async fn member_non_commander_cannot_end_session() {
    let pool = test_helpers::setup_test_db().await;
    test_helpers::clean_database(&pool).await;

    let config = test_helpers::build_test_config();
    let app = test_helpers::create_full_test_app(pool.clone(), config.clone()).await;

    let city = db_fixtures::insert_city(&pool, "Test City").await;
    let creator_id =
        db_fixtures::insert_user(&pool, "900070", "creator@test.com", "CITY_USER", Some(city))
            .await;
    let commander_id = db_fixtures::insert_user(
        &pool,
        "900071",
        "commander@test.com",
        "CITY_USER",
        Some(city),
    )
    .await;
    let member_id =
        db_fixtures::insert_user(&pool, "900072", "member@test.com", "CITY_USER", Some(city)).await;

    let mut creator_claims = test_helpers::build_city_user_claims(city);
    creator_claims.id = creator_id.to_string();
    let creator_token = test_helpers::generate_jwt(&creator_claims, &config.jwt_secret);

    let create_payload = serde_json::json!({
        "description": "Session with commander",
        "members": [
            {
                "user_id": creator_id,
                "function": "Patroller"
            },
            {
                "user_id": commander_id,
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
        &creator_token,
    )
    .to_request();

    let create_resp = test::call_service(&app, create_req).await;
    assert_eq!(create_resp.status(), StatusCode::CREATED);

    let mut member_claims = test_helpers::build_city_user_claims(city);
    member_claims.id = member_id.to_string();
    let member_token = test_helpers::generate_jwt(&member_claims, &config.jwt_secret);

    let end_req = test_helpers::with_auth_headers(
        test::TestRequest::post().uri("/api/v1/work-sessions/end"),
        &config,
        &member_token,
    )
    .to_request();

    let end_resp = test::call_service(&app, end_req).await;
    assert_eq!(end_resp.status(), StatusCode::FORBIDDEN);
}
