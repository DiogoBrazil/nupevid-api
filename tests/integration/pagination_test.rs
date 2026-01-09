use actix_web::{http::StatusCode, test};
use chrono::{NaiveDate, NaiveTime};
use sqlx::PgPool;
use uuid::Uuid;

use crate::common::{db_fixtures, test_helpers};

async fn insert_attendance_victim(pool: &PgPool, victim_id: Uuid) -> Uuid {
    let id = Uuid::new_v4();
    sqlx::query(
        "INSERT INTO attendance_victims (
            id, victim_id, was_victim_present, attendance_date, attendance_time, description,
            latitude, longitude, offender_id, protective_measure_id, is_remote, risk_level,
            offender_freedom_status, offender_has_firearm_access, needs_legal_assistance,
            needs_psychological_support, was_instructed_about_protective_measure_procedures,
            offender_violated_protective_measure, is_deleted
        ) VALUES (
            $1, $2, $3, $4, $5, $6,
            $7, $8, $9, $10, $11, $12::risk_level,
            $13::offender_freedom_status, $14::offender_firearm_access, $15,
            $16, $17, $18, false
        )",
    )
    .bind(id)
    .bind(victim_id)
    .bind(true)
    .bind(NaiveDate::from_ymd_opt(2025, 1, 1).expect("valid date"))
    .bind(NaiveTime::from_hms_opt(9, 0, 0).expect("valid time"))
    .bind(Some("Attendance test".to_string()))
    .bind(Some(1.0_f64))
    .bind(Some(1.0_f64))
    .bind(Option::<Uuid>::None)
    .bind(Option::<Uuid>::None)
    .bind(false)
    .bind("Low")
    .bind("Free")
    .bind("Unknown")
    .bind(false)
    .bind(false)
    .bind(false)
    .bind(false)
    .execute(pool)
    .await
    .expect("Failed to insert attendance victim");
    id
}

async fn insert_attendance_offender(pool: &PgPool, offender_id: Uuid, victim_id: Uuid) -> Uuid {
    let id = Uuid::new_v4();
    sqlx::query(
        "INSERT INTO attendance_offenders (
            id, offender_id, victim_id, protective_measure_id, was_offender_present,
            attendance_date, attendance_time, is_remote, assaults_children,
            violence_aggravator, violence_aggravator_other, description, is_deleted
        ) VALUES (
            $1, $2, $3, $4, $5,
            $6, $7, $8, $9,
            $10::violence_aggravator_enum, $11, $12, false
        )",
    )
    .bind(id)
    .bind(offender_id)
    .bind(victim_id)
    .bind(Option::<Uuid>::None)
    .bind(true)
    .bind(NaiveDate::from_ymd_opt(2025, 1, 2).expect("valid date"))
    .bind(NaiveTime::from_hms_opt(10, 0, 0).expect("valid time"))
    .bind(false)
    .bind(false)
    .bind("AlcoholUse")
    .bind(Option::<String>::None)
    .bind(Some("Attendance offender test".to_string()))
    .execute(pool)
    .await
    .expect("Failed to insert attendance offender");
    id
}

#[actix_rt::test]
async fn cities_list_pagination_clamps_page_size() {
    let pool = test_helpers::setup_test_db().await;
    test_helpers::clean_database(&pool).await;

    let config = test_helpers::build_test_config();
    let app = test_helpers::create_full_test_app(pool.clone(), config.clone()).await;

    db_fixtures::insert_city(&pool, "City A").await;
    db_fixtures::insert_city(&pool, "City B").await;
    db_fixtures::insert_city(&pool, "City C").await;

    let root_claims = test_helpers::build_root_claims();
    let root_token = test_helpers::generate_jwt(&root_claims, &config.jwt_secret);
    let req = test_helpers::with_auth_headers(
        test::TestRequest::get()
            .uri("/api/v1/cities?page=1&page_size=500"),
        &config,
        &root_token,
    )
    .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), StatusCode::OK);
    let body: serde_json::Value = test::read_body_json(resp).await;
    assert_eq!(body["page"].as_i64(), Some(1));
    assert_eq!(body["page_size"].as_i64(), Some(200));
    assert_eq!(body["total_items"].as_i64(), Some(3));
    assert_eq!(body["total_pages"].as_i64(), Some(1));
    assert_eq!(body["data"].as_array().unwrap().len(), 3);
}

#[actix_rt::test]
async fn users_list_pagination_returns_metadata() {
    let pool = test_helpers::setup_test_db().await;
    test_helpers::clean_database(&pool).await;

    let config = test_helpers::build_test_config();
    let app = test_helpers::create_full_test_app(pool.clone(), config.clone()).await;

    let city_id = db_fixtures::insert_city(&pool, "User City").await;
    db_fixtures::insert_user(&pool, "200001", "user1@test.com", "CITY_ADMIN", Some(city_id)).await;
    db_fixtures::insert_user(&pool, "200002", "user2@test.com", "CITY_USER", Some(city_id)).await;

    let root_claims = test_helpers::build_root_claims();
    let root_token = test_helpers::generate_jwt(&root_claims, &config.jwt_secret);
    let req = test_helpers::with_auth_headers(
        test::TestRequest::get()
            .uri("/api/v1/users?page=1&page_size=1"),
        &config,
        &root_token,
    )
    .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), StatusCode::OK);
    let body: serde_json::Value = test::read_body_json(resp).await;
    assert_eq!(body["page"].as_i64(), Some(1));
    assert_eq!(body["page_size"].as_i64(), Some(1));
    assert_eq!(body["total_items"].as_i64(), Some(2));
    assert_eq!(body["total_pages"].as_i64(), Some(2));
    assert_eq!(body["data"].as_array().unwrap().len(), 1);
}

#[actix_rt::test]
async fn victims_list_pagination_returns_metadata() {
    let pool = test_helpers::setup_test_db().await;
    test_helpers::clean_database(&pool).await;

    let config = test_helpers::build_test_config();
    let app = test_helpers::create_full_test_app(pool.clone(), config.clone()).await;

    let city_id = db_fixtures::insert_city(&pool, "Victim City").await;
    db_fixtures::insert_victim(&pool, "Victim A", city_id).await;
    db_fixtures::insert_victim(&pool, "Victim B", city_id).await;

    let root_claims = test_helpers::build_root_claims();
    let root_token = test_helpers::generate_jwt(&root_claims, &config.jwt_secret);
    let req = test_helpers::with_auth_headers(
        test::TestRequest::get()
            .uri("/api/v1/victims?page=1&page_size=1"),
        &config,
        &root_token,
    )
    .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), StatusCode::OK);
    let body: serde_json::Value = test::read_body_json(resp).await;
    assert_eq!(body["page"].as_i64(), Some(1));
    assert_eq!(body["page_size"].as_i64(), Some(1));
    assert_eq!(body["total_items"].as_i64(), Some(2));
    assert_eq!(body["total_pages"].as_i64(), Some(2));
    assert_eq!(body["data"].as_array().unwrap().len(), 1);
}

#[actix_rt::test]
async fn offenders_list_pagination_returns_metadata() {
    let pool = test_helpers::setup_test_db().await;
    test_helpers::clean_database(&pool).await;

    let config = test_helpers::build_test_config();
    let app = test_helpers::create_full_test_app(pool.clone(), config.clone()).await;

    let city_id = db_fixtures::insert_city(&pool, "Offender City").await;
    db_fixtures::insert_offender(&pool, "Offender A", city_id).await;
    db_fixtures::insert_offender(&pool, "Offender B", city_id).await;

    let root_claims = test_helpers::build_root_claims();
    let root_token = test_helpers::generate_jwt(&root_claims, &config.jwt_secret);
    let req = test_helpers::with_auth_headers(
        test::TestRequest::get()
            .uri("/api/v1/offenders?page=1&page_size=1"),
        &config,
        &root_token,
    )
    .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), StatusCode::OK);
    let body: serde_json::Value = test::read_body_json(resp).await;
    assert_eq!(body["page"].as_i64(), Some(1));
    assert_eq!(body["page_size"].as_i64(), Some(1));
    assert_eq!(body["total_items"].as_i64(), Some(2));
    assert_eq!(body["total_pages"].as_i64(), Some(2));
    assert_eq!(body["data"].as_array().unwrap().len(), 1);
}

#[actix_rt::test]
async fn protective_measures_list_pagination_returns_metadata() {
    let pool = test_helpers::setup_test_db().await;
    test_helpers::clean_database(&pool).await;

    let config = test_helpers::build_test_config();
    let app = test_helpers::create_full_test_app(pool.clone(), config.clone()).await;

    let city_id = db_fixtures::insert_city(&pool, "Measure City").await;
    let victim_a = db_fixtures::insert_victim(&pool, "Victim A", city_id).await;
    let victim_b = db_fixtures::insert_victim(&pool, "Victim B", city_id).await;
    let offender_a = db_fixtures::insert_offender(&pool, "Offender A", city_id).await;
    let offender_b = db_fixtures::insert_offender(&pool, "Offender B", city_id).await;

    db_fixtures::insert_protective_measure(&pool, victim_a, offender_a, city_id, "Valid").await;
    db_fixtures::insert_protective_measure(&pool, victim_b, offender_b, city_id, "Revoked").await;

    let root_claims = test_helpers::build_root_claims();
    let root_token = test_helpers::generate_jwt(&root_claims, &config.jwt_secret);
    let req = test_helpers::with_auth_headers(
        test::TestRequest::get()
            .uri("/api/v1/protective-measures?page=1&page_size=1"),
        &config,
        &root_token,
    )
    .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), StatusCode::OK);
    let body: serde_json::Value = test::read_body_json(resp).await;
    assert_eq!(body["page"].as_i64(), Some(1));
    assert_eq!(body["page_size"].as_i64(), Some(1));
    assert_eq!(body["total_items"].as_i64(), Some(2));
    assert_eq!(body["total_pages"].as_i64(), Some(2));
    assert_eq!(body["data"].as_array().unwrap().len(), 1);
}

#[actix_rt::test]
async fn attendance_victims_list_pagination_returns_metadata() {
    let pool = test_helpers::setup_test_db().await;
    test_helpers::clean_database(&pool).await;

    let config = test_helpers::build_test_config();
    let app = test_helpers::create_full_test_app(pool.clone(), config.clone()).await;

    let city_id = db_fixtures::insert_city(&pool, "Attendance Victim City").await;
    let victim_id = db_fixtures::insert_victim(&pool, "Victim A", city_id).await;
    insert_attendance_victim(&pool, victim_id).await;
    insert_attendance_victim(&pool, victim_id).await;

    let root_claims = test_helpers::build_root_claims();
    let root_token = test_helpers::generate_jwt(&root_claims, &config.jwt_secret);
    let req = test_helpers::with_auth_headers(
        test::TestRequest::get()
            .uri("/api/v1/attendance-victims?page=1&page_size=1"),
        &config,
        &root_token,
    )
    .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), StatusCode::OK);
    let body: serde_json::Value = test::read_body_json(resp).await;
    assert_eq!(body["page"].as_i64(), Some(1));
    assert_eq!(body["page_size"].as_i64(), Some(1));
    assert_eq!(body["total_items"].as_i64(), Some(2));
    assert_eq!(body["total_pages"].as_i64(), Some(2));
    assert_eq!(body["data"].as_array().unwrap().len(), 1);
}

#[actix_rt::test]
async fn attendance_offenders_list_pagination_returns_metadata() {
    let pool = test_helpers::setup_test_db().await;
    test_helpers::clean_database(&pool).await;

    let config = test_helpers::build_test_config();
    let app = test_helpers::create_full_test_app(pool.clone(), config.clone()).await;

    let city_id = db_fixtures::insert_city(&pool, "Attendance Offender City").await;
    let victim_id = db_fixtures::insert_victim(&pool, "Victim A", city_id).await;
    let offender_id = db_fixtures::insert_offender(&pool, "Offender A", city_id).await;
    insert_attendance_offender(&pool, offender_id, victim_id).await;
    insert_attendance_offender(&pool, offender_id, victim_id).await;

    let root_claims = test_helpers::build_root_claims();
    let root_token = test_helpers::generate_jwt(&root_claims, &config.jwt_secret);
    let req = test_helpers::with_auth_headers(
        test::TestRequest::get()
            .uri("/api/v1/attendance-offenders?page=1&page_size=1"),
        &config,
        &root_token,
    )
    .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), StatusCode::OK);
    let body: serde_json::Value = test::read_body_json(resp).await;
    assert_eq!(body["page"].as_i64(), Some(1));
    assert_eq!(body["page_size"].as_i64(), Some(1));
    assert_eq!(body["total_items"].as_i64(), Some(2));
    assert_eq!(body["total_pages"].as_i64(), Some(2));
    assert_eq!(body["data"].as_array().unwrap().len(), 1);
}

#[actix_rt::test]
async fn work_sessions_list_pagination_returns_metadata() {
    let pool = test_helpers::setup_test_db().await;
    test_helpers::clean_database(&pool).await;

    let config = test_helpers::build_test_config();
    let app = test_helpers::create_full_test_app(pool.clone(), config.clone()).await;

    let city_id = db_fixtures::insert_city(&pool, "Work City").await;
    let user_a = db_fixtures::insert_user(&pool, "300001", "ws1@test.com", "CITY_USER", Some(city_id)).await;
    let user_b = db_fixtures::insert_user(&pool, "300002", "ws2@test.com", "CITY_USER", Some(city_id)).await;
    test_helpers::create_work_session_for_user(&pool, user_a).await;
    test_helpers::create_work_session_for_user(&pool, user_b).await;

    let root_claims = test_helpers::build_root_claims();
    let root_token = test_helpers::generate_jwt(&root_claims, &config.jwt_secret);
    let req = test_helpers::with_auth_headers(
        test::TestRequest::get()
            .uri("/api/v1/work-sessions?page=1&page_size=1"),
        &config,
        &root_token,
    )
    .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), StatusCode::OK);
    let body: serde_json::Value = test::read_body_json(resp).await;
    assert_eq!(body["page"].as_i64(), Some(1));
    assert_eq!(body["page_size"].as_i64(), Some(1));
    assert_eq!(body["total_items"].as_i64(), Some(2));
    assert_eq!(body["total_pages"].as_i64(), Some(2));
    assert_eq!(body["data"].as_array().unwrap().len(), 1);
}
