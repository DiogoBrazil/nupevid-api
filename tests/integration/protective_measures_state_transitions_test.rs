use actix_web::{dev::Service, http::StatusCode, test};
use chrono::{Duration, Local, NaiveDate};
use uuid::Uuid;

use crate::common::{db_fixtures, test_helpers};

fn base_payload(
    victim_id: Uuid,
    court_district_id: Uuid,
    offender_id: Uuid,
    status: &str,
) -> serde_json::Value {
    serde_json::json!({
        "process_number": format!("PM-{}", Uuid::new_v4()),
        "issued_at": NaiveDate::from_ymd_opt(2025, 1, 1).unwrap(),
        "judicial_authority": "Juiz A",
        "court_district_id": court_district_id,
        "status": status,
        "violence_types": ["Physical"],
        "relationship_to_victim": "Spouse",
        "assaults_children": false,
        "was_drunk_during_assault": false,
        "victim_id": victim_id,
        "offender_id": offender_id,
    })
}

async fn setup_case() -> (
    sqlx::PgPool,
    impl Service<
        actix_http::Request,
        Response = actix_web::dev::ServiceResponse,
        Error = actix_web::Error,
    >,
    nupevid_api::config::config_env::Config,
    Uuid,
    Uuid,
    Uuid,
    String,
) {
    let pool = test_helpers::setup_test_db().await;
    test_helpers::clean_database(&pool).await;

    let config = test_helpers::build_test_config();
    let app = test_helpers::create_full_test_app(pool.clone(), config.clone()).await;

    let city_id = db_fixtures::insert_city(&pool, "Cidade State").await;
    let victim_id = db_fixtures::insert_victim(&pool, "Vitima State", city_id).await;
    let offender_id = db_fixtures::insert_offender(&pool, "Agressor State", city_id).await;
    let claims = test_helpers::build_city_admin_claims(city_id);
    let token = test_helpers::generate_jwt(&claims, &config.jwt_secret);

    (pool, app, config, city_id, victim_id, offender_id, token)
}

async fn create_measure(
    app: &impl Service<
        actix_http::Request,
        Response = actix_web::dev::ServiceResponse,
        Error = actix_web::Error,
    >,
    config: &nupevid_api::config::config_env::Config,
    token: &str,
    payload: serde_json::Value,
) -> actix_web::dev::ServiceResponse {
    let req = test_helpers::with_auth_headers(
        test::TestRequest::post()
            .uri("/api/v1/protective-measures")
            .set_json(&payload),
        config,
        token,
    )
    .to_request();

    test::call_service(app, req).await
}

async fn update_measure(
    app: &impl Service<
        actix_http::Request,
        Response = actix_web::dev::ServiceResponse,
        Error = actix_web::Error,
    >,
    config: &nupevid_api::config::config_env::Config,
    token: &str,
    measure_id: &str,
    payload: serde_json::Value,
) -> actix_web::dev::ServiceResponse {
    let req = test_helpers::with_auth_headers(
        test::TestRequest::put()
            .uri(&format!("/api/v1/protective-measures/{}", measure_id))
            .set_json(&payload),
        config,
        token,
    )
    .to_request();

    test::call_service(app, req).await
}

#[actix_rt::test]
async fn create_fails_when_issued_at_is_in_the_future_regardless_of_status() {
    let (_pool, app, config, city_id, victim_id, offender_id, token) = setup_case().await;
    let mut payload = base_payload(victim_id, city_id, offender_id, "Revoked");
    payload["issued_at"] = serde_json::json!(Local::now().date_naive() + Duration::days(1));

    let resp = create_measure(&app, &config, &token, payload).await;

    assert_eq!(resp.status(), StatusCode::BAD_REQUEST);
    let body: serde_json::Value = test::read_body_json(resp).await;
    assert!(body["message"].as_str().unwrap().contains("issued_at"));
}

#[actix_rt::test]
async fn create_succeeds_when_issued_at_is_today_with_status_valid() {
    let (_pool, app, config, city_id, victim_id, offender_id, token) = setup_case().await;
    let mut payload = base_payload(victim_id, city_id, offender_id, "Valid");
    payload["issued_at"] = serde_json::json!(Local::now().date_naive());

    let resp = create_measure(&app, &config, &token, payload).await;

    assert_eq!(resp.status(), StatusCode::CREATED);
    let body: serde_json::Value = test::read_body_json(resp).await;
    assert_eq!(body["data"]["status"].as_str().unwrap(), "Valid");
}

#[actix_rt::test]
async fn update_fails_when_issued_at_changed_to_future() {
    let (_pool, app, config, city_id, victim_id, offender_id, token) = setup_case().await;
    let create_resp = create_measure(
        &app,
        &config,
        &token,
        base_payload(victim_id, city_id, offender_id, "Revoked"),
    )
    .await;
    let body: serde_json::Value = test::read_body_json(create_resp).await;
    let measure_id = body["data"]["id"].as_str().unwrap().to_string();

    let mut payload = base_payload(victim_id, city_id, offender_id, "Revoked");
    payload["issued_at"] = serde_json::json!(Local::now().date_naive() + Duration::days(1));
    let resp = update_measure(&app, &config, &token, &measure_id, payload).await;

    assert_eq!(resp.status(), StatusCode::BAD_REQUEST);
}

#[actix_rt::test]
async fn create_succeeds_with_status_valid() {
    let (_pool, app, config, city_id, victim_id, offender_id, token) = setup_case().await;

    let resp = create_measure(
        &app,
        &config,
        &token,
        base_payload(victim_id, city_id, offender_id, "Valid"),
    )
    .await;

    assert_eq!(resp.status(), StatusCode::CREATED);
}

#[actix_rt::test]
async fn create_succeeds_with_status_revoked() {
    let (_pool, app, config, city_id, victim_id, offender_id, token) = setup_case().await;

    let resp = create_measure(
        &app,
        &config,
        &token,
        base_payload(victim_id, city_id, offender_id, "Revoked"),
    )
    .await;

    assert_eq!(resp.status(), StatusCode::CREATED);
}

#[actix_rt::test]
async fn create_rejects_status_expired_as_invalid_input() {
    let (_pool, app, config, city_id, victim_id, offender_id, token) = setup_case().await;

    let resp = create_measure(
        &app,
        &config,
        &token,
        base_payload(victim_id, city_id, offender_id, "Expired"),
    )
    .await;

    assert_eq!(resp.status(), StatusCode::UNPROCESSABLE_ENTITY);
    let body: serde_json::Value = test::read_body_json(resp).await;
    assert!(body["message"].as_str().unwrap().contains("Expired"));
}

#[actix_rt::test]
async fn sequence_valid_revoked_valid_revoked_is_accepted() {
    let (_pool, app, config, city_id, victim_id, offender_id, token) = setup_case().await;
    let create_resp = create_measure(
        &app,
        &config,
        &token,
        base_payload(victim_id, city_id, offender_id, "Valid"),
    )
    .await;
    assert_eq!(create_resp.status(), StatusCode::CREATED);
    let body: serde_json::Value = test::read_body_json(create_resp).await;
    let measure_id = body["data"]["id"].as_str().unwrap().to_string();

    for status in ["Revoked", "Valid", "Revoked"] {
        let resp = update_measure(
            &app,
            &config,
            &token,
            &measure_id,
            base_payload(victim_id, city_id, offender_id, status),
        )
        .await;
        assert_eq!(resp.status(), StatusCode::OK);
    }
}

#[actix_rt::test]
async fn transition_revoked_to_valid_respects_unique_active_per_pair() {
    let (_pool, app, config, city_id, victim_id, offender_id, token) = setup_case().await;
    let revoked_resp = create_measure(
        &app,
        &config,
        &token,
        base_payload(victim_id, city_id, offender_id, "Revoked"),
    )
    .await;
    let body: serde_json::Value = test::read_body_json(revoked_resp).await;
    let revoked_id = body["data"]["id"].as_str().unwrap().to_string();

    let active_resp = create_measure(
        &app,
        &config,
        &token,
        base_payload(victim_id, city_id, offender_id, "Valid"),
    )
    .await;
    assert_eq!(active_resp.status(), StatusCode::CREATED);

    let resp = update_measure(
        &app,
        &config,
        &token,
        &revoked_id,
        base_payload(victim_id, city_id, offender_id, "Valid"),
    )
    .await;

    assert_eq!(resp.status(), StatusCode::CONFLICT);
}

#[actix_rt::test]
async fn transition_valid_to_revoked_frees_slot_for_same_pair_new_measure() {
    let (_pool, app, config, city_id, victim_id, offender_id, token) = setup_case().await;
    let create_resp = create_measure(
        &app,
        &config,
        &token,
        base_payload(victim_id, city_id, offender_id, "Valid"),
    )
    .await;
    let body: serde_json::Value = test::read_body_json(create_resp).await;
    let measure_id = body["data"]["id"].as_str().unwrap().to_string();

    let update_resp = update_measure(
        &app,
        &config,
        &token,
        &measure_id,
        base_payload(victim_id, city_id, offender_id, "Revoked"),
    )
    .await;
    assert_eq!(update_resp.status(), StatusCode::OK);

    let new_resp = create_measure(
        &app,
        &config,
        &token,
        base_payload(victim_id, city_id, offender_id, "Valid"),
    )
    .await;
    assert_eq!(new_resp.status(), StatusCode::CREATED);
}

#[actix_rt::test]
async fn payload_with_unknown_field_valid_until_is_rejected_with_422() {
    let (_pool, app, config, city_id, victim_id, offender_id, token) = setup_case().await;
    let mut payload = base_payload(victim_id, city_id, offender_id, "Revoked");
    payload["valid_until"] = serde_json::json!(NaiveDate::from_ymd_opt(2026, 1, 1).unwrap());

    let resp = create_measure(&app, &config, &token, payload).await;

    assert_eq!(resp.status(), StatusCode::UNPROCESSABLE_ENTITY);
    let body: serde_json::Value = test::read_body_json(resp).await;
    assert_eq!(body["field"].as_str().unwrap(), "valid_until");
}

#[actix_rt::test]
async fn payload_with_unknown_field_any_other_name_is_rejected_with_422() {
    let (_pool, app, config, city_id, victim_id, offender_id, token) = setup_case().await;
    let mut payload = base_payload(victim_id, city_id, offender_id, "Revoked");
    payload["unexpected_field"] = serde_json::json!("unexpected");

    let resp = create_measure(&app, &config, &token, payload).await;

    assert_eq!(resp.status(), StatusCode::UNPROCESSABLE_ENTITY);
    let body: serde_json::Value = test::read_body_json(resp).await;
    assert_eq!(body["field"].as_str().unwrap(), "unexpected_field");
}

#[actix_rt::test]
async fn payload_missing_required_field_issued_at_is_rejected_with_422_and_mentions_field() {
    let (_pool, app, config, city_id, victim_id, offender_id, token) = setup_case().await;
    let mut payload = base_payload(victim_id, city_id, offender_id, "Revoked");
    payload.as_object_mut().unwrap().remove("issued_at");

    let resp = create_measure(&app, &config, &token, payload).await;

    assert_eq!(resp.status(), StatusCode::UNPROCESSABLE_ENTITY);
    let body: serde_json::Value = test::read_body_json(resp).await;
    assert_eq!(body["field"].as_str().unwrap(), "issued_at");
}
