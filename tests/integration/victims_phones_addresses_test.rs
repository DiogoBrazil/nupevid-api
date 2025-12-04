use actix_web::{http::StatusCode, test};
use uuid::Uuid;

use crate::common::{db_fixtures, test_helpers};

fn build_phone_payload() -> serde_json::Value {
    serde_json::json!({
        "phone": "11987654321",
        "phone_type": "whatsapp"
    })
}

fn build_address_payload(city_id: Uuid) -> serde_json::Value {
    serde_json::json!({
        "street": "Rua Nova",
        "number": "42",
        "district": "Bairro",
        "city_id": city_id,
        "zip_code": "12345-000",
        "complement": "Casa"
    })
}

#[actix_rt::test]
async fn can_add_update_delete_phone_for_victim() {
    let pool = test_helpers::setup_test_db().await;
    test_helpers::clean_database(&pool).await;
    let config = test_helpers::build_test_config();
    let app = test_helpers::create_full_test_app(pool.clone(), config.clone()).await;

    let city_id = db_fixtures::insert_city(&pool, "City For Phones").await;
    let victim_id = db_fixtures::insert_victim(&pool, "Victim For Phones", city_id).await;

    let root_claims = test_helpers::build_root_claims();
    let token = test_helpers::generate_jwt(&root_claims, &config.jwt_secret);

    // 1. Add phone
    let payload = build_phone_payload();
    let req = test_helpers::with_auth_headers(
        test::TestRequest::post()
            .uri(&format!("/api/v1/victims/{}/phones", victim_id))
            .set_json(&payload),
        &config,
        &token,
    )
    .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), StatusCode::CREATED);
    let body: serde_json::Value = test::read_body_json(resp).await;
    let phone_id = body["data"]["id"].as_str().unwrap();
    assert_eq!(body["data"]["phone"].as_str().unwrap(), "11987654321");

    // 2. Update phone
    let update_payload = serde_json::json!({
        "phone": "11123456789",
        "phone_type": "other"
    });
    let update_req = test_helpers::with_auth_headers(
        test::TestRequest::put()
            .uri(&format!("/api/v1/victims/{}/phones/{}", victim_id, phone_id))
            .set_json(&update_payload),
        &config,
        &token,
    )
    .to_request();
    let update_resp = test::call_service(&app, update_req).await;
    assert_eq!(update_resp.status(), StatusCode::OK);
    let updated_body: serde_json::Value = test::read_body_json(update_resp).await;
    assert_eq!(updated_body["data"]["phone"].as_str().unwrap(), "11123456789");
    assert_eq!(updated_body["data"]["phone_type"].as_str().unwrap(), "other");

    // 3. Delete phone
    let delete_req = test_helpers::with_auth_headers(
        test::TestRequest::delete()
            .uri(&format!("/api/v1/victims/{}/phones/{}", victim_id, phone_id)),
        &config,
        &token,
    )
    .to_request();
    let delete_resp = test::call_service(&app, delete_req).await;
    assert_eq!(delete_resp.status(), StatusCode::OK);

    // 4. Verify it's gone
    let get_victim_req = test_helpers::with_auth_headers(
        test::TestRequest::get()
            .uri(&format!("/api/v1/victims/{}", victim_id)),
        &config,
        &token,
    )
    .to_request();
    let get_victim_resp = test::call_service(&app, get_victim_req).await;
    let victim_body: serde_json::Value = test::read_body_json(get_victim_resp).await;
    assert!(victim_body["data"]["phones"].as_array().unwrap().is_empty());
}

#[actix_rt::test]
async fn can_add_update_delete_address_for_victim() {
    let pool = test_helpers::setup_test_db().await;
    test_helpers::clean_database(&pool).await;
    let config = test_helpers::build_test_config();
    let app = test_helpers::create_full_test_app(pool.clone(), config.clone()).await;

    let city_id = db_fixtures::insert_city(&pool, "City For Addresses").await;
    let victim_id = db_fixtures::insert_victim(&pool, "Victim For Addresses", city_id).await;

    let root_claims = test_helpers::build_root_claims();
    let token = test_helpers::generate_jwt(&root_claims, &config.jwt_secret);

    // 1. Add address
    let payload = build_address_payload(city_id);
    let req = test_helpers::with_auth_headers(
        test::TestRequest::post()
            .uri(&format!("/api/v1/victims/{}/addresses", victim_id))
            .set_json(&payload),
        &config,
        &token,
    )
    .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), StatusCode::CREATED);
    let body: serde_json::Value = test::read_body_json(resp).await;
    let address_id = body["data"]["id"].as_str().unwrap();
    assert_eq!(body["data"]["street"].as_str().unwrap(), "Rua Nova");

    // 2. Update address
    let update_payload = serde_json::json!({
        "street": "Rua Alterada",
        "number": "123",
        "district": "Centro",
        "city_id": city_id,
        "zip_code": "54321-000",
        "complement": ""
    });
    let update_req = test_helpers::with_auth_headers(
        test::TestRequest::put()
            .uri(&format!("/api/v1/victims/{}/addresses/{}", victim_id, address_id))
            .set_json(&update_payload),
        &config,
        &token,
    )
    .to_request();
    let update_resp = test::call_service(&app, update_req).await;
    assert_eq!(update_resp.status(), StatusCode::OK);
    let updated_body: serde_json::Value = test::read_body_json(update_resp).await;
    assert_eq!(updated_body["data"]["street"].as_str().unwrap(), "Rua Alterada");

    // 3. Delete address
    let delete_req = test_helpers::with_auth_headers(
        test::TestRequest::delete()
            .uri(&format!("/api/v1/victims/{}/addresses/{}", victim_id, address_id)),
        &config,
        &token,
    )
    .to_request();
    let delete_resp = test::call_service(&app, delete_req).await;
    assert_eq!(delete_resp.status(), StatusCode::OK);

    // 4. Verify it's gone
    let get_victim_req = test_helpers::with_auth_headers(
        test::TestRequest::get()
            .uri(&format!("/api/v1/victims/{}", victim_id)),
        &config,
        &token,
    )
    .to_request();
    let get_victim_resp = test::call_service(&app, get_victim_req).await;
    let victim_body: serde_json::Value = test::read_body_json(get_victim_resp).await;
    assert!(victim_body["data"]["addresses"].as_array().unwrap().is_empty());
}

#[actix_rt::test]
async fn city_admin_cannot_modify_phone_in_other_city() {
    let pool = test_helpers::setup_test_db().await;
    test_helpers::clean_database(&pool).await;
    let config = test_helpers::build_test_config();
    let app = test_helpers::create_full_test_app(pool.clone(), config.clone()).await;

    let city_a = db_fixtures::insert_city(&pool, "City A").await;
    let city_b = db_fixtures::insert_city(&pool, "City B").await;
    let victim_b_id = db_fixtures::insert_victim(&pool, "Victim in City B", city_b).await;

    let admin_a_claims = test_helpers::build_city_admin_claims(city_a);
    let token = test_helpers::generate_jwt(&admin_a_claims, &config.jwt_secret);

    // Try to add phone to victim in another city
    let payload = build_phone_payload();
    let req = test_helpers::with_auth_headers(
        test::TestRequest::post()
            .uri(&format!("/api/v1/victims/{}/phones", victim_b_id))
            .set_json(&payload),
        &config,
        &token,
    )
    .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), StatusCode::FORBIDDEN);
}

#[actix_rt::test]
async fn city_admin_cannot_modify_address_in_other_city() {
    let pool = test_helpers::setup_test_db().await;
    test_helpers::clean_database(&pool).await;
    let config = test_helpers::build_test_config();
    let app = test_helpers::create_full_test_app(pool.clone(), config.clone()).await;

    let city_a = db_fixtures::insert_city(&pool, "City A").await;
    let city_b = db_fixtures::insert_city(&pool, "City B").await;
    let victim_b_id = db_fixtures::insert_victim(&pool, "Victim in City B", city_b).await;

    let admin_a_claims = test_helpers::build_city_admin_claims(city_a);
    let token = test_helpers::generate_jwt(&admin_a_claims, &config.jwt_secret);

    // Try to add address to victim in another city
    let payload = build_address_payload(city_b);
    let req = test_helpers::with_auth_headers(
        test::TestRequest::post()
            .uri(&format!("/api/v1/victims/{}/addresses", victim_b_id))
            .set_json(&payload),
        &config,
        &token,
    )
    .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), StatusCode::FORBIDDEN);
}
