use actix_web::{http::StatusCode, test};

use crate::common::{db_fixtures, test_helpers};

/// Phase 11 - Test 1: CITY_USER cannot view another CITY_USER's session
#[actix_rt::test]
async fn city_user_cannot_view_other_city_user_session() {
    let pool = test_helpers::setup_test_db().await;
    test_helpers::clean_database(&pool).await;

    let config = test_helpers::build_test_config();
    let app = test_helpers::create_full_test_app(pool.clone(), config.clone()).await;

    let city = db_fixtures::insert_city(&pool, "Test City").await;
    let user_a =
        db_fixtures::insert_user(&pool, "700001", "user_a@test.com", "CITY_USER", Some(city)).await;
    let user_b =
        db_fixtures::insert_user(&pool, "700002", "user_b@test.com", "CITY_USER", Some(city)).await;

    // User A creates session
    let session_id = test_helpers::create_work_session_for_user(&pool, user_a).await;

    // User B tries to view User A's session
    let mut user_b_claims = test_helpers::build_city_user_claims(city);
    user_b_claims.id = user_b.to_string();
    let user_b_token = test_helpers::generate_jwt(&user_b_claims, &config.jwt_secret);

    let get_req = test_helpers::with_auth_headers(
        test::TestRequest::get().uri(&format!("/api/v1/work-sessions/{}", session_id)),
        &config,
        &user_b_token,
    )
    .to_request();

    let get_resp = test::call_service(&app, get_req).await;
    // CITY_USER doesn't have view_other_work_sessions permission
    assert_eq!(get_resp.status(), StatusCode::FORBIDDEN);
    let body: serde_json::Value = test::read_body_json(get_resp).await;
    assert_eq!(body["status_code"].as_u64().unwrap(), 403);
    assert_eq!(body["error"].as_str().unwrap(), "Forbidden");
}

/// Phase 11 - Test 2: CITY_ADMIN can view other users' sessions in same city
#[actix_rt::test]
async fn city_admin_can_view_other_user_session_in_same_city() {
    let pool = test_helpers::setup_test_db().await;
    test_helpers::clean_database(&pool).await;

    let config = test_helpers::build_test_config();
    let app = test_helpers::create_full_test_app(pool.clone(), config.clone()).await;

    let city = db_fixtures::insert_city(&pool, "Test City").await;
    let user =
        db_fixtures::insert_user(&pool, "700003", "user@test.com", "CITY_USER", Some(city)).await;
    let admin =
        db_fixtures::insert_user(&pool, "700004", "admin@test.com", "CITY_ADMIN", Some(city)).await;

    // User creates session
    let session_id = test_helpers::create_work_session_for_user(&pool, user).await;

    // Admin views user's session
    let mut admin_claims = test_helpers::build_city_admin_claims(city);
    admin_claims.id = admin.to_string();
    let admin_token = test_helpers::generate_jwt(&admin_claims, &config.jwt_secret);

    let get_req = test_helpers::with_auth_headers(
        test::TestRequest::get().uri(&format!("/api/v1/work-sessions/{}", session_id)),
        &config,
        &admin_token,
    )
    .to_request();

    let get_resp = test::call_service(&app, get_req).await;
    // CITY_ADMIN has view_other_work_sessions permission
    assert_eq!(get_resp.status(), StatusCode::OK);
}

/// Phase 11 - Test 3: User can view their own session
#[actix_rt::test]
async fn user_can_view_own_session() {
    let pool = test_helpers::setup_test_db().await;
    test_helpers::clean_database(&pool).await;

    let config = test_helpers::build_test_config();
    let app = test_helpers::create_full_test_app(pool.clone(), config.clone()).await;

    let city = db_fixtures::insert_city(&pool, "Test City").await;
    let user_id =
        db_fixtures::insert_user(&pool, "700005", "user@test.com", "CITY_USER", Some(city)).await;

    // User creates session
    let session_id = test_helpers::create_work_session_for_user(&pool, user_id).await;

    // User views their own session
    let mut user_claims = test_helpers::build_city_user_claims(city);
    user_claims.id = user_id.to_string();
    let user_token = test_helpers::generate_jwt(&user_claims, &config.jwt_secret);

    let get_req = test_helpers::with_auth_headers(
        test::TestRequest::get().uri(&format!("/api/v1/work-sessions/{}", session_id)),
        &config,
        &user_token,
    )
    .to_request();

    let get_resp = test::call_service(&app, get_req).await;
    // User can always view their own session
    assert_eq!(get_resp.status(), StatusCode::OK);

    let body: serde_json::Value = test::read_body_json(get_resp).await;
    assert_eq!(body["data"]["id"].as_str().unwrap(), session_id.to_string());
}
