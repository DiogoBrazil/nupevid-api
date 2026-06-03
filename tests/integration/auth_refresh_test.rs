use actix_web::{Error, dev::Service, http::StatusCode, test};
use chrono::{Duration, Utc};
use nupevid_api::adapters::password_hasher::{Argon2PasswordHasher, PasswordHasherPort};
use nupevid_api::config::config_env::Config;
use sqlx::PgPool;
use uuid::Uuid;

use crate::common::{db_fixtures, test_helpers};

const PASSWORD: &str = "senha123";

/// Logs a user in (without auto session) and returns the parsed JSON body.
async fn login<S>(app: &S, config: &Config, email: &str) -> serde_json::Value
where
    S: Service<actix_http::Request, Response = actix_web::dev::ServiceResponse, Error = Error>,
{
    let payload = serde_json::json!({
        "email": email,
        "password": PASSWORD,
        "auto_create_session": false,
    });

    let req = test::TestRequest::post()
        .uri("/api/v1/auth/login")
        .insert_header(("api_key", config.api_key.clone()))
        .set_json(&payload)
        .to_request();

    let resp = test::call_service(app, req).await;
    assert_eq!(resp.status(), StatusCode::OK);
    test::read_body_json(resp).await
}

async fn refresh<S>(
    app: &S,
    config: &Config,
    refresh_token: &str,
) -> actix_web::dev::ServiceResponse
where
    S: Service<actix_http::Request, Response = actix_web::dev::ServiceResponse, Error = Error>,
{
    let req = test::TestRequest::post()
        .uri("/api/v1/auth/refresh")
        .insert_header(("api_key", config.api_key.clone()))
        .set_json(serde_json::json!({ "refresh_token": refresh_token }))
        .to_request();
    test::call_service(app, req).await
}

async fn logout<S>(
    app: &S,
    config: &Config,
    refresh_token: &str,
) -> actix_web::dev::ServiceResponse
where
    S: Service<actix_http::Request, Response = actix_web::dev::ServiceResponse, Error = Error>,
{
    let req = test::TestRequest::post()
        .uri("/api/v1/auth/logout")
        .insert_header(("api_key", config.api_key.clone()))
        .set_json(serde_json::json!({ "refresh_token": refresh_token }))
        .to_request();
    test::call_service(app, req).await
}

/// Inserts a refresh token row directly with a known secret, returning the `{id}.{secret}` value.
async fn seed_refresh_token(
    pool: &PgPool,
    user_id: Uuid,
    expires_at: chrono::DateTime<Utc>,
) -> String {
    let id = Uuid::new_v4();
    let secret = "test-secret-value-1234567890";
    let hasher = Argon2PasswordHasher::new();
    let token_hash = hasher.hash_password(secret).expect("hash refresh secret");

    sqlx::query(
        "INSERT INTO refresh_tokens (id, user_id, token_hash, expires_at) VALUES ($1, $2, $3, $4)",
    )
    .bind(id)
    .bind(user_id)
    .bind(token_hash)
    .bind(expires_at)
    .execute(pool)
    .await
    .expect("seed refresh token");

    format!("{}.{}", id, secret)
}

#[sqlx::test]
async fn login_returns_access_and_refresh_tokens(pool: PgPool) {
    let config = test_helpers::build_test_config();
    let app = test_helpers::create_full_test_app(pool.clone(), config.clone()).await;

    let city_id = db_fixtures::insert_city(&pool, "City").await;
    db_fixtures::insert_user(&pool, "200001", "u1@test.com", "CITY_USER", Some(city_id)).await;

    let body = login(&app, &config, "u1@test.com").await;

    assert!(body["data"]["access_token"].as_str().is_some());
    assert!(body["data"]["refresh_token"].as_str().is_some());
    assert_eq!(body["data"]["token_type"].as_str().unwrap(), "Bearer");
    assert_eq!(body["data"]["expires_in"].as_i64().unwrap(), 900);

    // Refresh token must be in the `{uuid}.{secret}` format and never stored in plaintext.
    let refresh_token = body["data"]["refresh_token"].as_str().unwrap();
    assert!(refresh_token.contains('.'));
}

#[sqlx::test]
async fn refresh_rotates_tokens_and_revokes_old(pool: PgPool) {
    let config = test_helpers::build_test_config();
    let app = test_helpers::create_full_test_app(pool.clone(), config.clone()).await;

    let city_id = db_fixtures::insert_city(&pool, "City").await;
    db_fixtures::insert_user(&pool, "200002", "u2@test.com", "CITY_USER", Some(city_id)).await;

    let login_body = login(&app, &config, "u2@test.com").await;
    let old_refresh = login_body["data"]["refresh_token"].as_str().unwrap();
    let old_id = Uuid::parse_str(old_refresh.split_once('.').unwrap().0).unwrap();

    let resp = refresh(&app, &config, old_refresh).await;
    assert_eq!(resp.status(), StatusCode::OK);

    let body: serde_json::Value = test::read_body_json(resp).await;
    let new_refresh = body["data"]["refresh_token"].as_str().unwrap();
    assert!(body["data"]["access_token"].as_str().is_some());
    assert_eq!(body["data"]["token_type"].as_str().unwrap(), "Bearer");
    assert_ne!(new_refresh, old_refresh);

    // Old token is revoked and points to the replacement.
    let (revoked_at, replaced_by): (Option<chrono::DateTime<Utc>>, Option<Uuid>) =
        sqlx::query_as("SELECT revoked_at, replaced_by_token_id FROM refresh_tokens WHERE id = $1")
            .bind(old_id)
            .fetch_one(&pool)
            .await
            .unwrap();
    assert!(revoked_at.is_some(), "old refresh token should be revoked");
    let new_id = Uuid::parse_str(new_refresh.split_once('.').unwrap().0).unwrap();
    assert_eq!(replaced_by, Some(new_id));

    // The new refresh token works.
    let resp2 = refresh(&app, &config, new_refresh).await;
    assert_eq!(resp2.status(), StatusCode::OK);
}

#[sqlx::test]
async fn reused_refresh_token_is_rejected(pool: PgPool) {
    let config = test_helpers::build_test_config();
    let app = test_helpers::create_full_test_app(pool.clone(), config.clone()).await;

    let city_id = db_fixtures::insert_city(&pool, "City").await;
    db_fixtures::insert_user(&pool, "200003", "u3@test.com", "CITY_USER", Some(city_id)).await;

    let login_body = login(&app, &config, "u3@test.com").await;
    let old_refresh = login_body["data"]["refresh_token"].as_str().unwrap();

    // First rotation succeeds.
    let first = refresh(&app, &config, old_refresh).await;
    assert_eq!(first.status(), StatusCode::OK);

    // Reusing the now-rotated token must be rejected.
    let reused = refresh(&app, &config, old_refresh).await;
    assert_eq!(reused.status(), StatusCode::UNAUTHORIZED);

    let body: serde_json::Value = test::read_body_json(reused).await;
    assert_eq!(body["status_code"].as_u64().unwrap(), 401);
    assert!(body["error"].as_str().is_some());
    assert!(body["message"].as_str().is_some());
}

#[sqlx::test]
async fn expired_refresh_token_is_rejected(pool: PgPool) {
    let config = test_helpers::build_test_config();
    let app = test_helpers::create_full_test_app(pool.clone(), config.clone()).await;

    let city_id = db_fixtures::insert_city(&pool, "City").await;
    let user_id =
        db_fixtures::insert_user(&pool, "200004", "u4@test.com", "CITY_USER", Some(city_id)).await;

    let expired_token = seed_refresh_token(&pool, user_id, Utc::now() - Duration::days(1)).await;

    let resp = refresh(&app, &config, &expired_token).await;
    assert_eq!(resp.status(), StatusCode::UNAUTHORIZED);

    let body: serde_json::Value = test::read_body_json(resp).await;
    assert!(body["message"].as_str().unwrap().contains("expired"));
}

#[sqlx::test]
async fn logout_revokes_refresh_token(pool: PgPool) {
    let config = test_helpers::build_test_config();
    let app = test_helpers::create_full_test_app(pool.clone(), config.clone()).await;

    let city_id = db_fixtures::insert_city(&pool, "City").await;
    db_fixtures::insert_user(&pool, "200005", "u5@test.com", "CITY_USER", Some(city_id)).await;

    let login_body = login(&app, &config, "u5@test.com").await;
    let refresh_token = login_body["data"]["refresh_token"].as_str().unwrap();

    let logout_resp = logout(&app, &config, refresh_token).await;
    assert_eq!(logout_resp.status(), StatusCode::OK);

    // After logout, the refresh token can no longer be used.
    let resp = refresh(&app, &config, refresh_token).await;
    assert_eq!(resp.status(), StatusCode::UNAUTHORIZED);
}

#[sqlx::test]
async fn access_token_from_login_works_on_protected_route(pool: PgPool) {
    let config = test_helpers::build_test_config();
    let app = test_helpers::create_full_test_app(pool.clone(), config.clone()).await;

    let city_id = db_fixtures::insert_city(&pool, "City").await;
    db_fixtures::insert_user(&pool, "200006", "u6@test.com", "CITY_USER", Some(city_id)).await;

    let login_body = login(&app, &config, "u6@test.com").await;
    let access_token = login_body["data"]["access_token"].as_str().unwrap();

    // Protected route accepts the freshly issued access token.
    let req = test_helpers::with_auth_headers(
        test::TestRequest::get().uri("/api/v1/work-sessions/active"),
        &config,
        access_token,
    )
    .to_request();
    let resp = test::call_service(&app, req).await;
    assert_ne!(resp.status(), StatusCode::UNAUTHORIZED);

    // Without a token the same route is rejected by the auth middleware
    // (the middleware short-circuits with an error rather than a response).
    let req_no_token = test::TestRequest::get()
        .uri("/api/v1/work-sessions/active")
        .insert_header(("api_key", config.api_key.clone()))
        .to_request();
    let err = test::try_call_service(&app, req_no_token)
        .await
        .expect_err("protected route without Authorization must fail");
    assert!(err.to_string().contains("Invalid authorization header"));
}

#[sqlx::test]
async fn refresh_with_malformed_token_is_rejected(pool: PgPool) {
    let config = test_helpers::build_test_config();
    let app = test_helpers::create_full_test_app(pool.clone(), config.clone()).await;

    let resp = refresh(&app, &config, "not-a-valid-token").await;
    assert_eq!(resp.status(), StatusCode::UNAUTHORIZED);

    let body: serde_json::Value = test::read_body_json(resp).await;
    assert_eq!(body["status_code"].as_u64().unwrap(), 401);
}
