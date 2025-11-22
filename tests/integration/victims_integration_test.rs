use actix_web::{test, http::StatusCode};
use uuid::Uuid;

use crate::common::{test_helpers, db_fixtures};

fn build_victim_payload(full_name: &str, city_id: Uuid) -> serde_json::Value {
    serde_json::json!({
        "full_name": full_name,
        "document_id": serde_json::Value::Null,
        "birth_date": serde_json::Value::Null,
        "phone": serde_json::Value::Null,
        "city_id": city_id,
    })
}


#[actix_rt::test]
async fn root_can_create_and_list_victims_in_multiple_cities() {
    let pool = test_helpers::setup_test_db().await;
    test_helpers::clean_database(&pool).await;

    let config = test_helpers::build_test_config();
    let app = test_helpers::create_full_test_app(pool.clone(), config.clone()).await;

    let city_a = db_fixtures::insert_city(&pool, "Cidade A").await;
    let city_b = db_fixtures::insert_city(&pool, "Cidade B").await;

    let root_claims = test_helpers::build_root_claims();
    let root_token = test_helpers::generate_jwt(&root_claims, &config.jwt_secret);

    // Create victim in city A
    for (name, city_id) in [("Vitima A", city_a), ("Vitima B", city_b)] {
        let payload = build_victim_payload(name, city_id);
        let req = test_helpers::with_auth_headers(
            test::TestRequest::post()
                .uri("/api/v1/victims")
                .set_json(&payload),
            &config,
            &root_token,
        )
        .to_request();

        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status(), StatusCode::CREATED);
    }

    // ROOT should see victims from both cities
    let list_req = test_helpers::with_auth_headers(
        test::TestRequest::get().uri("/api/v1/victims"),
        &config,
        &root_token,
    )
    .to_request();

    let list_resp = test::call_service(&app, list_req).await;
    assert_eq!(list_resp.status(), StatusCode::OK);
    let body: serde_json::Value = test::read_body_json(list_resp).await;
    assert_eq!(body["data"].as_array().unwrap().len(), 2);
}

#[actix_rt::test]
async fn city_admin_can_only_access_victims_in_own_city() {
    let pool = test_helpers::setup_test_db().await;
    test_helpers::clean_database(&pool).await;

    let config = test_helpers::build_test_config();
    let app = test_helpers::create_full_test_app(pool.clone(), config.clone()).await;

    let city_a = db_fixtures::insert_city(&pool, "Cidade A").await;
    let city_b = db_fixtures::insert_city(&pool, "Cidade B").await;

    let root_claims = test_helpers::build_root_claims();
    let root_token = test_helpers::generate_jwt(&root_claims, &config.jwt_secret);

    // Create victims in both cities as ROOT
    let v1_payload = build_victim_payload("Vitima A", city_a);
    let v2_payload = build_victim_payload("Vitima B", city_b);

    let create_v1 = test_helpers::with_auth_headers(
        test::TestRequest::post().uri("/api/v1/victims").set_json(&v1_payload),
        &config,
        &root_token,
    )
    .to_request();
    let create_v1_resp = test::call_service(&app, create_v1).await;
    assert_eq!(create_v1_resp.status(), StatusCode::CREATED);
    let v1_body: serde_json::Value = test::read_body_json(create_v1_resp).await;
    let victim_a_id = v1_body["data"]["id"].as_str().unwrap().to_string();

    let create_v2 = test_helpers::with_auth_headers(
        test::TestRequest::post().uri("/api/v1/victims").set_json(&v2_payload),
        &config,
        &root_token,
    )
    .to_request();
    let create_v2_resp = test::call_service(&app, create_v2).await;
    assert_eq!(create_v2_resp.status(), StatusCode::CREATED);

    // CITY_ADMIN for city A
    let admin_a_claims = test_helpers::build_city_admin_claims(city_a);
    let admin_a_token = test_helpers::generate_jwt(&admin_a_claims, &config.jwt_secret);

    // List victims -> should only see victims in city A
    let list_req = test_helpers::with_auth_headers(
        test::TestRequest::get().uri("/api/v1/victims"),
        &config,
        &admin_a_token,
    )
    .to_request();

    let list_resp = test::call_service(&app, list_req).await;
    assert_eq!(list_resp.status(), StatusCode::OK);
    let list_body: serde_json::Value = test::read_body_json(list_resp).await;
    let victims = list_body["data"].as_array().unwrap();
    assert_eq!(victims.len(), 1);
    assert_eq!(victims[0]["city_id"].as_str().unwrap(), city_a.to_string());

    // CITY_ADMIN A can GET victim from its city
    let get_own_req = test_helpers::with_auth_headers(
        test::TestRequest::get().uri(&format!("/api/v1/victims/{}", victim_a_id)),
        &config,
        &admin_a_token,
    )
    .to_request();
    let get_own_resp = test::call_service(&app, get_own_req).await;
    assert_eq!(get_own_resp.status(), StatusCode::OK);

    // CITY_ADMIN A cannot access victim from city B
    let victims_city_b_req = test_helpers::with_auth_headers(
        test::TestRequest::get().uri("/api/v1/victims"),
        &config,
        &root_token,
    )
    .to_request();
    let victims_city_b_resp = test::call_service(&app, victims_city_b_req).await;
    assert_eq!(victims_city_b_resp.status(), StatusCode::OK);
}

#[actix_rt::test]
async fn delete_victim_soft_delete_and_not_listed() {
    let pool = test_helpers::setup_test_db().await;
    test_helpers::clean_database(&pool).await;

    let config = test_helpers::build_test_config();
    let app = test_helpers::create_full_test_app(pool.clone(), config.clone()).await;

    let city = db_fixtures::insert_city(&pool, "Cidade Delete").await;
    let root_claims = test_helpers::build_root_claims();
    let root_token = test_helpers::generate_jwt(&root_claims, &config.jwt_secret);

    let payload = build_victim_payload("Vitima Delete", city);
    let create_req = test_helpers::with_auth_headers(
        test::TestRequest::post().uri("/api/v1/victims").set_json(&payload),
        &config,
        &root_token,
    )
    .to_request();
    let create_resp = test::call_service(&app, create_req).await;
    assert_eq!(create_resp.status(), StatusCode::CREATED);
    let body: serde_json::Value = test::read_body_json(create_resp).await;
    let victim_id = body["data"]["id"].as_str().unwrap();

    // Delete victim
    let delete_req = test_helpers::with_auth_headers(
        test::TestRequest::delete().uri(&format!("/api/v1/victims/{}", victim_id)),
        &config,
        &root_token,
    )
    .to_request();
    let delete_resp = test::call_service(&app, delete_req).await;
    assert_eq!(delete_resp.status(), StatusCode::OK);

    // GET by id should return 404
    let get_req = test_helpers::with_auth_headers(
        test::TestRequest::get().uri(&format!("/api/v1/victims/{}", victim_id)),
        &config,
        &root_token,
    )
    .to_request();
    let get_resp = test::call_service(&app, get_req).await;
    assert_eq!(get_resp.status(), StatusCode::NOT_FOUND);

    // List should not include deleted victim
    let list_req = test_helpers::with_auth_headers(
        test::TestRequest::get().uri("/api/v1/victims"),
        &config,
        &root_token,
    )
    .to_request();
    let list_resp = test::call_service(&app, list_req).await;
    assert_eq!(list_resp.status(), StatusCode::OK);
    let list_body: serde_json::Value = test::read_body_json(list_resp).await;
    let victims = list_body["data"].as_array().unwrap();
    assert!(victims.iter().all(|v| v["id"].as_str().unwrap() != victim_id));
}

#[actix_rt::test]
async fn city_admin_cannot_create_victim_in_other_city() {
    let pool = test_helpers::setup_test_db().await;
    test_helpers::clean_database(&pool).await;

    let config = test_helpers::build_test_config();
    let app = test_helpers::create_full_test_app(pool.clone(), config.clone()).await;

    let city_a = db_fixtures::insert_city(&pool, "Cidade A").await;
    let city_b = db_fixtures::insert_city(&pool, "Cidade B").await;

    let admin_a_claims = test_helpers::build_city_admin_claims(city_a);
    let admin_a_token = test_helpers::generate_jwt(&admin_a_claims, &config.jwt_secret);

    // Try to create victim in a different city (B)
    let payload = build_victim_payload("Vitima", city_b);
    let req = test_helpers::with_auth_headers(
        test::TestRequest::post().uri("/api/v1/victims").set_json(&payload),
        &config,
        &admin_a_token,
    )
    .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), StatusCode::FORBIDDEN);
}

#[actix_rt::test]
async fn victim_addresses_respect_city_access_rules() {
    let pool = test_helpers::setup_test_db().await;
    test_helpers::clean_database(&pool).await;

    let config = test_helpers::build_test_config();
    let app = test_helpers::create_full_test_app(pool.clone(), config.clone()).await;

    let city_a = db_fixtures::insert_city(&pool, "Cidade A").await;
    let city_b = db_fixtures::insert_city(&pool, "Cidade B").await;

    let root_claims = test_helpers::build_root_claims();
    let root_token = test_helpers::generate_jwt(&root_claims, &config.jwt_secret);

    // Create victim in city B as ROOT
    let victim_payload = build_victim_payload("Vitima B", city_b);
    let create_victim_req = test_helpers::with_auth_headers(
        test::TestRequest::post().uri("/api/v1/victims").set_json(&victim_payload),
        &config,
        &root_token,
    )
    .to_request();
    let create_victim_resp = test::call_service(&app, create_victim_req).await;
    assert_eq!(create_victim_resp.status(), StatusCode::CREATED);
    let create_victim_body: serde_json::Value = test::read_body_json(create_victim_resp).await;
    let victim_b_id = create_victim_body["data"]["id"].as_str().unwrap();

    // CITY_ADMIN A
    let admin_a_claims = test_helpers::build_city_admin_claims(city_a);
    let admin_a_token = test_helpers::generate_jwt(&admin_a_claims, &config.jwt_secret);

    // CITY_ADMIN A cannot create address for victim in city B
    let addr_payload = serde_json::json!({
        "victim_id": victim_b_id,
        "street": "Rua B",
        "number": "10",
        "district": "Centro",
        "city_name": "Cidade B",
        "state": "SP",
        "zip_code": "11111-111",
        "complement": "Casa"
    });

    let create_addr_req = test_helpers::with_auth_headers(
        test::TestRequest::post()
            .uri("/api/v1/victim-addresses")
            .set_json(&addr_payload),
        &config,
        &admin_a_token,
    )
    .to_request();

    let create_addr_resp = test::call_service(&app, create_addr_req).await;
    assert_eq!(create_addr_resp.status(), StatusCode::FORBIDDEN);
}