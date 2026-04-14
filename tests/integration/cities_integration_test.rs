use actix_web::{http::StatusCode, test};

use crate::common::test_helpers;
use nupevid_api::core::entities::auth::ClaimsToUserToken;
use nupevid_api::core::value_objects::profiles::Profile;
use nupevid_api::core::value_objects::ranks::Rank;
use std::time::{SystemTime, UNIX_EPOCH};
use uuid::Uuid;

fn new_city_payload(name: &str) -> serde_json::Value {
    serde_json::json!({
        "name": name,
        "state": "RO",
        "battalion": "1ºBPM",
    })
}

#[actix_rt::test]
async fn non_root_cannot_create_city() {
    let pool = test_helpers::setup_test_db().await;
    test_helpers::clean_database(&pool).await;

    let config = test_helpers::build_test_config();
    let app = test_helpers::create_full_test_app(pool.clone(), config.clone()).await;

    // Criar uma cidade para obter um city_id e compor os tokens de não-ROOT
    let root_claims = test_helpers::build_root_claims();
    let root_token = test_helpers::generate_jwt(&root_claims, &config.jwt_secret);
    let base_city_payload = new_city_payload("PORTO VELHO");
    let base_req = test_helpers::with_auth_headers(
        test::TestRequest::post()
            .uri("/api/v1/cities")
            .set_json(&base_city_payload),
        &config,
        &root_token,
    )
    .to_request();
    let base_resp = test::call_service(&app, base_req).await;
    assert!(base_resp.status().is_success());
    let base_body: serde_json::Value = test::read_body_json(base_resp).await;
    let city_id: Uuid = base_body["data"]["id"].as_str().unwrap().parse().unwrap();

    // CITY_ADMIN tenta criar cidade -> FORBIDDEN
    let admin_claims = test_helpers::build_city_admin_claims(city_id);
    let admin_token = test_helpers::generate_jwt(&admin_claims, &config.jwt_secret);
    let admin_create_payload = new_city_payload("ARIQUEMES");
    let admin_req = test_helpers::with_auth_headers(
        test::TestRequest::post()
            .uri("/api/v1/cities")
            .set_json(&admin_create_payload),
        &config,
        &admin_token,
    )
    .to_request();
    let admin_resp = test::call_service(&app, admin_req).await;
    assert_eq!(admin_resp.status(), StatusCode::FORBIDDEN);

    // CITY_USER tenta criar cidade -> FORBIDDEN
    let claims_user = ClaimsToUserToken {
        id: Uuid::new_v4().to_string(),
        exp: (SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs() as usize)
            + 3600,
        rank: Rank::CbPm,
        registration: "100009999".to_string(),
        full_name: "Any User".to_string(),
        profile: Profile::CityUser,
        email: "any.user@test.com".to_string(),
        city_id: Some(city_id.to_string()),
    };
    let user_token = test_helpers::generate_jwt(&claims_user, &config.jwt_secret);
    let user_create_payload = new_city_payload("JI-PARANÁ");
    let user_req = test_helpers::with_auth_headers(
        test::TestRequest::post()
            .uri("/api/v1/cities")
            .set_json(&user_create_payload),
        &config,
        &user_token,
    )
    .to_request();
    let user_resp = test::call_service(&app, user_req).await;
    assert_eq!(user_resp.status(), StatusCode::FORBIDDEN);
}

#[actix_rt::test]
async fn city_admin_cannot_update_or_delete_cities() {
    let pool = test_helpers::setup_test_db().await;
    test_helpers::clean_database(&pool).await;

    let config = test_helpers::build_test_config();
    let app = test_helpers::create_full_test_app(pool.clone(), config.clone()).await;

    // Cria cidade como ROOT
    let root_claims = test_helpers::build_root_claims();
    let root_token = test_helpers::generate_jwt(&root_claims, &config.jwt_secret);
    let city_payload = new_city_payload("PORTO VELHO");
    let create_req = test_helpers::with_auth_headers(
        test::TestRequest::post()
            .uri("/api/v1/cities")
            .set_json(&city_payload),
        &config,
        &root_token,
    )
    .to_request();
    let create_resp = test::call_service(&app, create_req).await;
    assert!(create_resp.status().is_success());
    let created: serde_json::Value = test::read_body_json(create_resp).await;
    let city_id: Uuid = created["data"]["id"].as_str().unwrap().parse().unwrap();

    // CITY_ADMIN da própria cidade
    let admin_claims = test_helpers::build_city_admin_claims(city_id);
    let admin_token = test_helpers::generate_jwt(&admin_claims, &config.jwt_secret);

    // Tenta atualizar cidade - deve ser FORBIDDEN (apenas ROOT pode)
    let update_payload = serde_json::json!({
        "name": "PORTO VELHO",
        "state": "RO",
        "battalion": "2ºBPM",
    });
    let update_req = test_helpers::with_auth_headers(
        test::TestRequest::put()
            .uri(&format!("/api/v1/cities/{}", city_id))
            .set_json(&update_payload),
        &config,
        &admin_token,
    )
    .to_request();
    let update_resp = test::call_service(&app, update_req).await;
    assert_eq!(update_resp.status(), StatusCode::FORBIDDEN);

    // Tenta deletar cidade - deve ser FORBIDDEN (apenas ROOT pode)
    let delete_req = test_helpers::with_auth_headers(
        test::TestRequest::delete().uri(&format!("/api/v1/cities/{}", city_id)),
        &config,
        &admin_token,
    )
    .to_request();
    let delete_resp = test::call_service(&app, delete_req).await;
    assert_eq!(delete_resp.status(), StatusCode::FORBIDDEN);
}

#[actix_rt::test]
async fn city_admin_cannot_update_or_delete_any_city() {
    let pool = test_helpers::setup_test_db().await;
    test_helpers::clean_database(&pool).await;

    let config = test_helpers::build_test_config();
    let app = test_helpers::create_full_test_app(pool.clone(), config.clone()).await;

    // Cria duas cidades como ROOT
    let root_claims = test_helpers::build_root_claims();
    let root_token = test_helpers::generate_jwt(&root_claims, &config.jwt_secret);
    let city_a_payload = new_city_payload("PORTO VELHO");
    let city_b_payload = new_city_payload("ARIQUEMES");
    let create_a = test_helpers::with_auth_headers(
        test::TestRequest::post()
            .uri("/api/v1/cities")
            .set_json(&city_a_payload),
        &config,
        &root_token,
    )
    .to_request();
    let resp_a = test::call_service(&app, create_a).await;
    let body_a: serde_json::Value = test::read_body_json(resp_a).await;
    let city_a: Uuid = body_a["data"]["id"].as_str().unwrap().parse().unwrap();
    let create_b = test_helpers::with_auth_headers(
        test::TestRequest::post()
            .uri("/api/v1/cities")
            .set_json(&city_b_payload),
        &config,
        &root_token,
    )
    .to_request();
    let resp_b = test::call_service(&app, create_b).await;
    let body_b: serde_json::Value = test::read_body_json(resp_b).await;
    let city_b: Uuid = body_b["data"]["id"].as_str().unwrap().parse().unwrap();

    // CITY_ADMIN de city_a tentando alterar/excluir city_b (deve falhar - apenas ROOT pode)
    let admin_a_claims = test_helpers::build_city_admin_claims(city_a);
    let admin_a_token = test_helpers::generate_jwt(&admin_a_claims, &config.jwt_secret);

    let update_payload = serde_json::json!({
        "name": "ARIQUEMES",
        "state": "RO",
        "battalion": "2ºBPM",
    });

    let update_req = test_helpers::with_auth_headers(
        test::TestRequest::put()
            .uri(&format!("/api/v1/cities/{}", city_b))
            .set_json(&update_payload),
        &config,
        &admin_a_token,
    )
    .to_request();
    let update_resp = test::call_service(&app, update_req).await;
    assert_eq!(update_resp.status(), StatusCode::FORBIDDEN);

    let delete_req = test_helpers::with_auth_headers(
        test::TestRequest::delete().uri(&format!("/api/v1/cities/{}", city_b)),
        &config,
        &admin_a_token,
    )
    .to_request();
    let delete_resp = test::call_service(&app, delete_req).await;
    assert_eq!(delete_resp.status(), StatusCode::FORBIDDEN);
}

#[actix_rt::test]
async fn create_city_success() {
    let pool = test_helpers::setup_test_db().await;
    test_helpers::clean_database(&pool).await;

    let config = test_helpers::build_test_config();
    let app = test_helpers::create_full_test_app(pool.clone(), config.clone()).await;

    let claims = test_helpers::build_root_claims();
    let token = test_helpers::generate_jwt(&claims, &config.jwt_secret);

    let payload = new_city_payload("PORTO VELHO");

    let req = test_helpers::with_auth_headers(
        test::TestRequest::post()
            .uri("/api/v1/cities")
            .set_json(&payload),
        &config,
        &token,
    )
    .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), StatusCode::CREATED);

    let body: serde_json::Value = test::read_body_json(resp).await;
    assert_eq!(body["status"].as_u64().unwrap(), 201);
    assert_eq!(body["data"]["name"].as_str().unwrap(), "PORTO VELHO");
}

#[actix_rt::test]
async fn create_city_invalid_state_returns_bad_request() {
    let pool = test_helpers::setup_test_db().await;
    test_helpers::clean_database(&pool).await;

    let config = test_helpers::build_test_config();
    let app = test_helpers::create_full_test_app(pool.clone(), config.clone()).await;

    let claims = test_helpers::build_root_claims();
    let token = test_helpers::generate_jwt(&claims, &config.jwt_secret);

    let payload = serde_json::json!({
        "name": "PORTO VELHO",
        "state": "SP", // invalid state
        "battalion": "1ºBPM",
    });

    let req = test_helpers::with_auth_headers(
        test::TestRequest::post()
            .uri("/api/v1/cities")
            .set_json(&payload),
        &config,
        &token,
    )
    .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), StatusCode::BAD_REQUEST);

    let body: serde_json::Value = test::read_body_json(resp).await;
    assert_eq!(body["status_code"].as_u64().unwrap(), 400);
    assert!(body["message"].as_str().unwrap().contains("invalid state"));
}

#[actix_rt::test]
async fn create_city_invalid_name_returns_bad_request() {
    let pool = test_helpers::setup_test_db().await;
    test_helpers::clean_database(&pool).await;

    let config = test_helpers::build_test_config();
    let app = test_helpers::create_full_test_app(pool.clone(), config.clone()).await;

    let claims = test_helpers::build_root_claims();
    let token = test_helpers::generate_jwt(&claims, &config.jwt_secret);

    let payload = serde_json::json!({
        "name": "Cidade Inexistente",
        "state": "RO",
        "battalion": "1ºBPM",
    });

    let req = test_helpers::with_auth_headers(
        test::TestRequest::post()
            .uri("/api/v1/cities")
            .set_json(&payload),
        &config,
        &token,
    )
    .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), StatusCode::BAD_REQUEST);

    let body: serde_json::Value = test::read_body_json(resp).await;
    assert_eq!(body["status_code"].as_u64().unwrap(), 400);
    assert!(
        body["message"]
            .as_str()
            .unwrap()
            .contains("invalid city name")
    );
}

#[actix_rt::test]
async fn create_city_invalid_battalion_returns_bad_request() {
    let pool = test_helpers::setup_test_db().await;
    test_helpers::clean_database(&pool).await;

    let config = test_helpers::build_test_config();
    let app = test_helpers::create_full_test_app(pool.clone(), config.clone()).await;

    let claims = test_helpers::build_root_claims();
    let token = test_helpers::generate_jwt(&claims, &config.jwt_secret);

    let payload = serde_json::json!({
        "name": "PORTO VELHO",
        "state": "RO",
        "battalion": "99ºBPM", // invalid battalion
    });

    let req = test_helpers::with_auth_headers(
        test::TestRequest::post()
            .uri("/api/v1/cities")
            .set_json(&payload),
        &config,
        &token,
    )
    .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), StatusCode::BAD_REQUEST);

    let body: serde_json::Value = test::read_body_json(resp).await;
    assert_eq!(body["status_code"].as_u64().unwrap(), 400);
    assert!(
        body["message"]
            .as_str()
            .unwrap()
            .contains("invalid battalion")
    );
}

#[actix_rt::test]
async fn create_city_missing_fields_returns_bad_request() {
    let pool = test_helpers::setup_test_db().await;
    test_helpers::clean_database(&pool).await;

    let config = test_helpers::build_test_config();
    let app = test_helpers::create_full_test_app(pool.clone(), config.clone()).await;

    let claims = test_helpers::build_root_claims();
    let token = test_helpers::generate_jwt(&claims, &config.jwt_secret);

    let payload = serde_json::json!({
        "name": "",
        "state": "",
        "battalion": "",
    });

    let req = test_helpers::with_auth_headers(
        test::TestRequest::post()
            .uri("/api/v1/cities")
            .set_json(&payload),
        &config,
        &token,
    )
    .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), StatusCode::BAD_REQUEST);

    let body: serde_json::Value = test::read_body_json(resp).await;
    assert_eq!(body["status_code"].as_u64().unwrap(), 400);
    assert!(
        body["message"]
            .as_str()
            .unwrap()
            .contains("cannot be empty")
    );
}

#[actix_rt::test]
async fn get_all_cities_empty_returns_empty_list() {
    let pool = test_helpers::setup_test_db().await;
    test_helpers::clean_database(&pool).await;

    let config = test_helpers::build_test_config();
    let app = test_helpers::create_full_test_app(pool.clone(), config.clone()).await;

    let claims = test_helpers::build_root_claims();
    let token = test_helpers::generate_jwt(&claims, &config.jwt_secret);

    let req = test_helpers::with_auth_headers(
        test::TestRequest::get().uri("/api/v1/cities"),
        &config,
        &token,
    )
    .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), StatusCode::OK);

    let body: serde_json::Value = test::read_body_json(resp).await;
    assert_eq!(body["status"].as_u64().unwrap(), 200);
    assert_eq!(body["data"].as_array().unwrap().len(), 0);
}

#[actix_rt::test]
async fn get_all_cities_with_data_returns_list() {
    let pool = test_helpers::setup_test_db().await;
    test_helpers::clean_database(&pool).await;

    let config = test_helpers::build_test_config();
    let app = test_helpers::create_full_test_app(pool.clone(), config.clone()).await;

    let claims = test_helpers::build_root_claims();
    let token = test_helpers::generate_jwt(&claims, &config.jwt_secret);

    // create two cities
    for name in ["PORTO VELHO", "ARIQUEMES"] {
        let payload = new_city_payload(name);
        let req = test_helpers::with_auth_headers(
            test::TestRequest::post()
                .uri("/api/v1/cities")
                .set_json(&payload),
            &config,
            &token,
        )
        .to_request();
        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status(), StatusCode::CREATED);
    }

    let req = test_helpers::with_auth_headers(
        test::TestRequest::get().uri("/api/v1/cities"),
        &config,
        &token,
    )
    .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), StatusCode::OK);

    let body: serde_json::Value = test::read_body_json(resp).await;
    assert_eq!(body["status"].as_u64().unwrap(), 200);
    assert_eq!(body["data"].as_array().unwrap().len(), 2);
}

#[actix_rt::test]
async fn get_city_by_id_success() {
    let pool = test_helpers::setup_test_db().await;
    test_helpers::clean_database(&pool).await;

    let config = test_helpers::build_test_config();
    let app = test_helpers::create_full_test_app(pool.clone(), config.clone()).await;

    let claims = test_helpers::build_root_claims();
    let token = test_helpers::generate_jwt(&claims, &config.jwt_secret);

    let payload = new_city_payload("CACOAL");
    let create_req = test_helpers::with_auth_headers(
        test::TestRequest::post()
            .uri("/api/v1/cities")
            .set_json(&payload),
        &config,
        &token,
    )
    .to_request();

    let create_resp = test::call_service(&app, create_req).await;
    assert_eq!(create_resp.status(), StatusCode::CREATED);
    let create_body: serde_json::Value = test::read_body_json(create_resp).await;
    let city_id = create_body["data"]["id"].as_str().unwrap();

    let get_req = test_helpers::with_auth_headers(
        test::TestRequest::get().uri(&format!("/api/v1/cities/{}", city_id)),
        &config,
        &token,
    )
    .to_request();

    let get_resp = test::call_service(&app, get_req).await;
    assert_eq!(get_resp.status(), StatusCode::OK);

    let get_body: serde_json::Value = test::read_body_json(get_resp).await;
    assert_eq!(get_body["status"].as_u64().unwrap(), 200);
    assert_eq!(get_body["data"]["id"].as_str().unwrap(), city_id);
}

#[actix_rt::test]
async fn get_city_by_id_not_found_returns_404() {
    let pool = test_helpers::setup_test_db().await;
    test_helpers::clean_database(&pool).await;

    let config = test_helpers::build_test_config();
    let app = test_helpers::create_full_test_app(pool.clone(), config.clone()).await;

    let claims = test_helpers::build_root_claims();
    let token = test_helpers::generate_jwt(&claims, &config.jwt_secret);

    let random_id = uuid::Uuid::new_v4();
    let req = test_helpers::with_auth_headers(
        test::TestRequest::get().uri(&format!("/api/v1/cities/{}", random_id)),
        &config,
        &token,
    )
    .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), StatusCode::NOT_FOUND);

    let body: serde_json::Value = test::read_body_json(resp).await;
    assert_eq!(body["status_code"].as_u64().unwrap(), 404);
    assert!(body["message"].as_str().unwrap().contains("not found"));
}

#[actix_rt::test]
async fn update_city_success() {
    let pool = test_helpers::setup_test_db().await;
    test_helpers::clean_database(&pool).await;

    let config = test_helpers::build_test_config();
    let app = test_helpers::create_full_test_app(pool.clone(), config.clone()).await;

    let claims = test_helpers::build_root_claims();
    let token = test_helpers::generate_jwt(&claims, &config.jwt_secret);

    let payload = new_city_payload("VILHENA");
    let create_req = test_helpers::with_auth_headers(
        test::TestRequest::post()
            .uri("/api/v1/cities")
            .set_json(&payload),
        &config,
        &token,
    )
    .to_request();

    let create_resp = test::call_service(&app, create_req).await;
    assert_eq!(create_resp.status(), StatusCode::CREATED);
    let create_body: serde_json::Value = test::read_body_json(create_resp).await;
    let city_id = create_body["data"]["id"].as_str().unwrap();

    let update_payload = serde_json::json!({
        "name": "JI-PARANÁ",
        "state": "RO",
        "battalion": "2ºBPM",
    });

    let update_req = test_helpers::with_auth_headers(
        test::TestRequest::put()
            .uri(&format!("/api/v1/cities/{}", city_id))
            .set_json(&update_payload),
        &config,
        &token,
    )
    .to_request();

    let update_resp = test::call_service(&app, update_req).await;
    assert_eq!(update_resp.status(), StatusCode::OK);

    let update_body: serde_json::Value = test::read_body_json(update_resp).await;
    assert_eq!(update_body["status"].as_u64().unwrap(), 200);
    assert_eq!(update_body["data"]["name"].as_str().unwrap(), "JI-PARANÁ");
}

#[actix_rt::test]
async fn update_city_invalid_state_returns_bad_request() {
    let pool = test_helpers::setup_test_db().await;
    test_helpers::clean_database(&pool).await;

    let config = test_helpers::build_test_config();
    let app = test_helpers::create_full_test_app(pool.clone(), config.clone()).await;

    let claims = test_helpers::build_root_claims();
    let token = test_helpers::generate_jwt(&claims, &config.jwt_secret);

    let payload = new_city_payload("JARU");
    let create_req = test_helpers::with_auth_headers(
        test::TestRequest::post()
            .uri("/api/v1/cities")
            .set_json(&payload),
        &config,
        &token,
    )
    .to_request();

    let create_resp = test::call_service(&app, create_req).await;
    assert_eq!(create_resp.status(), StatusCode::CREATED);
    let create_body: serde_json::Value = test::read_body_json(create_resp).await;
    let city_id = create_body["data"]["id"].as_str().unwrap();

    let update_payload = serde_json::json!({
        "name": "JARU",
        "state": "SP", // invalid state
        "battalion": "1ºBPM",
    });

    let update_req = test_helpers::with_auth_headers(
        test::TestRequest::put()
            .uri(&format!("/api/v1/cities/{}", city_id))
            .set_json(&update_payload),
        &config,
        &token,
    )
    .to_request();

    let update_resp = test::call_service(&app, update_req).await;
    assert_eq!(update_resp.status(), StatusCode::BAD_REQUEST);

    let update_body: serde_json::Value = test::read_body_json(update_resp).await;
    assert_eq!(update_body["status_code"].as_u64().unwrap(), 400);
    assert!(
        update_body["message"]
            .as_str()
            .unwrap()
            .contains("invalid state")
    );
}

#[actix_rt::test]
async fn update_city_not_found_returns_404() {
    let pool = test_helpers::setup_test_db().await;
    test_helpers::clean_database(&pool).await;

    let config = test_helpers::build_test_config();
    let app = test_helpers::create_full_test_app(pool.clone(), config.clone()).await;

    let claims = test_helpers::build_root_claims();
    let token = test_helpers::generate_jwt(&claims, &config.jwt_secret);

    let update_payload = new_city_payload("BURITIS");

    let random_id = uuid::Uuid::new_v4();
    let update_req = test_helpers::with_auth_headers(
        test::TestRequest::put()
            .uri(&format!("/api/v1/cities/{}", random_id))
            .set_json(&update_payload),
        &config,
        &token,
    )
    .to_request();

    let update_resp = test::call_service(&app, update_req).await;
    assert_eq!(update_resp.status(), StatusCode::NOT_FOUND);

    let update_body: serde_json::Value = test::read_body_json(update_resp).await;
    assert_eq!(update_body["status_code"].as_u64().unwrap(), 404);
    assert!(
        update_body["message"]
            .as_str()
            .unwrap()
            .contains("not found")
    );
}

#[actix_rt::test]
async fn delete_city_success_performs_soft_delete() {
    let pool = test_helpers::setup_test_db().await;
    test_helpers::clean_database(&pool).await;

    let config = test_helpers::build_test_config();
    let app = test_helpers::create_full_test_app(pool.clone(), config.clone()).await;

    let claims = test_helpers::build_root_claims();
    let token = test_helpers::generate_jwt(&claims, &config.jwt_secret);

    let payload = new_city_payload("CEREJEIRAS");
    let create_req = test_helpers::with_auth_headers(
        test::TestRequest::post()
            .uri("/api/v1/cities")
            .set_json(&payload),
        &config,
        &token,
    )
    .to_request();

    let create_resp = test::call_service(&app, create_req).await;
    assert_eq!(create_resp.status(), StatusCode::CREATED);
    let create_body: serde_json::Value = test::read_body_json(create_resp).await;
    let city_id = create_body["data"]["id"].as_str().unwrap();

    let delete_req = test_helpers::with_auth_headers(
        test::TestRequest::delete().uri(&format!("/api/v1/cities/{}", city_id)),
        &config,
        &token,
    )
    .to_request();

    let delete_resp = test::call_service(&app, delete_req).await;
    assert_eq!(delete_resp.status(), StatusCode::OK);

    // City should not appear in list anymore
    let list_req = test_helpers::with_auth_headers(
        test::TestRequest::get().uri("/api/v1/cities"),
        &config,
        &token,
    )
    .to_request();

    let list_resp = test::call_service(&app, list_req).await;
    assert_eq!(list_resp.status(), StatusCode::OK);
    let list_body: serde_json::Value = test::read_body_json(list_resp).await;
    let cities = list_body["data"].as_array().unwrap();
    assert!(cities.iter().all(|c| c["id"].as_str().unwrap() != city_id));
}

#[actix_rt::test]
async fn delete_city_not_found_returns_404() {
    let pool = test_helpers::setup_test_db().await;
    test_helpers::clean_database(&pool).await;

    let config = test_helpers::build_test_config();
    let app = test_helpers::create_full_test_app(pool.clone(), config.clone()).await;

    let claims = test_helpers::build_root_claims();
    let token = test_helpers::generate_jwt(&claims, &config.jwt_secret);

    let random_id = uuid::Uuid::new_v4();
    let delete_req = test_helpers::with_auth_headers(
        test::TestRequest::delete().uri(&format!("/api/v1/cities/{}", random_id)),
        &config,
        &token,
    )
    .to_request();

    let delete_resp = test::call_service(&app, delete_req).await;
    assert_eq!(delete_resp.status(), StatusCode::NOT_FOUND);

    let delete_body: serde_json::Value = test::read_body_json(delete_resp).await;
    assert_eq!(delete_body["status_code"].as_u64().unwrap(), 404);
    assert!(
        delete_body["message"]
            .as_str()
            .unwrap()
            .contains("not found")
    );
}

#[actix_rt::test]
async fn create_duplicate_city_returns_bad_request() {
    let pool = test_helpers::setup_test_db().await;
    test_helpers::clean_database(&pool).await;

    let config = test_helpers::build_test_config();
    let app = test_helpers::create_full_test_app(pool.clone(), config.clone()).await;

    let claims = test_helpers::build_root_claims();
    let token = test_helpers::generate_jwt(&claims, &config.jwt_secret);

    // Create first city
    let payload = new_city_payload("PORTO VELHO");
    let req1 = test_helpers::with_auth_headers(
        test::TestRequest::post()
            .uri("/api/v1/cities")
            .set_json(&payload),
        &config,
        &token,
    )
    .to_request();

    let resp1 = test::call_service(&app, req1).await;
    assert_eq!(resp1.status(), StatusCode::CREATED);

    // Try to create the same city again (same name and battalion)
    let req2 = test_helpers::with_auth_headers(
        test::TestRequest::post()
            .uri("/api/v1/cities")
            .set_json(&payload),
        &config,
        &token,
    )
    .to_request();

    let resp2 = test::call_service(&app, req2).await;
    assert_eq!(resp2.status(), StatusCode::BAD_REQUEST);

    let body: serde_json::Value = test::read_body_json(resp2).await;
    assert_eq!(body["status_code"].as_u64().unwrap(), 400);
    assert!(body["message"].as_str().unwrap().contains("already exists"));
    assert!(body["message"].as_str().unwrap().contains("PORTO VELHO"));
    assert!(body["message"].as_str().unwrap().contains("1ºBPM"));
}

#[actix_rt::test]
async fn update_city_to_duplicate_name_and_battalion_returns_bad_request() {
    let pool = test_helpers::setup_test_db().await;
    test_helpers::clean_database(&pool).await;

    let config = test_helpers::build_test_config();
    let app = test_helpers::create_full_test_app(pool.clone(), config.clone()).await;

    let claims = test_helpers::build_root_claims();
    let token = test_helpers::generate_jwt(&claims, &config.jwt_secret);

    // Create first city
    let payload1 = new_city_payload("PORTO VELHO");
    let req1 = test_helpers::with_auth_headers(
        test::TestRequest::post()
            .uri("/api/v1/cities")
            .set_json(&payload1),
        &config,
        &token,
    )
    .to_request();

    let resp1 = test::call_service(&app, req1).await;
    assert_eq!(resp1.status(), StatusCode::CREATED);

    // Create second city with different name but same battalion
    let payload2 = serde_json::json!({
        "name": "ARIQUEMES",
        "state": "RO",
        "battalion": "1ºBPM",
    });
    let req2 = test_helpers::with_auth_headers(
        test::TestRequest::post()
            .uri("/api/v1/cities")
            .set_json(&payload2),
        &config,
        &token,
    )
    .to_request();

    let resp2 = test::call_service(&app, req2).await;
    assert_eq!(resp2.status(), StatusCode::CREATED);
    let body2: serde_json::Value = test::read_body_json(resp2).await;
    let city2_id = body2["data"]["id"].as_str().unwrap();

    // Try to update second city to have the same name and battalion as first city
    let update_payload = new_city_payload("PORTO VELHO");
    let update_req = test_helpers::with_auth_headers(
        test::TestRequest::put()
            .uri(&format!("/api/v1/cities/{}", city2_id))
            .set_json(&update_payload),
        &config,
        &token,
    )
    .to_request();

    let update_resp = test::call_service(&app, update_req).await;
    assert_eq!(update_resp.status(), StatusCode::BAD_REQUEST);

    let update_body: serde_json::Value = test::read_body_json(update_resp).await;
    assert_eq!(update_body["status_code"].as_u64().unwrap(), 400);
    assert!(
        update_body["message"]
            .as_str()
            .unwrap()
            .contains("already exists")
    );
    assert!(
        update_body["message"]
            .as_str()
            .unwrap()
            .contains("PORTO VELHO")
    );
    assert!(update_body["message"].as_str().unwrap().contains("1ºBPM"));
}

#[actix_rt::test]
async fn update_city_to_same_name_and_battalion_succeeds() {
    let pool = test_helpers::setup_test_db().await;
    test_helpers::clean_database(&pool).await;

    let config = test_helpers::build_test_config();
    let app = test_helpers::create_full_test_app(pool.clone(), config.clone()).await;

    let claims = test_helpers::build_root_claims();
    let token = test_helpers::generate_jwt(&claims, &config.jwt_secret);

    // Create a city
    let payload = new_city_payload("PORTO VELHO");
    let req = test_helpers::with_auth_headers(
        test::TestRequest::post()
            .uri("/api/v1/cities")
            .set_json(&payload),
        &config,
        &token,
    )
    .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), StatusCode::CREATED);
    let body: serde_json::Value = test::read_body_json(resp).await;
    let city_id = body["data"]["id"].as_str().unwrap();

    // Update the city with the same name and battalion (should succeed)
    let update_payload = new_city_payload("PORTO VELHO");
    let update_req = test_helpers::with_auth_headers(
        test::TestRequest::put()
            .uri(&format!("/api/v1/cities/{}", city_id))
            .set_json(&update_payload),
        &config,
        &token,
    )
    .to_request();

    let update_resp = test::call_service(&app, update_req).await;
    assert_eq!(update_resp.status(), StatusCode::OK);

    let update_body: serde_json::Value = test::read_body_json(update_resp).await;
    assert_eq!(update_body["status"].as_u64().unwrap(), 200);
    assert_eq!(update_body["data"]["name"].as_str().unwrap(), "PORTO VELHO");
}
