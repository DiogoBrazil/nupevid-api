use actix_web::{test, http::StatusCode};
use uuid::Uuid;

use crate::common::{fixtures, test_helpers};

#[actix_rt::test]
async fn test_update_user_success() {
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

    // Update user
    let update_data = fixtures::valid_update_user();
    let update_req = test::TestRequest::put()
        .uri(&format!("/users/{}", user_id))
        .set_json(&update_data)
        .to_request();

    let update_resp = test::call_service(&app, update_req).await;
    assert_eq!(update_resp.status(), StatusCode::OK);

    let update_body: serde_json::Value = test::read_body_json(update_resp).await;
    assert_eq!(update_body["status"].as_u64().unwrap(), 200);
    assert_eq!(update_body["data"]["full_name"].as_str().unwrap(), update_data.full_name);
    assert_eq!(update_body["data"]["email"].as_str().unwrap(), update_data.email);
}

#[actix_rt::test]
async fn test_update_user_not_found() {
    let pool = test_helpers::setup_test_db().await;
    test_helpers::clean_users_table(&pool).await;
    let app = test_helpers::create_test_app(pool.clone()).await;

    let random_uuid = Uuid::new_v4();
    let update_data = fixtures::valid_update_user();
    let req = test::TestRequest::put()
        .uri(&format!("/users/{}", random_uuid))
        .set_json(&update_data)
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), StatusCode::NOT_FOUND);

    let body: serde_json::Value = test::read_body_json(resp).await;
    assert_eq!(body["status_code"].as_u64().unwrap(), 404);
    assert!(body["message"].as_str().unwrap().contains("not found"));
}

#[actix_rt::test]
async fn test_update_user_duplicate_email() {
    let pool = test_helpers::setup_test_db().await;
    test_helpers::clean_users_table(&pool).await;
    let app = test_helpers::create_test_app(pool.clone()).await;

    // Create two users
    let user1 = fixtures::valid_create_user();
    let user2 = fixtures::valid_create_user_2();
    
    let create_req1 = test::TestRequest::post()
        .uri("/users")
        .set_json(&user1)
        .to_request();
    let create_resp1 = test::call_service(&app, create_req1).await;
    let create_body1: serde_json::Value = test::read_body_json(create_resp1).await;
    let user1_id = create_body1["data"]["id"].as_str().unwrap();

    let create_req2 = test::TestRequest::post()
        .uri("/users")
        .set_json(&user2)
        .to_request();
    test::call_service(&app, create_req2).await;

    // Try to update user1 with user2's email
    let mut update_data = fixtures::valid_update_user();
    update_data.email = user2.email.clone();
    
    let update_req = test::TestRequest::put()
        .uri(&format!("/users/{}", user1_id))
        .set_json(&update_data)
        .to_request();

    let update_resp = test::call_service(&app, update_req).await;
    assert_eq!(update_resp.status(), StatusCode::BAD_REQUEST);

    let update_body: serde_json::Value = test::read_body_json(update_resp).await;
    assert_eq!(update_body["status_code"].as_u64().unwrap(), 400);
    assert!(update_body["message"].as_str().unwrap().contains("already in use"));
}

#[actix_rt::test]
async fn test_update_user_duplicate_registration() {
    let pool = test_helpers::setup_test_db().await;
    test_helpers::clean_users_table(&pool).await;
    let app = test_helpers::create_test_app(pool.clone()).await;

    // Create two users
    let user1 = fixtures::valid_create_user();
    let user2 = fixtures::valid_create_user_2();
    
    let create_req1 = test::TestRequest::post()
        .uri("/users")
        .set_json(&user1)
        .to_request();
    let create_resp1 = test::call_service(&app, create_req1).await;
    let create_body1: serde_json::Value = test::read_body_json(create_resp1).await;
    let user1_id = create_body1["data"]["id"].as_str().unwrap();

    let create_req2 = test::TestRequest::post()
        .uri("/users")
        .set_json(&user2)
        .to_request();
    test::call_service(&app, create_req2).await;

    // Try to update user1 with user2's registration
    let mut update_data = fixtures::valid_update_user();
    update_data.registration = user2.registration.clone();
    
    let update_req = test::TestRequest::put()
        .uri(&format!("/users/{}", user1_id))
        .set_json(&update_data)
        .to_request();

    let update_resp = test::call_service(&app, update_req).await;
    assert_eq!(update_resp.status(), StatusCode::BAD_REQUEST);

    let update_body: serde_json::Value = test::read_body_json(update_resp).await;
    assert_eq!(update_body["status_code"].as_u64().unwrap(), 400);
    assert!(update_body["message"].as_str().unwrap().contains("registration"));
    assert!(update_body["message"].as_str().unwrap().contains("already exists"));
}

#[actix_rt::test]
async fn test_delete_user_success() {
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

    // Delete user
    let delete_req = test::TestRequest::delete()
        .uri(&format!("/users/{}", user_id))
        .to_request();

    let delete_resp = test::call_service(&app, delete_req).await;
    assert_eq!(delete_resp.status(), StatusCode::OK);

    let delete_body: serde_json::Value = test::read_body_json(delete_resp).await;
    assert_eq!(delete_body["status"].as_u64().unwrap(), 200);
    assert_eq!(delete_body["data"]["id"].as_str().unwrap(), user_id);

    // Verify user is deleted
    let get_req = test::TestRequest::get()
        .uri(&format!("/users/{}", user_id))
        .to_request();
    let get_resp = test::call_service(&app, get_req).await;
    assert_eq!(get_resp.status(), StatusCode::NOT_FOUND);
}

#[actix_rt::test]
async fn test_delete_user_not_found() {
    let pool = test_helpers::setup_test_db().await;
    test_helpers::clean_users_table(&pool).await;
    let app = test_helpers::create_test_app(pool.clone()).await;

    let random_uuid = Uuid::new_v4();
    let req = test::TestRequest::delete()
        .uri(&format!("/users/{}", random_uuid))
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), StatusCode::NOT_FOUND);

    let body: serde_json::Value = test::read_body_json(resp).await;
    assert_eq!(body["status_code"].as_u64().unwrap(), 404);
    assert!(body["message"].as_str().unwrap().contains("not found"));
}
