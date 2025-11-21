use actix_web::{test, http::StatusCode};
use uuid::Uuid;

use crate::common::{fixtures, test_helpers};

#[actix_rt::test]
async fn test_update_password_success() {
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

    // Update password
    let password_data = fixtures::valid_update_password();
    let update_req = test::TestRequest::patch()
        .uri(&format!("/users/{}/password", user_id))
        .set_json(&password_data)
        .to_request();

    let update_resp = test::call_service(&app, update_req).await;
    assert_eq!(update_resp.status(), StatusCode::OK);

    let update_body: serde_json::Value = test::read_body_json(update_resp).await;
    assert_eq!(update_body["status"].as_u64().unwrap(), 200);
    assert_eq!(update_body["data"]["id"].as_str().unwrap(), user_id);
}

#[actix_rt::test]
async fn test_update_password_incorrect_current() {
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

    // Try to update password with incorrect current password
    let password_data = fixtures::invalid_update_password();
    let update_req = test::TestRequest::patch()
        .uri(&format!("/users/{}/password", user_id))
        .set_json(&password_data)
        .to_request();

    let update_resp = test::call_service(&app, update_req).await;
    assert_eq!(update_resp.status(), StatusCode::BAD_REQUEST);

    let update_body: serde_json::Value = test::read_body_json(update_resp).await;
    assert_eq!(update_body["status_code"].as_u64().unwrap(), 400);
    assert!(update_body["message"].as_str().unwrap().contains("incorrect"));
}

#[actix_rt::test]
async fn test_update_password_user_not_found() {
    let pool = test_helpers::setup_test_db().await;
    test_helpers::clean_users_table(&pool).await;
    let app = test_helpers::create_test_app(pool.clone()).await;

    let random_uuid = Uuid::new_v4();
    let password_data = fixtures::valid_update_password();
    let req = test::TestRequest::patch()
        .uri(&format!("/users/{}/password", random_uuid))
        .set_json(&password_data)
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), StatusCode::NOT_FOUND);

    let body: serde_json::Value = test::read_body_json(resp).await;
    assert_eq!(body["status_code"].as_u64().unwrap(), 404);
    assert!(body["message"].as_str().unwrap().contains("not found"));
}

#[actix_rt::test]
async fn test_update_password_empty_fields() {
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

    // Try to update with empty password fields
    let password_data = serde_json::json!({
        "current_password": "",
        "new_password": ""
    });
    let update_req = test::TestRequest::patch()
        .uri(&format!("/users/{}/password", user_id))
        .set_json(&password_data)
        .to_request();

    let update_resp = test::call_service(&app, update_req).await;
    assert_eq!(update_resp.status(), StatusCode::BAD_REQUEST);

    let update_body: serde_json::Value = test::read_body_json(update_resp).await;
    assert_eq!(update_body["status_code"].as_u64().unwrap(), 400);
    assert!(update_body["message"].as_str().unwrap().contains("required"));
}
