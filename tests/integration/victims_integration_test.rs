use actix_web::{http::StatusCode, test};
use sqlx::PgPool;
use uuid::Uuid;

use crate::common::{db_fixtures, test_helpers};

fn build_victim_payload(full_name: &str, city_id: Uuid) -> serde_json::Value {
    serde_json::json!({
        "full_name": full_name,
        "cpf": serde_json::Value::Null,
        "birth_date": serde_json::Value::Null,
        "city_id": city_id,
        "phones": serde_json::Value::Null,
        "addresses": serde_json::Value::Null,
        "education_level": serde_json::Value::Null,
        "occupation": serde_json::Value::Null,
        "children_count": serde_json::Value::Null,
        "special_needs_type": serde_json::Value::Null,
        "uses_alcohol": false,
        "uses_drugs": false,
        "psychiatric_issues_type": serde_json::Value::Null,
    })
}

fn build_victim_payload_with_address(full_name: &str, city_id: Uuid) -> serde_json::Value {
    serde_json::json!({
        "full_name": full_name,
        "cpf": "529.982.247-25",
        "birth_date": "1990-01-01",
        "city_id": city_id,
        "phones": [{
            "phone": "11999999999",
            "phone_type": "Mobile"
        }],
        "addresses": [{
            "street": "Rua Teste",
            "number": "100",
            "district": "Centro",
            "city_id": city_id,
            "zip_code": "01000-000",
            "complement": "Apt 10",
            "address_type": "Residential"
        }],
        "education_level": "College",
        "occupation": "Professora",
        "children_count": 2,
        "special_needs_type": serde_json::Value::Null,
        "uses_alcohol": false,
        "uses_drugs": false,
        "psychiatric_issues_type": serde_json::Value::Null,
    })
}

#[sqlx::test]
async fn search_victims_by_name_returns_matches(pool: PgPool) {

    let config = test_helpers::build_test_config();
    let app = test_helpers::create_full_test_app(pool.clone(), config.clone()).await;

    let city = db_fixtures::insert_city(&pool, "Cidade Busca").await;
    let root_claims = test_helpers::build_root_claims();
    let root_token = test_helpers::generate_jwt(&root_claims, &config.jwt_secret);

    for name in ["Maria Silva", "Joao Souza"] {
        let payload = build_victim_payload(name, city);
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

    let search_req = test_helpers::with_auth_headers(
        test::TestRequest::get().uri("/api/v1/victims/search?name=maria"),
        &config,
        &root_token,
    )
    .to_request();

    let search_resp = test::call_service(&app, search_req).await;
    assert_eq!(search_resp.status(), StatusCode::OK);
    let body: serde_json::Value = test::read_body_json(search_resp).await;
    let results = body["data"].as_array().unwrap();
    assert_eq!(results.len(), 1);
    assert_eq!(results[0]["full_name"].as_str().unwrap(), "MARIA SILVA");
}

#[sqlx::test]
async fn search_victims_by_cpf_returns_match(pool: PgPool) {

    let config = test_helpers::build_test_config();
    let app = test_helpers::create_full_test_app(pool.clone(), config.clone()).await;

    let city = db_fixtures::insert_city(&pool, "Cidade CPF").await;
    let root_claims = test_helpers::build_root_claims();
    let root_token = test_helpers::generate_jwt(&root_claims, &config.jwt_secret);

    let mut payload = build_victim_payload("Vitima CPF", city);
    payload["cpf"] = serde_json::json!("529.982.247-25");

    let create_req = test_helpers::with_auth_headers(
        test::TestRequest::post()
            .uri("/api/v1/victims")
            .set_json(&payload),
        &config,
        &root_token,
    )
    .to_request();

    let create_resp = test::call_service(&app, create_req).await;
    assert_eq!(create_resp.status(), StatusCode::CREATED);

    let search_req = test_helpers::with_auth_headers(
        test::TestRequest::get().uri("/api/v1/victims/search?cpf=529.982.247-25"),
        &config,
        &root_token,
    )
    .to_request();

    let search_resp = test::call_service(&app, search_req).await;
    assert_eq!(search_resp.status(), StatusCode::OK);
    let body: serde_json::Value = test::read_body_json(search_resp).await;
    let results = body["data"].as_array().unwrap();
    assert_eq!(results.len(), 1);
    assert_eq!(results[0]["cpf"].as_str().unwrap(), "529.982.247-25");
}

#[sqlx::test]
async fn search_victims_rejects_missing_filters(pool: PgPool) {

    let config = test_helpers::build_test_config();
    let app = test_helpers::create_full_test_app(pool.clone(), config.clone()).await;

    let root_claims = test_helpers::build_root_claims();
    let root_token = test_helpers::generate_jwt(&root_claims, &config.jwt_secret);

    let search_req = test_helpers::with_auth_headers(
        test::TestRequest::get().uri("/api/v1/victims/search"),
        &config,
        &root_token,
    )
    .to_request();

    let search_resp = test::call_service(&app, search_req).await;
    assert_eq!(search_resp.status(), StatusCode::BAD_REQUEST);
    let body: serde_json::Value = test::read_body_json(search_resp).await;
    assert!(
        body["message"]
            .as_str()
            .unwrap()
            .contains("query parameter 'name' or 'cpf' is required")
    );
}

#[sqlx::test]
async fn search_victims_rejects_conflicting_filters(pool: PgPool) {

    let config = test_helpers::build_test_config();
    let app = test_helpers::create_full_test_app(pool.clone(), config.clone()).await;

    let root_claims = test_helpers::build_root_claims();
    let root_token = test_helpers::generate_jwt(&root_claims, &config.jwt_secret);

    let search_req = test_helpers::with_auth_headers(
        test::TestRequest::get().uri("/api/v1/victims/search?name=maria&cpf=529.982.247-25"),
        &config,
        &root_token,
    )
    .to_request();

    let search_resp = test::call_service(&app, search_req).await;
    assert_eq!(search_resp.status(), StatusCode::BAD_REQUEST);
    let body: serde_json::Value = test::read_body_json(search_resp).await;
    assert!(
        body["message"]
            .as_str()
            .unwrap()
            .contains("provide either 'name' or 'cpf', not both")
    );
}

#[sqlx::test]
async fn search_victims_rejects_empty_name_filter(pool: PgPool) {

    let config = test_helpers::build_test_config();
    let app = test_helpers::create_full_test_app(pool.clone(), config.clone()).await;

    let root_claims = test_helpers::build_root_claims();
    let root_token = test_helpers::generate_jwt(&root_claims, &config.jwt_secret);

    let search_req = test_helpers::with_auth_headers(
        test::TestRequest::get().uri("/api/v1/victims/search?name=%20%20"),
        &config,
        &root_token,
    )
    .to_request();

    let search_resp = test::call_service(&app, search_req).await;
    assert_eq!(search_resp.status(), StatusCode::BAD_REQUEST);
    let body: serde_json::Value = test::read_body_json(search_resp).await;
    assert!(
        body["message"]
            .as_str()
            .unwrap()
            .contains("query parameter 'name' cannot be empty")
    );
}

#[sqlx::test]
async fn root_can_create_and_list_victims_in_multiple_cities(pool: PgPool) {

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

#[sqlx::test]
async fn city_admin_can_only_access_victims_in_own_city(pool: PgPool) {

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
        test::TestRequest::post()
            .uri("/api/v1/victims")
            .set_json(&v1_payload),
        &config,
        &root_token,
    )
    .to_request();
    let create_v1_resp = test::call_service(&app, create_v1).await;
    assert_eq!(create_v1_resp.status(), StatusCode::CREATED);
    let v1_body: serde_json::Value = test::read_body_json(create_v1_resp).await;
    let victim_a_id = v1_body["data"]["id"].as_str().unwrap().to_string();

    let create_v2 = test_helpers::with_auth_headers(
        test::TestRequest::post()
            .uri("/api/v1/victims")
            .set_json(&v2_payload),
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

#[sqlx::test]
async fn delete_victim_soft_delete_and_not_listed(pool: PgPool) {

    let config = test_helpers::build_test_config();
    let app = test_helpers::create_full_test_app(pool.clone(), config.clone()).await;

    let city = db_fixtures::insert_city(&pool, "Cidade Delete").await;
    let root_claims = test_helpers::build_root_claims();
    let root_token = test_helpers::generate_jwt(&root_claims, &config.jwt_secret);

    let payload = build_victim_payload("Vitima Delete", city);
    let create_req = test_helpers::with_auth_headers(
        test::TestRequest::post()
            .uri("/api/v1/victims")
            .set_json(&payload),
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
    assert!(
        victims
            .iter()
            .all(|v| v["id"].as_str().unwrap() != victim_id)
    );
}

#[sqlx::test]
async fn city_admin_cannot_create_victim_in_other_city(pool: PgPool) {

    let config = test_helpers::build_test_config();
    let app = test_helpers::create_full_test_app(pool.clone(), config.clone()).await;

    let city_a = db_fixtures::insert_city(&pool, "Cidade A").await;
    let city_b = db_fixtures::insert_city(&pool, "Cidade B").await;

    let admin_a_claims = test_helpers::build_city_admin_claims(city_a);
    let admin_a_token = test_helpers::generate_jwt(&admin_a_claims, &config.jwt_secret);

    // Try to create victim in a different city (B)
    let payload = build_victim_payload("Vitima", city_b);
    let req = test_helpers::with_auth_headers(
        test::TestRequest::post()
            .uri("/api/v1/victims")
            .set_json(&payload),
        &config,
        &admin_a_token,
    )
    .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), StatusCode::FORBIDDEN);
}

#[sqlx::test]
async fn create_victim_with_address_in_single_request(pool: PgPool) {

    let config = test_helpers::build_test_config();
    let app = test_helpers::create_full_test_app(pool.clone(), config.clone()).await;

    let city = db_fixtures::insert_city(&pool, "Cidade Addr").await;
    let root_claims = test_helpers::build_root_claims();
    let root_token = test_helpers::generate_jwt(&root_claims, &config.jwt_secret);

    // Create victim with address
    let payload = build_victim_payload_with_address("Vitima Com Endereco", city);
    let create_req = test_helpers::with_auth_headers(
        test::TestRequest::post()
            .uri("/api/v1/victims")
            .set_json(&payload),
        &config,
        &root_token,
    )
    .to_request();

    let create_resp = test::call_service(&app, create_req).await;
    assert_eq!(create_resp.status(), StatusCode::CREATED);

    let body: serde_json::Value = test::read_body_json(create_resp).await;

    // Verify victim data
    assert_eq!(
        body["data"]["full_name"].as_str().unwrap(),
        "VITIMA COM ENDERECO"
    );
    assert_eq!(body["data"]["cpf"].as_str().unwrap(), "529.982.247-25");

    // Verify address data is included (now as array)
    assert!(body["data"]["addresses"].is_array());
    assert_eq!(body["data"]["addresses"].as_array().unwrap().len(), 1);
    let address = &body["data"]["addresses"][0];
    assert_eq!(address["street"].as_str().unwrap(), "Rua Teste");
    assert_eq!(address["number"].as_str().unwrap(), "100");
    assert_eq!(address["district"].as_str().unwrap(), "Centro");
    assert_eq!(address["city_id"].as_str().unwrap(), city.to_string());
}

#[sqlx::test]
async fn get_victim_returns_address_when_exists(pool: PgPool) {

    let config = test_helpers::build_test_config();
    let app = test_helpers::create_full_test_app(pool.clone(), config.clone()).await;

    let city = db_fixtures::insert_city(&pool, "Cidade Get").await;
    let root_claims = test_helpers::build_root_claims();
    let root_token = test_helpers::generate_jwt(&root_claims, &config.jwt_secret);

    // Create victim with address
    let payload = build_victim_payload_with_address("Vitima Get", city);
    let create_req = test_helpers::with_auth_headers(
        test::TestRequest::post()
            .uri("/api/v1/victims")
            .set_json(&payload),
        &config,
        &root_token,
    )
    .to_request();

    let create_resp = test::call_service(&app, create_req).await;
    let create_body: serde_json::Value = test::read_body_json(create_resp).await;
    let victim_id = create_body["data"]["id"].as_str().unwrap();

    // GET victim should include address
    let get_req = test_helpers::with_auth_headers(
        test::TestRequest::get().uri(&format!("/api/v1/victims/{}", victim_id)),
        &config,
        &root_token,
    )
    .to_request();

    let get_resp = test::call_service(&app, get_req).await;
    assert_eq!(get_resp.status(), StatusCode::OK);

    let body: serde_json::Value = test::read_body_json(get_resp).await;
    assert!(body["data"]["addresses"].is_array());
    assert_eq!(body["data"]["addresses"].as_array().unwrap().len(), 1);
    assert_eq!(
        body["data"]["addresses"][0]["street"].as_str().unwrap(),
        "Rua Teste"
    );
}

#[sqlx::test]
async fn update_victim_can_add_or_update_address(pool: PgPool) {

    let config = test_helpers::build_test_config();
    let app = test_helpers::create_full_test_app(pool.clone(), config.clone()).await;

    let city = db_fixtures::insert_city(&pool, "Cidade Update").await;
    let root_claims = test_helpers::build_root_claims();
    let root_token = test_helpers::generate_jwt(&root_claims, &config.jwt_secret);

    // Create victim without address
    let payload = build_victim_payload("Vitima Update", city);
    let create_req = test_helpers::with_auth_headers(
        test::TestRequest::post()
            .uri("/api/v1/victims")
            .set_json(&payload),
        &config,
        &root_token,
    )
    .to_request();

    let create_resp = test::call_service(&app, create_req).await;
    let create_body: serde_json::Value = test::read_body_json(create_resp).await;
    let victim_id = create_body["data"]["id"].as_str().unwrap();

    // Verify no address initially (now empty array)
    assert!(create_body["data"]["addresses"].is_array());
    assert_eq!(
        create_body["data"]["addresses"].as_array().unwrap().len(),
        0
    );

    // Update victim adding address
    let update_payload = serde_json::json!({
        "full_name": "Vitima Update Renamed",
        "cpf": null,
        "birth_date": null,
        "city_id": city,
        "phones": null,
        "addresses": [{
            "street": "Nova Rua",
            "number": "200",
            "district": "Bairro Novo",
            "city_id": city,
            "zip_code": "20000-000",
            "complement": null,
            "address_type": "Residential"
        }],
        "education_level": null,
        "occupation": null,
        "children_count": null,
        "special_needs_type": null,
        "uses_alcohol": false,
        "uses_drugs": false,
        "psychiatric_issues_type": null,
    });

    let update_req = test_helpers::with_auth_headers(
        test::TestRequest::put()
            .uri(&format!("/api/v1/victims/{}", victim_id))
            .set_json(&update_payload),
        &config,
        &root_token,
    )
    .to_request();

    let update_resp = test::call_service(&app, update_req).await;
    assert_eq!(update_resp.status(), StatusCode::OK);

    let update_body: serde_json::Value = test::read_body_json(update_resp).await;
    assert_eq!(
        update_body["data"]["full_name"].as_str().unwrap(),
        "VITIMA UPDATE RENAMED"
    );
    assert!(update_body["data"]["addresses"].is_array());
    assert_eq!(
        update_body["data"]["addresses"].as_array().unwrap().len(),
        1
    );
    assert_eq!(
        update_body["data"]["addresses"][0]["street"]
            .as_str()
            .unwrap(),
        "Nova Rua"
    );
    assert_eq!(
        update_body["data"]["addresses"][0]["city_id"]
            .as_str()
            .unwrap(),
        city.to_string()
    );
}

#[sqlx::test]
async fn city_admin_cannot_create_victim_with_address_in_other_city(pool: PgPool) {

    let config = test_helpers::build_test_config();
    let app = test_helpers::create_full_test_app(pool.clone(), config.clone()).await;

    let city_a = db_fixtures::insert_city(&pool, "Cidade A").await;
    let city_b = db_fixtures::insert_city(&pool, "Cidade B").await;

    let admin_a_claims = test_helpers::build_city_admin_claims(city_a);
    let admin_a_token = test_helpers::generate_jwt(&admin_a_claims, &config.jwt_secret);

    // Try to create victim with address in city B
    let payload = build_victim_payload_with_address("Vitima", city_b);
    let req = test_helpers::with_auth_headers(
        test::TestRequest::post()
            .uri("/api/v1/victims")
            .set_json(&payload),
        &config,
        &admin_a_token,
    )
    .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), StatusCode::FORBIDDEN);
}
