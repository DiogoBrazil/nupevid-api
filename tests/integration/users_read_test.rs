use actix_web::{test, http::StatusCode};
use uuid::Uuid;

use crate::common::{fixtures, test_helpers};

#[actix_rt::test]
async fn test_get_all_users_success() {
    let pool = test_helpers::setup_test_db().await;
    test_helpers::clean_users_table(&pool).await;
    let app = test_helpers::create_test_app(pool.clone()).await;

    // Create two users
    let user1 = fixtures::valid_create_user();
    let user2 = fixtures::valid_create_user_2();
    
    for user in [user1, user2] {
        let req = test::TestRequest::post()
            .uri("/users")
            .set_json(&user)
            .to_request();
        test::call_service(&app, req).await;
    }

    // Get all users
    let req = test::TestRequest::get()
        .uri("/users")
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), StatusCode::OK);

    let body: serde_json::Value = test::read_body_json(resp).await;
    assert_eq!(body["status"].as_u64().unwrap(), 200);
    assert_eq!(body["data"].as_array().unwrap().len(), 2);
}

#[actix_rt::test]
async fn test_get_all_users_empty() {
    let pool = test_helpers::setup_test_db().await;
    test_helpers::clean_users_table(&pool).await;
    let app = test_helpers::create_test_app(pool.clone()).await;

    let req = test::TestRequest::get()
        .uri("/users")
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), StatusCode::OK);

    let body: serde_json::Value = test::read_body_json(resp).await;
    assert_eq!(body["status"].as_u64().unwrap(), 200);
    assert_eq!(body["data"].as_array().unwrap().len(), 0);
}

#[actix_rt::test]
async fn test_get_user_by_id_success() {
    let pool = test_helpers::setup_test_db().await;
    test_helpers::clean_users_table(&pool).await;
    let app = test_helpers::create_test_app(pool.clone()).await;

    // Create a user
    let user = fixtures::valid_create_user();
    let create_req = test::TestRequest::post()
        .uri("/users")
        .set_json(&user)
        .to_request();
    let create_resp = test::call_service(&app, create_req).await;
    let create_body: serde_json::Value = test::read_body_json(create_resp).await;
    let user_id = create_body["data"]["id"].as_str().unwrap();

    // Get user by ID
    let get_req = test::TestRequest::get()
        .uri(&format!("/users/{}", user_id))
        .to_request();

    let get_resp = test::call_service(&app, get_req).await;
    assert_eq!(get_resp.status(), StatusCode::OK);

    let get_body: serde_json::Value = test::read_body_json(get_resp).await;
    assert_eq!(get_body["status"].as_u64().unwrap(), 200);
    assert_eq!(get_body["data"]["id"].as_str().unwrap(), user_id);
    assert_eq!(get_body["data"]["email"].as_str().unwrap(), user.email);
}

#[actix_rt::test]
async fn test_get_user_by_id_not_found() {
    let pool = test_helpers::setup_test_db().await;
    test_helpers::clean_users_table(&pool).await;
    let app = test_helpers::create_test_app(pool.clone()).await;

    let random_uuid = Uuid::new_v4();
    let req = test::TestRequest::get()
        .uri(&format!("/users/{}", random_uuid))
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), StatusCode::NOT_FOUND);

    let body: serde_json::Value = test::read_body_json(resp).await;
    assert_eq!(body["status_code"].as_u64().unwrap(), 404);
    assert!(body["message"].as_str().unwrap().contains("not found"));
}

#[actix_rt::test]
async fn test_get_user_by_invalid_uuid() {
    let pool = test_helpers::setup_test_db().await;
    let app = test_helpers::create_test_app(pool.clone()).await;

    let req = test::TestRequest::get()
        .uri("/users/invalid-uuid-format")
        .to_request();

    let resp = test::call_service(&app, req).await;
    // Actix will return BAD_REQUEST for invalid UUID in path
    assert_eq!(resp.status(), StatusCode::NOT_FOUND); // Route not matched
}
