//! Phase 4.§6.3 — Soft-delete sweep.
//!
//! Consolida testes transversais que garantem que após `DELETE` (soft-delete):
//! - Listagens, buscas filtradas e GET-by-id não retornam a entidade.
//! - Tentativas de UPDATE retornam `NotFound`.
//!
//! Os testes de list/get-by-id para `victims` e `offenders` já existiam no
//! baseline (ver `victims_integration_test.rs::delete_victim_soft_delete_and_not_listed`
//! e `offenders_basic_test.rs::test_delete_offender`); este arquivo fecha a
//! lacuna com os cenários de update e search explicitamente citados no plano.

use actix_web::{http::StatusCode, test};
use uuid::Uuid;

use crate::common::{db_fixtures, test_helpers};

fn build_victim_update_payload(city_id: Uuid) -> serde_json::Value {
    serde_json::json!({
        "full_name": "Vitima Update",
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

fn build_offender_update_payload(city_id: Uuid) -> serde_json::Value {
    serde_json::json!({
        "full_name": "Offender Update",
        "city_id": city_id.to_string(),
        "imprisoned": false,
        "uses_alcohol": false,
        "uses_drugs": false,
        "has_psychiatric_issues": false,
        "education_level": "High School",
    })
}

#[actix_rt::test]
async fn cannot_update_victim_after_soft_delete() {
    let pool = test_helpers::setup_test_db().await;
    test_helpers::clean_database(&pool).await;

    let config = test_helpers::build_test_config();
    let app = test_helpers::create_full_test_app(pool.clone(), config.clone()).await;

    let city = db_fixtures::insert_city(&pool, "SoftDel City V").await;
    let victim_id = db_fixtures::insert_victim(&pool, "Vitima X", city).await;

    let token = test_helpers::generate_jwt(&test_helpers::build_root_claims(), &config.jwt_secret);

    // Soft-delete
    let delete_req = test_helpers::with_auth_headers(
        test::TestRequest::delete().uri(&format!("/api/v1/victims/{}", victim_id)),
        &config,
        &token,
    )
    .to_request();
    let delete_resp = test::call_service(&app, delete_req).await;
    assert_eq!(delete_resp.status(), StatusCode::OK);

    // Try to update
    let update_req = test_helpers::with_auth_headers(
        test::TestRequest::put()
            .uri(&format!("/api/v1/victims/{}", victim_id))
            .set_json(&build_victim_update_payload(city)),
        &config,
        &token,
    )
    .to_request();
    let update_resp = test::call_service(&app, update_req).await;
    assert_eq!(update_resp.status(), StatusCode::NOT_FOUND);
}

#[actix_rt::test]
async fn cannot_update_offender_after_soft_delete() {
    let pool = test_helpers::setup_test_db().await;
    test_helpers::clean_database(&pool).await;

    let config = test_helpers::build_test_config();
    let app = test_helpers::create_full_test_app(pool.clone(), config.clone()).await;

    let city = db_fixtures::insert_city(&pool, "SoftDel City O").await;
    let offender_id = db_fixtures::insert_offender(&pool, "Offender X", city).await;

    let token = test_helpers::generate_jwt(&test_helpers::build_root_claims(), &config.jwt_secret);

    // Soft-delete
    let delete_req = test_helpers::with_auth_headers(
        test::TestRequest::delete().uri(&format!("/api/v1/offenders/{}", offender_id)),
        &config,
        &token,
    )
    .to_request();
    assert_eq!(
        test::call_service(&app, delete_req).await.status(),
        StatusCode::OK
    );

    // Try to update
    let update_req = test_helpers::with_auth_headers(
        test::TestRequest::put()
            .uri(&format!("/api/v1/offenders/{}", offender_id))
            .set_json(&build_offender_update_payload(city)),
        &config,
        &token,
    )
    .to_request();
    let update_resp = test::call_service(&app, update_req).await;
    assert_eq!(update_resp.status(), StatusCode::NOT_FOUND);
}

#[actix_rt::test]
async fn cannot_update_protective_measure_after_soft_delete() {
    let pool = test_helpers::setup_test_db().await;
    test_helpers::clean_database(&pool).await;

    let config = test_helpers::build_test_config();
    let app = test_helpers::create_full_test_app(pool.clone(), config.clone()).await;

    let city = db_fixtures::insert_city(&pool, "SoftDel City PM").await;
    let victim_id = db_fixtures::insert_victim(&pool, "Victim for PM", city).await;
    let offender_id = db_fixtures::insert_offender(&pool, "Offender for PM", city).await;
    let pm_id =
        db_fixtures::insert_protective_measure(&pool, victim_id, offender_id, city, "Revoked")
            .await;

    let token = test_helpers::generate_jwt(&test_helpers::build_root_claims(), &config.jwt_secret);

    // Soft-delete the measure
    let delete_req = test_helpers::with_auth_headers(
        test::TestRequest::delete().uri(&format!("/api/v1/protective-measures/{}", pm_id)),
        &config,
        &token,
    )
    .to_request();
    assert_eq!(
        test::call_service(&app, delete_req).await.status(),
        StatusCode::OK
    );

    // Try to update
    let update_payload = serde_json::json!({
        "process_number": "2025.000.000001-0",
        "sei_process_number": serde_json::Value::Null,
        "occurrence_report_number": serde_json::Value::Null,
        "issued_at": "2025-01-02",
        "judicial_authority": "1st Criminal Court",
        "court_district_id": city,
        "distance_meters": serde_json::Value::Null,
        "status": "Revoked",
        "violence_types": ["Physical"],
        "relationship_to_victim": "Spouse",
        "assaults_children": false,
        "was_drunk_during_assault": false,
        "victim_id": victim_id,
        "offender_id": offender_id,
    });

    let update_req = test_helpers::with_auth_headers(
        test::TestRequest::put()
            .uri(&format!("/api/v1/protective-measures/{}", pm_id))
            .set_json(&update_payload),
        &config,
        &token,
    )
    .to_request();
    let update_resp = test::call_service(&app, update_req).await;
    assert_eq!(update_resp.status(), StatusCode::NOT_FOUND);
}

#[actix_rt::test]
async fn cannot_update_user_after_soft_delete() {
    let pool = test_helpers::setup_test_db().await;
    test_helpers::clean_database(&pool).await;

    let config = test_helpers::build_test_config();
    let app = test_helpers::create_full_test_app(pool.clone(), config.clone()).await;

    let city = db_fixtures::insert_city(&pool, "SoftDel City U").await;
    let target_id = db_fixtures::insert_user(
        &pool,
        "900001",
        "target-softdel@test.com",
        "CITY_USER",
        Some(city),
    )
    .await;

    let token = test_helpers::generate_jwt(&test_helpers::build_root_claims(), &config.jwt_secret);

    // Soft-delete the user
    let delete_req = test_helpers::with_auth_headers(
        test::TestRequest::delete().uri(&format!("/api/v1/users/{}", target_id)),
        &config,
        &token,
    )
    .to_request();
    assert_eq!(
        test::call_service(&app, delete_req).await.status(),
        StatusCode::OK
    );

    // Try to update
    let update_payload = serde_json::json!({
        "rank": "SD PM",
        "registration": "900001",
        "full_name": "Novo Nome",
        "profile": "CITY_USER",
        "email": "target-softdel@test.com",
        "city_id": city,
    });

    let update_req = test_helpers::with_auth_headers(
        test::TestRequest::put()
            .uri(&format!("/api/v1/users/{}", target_id))
            .set_json(&update_payload),
        &config,
        &token,
    )
    .to_request();
    let update_resp = test::call_service(&app, update_req).await;
    assert_eq!(update_resp.status(), StatusCode::NOT_FOUND);
}

#[actix_rt::test]
async fn cannot_update_city_after_soft_delete() {
    let pool = test_helpers::setup_test_db().await;
    test_helpers::clean_database(&pool).await;

    let config = test_helpers::build_test_config();
    let app = test_helpers::create_full_test_app(pool.clone(), config.clone()).await;

    // Use a city name present in VALID_CITIES so the validator does not short-circuit
    // with BadRequest before the NotFound check can fire.
    let city_id = db_fixtures::insert_city(&pool, "PORTO VELHO").await;

    let token = test_helpers::generate_jwt(&test_helpers::build_root_claims(), &config.jwt_secret);

    // Soft-delete the city
    let delete_req = test_helpers::with_auth_headers(
        test::TestRequest::delete().uri(&format!("/api/v1/cities/{}", city_id)),
        &config,
        &token,
    )
    .to_request();
    assert_eq!(
        test::call_service(&app, delete_req).await.status(),
        StatusCode::OK
    );

    // Try to update with another valid city name so we exercise the soft-delete
    // path rather than the field validator.
    let update_payload = serde_json::json!({
        "name": "ARIQUEMES",
        "state": "RO",
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
    assert_eq!(update_resp.status(), StatusCode::NOT_FOUND);
}

#[actix_rt::test]
async fn victim_soft_deleted_not_returned_by_search_by_name() {
    let pool = test_helpers::setup_test_db().await;
    test_helpers::clean_database(&pool).await;

    let config = test_helpers::build_test_config();
    let app = test_helpers::create_full_test_app(pool.clone(), config.clone()).await;

    let city = db_fixtures::insert_city(&pool, "Search City V").await;
    let victim_id = db_fixtures::insert_victim(&pool, "Maria Sumida", city).await;

    let token = test_helpers::generate_jwt(&test_helpers::build_root_claims(), &config.jwt_secret);

    // Soft-delete
    let delete_req = test_helpers::with_auth_headers(
        test::TestRequest::delete().uri(&format!("/api/v1/victims/{}", victim_id)),
        &config,
        &token,
    )
    .to_request();
    assert_eq!(
        test::call_service(&app, delete_req).await.status(),
        StatusCode::OK
    );

    // Search by name must not return the soft-deleted victim
    let search_req = test_helpers::with_auth_headers(
        test::TestRequest::get().uri("/api/v1/victims/search?name=sumida"),
        &config,
        &token,
    )
    .to_request();
    let search_resp = test::call_service(&app, search_req).await;
    assert_eq!(search_resp.status(), StatusCode::OK);
    let body: serde_json::Value = test::read_body_json(search_resp).await;
    let results = body["data"].as_array().expect("data array");
    assert!(
        results
            .iter()
            .all(|v| v["id"].as_str().unwrap() != victim_id.to_string()),
        "soft-deleted victim should not be returned by search"
    );
}

#[actix_rt::test]
async fn offender_soft_deleted_not_returned_by_search_by_name() {
    let pool = test_helpers::setup_test_db().await;
    test_helpers::clean_database(&pool).await;

    let config = test_helpers::build_test_config();
    let app = test_helpers::create_full_test_app(pool.clone(), config.clone()).await;

    let city = db_fixtures::insert_city(&pool, "Search City O").await;
    let offender_id = db_fixtures::insert_offender(&pool, "Joao Sumido", city).await;

    let token = test_helpers::generate_jwt(&test_helpers::build_root_claims(), &config.jwt_secret);

    // Soft-delete
    let delete_req = test_helpers::with_auth_headers(
        test::TestRequest::delete().uri(&format!("/api/v1/offenders/{}", offender_id)),
        &config,
        &token,
    )
    .to_request();
    assert_eq!(
        test::call_service(&app, delete_req).await.status(),
        StatusCode::OK
    );

    // Search by name must not return the soft-deleted offender
    let search_req = test_helpers::with_auth_headers(
        test::TestRequest::get().uri("/api/v1/offenders/search?name=sumido"),
        &config,
        &token,
    )
    .to_request();
    let search_resp = test::call_service(&app, search_req).await;
    assert_eq!(search_resp.status(), StatusCode::OK);
    let body: serde_json::Value = test::read_body_json(search_resp).await;
    let results = body["data"].as_array().expect("data array");
    assert!(
        results
            .iter()
            .all(|o| o["id"].as_str().unwrap() != offender_id.to_string()),
        "soft-deleted offender should not be returned by search"
    );
}

#[actix_rt::test]
async fn protective_measure_soft_deleted_not_returned_by_list() {
    let pool = test_helpers::setup_test_db().await;
    test_helpers::clean_database(&pool).await;

    let config = test_helpers::build_test_config();
    let app = test_helpers::create_full_test_app(pool.clone(), config.clone()).await;

    let city = db_fixtures::insert_city(&pool, "List City PM").await;
    let victim_id = db_fixtures::insert_victim(&pool, "VictimList", city).await;
    let offender_id = db_fixtures::insert_offender(&pool, "OffenderList", city).await;
    let pm_id =
        db_fixtures::insert_protective_measure(&pool, victim_id, offender_id, city, "Revoked")
            .await;

    let token = test_helpers::generate_jwt(&test_helpers::build_root_claims(), &config.jwt_secret);

    // Soft-delete
    let delete_req = test_helpers::with_auth_headers(
        test::TestRequest::delete().uri(&format!("/api/v1/protective-measures/{}", pm_id)),
        &config,
        &token,
    )
    .to_request();
    assert_eq!(
        test::call_service(&app, delete_req).await.status(),
        StatusCode::OK
    );

    // List must not include the soft-deleted measure
    let list_req = test_helpers::with_auth_headers(
        test::TestRequest::get().uri("/api/v1/protective-measures"),
        &config,
        &token,
    )
    .to_request();
    let list_resp = test::call_service(&app, list_req).await;
    assert_eq!(list_resp.status(), StatusCode::OK);
    let body: serde_json::Value = test::read_body_json(list_resp).await;
    let results = body["data"].as_array().expect("data array");
    assert!(
        results
            .iter()
            .all(|pm| pm["id"].as_str().unwrap() != pm_id.to_string()),
        "soft-deleted protective measure should not be returned by list"
    );
}
