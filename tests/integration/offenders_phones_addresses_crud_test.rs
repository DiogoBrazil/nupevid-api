use actix_web::{http::StatusCode, test};
use sqlx::PgPool;
use uuid::Uuid;

use crate::common::{db_fixtures, test_helpers};

fn build_phone_payload() -> serde_json::Value {
    serde_json::json!({
        "phone": "11987654321",
        "phone_type": "Mobile"
    })
}

fn build_address_payload(city_id: Uuid) -> serde_json::Value {
    serde_json::json!({
        "street": "Rua Nova",
        "number": "42",
        "district": "Bairro",
        "city_id": city_id,
        "zip_code": "12345-000",
        "complement": "Casa",
        "address_type": "Residential"
    })
}

#[sqlx::test]
async fn can_add_update_delete_phone_for_offender(pool: PgPool) {
    let config = test_helpers::build_test_config();
    let app = test_helpers::create_full_test_app(pool.clone(), config.clone()).await;

    let city_id = db_fixtures::insert_city(&pool, "City Offender Phones").await;
    let offender_id = db_fixtures::insert_offender(&pool, "Offender For Phones", city_id).await;

    let root_claims = test_helpers::build_root_claims();
    let token = test_helpers::generate_jwt(&root_claims, &config.jwt_secret);

    // 1. Add phone
    let payload = build_phone_payload();
    let req = test_helpers::with_auth_headers(
        test::TestRequest::post()
            .uri(&format!("/api/v1/offenders/{}/phones", offender_id))
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
        "phone_type": "Work"
    });
    let update_req = test_helpers::with_auth_headers(
        test::TestRequest::put()
            .uri(&format!(
                "/api/v1/offenders/{}/phones/{}",
                offender_id, phone_id
            ))
            .set_json(&update_payload),
        &config,
        &token,
    )
    .to_request();
    let update_resp = test::call_service(&app, update_req).await;
    assert_eq!(update_resp.status(), StatusCode::OK);
    let updated_body: serde_json::Value = test::read_body_json(update_resp).await;
    assert_eq!(
        updated_body["data"]["phone"].as_str().unwrap(),
        "11123456789"
    );
    assert_eq!(updated_body["data"]["phone_type"].as_str().unwrap(), "Work");

    // 3. Delete phone
    let delete_req = test_helpers::with_auth_headers(
        test::TestRequest::delete().uri(&format!(
            "/api/v1/offenders/{}/phones/{}",
            offender_id, phone_id
        )),
        &config,
        &token,
    )
    .to_request();
    let delete_resp = test::call_service(&app, delete_req).await;
    assert_eq!(delete_resp.status(), StatusCode::OK);

    // 4. Verify it's gone
    let get_req = test_helpers::with_auth_headers(
        test::TestRequest::get().uri(&format!("/api/v1/offenders/{}", offender_id)),
        &config,
        &token,
    )
    .to_request();
    let get_resp = test::call_service(&app, get_req).await;
    let offender_body: serde_json::Value = test::read_body_json(get_resp).await;
    assert!(
        offender_body["data"]["phones"]
            .as_array()
            .unwrap()
            .is_empty()
    );
}

#[sqlx::test]
async fn can_add_update_delete_address_for_offender(pool: PgPool) {
    let config = test_helpers::build_test_config();
    let app = test_helpers::create_full_test_app(pool.clone(), config.clone()).await;

    let city_id = db_fixtures::insert_city(&pool, "City Offender Addresses").await;
    let offender_id = db_fixtures::insert_offender(&pool, "Offender For Addresses", city_id).await;

    let root_claims = test_helpers::build_root_claims();
    let token = test_helpers::generate_jwt(&root_claims, &config.jwt_secret);

    // 1. Add address
    let payload = build_address_payload(city_id);
    let req = test_helpers::with_auth_headers(
        test::TestRequest::post()
            .uri(&format!("/api/v1/offenders/{}/addresses", offender_id))
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
        "complement": "",
        "address_type": "Residential"
    });
    let update_req = test_helpers::with_auth_headers(
        test::TestRequest::put()
            .uri(&format!(
                "/api/v1/offenders/{}/addresses/{}",
                offender_id, address_id
            ))
            .set_json(&update_payload),
        &config,
        &token,
    )
    .to_request();
    let update_resp = test::call_service(&app, update_req).await;
    assert_eq!(update_resp.status(), StatusCode::OK);
    let updated_body: serde_json::Value = test::read_body_json(update_resp).await;
    assert_eq!(
        updated_body["data"]["street"].as_str().unwrap(),
        "Rua Alterada"
    );

    // 3. Delete address
    let delete_req = test_helpers::with_auth_headers(
        test::TestRequest::delete().uri(&format!(
            "/api/v1/offenders/{}/addresses/{}",
            offender_id, address_id
        )),
        &config,
        &token,
    )
    .to_request();
    let delete_resp = test::call_service(&app, delete_req).await;
    assert_eq!(delete_resp.status(), StatusCode::OK);

    // 4. Verify it's gone
    let get_req = test_helpers::with_auth_headers(
        test::TestRequest::get().uri(&format!("/api/v1/offenders/{}", offender_id)),
        &config,
        &token,
    )
    .to_request();
    let get_resp = test::call_service(&app, get_req).await;
    let offender_body: serde_json::Value = test::read_body_json(get_resp).await;
    assert!(
        offender_body["data"]["addresses"]
            .as_array()
            .unwrap()
            .is_empty()
    );
}

#[sqlx::test]
async fn city_admin_cannot_modify_offender_phone_in_other_city(pool: PgPool) {
    let config = test_helpers::build_test_config();
    let app = test_helpers::create_full_test_app(pool.clone(), config.clone()).await;

    let city_a = db_fixtures::insert_city(&pool, "City A Off Phone").await;
    let city_b = db_fixtures::insert_city(&pool, "City B Off Phone").await;
    let offender_b_id = db_fixtures::insert_offender(&pool, "Offender in City B", city_b).await;

    let admin_a_claims = test_helpers::build_city_admin_claims(city_a);
    let token = test_helpers::generate_jwt(&admin_a_claims, &config.jwt_secret);

    let payload = build_phone_payload();
    let req = test_helpers::with_auth_headers(
        test::TestRequest::post()
            .uri(&format!("/api/v1/offenders/{}/phones", offender_b_id))
            .set_json(&payload),
        &config,
        &token,
    )
    .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), StatusCode::FORBIDDEN);
}

#[sqlx::test]
async fn city_admin_cannot_modify_offender_address_in_other_city(pool: PgPool) {
    let config = test_helpers::build_test_config();
    let app = test_helpers::create_full_test_app(pool.clone(), config.clone()).await;

    let city_a = db_fixtures::insert_city(&pool, "City A Off Addr").await;
    let city_b = db_fixtures::insert_city(&pool, "City B Off Addr").await;
    let offender_b_id = db_fixtures::insert_offender(&pool, "Offender in City B", city_b).await;

    let admin_a_claims = test_helpers::build_city_admin_claims(city_a);
    let token = test_helpers::generate_jwt(&admin_a_claims, &config.jwt_secret);

    let payload = build_address_payload(city_b);
    let req = test_helpers::with_auth_headers(
        test::TestRequest::post()
            .uri(&format!("/api/v1/offenders/{}/addresses", offender_b_id))
            .set_json(&payload),
        &config,
        &token,
    )
    .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), StatusCode::FORBIDDEN);
}
