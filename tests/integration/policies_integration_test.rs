use actix_web::{http::StatusCode, test};
use serde_json::json;
use uuid::Uuid;

use crate::common::{db_fixtures, test_helpers};

// Helper: build JWT for a persisted user
fn build_token_for_user(id: Uuid, profile: &str, email: &str, full_name: &str, registration: &str, rank: &str, city_id: Option<Uuid>, jwt_secret: &str) -> String {
    let claims = nupevid_api::core::entities::auth::ClaimsToUserToken {
        id: id.to_string(),
        exp: {
            use std::time::{SystemTime, UNIX_EPOCH};
            (SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs() as usize) + 3600
        },
        rank: rank.to_string(),
        registration: registration.to_string(),
        full_name: full_name.to_string(),
        profile: profile.to_string(),
        email: email.to_string(),
        city_id: city_id.map(|c| c.to_string()),
    };
    test_helpers::generate_jwt(&claims, jwt_secret)
}

#[actix_rt::test]
async fn root_cannot_assign_city_management_policies() {
    let pool = test_helpers::setup_test_db().await;
    test_helpers::clean_database(&pool).await;

    let config = test_helpers::build_test_config();
    let app = test_helpers::create_full_test_app(pool.clone(), config.clone()).await;

    // Cria cidade e usuário alvo
    let city = db_fixtures::insert_city(&pool, "PORTO VELHO").await;
    let root_token = test_helpers::generate_jwt(&test_helpers::build_root_claims(), &config.jwt_secret);

    let user_payload = json!({
        "rank": "CB PM",
        "registration": "900000001",
        "full_name": "Target User",
        "profile": "CITY_USER",
        "email": "target.assign.cities@test.com",
        "password": "Secret123!",
        "city_id": city,
    });
    let create_user = test_helpers::with_auth_headers(
        test::TestRequest::post().uri("/api/v1/users").set_json(&user_payload),
        &config, &root_token).to_request();
    let user_resp = test::call_service(&app, create_user).await;
    let user_body: serde_json::Value = test::read_body_json(user_resp).await;
    let user_id: Uuid = user_body["data"]["id"].as_str().unwrap().parse().unwrap();

    // Tenta conceder políticas de gestão de cidades (não atribuíveis)
    for p in ["create_cities", "update_cities", "delete_cities"] {        
        let append_payload = json!({ "city_ids": [city] });
        let append_req = test_helpers::with_auth_headers(
            test::TestRequest::post().uri(&format!("/api/v1/users/{}/policies/{}/cities", user_id, p))
                .set_json(&append_payload),
            &config, &root_token).to_request();
        let append_resp = test::call_service(&app, append_req).await;
        assert_eq!(append_resp.status(), StatusCode::BAD_REQUEST);
    }
}

#[actix_rt::test]
async fn root_cannot_remove_city_management_policies() {
    let pool = test_helpers::setup_test_db().await;
    test_helpers::clean_database(&pool).await;

    let config = test_helpers::build_test_config();
    let app = test_helpers::create_full_test_app(pool.clone(), config.clone()).await;

    // Cria cidade e usuário alvo
    let city = db_fixtures::insert_city(&pool, "ARIQUEMES").await;
    let root_token = test_helpers::generate_jwt(&test_helpers::build_root_claims(), &config.jwt_secret);

    let user_payload = json!({
        "rank": "CB PM",
        "registration": "900000002",
        "full_name": "Target User 2",
        "profile": "CITY_USER",
        "email": "target.remove.cities@test.com",
        "password": "Secret123!",
        "city_id": city,
    });
    let create_user = test_helpers::with_auth_headers(
        test::TestRequest::post().uri("/api/v1/users").set_json(&user_payload),
        &config, &root_token).to_request();
    let user_resp = test::call_service(&app, create_user).await;
    let user_body: serde_json::Value = test::read_body_json(user_resp).await;
    let user_id: Uuid = user_body["data"]["id"].as_str().unwrap().parse().unwrap();

    // Tenta remover políticas de gestão de cidades (também não removíveis via endpoint)
    for p in ["create_cities", "update_cities", "delete_cities"] {
        let remove_payload = json!({ "city_ids": [city] });
        let remove_req = test_helpers::with_auth_headers(
            test::TestRequest::delete().uri(&format!("/api/v1/users/{}/policies/{}/cities", user_id, p))
                .set_json(&remove_payload),
            &config, &root_token).to_request();
        let remove_resp = test::call_service(&app, remove_req).await;
        assert_eq!(remove_resp.status(), StatusCode::BAD_REQUEST);
    }
}

#[actix_rt::test]
async fn root_can_grant_extra_read_victims_to_city_admin_and_admin_assigns_to_city_user() {
    let pool = test_helpers::setup_test_db().await;
    test_helpers::clean_database(&pool).await;

    let config = test_helpers::build_test_config();
    let app = test_helpers::create_full_test_app(pool.clone(), config.clone()).await;

    // Cidades A e B
    let city_a = db_fixtures::insert_city(&pool, "PORTO VELHO").await;
    let city_b = db_fixtures::insert_city(&pool, "ARIQUEMES").await;

    // ROOT token
    let root_claims = test_helpers::build_root_claims();
    let root_token = test_helpers::generate_jwt(&root_claims, &config.jwt_secret);

    // Cria CITY_ADMIN A
    let admin_payload = json!({
        "rank": "CAP PM",
        "registration": "100000101",
        "full_name": "Admin A",
        "profile": "CITY_ADMIN",
        "email": "admin.a@test.com",
        "password": "Secret123!",
        "city_id": city_a,
    });
    let create_admin_req = test_helpers::with_auth_headers(
        test::TestRequest::post().uri("/api/v1/users").set_json(&admin_payload),
        &config, &root_token).to_request();
    let create_admin_resp = test::call_service(&app, create_admin_req).await;
    assert_eq!(create_admin_resp.status(), StatusCode::CREATED);
    let admin_body: serde_json::Value = test::read_body_json(create_admin_resp).await;
    let admin_id: Uuid = admin_body["data"]["id"].as_str().unwrap().parse().unwrap();

    // Atualiza policies do ADMIN: add read_victims em city_b (mantendo city_a)
    // Primeiro obter policies atuais via GET /users/{id}
    let get_admin_req = test_helpers::with_auth_headers(
        test::TestRequest::get().uri(&format!("/api/v1/users/{}", admin_id)),
        &config, &root_token).to_request();
    let get_admin_resp = test::call_service(&app, get_admin_req).await;
    assert_eq!(get_admin_resp.status(), StatusCode::OK);
    let get_admin_body: serde_json::Value = test::read_body_json(get_admin_resp).await;
    let mut policies = get_admin_body["data"]["permission_policies"].clone();
    // garantir array para read_victims
    {
        let arr = policies["read_victims"].as_array().cloned().unwrap_or_else(|| vec![json!(city_a.to_string())]);
        let mut new_arr = arr.clone();
        if !new_arr.iter().any(|v| v.as_str() == Some(&city_b.to_string())) {
            new_arr.push(json!(city_b.to_string()));
        }
        policies["read_victims"] = json!(new_arr);
    }
    let update_admin_payload = json!({
        "rank": "CAP PM",
        "registration": "100000101",
        "full_name": "Admin A",
        "profile": "CITY_ADMIN",
        "email": "admin.a@test.com",
        "city_id": city_a,
        "permission_policies": policies,
    });
    let update_admin_req = test_helpers::with_auth_headers(
        test::TestRequest::put().uri(&format!("/api/v1/users/{}", admin_id)).set_json(&update_admin_payload),
        &config, &root_token).to_request();
    let update_admin_resp = test::call_service(&app, update_admin_req).await;
    assert_eq!(update_admin_resp.status(), StatusCode::OK);

    // Token do ADMIN com id persistido
    let admin_token = build_token_for_user(admin_id, "CITY_ADMIN", "admin.a@test.com", "Admin A", "100000101", "CAP PM", Some(city_a), &config.jwt_secret);

    // ADMIN cria CITY_USER e atribui read_victims em city_b
    let city_user_payload = json!({
        "rank": "CB PM",
        "registration": "100000201",
        "full_name": "User A",
        "profile": "CITY_USER",
        "email": "user.a@test.com",
        "password": "Secret123!",
        "permission_policies": {
            "read_victims": [city_b]
        }
    });
    let create_user_req = test_helpers::with_auth_headers(
        test::TestRequest::post().uri("/api/v1/users").set_json(&city_user_payload),
        &config, &admin_token).to_request();
    let create_user_resp = test::call_service(&app, create_user_req).await;
    assert_eq!(create_user_resp.status(), StatusCode::CREATED);
    let user_body: serde_json::Value = test::read_body_json(create_user_resp).await;
    let user_id: Uuid = user_body["data"]["id"].as_str().unwrap().parse().unwrap();

    // Cria vítima em city_b
    let victim_payload = json!({
        "full_name": "Vitima B",
        "cpf": serde_json::Value::Null,
        "birth_date": serde_json::Value::Null,
        "phone": serde_json::Value::Null,
        "city_id": city_b,
        "address": serde_json::Value::Null,
    });
    let create_victim_req = test_helpers::with_auth_headers(
        test::TestRequest::post().uri("/api/v1/victims").set_json(&victim_payload),
        &config, &root_token).to_request();
    let create_victim_resp = test::call_service(&app, create_victim_req).await;
    assert_eq!(create_victim_resp.status(), StatusCode::CREATED);

    // Token do CITY_USER persistido
    let user_token = build_token_for_user(user_id, "CITY_USER", "user.a@test.com", "User A", "100000201", "CB PM", Some(city_a), &config.jwt_secret);

    // CITY_USER lista vítimas -> deve ver vítima de city_b por extra policy
    let list_req = test_helpers::with_auth_headers(test::TestRequest::get().uri("/api/v1/victims"), &config, &user_token).to_request();
    let list_resp = test::call_service(&app, list_req).await;
    assert_eq!(list_resp.status(), StatusCode::OK);
    let list_body: serde_json::Value = test::read_body_json(list_resp).await;
    assert!(list_body["data"].as_array().unwrap().iter().any(|v| v["city_id"].as_str().unwrap() == city_b.to_string()));
}

#[actix_rt::test]
async fn city_admin_cannot_assign_policy_not_possessed() {
    let pool = test_helpers::setup_test_db().await;
    test_helpers::clean_database(&pool).await;
    let config = test_helpers::build_test_config();
    let app = test_helpers::create_full_test_app(pool.clone(), config.clone()).await;

    let city_a = db_fixtures::insert_city(&pool, "PORTO VELHO").await;
    let city_b = db_fixtures::insert_city(&pool, "ARIQUEMES").await;
    let root_token = test_helpers::generate_jwt(&test_helpers::build_root_claims(), &config.jwt_secret);

    // Cria ADMIN A
    let admin_payload = json!({
        "rank": "CAP PM",
        "registration": "100000301",
        "full_name": "Admin B",
        "profile": "CITY_ADMIN",
        "email": "admin.b@test.com",
        "password": "Secret123!",
        "city_id": city_a,
    });
    let create_admin_req = test_helpers::with_auth_headers(
        test::TestRequest::post().uri("/api/v1/users").set_json(&admin_payload),
        &config, &root_token).to_request();
    let admin_resp = test::call_service(&app, create_admin_req).await;
    let admin_body: serde_json::Value = test::read_body_json(admin_resp).await;
    let admin_id: Uuid = admin_body["data"]["id"].as_str().unwrap().parse().unwrap();
    let admin_token = build_token_for_user(admin_id, "CITY_ADMIN", "admin.b@test.com", "Admin B", "100000301", "CAP PM", Some(city_a), &config.jwt_secret);

    // ADMIN tenta criar CITY_USER com create_victims em city_b (não possui)
    let user_payload = json!({
        "rank": "CB PM",
        "registration": "100000302",
        "full_name": "User B",
        "profile": "CITY_USER",
        "email": "user.b@test.com",
        "password": "Secret123!",
        "permission_policies": {
            "create_victims": [city_b]
        }
    });
    let create_user_req = test_helpers::with_auth_headers(
        test::TestRequest::post().uri("/api/v1/users").set_json(&user_payload),
        &config, &admin_token).to_request();
    let resp = test::call_service(&app, create_user_req).await;
    assert_eq!(resp.status(), StatusCode::FORBIDDEN);
}

#[actix_rt::test]
async fn city_user_cannot_assign_policies() {
    let pool = test_helpers::setup_test_db().await;
    test_helpers::clean_database(&pool).await;
    let config = test_helpers::build_test_config();
    let app = test_helpers::create_full_test_app(pool.clone(), config.clone()).await;

    let city = db_fixtures::insert_city(&pool, "PORTO VELHO").await;
    let root_token = test_helpers::generate_jwt(&test_helpers::build_root_claims(), &config.jwt_secret);

    // Cria CITY_USER
    let user_payload = json!({
        "rank": "CB PM",
        "registration": "100000401",
        "full_name": "User C",
        "profile": "CITY_USER",
        "email": "user.c@test.com",
        "password": "Secret123!",
        "city_id": city,
    });
    let create_user_req = test_helpers::with_auth_headers(
        test::TestRequest::post().uri("/api/v1/users").set_json(&user_payload),
        &config, &root_token).to_request();
    let resp = test::call_service(&app, create_user_req).await;
    assert_eq!(resp.status(), StatusCode::CREATED);
    let body: serde_json::Value = test::read_body_json(resp).await;
    let user_id: Uuid = body["data"]["id"].as_str().unwrap().parse().unwrap();

    // Token do CITY_USER persistido
    let token = build_token_for_user(user_id, "CITY_USER", "user.c@test.com", "User C", "100000401", "CB PM", Some(city), &config.jwt_secret);

    // Tenta criar outro usuário com policies -> deve ser proibido
    let payload = json!({
        "rank": "CB PM",
        "registration": "100000402",
        "full_name": "User D",
        "profile": "CITY_USER",
        "email": "user.d@test.com",
        "password": "Secret123!",
        "permission_policies": {
            "read_victims": [city]
        }
    });
    let create_req = test_helpers::with_auth_headers(
        test::TestRequest::post().uri("/api/v1/users").set_json(&payload),
        &config, &token).to_request();
    let create_resp = test::call_service(&app, create_req).await;
    assert_eq!(create_resp.status(), StatusCode::FORBIDDEN);
}

#[actix_rt::test]
async fn city_user_read_cities_only_own_city() {
    let pool = test_helpers::setup_test_db().await;
    test_helpers::clean_database(&pool).await;
    let config = test_helpers::build_test_config();
    let app = test_helpers::create_full_test_app(pool.clone(), config.clone()).await;

    let city_a = db_fixtures::insert_city(&pool, "PORTO VELHO").await;
    let _city_b = db_fixtures::insert_city(&pool, "ARIQUEMES").await;

    let root_token = test_helpers::generate_jwt(&test_helpers::build_root_claims(), &config.jwt_secret);

    // Cria CITY_USER em city_a
    let user_payload = json!({
        "rank": "CB PM",
        "registration": "100000501",
        "full_name": "User E",
        "profile": "CITY_USER",
        "email": "user.e@test.com",
        "password": "Secret123!",
        "city_id": city_a,
    });
    let create_user_req = test_helpers::with_auth_headers(
        test::TestRequest::post().uri("/api/v1/users").set_json(&user_payload),
        &config, &root_token).to_request();
    let resp = test::call_service(&app, create_user_req).await;
    assert_eq!(resp.status(), StatusCode::CREATED);
    let body: serde_json::Value = test::read_body_json(resp).await;
    let user_id: Uuid = body["data"]["id"].as_str().unwrap().parse().unwrap();
    let token = build_token_for_user(user_id, "CITY_USER", "user.e@test.com", "User E", "100000501", "CB PM", Some(city_a), &config.jwt_secret);

    // Lista cidades -> deve vir apenas a própria (fallback + filtragem)
    let list_req = test_helpers::with_auth_headers(test::TestRequest::get().uri("/api/v1/cities"), &config, &token).to_request();
    let list_resp = test::call_service(&app, list_req).await;
    assert_eq!(list_resp.status(), StatusCode::OK);
    let list_body: serde_json::Value = test::read_body_json(list_resp).await;
    let cities = list_body["data"].as_array().unwrap();
    assert_eq!(cities.len(), 1);
    assert_eq!(cities[0]["id"].as_str().unwrap(), city_a.to_string());
}

#[actix_rt::test]
async fn city_admin_with_extra_read_attendances_can_list_other_city() {
    let pool = test_helpers::setup_test_db().await;
    test_helpers::clean_database(&pool).await;
    let config = test_helpers::build_test_config();
    let app = test_helpers::create_full_test_app(pool.clone(), config.clone()).await;

    let city_a = db_fixtures::insert_city(&pool, "PORTO VELHO").await;
    let city_b = db_fixtures::insert_city(&pool, "ARIQUEMES").await;
    let root_token = test_helpers::generate_jwt(&test_helpers::build_root_claims(), &config.jwt_secret);

    // Cria ADMIN A
    let admin_payload = json!({
        "rank": "CAP PM",
        "registration": "100000601",
        "full_name": "Admin C",
        "profile": "CITY_ADMIN",
        "email": "admin.c@test.com",
        "password": "Secret123!",
        "city_id": city_a,
    });
    let create_admin_req = test_helpers::with_auth_headers(test::TestRequest::post().uri("/api/v1/users").set_json(&admin_payload), &config, &root_token).to_request();
    let admin_resp = test::call_service(&app, create_admin_req).await;
    let admin_body: serde_json::Value = test::read_body_json(admin_resp).await;
    let admin_id: Uuid = admin_body["data"]["id"].as_str().unwrap().parse().unwrap();

    // Atualiza admin adicionando read_attendances em city_b
    let get_admin_req = test_helpers::with_auth_headers(test::TestRequest::get().uri(&format!("/api/v1/users/{}", admin_id)), &config, &root_token).to_request();
    let get_admin_resp = test::call_service(&app, get_admin_req).await;
    let get_admin_body: serde_json::Value = test::read_body_json(get_admin_resp).await;
    let mut policies = get_admin_body["data"]["permission_policies"].clone();
    let arr = policies["read_attendances"].as_array().cloned().unwrap_or_else(|| vec![json!(city_a.to_string())]);
    let mut new_arr = arr.clone();
    if !new_arr.iter().any(|v| v.as_str() == Some(&city_b.to_string())) { new_arr.push(json!(city_b.to_string())); }
    policies["read_attendances"] = json!(new_arr);
    let update_admin_payload = json!({
        "rank": "CAP PM",
        "registration": "100000601",
        "full_name": "Admin C",
        "profile": "CITY_ADMIN",
        "email": "admin.c@test.com",
        "city_id": city_a,
        "permission_policies": policies,
    });
    let update_admin_req = test_helpers::with_auth_headers(test::TestRequest::put().uri(&format!("/api/v1/users/{}", admin_id)).set_json(&update_admin_payload), &config, &root_token).to_request();
    let update_admin_resp = test::call_service(&app, update_admin_req).await;
    assert_eq!(update_admin_resp.status(), StatusCode::OK);

    // Cria vítima e atendimento em city_b
    let victim_id = db_fixtures::insert_victim(&pool, "Vitima B", city_b).await;
    let attendance_payload = json!({
        "victim_id": victim_id,
        "was_victim_present": true,
        "attendance_date": "2024-01-01",
        "attendance_time": "10:00:00",
        "description": "Atendimento B",
        "latitude": null,
        "longitude": null,
        "address": null
    });
    let create_att_req = test_helpers::with_auth_headers(test::TestRequest::post().uri("/api/v1/attendances").set_json(&attendance_payload), &config, &root_token).to_request();
    let create_att_resp = test::call_service(&app, create_att_req).await;
    assert_eq!(create_att_resp.status(), StatusCode::CREATED);

    let admin_token = build_token_for_user(admin_id, "CITY_ADMIN", "admin.c@test.com", "Admin C", "100000601", "CAP PM", Some(city_a), &config.jwt_secret);

    // Lista atendimentos -> deve incluir o de city_b por extra policy
    let list_req = test_helpers::with_auth_headers(test::TestRequest::get().uri("/api/v1/attendances"), &config, &admin_token).to_request();
    let list_resp = test::call_service(&app, list_req).await;
    assert_eq!(list_resp.status(), StatusCode::OK);
    let list_body: serde_json::Value = test::read_body_json(list_resp).await;
    assert!(!list_body["data"].as_array().unwrap().is_empty());
}

#[actix_rt::test]
async fn city_admin_with_extra_read_protective_measures_can_list_other_city() {
    let pool = test_helpers::setup_test_db().await;
    test_helpers::clean_database(&pool).await;
    let config = test_helpers::build_test_config();
    let app = test_helpers::create_full_test_app(pool.clone(), config.clone()).await;

    let city_a = db_fixtures::insert_city(&pool, "PORTO VELHO").await;
    let city_b = db_fixtures::insert_city(&pool, "ARIQUEMES").await;
    let root_token = test_helpers::generate_jwt(&test_helpers::build_root_claims(), &config.jwt_secret);

    // Cria ADMIN A
    let admin_payload = json!({
        "rank": "CAP PM",
        "registration": "100000701",
        "full_name": "Admin D",
        "profile": "CITY_ADMIN",
        "email": "admin.d@test.com",
        "password": "Secret123!",
        "city_id": city_a,
    });
    let create_admin_req = test_helpers::with_auth_headers(test::TestRequest::post().uri("/api/v1/users").set_json(&admin_payload), &config, &root_token).to_request();
    let admin_resp = test::call_service(&app, create_admin_req).await;
    let admin_body: serde_json::Value = test::read_body_json(admin_resp).await;
    let admin_id: Uuid = admin_body["data"]["id"].as_str().unwrap().parse().unwrap();

    // Atualiza admin adicionando read_protective_measures em city_b
    let get_admin_req = test_helpers::with_auth_headers(test::TestRequest::get().uri(&format!("/api/v1/users/{}", admin_id)), &config, &root_token).to_request();
    let get_admin_resp = test::call_service(&app, get_admin_req).await;
    let get_admin_body: serde_json::Value = test::read_body_json(get_admin_resp).await;
    let mut policies = get_admin_body["data"]["permission_policies"].clone();
    let arr = policies["read_protective_measures"].as_array().cloned().unwrap_or_else(|| vec![json!(city_a.to_string())]);
    let mut new_arr = arr.clone();
    if !new_arr.iter().any(|v| v.as_str() == Some(&city_b.to_string())) { new_arr.push(json!(city_b.to_string())); }
    policies["read_protective_measures"] = json!(new_arr);
    let update_admin_payload = json!({
        "rank": "CAP PM",
        "registration": "100000701",
        "full_name": "Admin D",
        "profile": "CITY_ADMIN",
        "email": "admin.d@test.com",
        "city_id": city_a,
        "permission_policies": policies,
    });
    let update_admin_req = test_helpers::with_auth_headers(test::TestRequest::put().uri(&format!("/api/v1/users/{}", admin_id)).set_json(&update_admin_payload), &config, &root_token).to_request();
    let update_admin_resp = test::call_service(&app, update_admin_req).await;
    assert_eq!(update_admin_resp.status(), StatusCode::OK);

    // Cria vítima e medida em city_b
    let victim_id = db_fixtures::insert_victim(&pool, "Vitima PB", city_b).await;
    let measure_payload = json!({
        "process_number": "99887-65.2025.8.26.0000",
        "issued_at": "2025-01-01",
        "judicial_authority": "Juiz B",
        "court_district_id": city_b,
        "is_active": true,
        "victim_id": victim_id,
    });
    let create_measure_req = test_helpers::with_auth_headers(test::TestRequest::post().uri("/api/v1/protective-measures").set_json(&measure_payload), &config, &root_token).to_request();
    let create_measure_resp = test::call_service(&app, create_measure_req).await;
    assert_eq!(create_measure_resp.status(), StatusCode::CREATED);

    let admin_token = build_token_for_user(admin_id, "CITY_ADMIN", "admin.d@test.com", "Admin D", "100000701", "CAP PM", Some(city_a), &config.jwt_secret);

    // Lista medidas -> deve incluir city_b
    let list_req = test_helpers::with_auth_headers(test::TestRequest::get().uri("/api/v1/protective-measures"), &config, &admin_token).to_request();
    let list_resp = test::call_service(&app, list_req).await;
    assert_eq!(list_resp.status(), StatusCode::OK);
    let list_body: serde_json::Value = test::read_body_json(list_resp).await;
    assert!(!list_body["data"].as_array().unwrap().is_empty());
}

#[actix_rt::test]
async fn invalid_policy_name_rejected_on_create_and_update_user() {
    let pool = test_helpers::setup_test_db().await;
    test_helpers::clean_database(&pool).await;
    let config = test_helpers::build_test_config();
    let app = test_helpers::create_full_test_app(pool.clone(), config.clone()).await;

    let root_token = test_helpers::generate_jwt(&test_helpers::build_root_claims(), &config.jwt_secret);

    // Tentativa de criação com policy inválida
    let invalid_create = json!({
        "rank": "CAP PM",
        "registration": "100000801",
        "full_name": "Invalid Policy User",
        "profile": "CITY_USER",
        "email": "invalid.policy@test.com",
        "password": "Secret123!",
        "city_id": null,
        "permission_policies": {
            "invalid_policy": [Uuid::new_v4()]
        }
    });
    let create_req = test_helpers::with_auth_headers(test::TestRequest::post().uri("/api/v1/users").set_json(&invalid_create), &config, &root_token).to_request();
    let create_resp = test::call_service(&app, create_req).await;
    assert_eq!(create_resp.status(), StatusCode::BAD_REQUEST);

    // Cria usuário válido
    let valid_user = json!({
        "rank": "CAP PM",
        "registration": "100000802",
        "full_name": "Valid User",
        "profile": "CITY_USER",
        "email": "valid.user@test.com",
        "password": "Secret123!",
        "city_id": null
    });
    let valid_req = test_helpers::with_auth_headers(test::TestRequest::post().uri("/api/v1/users").set_json(&valid_user), &config, &root_token).to_request();
    let valid_resp = test::call_service(&app, valid_req).await;
    assert!(valid_resp.status().is_success());
    let valid_body: serde_json::Value = test::read_body_json(valid_resp).await;
    let user_id: Uuid = valid_body["data"]["id"].as_str().unwrap().parse().unwrap();

    // Tentativa de update com policy inválida
    let invalid_update = json!({
        "rank": "CAP PM",
        "registration": "100000802",
        "full_name": "Valid User",
        "profile": "CITY_USER",
        "email": "valid.user@test.com",
        "city_id": null,
        "permission_policies": { "invalid_policy": [Uuid::new_v4()] }
    });
    let update_req = test_helpers::with_auth_headers(test::TestRequest::put().uri(&format!("/api/v1/users/{}", user_id)).set_json(&invalid_update), &config, &root_token).to_request();
    let update_resp = test::call_service(&app, update_req).await;
    assert_eq!(update_resp.status(), StatusCode::BAD_REQUEST);
}

#[actix_rt::test]
async fn update_protective_measure_changing_victim_requires_policy_in_both_cities() {
    let pool = test_helpers::setup_test_db().await;
    test_helpers::clean_database(&pool).await;
    let config = test_helpers::build_test_config();
    let app = test_helpers::create_full_test_app(pool.clone(), config.clone()).await;

    let city_a = db_fixtures::insert_city(&pool, "PORTO VELHO").await;
    let city_b = db_fixtures::insert_city(&pool, "ARIQUEMES").await;
    let root_token = test_helpers::generate_jwt(&test_helpers::build_root_claims(), &config.jwt_secret);

    // Cria ADMIN A
    let admin_payload = json!({
        "rank": "CAP PM",
        "registration": "100000901",
        "full_name": "Admin E",
        "profile": "CITY_ADMIN",
        "email": "admin.e@test.com",
        "password": "Secret123!",
        "city_id": city_a,
    });
    let create_admin_req = test_helpers::with_auth_headers(test::TestRequest::post().uri("/api/v1/users").set_json(&admin_payload), &config, &root_token).to_request();
    let admin_resp = test::call_service(&app, create_admin_req).await;
    assert!(admin_resp.status().is_success());
    let admin_body: serde_json::Value = test::read_body_json(admin_resp).await;
    let admin_id: Uuid = admin_body["data"]["id"].as_str().unwrap().parse().unwrap();
    let admin_token = build_token_for_user(admin_id, "CITY_ADMIN", "admin.e@test.com", "Admin E", "100000901", "CAP PM", Some(city_a), &config.jwt_secret);

    // Cria vítimas e medida em city_a
    let victim_a = db_fixtures::insert_victim(&pool, "Vitima A", city_a).await;
    let victim_b = db_fixtures::insert_victim(&pool, "Vitima B", city_b).await;
    let measure_payload = json!({
        "process_number": "55555-55.2025.8.26.0000",
        "issued_at": "2025-01-01",
        "judicial_authority": "Juiz C",
        "court_district_id": city_a,
        "is_active": true,
        "victim_id": victim_a,
    });
    let create_req = test_helpers::with_auth_headers(test::TestRequest::post().uri("/api/v1/protective-measures").set_json(&measure_payload), &config, &admin_token).to_request();
    let create_resp = test::call_service(&app, create_req).await;
    assert_eq!(create_resp.status(), 201);
    let created: serde_json::Value = test::read_body_json(create_resp).await;
    let measure_id = created["data"]["id"].as_str().unwrap();

    // Tenta atualizar medida mudando vítima para city_b sem policy extra -> FORBIDDEN
    let update_payload_forbidden = json!({
        "process_number": "55555-55.2025.8.26.0000",
        "issued_at": "2025-01-01",
        "judicial_authority": "Juiz C",
        "court_district_id": city_b,
        "is_active": true,
        "victim_id": victim_b,
    });
    let update_req_forbidden = test_helpers::with_auth_headers(test::TestRequest::put().uri(&format!("/api/v1/protective-measures/{}", measure_id)).set_json(&update_payload_forbidden), &config, &admin_token).to_request();
    let update_resp_forbidden = test::call_service(&app, update_req_forbidden).await;
    assert_eq!(update_resp_forbidden.status(), 403);

    // ROOT concede update_protective_measures para city_b
    let get_admin_req = test_helpers::with_auth_headers(test::TestRequest::get().uri(&format!("/api/v1/users/{}", admin_id)), &config, &root_token).to_request();
    let get_admin_resp = test::call_service(&app, get_admin_req).await;
    let get_admin_body: serde_json::Value = test::read_body_json(get_admin_resp).await;
    let mut policies = get_admin_body["data"]["permission_policies"].clone();
    let arr = policies["update_protective_measures"].as_array().cloned().unwrap_or_else(|| vec![json!(city_a.to_string())]);
    let mut new_arr = arr.clone();
    if !new_arr.iter().any(|v| v.as_str() == Some(&city_b.to_string())) { new_arr.push(json!(city_b.to_string())); }
    policies["update_protective_measures"] = json!(new_arr);
    let update_admin_payload = json!({
        "rank": "CAP PM",
        "registration": "100000901",
        "full_name": "Admin E",
        "profile": "CITY_ADMIN",
        "email": "admin.e@test.com",
        "city_id": city_a,
        "permission_policies": policies,
    });
    let update_admin_req = test_helpers::with_auth_headers(test::TestRequest::put().uri(&format!("/api/v1/users/{}", admin_id)).set_json(&update_admin_payload), &config, &root_token).to_request();
    let update_admin_resp = test::call_service(&app, update_admin_req).await;
    assert_eq!(update_admin_resp.status(), 200);

    // Agora consegue atualizar a medida para vítima em city_b
    let update_req_ok = test_helpers::with_auth_headers(test::TestRequest::put().uri(&format!("/api/v1/protective-measures/{}", measure_id)).set_json(&update_payload_forbidden), &config, &admin_token).to_request();
    let update_resp_ok = test::call_service(&app, update_req_ok).await;
    assert_eq!(update_resp_ok.status(), 200);
}

#[actix_rt::test]
async fn city_admin_list_victims_only_allowed_cities_multi_extra() {
    let pool = test_helpers::setup_test_db().await;
    test_helpers::clean_database(&pool).await;
    let config = test_helpers::build_test_config();
    let app = test_helpers::create_full_test_app(pool.clone(), config.clone()).await;

    let city_a = db_fixtures::insert_city(&pool, "PORTO VELHO").await;
    let city_b = db_fixtures::insert_city(&pool, "ARIQUEMES").await;
    let city_c = db_fixtures::insert_city(&pool, "JARU").await;
    let root_token = test_helpers::generate_jwt(&test_helpers::build_root_claims(), &config.jwt_secret);

    // Cria ADMIN A
    let admin_payload = json!({
        "rank": "CAP PM",
        "registration": "100001001",
        "full_name": "Admin F",
        "profile": "CITY_ADMIN",
        "email": "admin.f@test.com",
        "password": "Secret123!",
        "city_id": city_a,
    });
    let create_admin_req = test_helpers::with_auth_headers(test::TestRequest::post().uri("/api/v1/users").set_json(&admin_payload), &config, &root_token).to_request();
    let admin_resp = test::call_service(&app, create_admin_req).await;
    let admin_body: serde_json::Value = test::read_body_json(admin_resp).await;
    let admin_id: Uuid = admin_body["data"]["id"].as_str().unwrap().parse().unwrap();

    // ROOT adiciona read_victims em city_b, mas não em city_c
    let get_admin_req = test_helpers::with_auth_headers(test::TestRequest::get().uri(&format!("/api/v1/users/{}", admin_id)), &config, &root_token).to_request();
    let get_admin_resp = test::call_service(&app, get_admin_req).await;
    let get_admin_body: serde_json::Value = test::read_body_json(get_admin_resp).await;
    let mut policies = get_admin_body["data"]["permission_policies"].clone();
    let mut arr = policies["read_victims"].as_array().cloned().unwrap_or_else(|| vec![json!(city_a.to_string())]);
    if !arr.iter().any(|v| v.as_str() == Some(&city_b.to_string())) { arr.push(json!(city_b.to_string())); }
    policies["read_victims"] = json!(arr);
    let update_admin_payload = json!({
        "rank": "CAP PM",
        "registration": "100001001",
        "full_name": "Admin F",
        "profile": "CITY_ADMIN",
        "email": "admin.f@test.com",
        "city_id": city_a,
        "permission_policies": policies,
    });
    let update_admin_req = test_helpers::with_auth_headers(test::TestRequest::put().uri(&format!("/api/v1/users/{}", admin_id)).set_json(&update_admin_payload), &config, &root_token).to_request();
    let update_admin_resp = test::call_service(&app, update_admin_req).await;
    assert_eq!(update_admin_resp.status(), 200);

    // Insere vítimas em A, B e C
    db_fixtures::insert_victim(&pool, "Vitima A", city_a).await;
    db_fixtures::insert_victim(&pool, "Vitima B", city_b).await;
    db_fixtures::insert_victim(&pool, "Vitima C", city_c).await;

    let admin_token = build_token_for_user(admin_id, "CITY_ADMIN", "admin.f@test.com", "Admin F", "100001001", "CAP PM", Some(city_a), &config.jwt_secret);

    // Lista vítimas -> deve retornar apenas A e B, não C
    let list_req = test_helpers::with_auth_headers(test::TestRequest::get().uri("/api/v1/victims"), &config, &admin_token).to_request();
    let list_resp = test::call_service(&app, list_req).await;
    assert_eq!(list_resp.status(), 200);
    let body: serde_json::Value = test::read_body_json(list_resp).await;
    let cities: std::collections::HashSet<_> = body["data"].as_array().unwrap().iter().map(|v| v["city_id"].as_str().unwrap().to_string()).collect();
    assert!(cities.contains(&city_a.to_string()));
    assert!(cities.contains(&city_b.to_string()));
    assert!(!cities.contains(&city_c.to_string()));
}

#[actix_rt::test]
async fn city_user_cannot_append_policies_via_endpoint() {
    let pool = test_helpers::setup_test_db().await;
    test_helpers::clean_database(&pool).await;
    let config = test_helpers::build_test_config();
    let app = test_helpers::create_full_test_app(pool.clone(), config.clone()).await;

    let city = db_fixtures::insert_city(&pool, "PORTO VELHO").await;
    let root_token = test_helpers::generate_jwt(&test_helpers::build_root_claims(), &config.jwt_secret);

    // Cria CITY_USER A
    let user_a_payload = json!({
        "rank": "CB PM",
        "registration": "100001101",
        "full_name": "User A",
        "profile": "CITY_USER",
        "email": "user.a@test.com",
        "password": "Secret123!",
        "city_id": city,
    });
    let create_user_a = test_helpers::with_auth_headers(
        test::TestRequest::post().uri("/api/v1/users").set_json(&user_a_payload),
        &config, &root_token).to_request();
    let resp_a = test::call_service(&app, create_user_a).await;
    let body_a: serde_json::Value = test::read_body_json(resp_a).await;
    let user_a_id: Uuid = body_a["data"]["id"].as_str().unwrap().parse().unwrap();

    // Cria CITY_USER B (alvo)
    let user_b_payload = json!({
        "rank": "CB PM",
        "registration": "100001102",
        "full_name": "User B",
        "profile": "CITY_USER",
        "email": "user.b@test.com",
        "password": "Secret123!",
        "city_id": city,
    });
    let create_user_b = test_helpers::with_auth_headers(
        test::TestRequest::post().uri("/api/v1/users").set_json(&user_b_payload),
        &config, &root_token).to_request();
    let resp_b = test::call_service(&app, create_user_b).await;
    let body_b: serde_json::Value = test::read_body_json(resp_b).await;
    let user_b_id: Uuid = body_b["data"]["id"].as_str().unwrap().parse().unwrap();

    // Token do CITY_USER A
    let token_a = build_token_for_user(user_a_id, "CITY_USER", "user.a@test.com", "User A", "100001101", "CB PM", Some(city), &config.jwt_secret);

    // CITY_USER A tenta adicionar policy para CITY_USER B -> deve ser FORBIDDEN
    let append_payload = json!({
        "city_ids": [city]
    });
    let append_req = test_helpers::with_auth_headers(
        test::TestRequest::post().uri(&format!("/api/v1/users/{}/policies/create_victims/cities", user_b_id))
            .set_json(&append_payload),
        &config, &token_a).to_request();
    let append_resp = test::call_service(&app, append_req).await;
    assert_eq!(append_resp.status(), StatusCode::FORBIDDEN);
}

#[actix_rt::test]
async fn city_user_cannot_remove_policies_via_endpoint() {
    let pool = test_helpers::setup_test_db().await;
    test_helpers::clean_database(&pool).await;
    let config = test_helpers::build_test_config();
    let app = test_helpers::create_full_test_app(pool.clone(), config.clone()).await;

    let city = db_fixtures::insert_city(&pool, "PORTO VELHO").await;
    let root_token = test_helpers::generate_jwt(&test_helpers::build_root_claims(), &config.jwt_secret);

    // Cria CITY_USER A
    let user_a_payload = json!({
        "rank": "CB PM",
        "registration": "100001201",
        "full_name": "User C",
        "profile": "CITY_USER",
        "email": "user.c@test.com",
        "password": "Secret123!",
        "city_id": city,
    });
    let create_user_a = test_helpers::with_auth_headers(
        test::TestRequest::post().uri("/api/v1/users").set_json(&user_a_payload),
        &config, &root_token).to_request();
    let resp_a = test::call_service(&app, create_user_a).await;
    let body_a: serde_json::Value = test::read_body_json(resp_a).await;
    let user_a_id: Uuid = body_a["data"]["id"].as_str().unwrap().parse().unwrap();

    // Cria CITY_USER B (alvo) com policies extras
    let user_b_payload = json!({
        "rank": "CB PM",
        "registration": "100001202",
        "full_name": "User D",
        "profile": "CITY_USER",
        "email": "user.d@test.com",
        "password": "Secret123!",
        "city_id": city,
        "permission_policies": {
            "read_victims": [city],
            "create_victims": [city]
        }
    });
    let create_user_b = test_helpers::with_auth_headers(
        test::TestRequest::post().uri("/api/v1/users").set_json(&user_b_payload),
        &config, &root_token).to_request();
    let resp_b = test::call_service(&app, create_user_b).await;
    let body_b: serde_json::Value = test::read_body_json(resp_b).await;
    let user_b_id: Uuid = body_b["data"]["id"].as_str().unwrap().parse().unwrap();

    // Token do CITY_USER A
    let token_a = build_token_for_user(user_a_id, "CITY_USER", "user.c@test.com", "User C", "100001201", "CB PM", Some(city), &config.jwt_secret);

    // CITY_USER A tenta remover policy de CITY_USER B -> deve ser FORBIDDEN
    let remove_payload = json!({
        "city_ids": [city]
    });
    let remove_req = test_helpers::with_auth_headers(
        test::TestRequest::delete().uri(&format!("/api/v1/users/{}/policies/create_victims/cities", user_b_id))
            .set_json(&remove_payload),
        &config, &token_a).to_request();
    let remove_resp = test::call_service(&app, remove_req).await;
    assert_eq!(remove_resp.status(), StatusCode::FORBIDDEN);
}

#[actix_rt::test]
async fn city_admin_can_append_and_remove_policies_for_city_user() {
    let pool = test_helpers::setup_test_db().await;
    test_helpers::clean_database(&pool).await;
    let config = test_helpers::build_test_config();
    let app = test_helpers::create_full_test_app(pool.clone(), config.clone()).await;

    let city_a = db_fixtures::insert_city(&pool, "PORTO VELHO").await;
    let city_b = db_fixtures::insert_city(&pool, "ARIQUEMES").await;
    let root_token = test_helpers::generate_jwt(&test_helpers::build_root_claims(), &config.jwt_secret);

    // Cria CITY_ADMIN A
    let admin_payload = json!({
        "rank": "CAP PM",
        "registration": "100001301",
        "full_name": "Admin G",
        "profile": "CITY_ADMIN",
        "email": "admin.g@test.com",
        "password": "Secret123!",
        "city_id": city_a,
    });
    let create_admin = test_helpers::with_auth_headers(
        test::TestRequest::post().uri("/api/v1/users").set_json(&admin_payload),
        &config, &root_token).to_request();
    let admin_resp = test::call_service(&app, create_admin).await;
    let admin_body: serde_json::Value = test::read_body_json(admin_resp).await;
    let admin_id: Uuid = admin_body["data"]["id"].as_str().unwrap().parse().unwrap();

    // ROOT adiciona read_victims em city_b para o ADMIN
    let get_admin_req = test_helpers::with_auth_headers(
        test::TestRequest::get().uri(&format!("/api/v1/users/{}", admin_id)),
        &config, &root_token).to_request();
    let get_admin_resp = test::call_service(&app, get_admin_req).await;
    let get_admin_body: serde_json::Value = test::read_body_json(get_admin_resp).await;
    let mut policies = get_admin_body["data"]["permission_policies"].clone();
    let mut arr = policies["read_victims"].as_array().cloned().unwrap_or_else(|| vec![json!(city_a.to_string())]);
    if !arr.iter().any(|v| v.as_str() == Some(&city_b.to_string())) {
        arr.push(json!(city_b.to_string()));
    }
    policies["read_victims"] = json!(arr);
    let update_admin_payload = json!({
        "rank": "CAP PM",
        "registration": "100001301",
        "full_name": "Admin G",
        "profile": "CITY_ADMIN",
        "email": "admin.g@test.com",
        "city_id": city_a,
        "permission_policies": policies,
    });
    let update_admin_req = test_helpers::with_auth_headers(
        test::TestRequest::put().uri(&format!("/api/v1/users/{}", admin_id)).set_json(&update_admin_payload),
        &config, &root_token).to_request();
    let update_admin_resp = test::call_service(&app, update_admin_req).await;
    assert_eq!(update_admin_resp.status(), StatusCode::OK);

    // Cria CITY_USER
    let user_payload = json!({
        "rank": "CB PM",
        "registration": "100001302",
        "full_name": "User E",
        "profile": "CITY_USER",
        "email": "user.e@test.com",
        "password": "Secret123!",
        "city_id": city_a,
    });
    let create_user = test_helpers::with_auth_headers(
        test::TestRequest::post().uri("/api/v1/users").set_json(&user_payload),
        &config, &root_token).to_request();
    let user_resp = test::call_service(&app, create_user).await;
    let user_body: serde_json::Value = test::read_body_json(user_resp).await;
    let user_id: Uuid = user_body["data"]["id"].as_str().unwrap().parse().unwrap();

    // Token do CITY_ADMIN
    let admin_token = build_token_for_user(admin_id, "CITY_ADMIN", "admin.g@test.com", "Admin G", "100001301", "CAP PM", Some(city_a), &config.jwt_secret);

    // CITY_ADMIN adiciona read_victims em city_b para CITY_USER -> deve ser OK
    let append_payload = json!({
        "city_ids": [city_b]
    });
    let append_req = test_helpers::with_auth_headers(
        test::TestRequest::post().uri(&format!("/api/v1/users/{}/policies/read_victims/cities", user_id))
            .set_json(&append_payload),
        &config, &admin_token).to_request();
    let append_resp = test::call_service(&app, append_req).await;
    assert_eq!(append_resp.status(), StatusCode::OK);

    // Verifica que a policy foi adicionada
    let get_user_req = test_helpers::with_auth_headers(
        test::TestRequest::get().uri(&format!("/api/v1/users/{}", user_id)),
        &config, &root_token).to_request();
    let get_user_resp = test::call_service(&app, get_user_req).await;
    let get_user_body: serde_json::Value = test::read_body_json(get_user_resp).await;
    let user_policies = get_user_body["data"]["permission_policies"]["read_victims"].as_array().unwrap();
    assert!(user_policies.iter().any(|v| v.as_str().unwrap() == city_b.to_string()));

    // CITY_ADMIN remove read_victims em city_b de CITY_USER -> deve ser OK
    let remove_payload = json!({
        "city_ids": [city_b]
    });
    let remove_req = test_helpers::with_auth_headers(
        test::TestRequest::delete().uri(&format!("/api/v1/users/{}/policies/read_victims/cities", user_id))
            .set_json(&remove_payload),
        &config, &admin_token).to_request();
    let remove_resp = test::call_service(&app, remove_req).await;
    assert_eq!(remove_resp.status(), StatusCode::OK);

    // Verifica que a policy foi removida
    let get_user_req2 = test_helpers::with_auth_headers(
        test::TestRequest::get().uri(&format!("/api/v1/users/{}", user_id)),
        &config, &root_token).to_request();
    let get_user_resp2 = test::call_service(&app, get_user_req2).await;
    let get_user_body2: serde_json::Value = test::read_body_json(get_user_resp2).await;
    let user_policies2 = get_user_body2["data"]["permission_policies"]["read_victims"].as_array().unwrap();
    assert!(!user_policies2.iter().any(|v| v.as_str().unwrap() == city_b.to_string()));
}
