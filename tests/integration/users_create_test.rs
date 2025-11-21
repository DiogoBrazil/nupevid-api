use actix_web::{test, http::StatusCode};

use crate::common::{fixtures, test_helpers};

#[actix_rt::test]
async fn test_create_user_success() {
    let pool = test_helpers::setup_test_db().await;
    test_helpers::clean_users_table(&pool).await;
    let app = test_helpers::create_test_app(pool.clone()).await;

    let user = fixtures::valid_create_user();
    let req = test::TestRequest::post()
        .uri("/users")
        .set_json(&user)
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), StatusCode::CREATED);

    let body: serde_json::Value = test::read_body_json(resp).await;
    assert_eq!(body["status"].as_u64().unwrap(), 201);
    assert_eq!(body["data"]["email"].as_str().unwrap(), user.email);
    assert_eq!(body["data"]["full_name"].as_str().unwrap(), user.full_name);
    assert!(body["data"]["password"].is_null()); // Password should not be returned
}

#[actix_rt::test]
async fn test_create_user_duplicate_email() {
    let pool = test_helpers::setup_test_db().await;
    test_helpers::clean_users_table(&pool).await;
    let app = test_helpers::create_test_app(pool.clone()).await;

    let user = fixtures::valid_create_user();
    
    // Create first user
    let req1 = test::TestRequest::post()
        .uri("/users")
        .set_json(&user)
        .to_request();
    let resp1 = test::call_service(&app, req1).await;
    assert_eq!(resp1.status(), StatusCode::CREATED);

    // Try to create user with same email
    let req2 = test::TestRequest::post()
        .uri("/users")
        .set_json(&user)
        .to_request();
    let resp2 = test::call_service(&app, req2).await;
    assert_eq!(resp2.status(), StatusCode::BAD_REQUEST);

    let body: serde_json::Value = test::read_body_json(resp2).await;
    assert_eq!(body["status_code"].as_u64().unwrap(), 400);
    assert!(body["message"].as_str().unwrap().contains("email"));
    assert!(body["message"].as_str().unwrap().contains("already exists"));
}

#[actix_rt::test]
async fn test_create_user_duplicate_registration() {
    let pool = test_helpers::setup_test_db().await;
    test_helpers::clean_users_table(&pool).await;
    let app = test_helpers::create_test_app(pool.clone()).await;

    let user1 = fixtures::valid_create_user();
    let mut user2 = fixtures::valid_create_user();
    user2.email = "different@test.com".to_string(); // Different email, same registration

    // Create first user
    let req1 = test::TestRequest::post()
        .uri("/users")
        .set_json(&user1)
        .to_request();
    let resp1 = test::call_service(&app, req1).await;
    assert_eq!(resp1.status(), StatusCode::CREATED);

    // Try to create user with same registration
    let req2 = test::TestRequest::post()
        .uri("/users")
        .set_json(&user2)
        .to_request();
    let resp2 = test::call_service(&app, req2).await;
    assert_eq!(resp2.status(), StatusCode::BAD_REQUEST);

    let body: serde_json::Value = test::read_body_json(resp2).await;
    assert_eq!(body["status_code"].as_u64().unwrap(), 400);
    assert!(body["message"].as_str().unwrap().contains("registration"));
    assert!(body["message"].as_str().unwrap().contains("already exists"));
}

#[actix_rt::test]
async fn test_create_user_invalid_email() {
    let pool = test_helpers::setup_test_db().await;
    test_helpers::clean_users_table(&pool).await;
    let app = test_helpers::create_test_app(pool.clone()).await;

    let user = fixtures::create_user_with_invalid_email();
    let req = test::TestRequest::post()
        .uri("/users")
        .set_json(&user)
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), StatusCode::BAD_REQUEST);

    let body: serde_json::Value = test::read_body_json(resp).await;
    assert_eq!(body["status_code"].as_u64().unwrap(), 400);
    assert!(body["message"].as_str().unwrap().contains("not a valid email"));
}

#[actix_rt::test]
async fn test_create_user_empty_fields() {
    let pool = test_helpers::setup_test_db().await;
    test_helpers::clean_users_table(&pool).await;
    let app = test_helpers::create_test_app(pool.clone()).await;

    let user = fixtures::create_user_with_empty_fields();
    let req = test::TestRequest::post()
        .uri("/users")
        .set_json(&user)
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), StatusCode::BAD_REQUEST);

    let body: serde_json::Value = test::read_body_json(resp).await;
    assert_eq!(body["status_code"].as_u64().unwrap(), 400);
    assert!(body["message"].as_str().unwrap().contains("cannot be empty"));
}
