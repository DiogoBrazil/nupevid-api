use actix_web::{dev::Service, http::StatusCode, test};
use futures::future::join_all;
use uuid::Uuid;

use crate::common::{db_fixtures, test_helpers};

fn measure_payload(
    victim_id: Uuid,
    court_district_id: Uuid,
    offender_id: Uuid,
) -> serde_json::Value {
    serde_json::json!({
        "process_number": format!("PM-{}", Uuid::new_v4()),
        "issued_at": chrono::NaiveDate::from_ymd_opt(2025, 1, 1).unwrap(),
        "judicial_authority": "Juiz A",
        "court_district_id": court_district_id,
        "status": "Valid",
        "violence_types": ["Physical"],
        "relationship_to_victim": "Spouse",
        "assaults_children": false,
        "was_drunk_during_assault": false,
        "victim_id": victim_id,
        "offender_id": offender_id,
    })
}

async fn post_measure(
    app: &impl Service<
        actix_http::Request,
        Response = actix_web::dev::ServiceResponse,
        Error = actix_web::Error,
    >,
    config: &nupevid_api::config::config_env::Config,
    token: &str,
    payload: serde_json::Value,
) -> StatusCode {
    let req = test_helpers::with_auth_headers(
        test::TestRequest::post()
            .uri("/api/v1/protective-measures")
            .set_json(&payload),
        config,
        token,
    )
    .to_request();

    test::call_service(app, req).await.status()
}

#[actix_rt::test]
async fn concurrent_valid_measure_create_same_pair_allows_one_success_and_conflicts_rest() {
    let pool = test_helpers::setup_test_db().await;
    test_helpers::clean_database(&pool).await;

    let config = test_helpers::build_test_config();
    let app = test_helpers::create_full_test_app(pool.clone(), config.clone()).await;

    let city_id = db_fixtures::insert_city(&pool, "Cidade Concorrencia").await;
    let victim_id = db_fixtures::insert_victim(&pool, "Vitima Concorrencia", city_id).await;
    let offender_id = db_fixtures::insert_offender(&pool, "Agressor Concorrencia", city_id).await;
    let claims = test_helpers::build_city_admin_claims(city_id);
    let token = test_helpers::generate_jwt(&claims, &config.jwt_secret);

    let futures = (0..6).map(|_| {
        post_measure(
            &app,
            &config,
            &token,
            measure_payload(victim_id, city_id, offender_id),
        )
    });

    let statuses = join_all(futures).await;
    let created = statuses
        .iter()
        .filter(|status| **status == StatusCode::CREATED)
        .count();
    let conflicts = statuses
        .iter()
        .filter(|status| **status == StatusCode::CONFLICT)
        .count();

    assert_eq!(created, 1);
    assert_eq!(conflicts, 5);
}

#[actix_rt::test]
async fn concurrent_valid_measure_create_same_victim_different_offenders_allows_all_success() {
    let pool = test_helpers::setup_test_db().await;
    test_helpers::clean_database(&pool).await;

    let config = test_helpers::build_test_config();
    let app = test_helpers::create_full_test_app(pool.clone(), config.clone()).await;

    let city_id = db_fixtures::insert_city(&pool, "Cidade Concorrencia 2").await;
    let victim_id = db_fixtures::insert_victim(&pool, "Vitima Concorrencia 2", city_id).await;
    let offender_ids = join_all((0..4).map(|idx| {
        let pool = pool.clone();
        async move {
            let name = format!("Agressor {}", idx);
            db_fixtures::insert_offender(&pool, &name, city_id).await
        }
    }))
    .await;
    let claims = test_helpers::build_city_admin_claims(city_id);
    let token = test_helpers::generate_jwt(&claims, &config.jwt_secret);

    let futures = offender_ids.into_iter().map(|offender_id| {
        post_measure(
            &app,
            &config,
            &token,
            measure_payload(victim_id, city_id, offender_id),
        )
    });

    let statuses = join_all(futures).await;

    assert!(statuses.iter().all(|status| *status == StatusCode::CREATED));
}

#[actix_rt::test]
async fn concurrent_work_session_create_same_user_allows_one_success_and_conflicts_rest() {
    let pool = test_helpers::setup_test_db().await;
    test_helpers::clean_database(&pool).await;

    let config = test_helpers::build_test_config();
    let app = test_helpers::create_full_test_app(pool.clone(), config.clone()).await;

    let city_id = db_fixtures::insert_city(&pool, "Cidade Work Session Concorrencia").await;
    let user_id = db_fixtures::insert_user(
        &pool,
        "900001",
        "concurrency.user@test.com",
        "CITY_USER",
        Some(city_id),
    )
    .await;

    let mut claims = test_helpers::build_city_user_claims(city_id);
    claims.id = user_id.to_string();
    let token = test_helpers::generate_jwt(&claims, &config.jwt_secret);

    let futures = (0..5).map(|idx| {
        let payload = serde_json::json!({
            "description": format!("Turno concorrente {}", idx),
            "members": [
                {
                    "user_id": user_id,
                    "function": "Commander"
                }
            ]
        });
        let req = test_helpers::with_auth_headers(
            test::TestRequest::post()
                .uri("/api/v1/work-sessions")
                .set_json(&payload),
            &config,
            &token,
        )
        .to_request();
        test::call_service(&app, req)
    });

    let responses = join_all(futures).await;
    let statuses: Vec<StatusCode> = responses.into_iter().map(|resp| resp.status()).collect();
    let created = statuses
        .iter()
        .filter(|status| **status == StatusCode::CREATED)
        .count();
    let conflicts = statuses
        .iter()
        .filter(|status| **status == StatusCode::CONFLICT)
        .count();

    assert_eq!(created, 1, "statuses: {:?}", statuses);
    assert_eq!(conflicts, 4, "statuses: {:?}", statuses);
}
