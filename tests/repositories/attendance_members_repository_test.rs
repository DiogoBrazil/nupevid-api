//! Phase 5 — attendance_members repository tests.
//!
//! Foco (plano §7):
//! - inserção em lote preserva ordem (ordenação por created_at)
//! - `get_victim_attendance_members` retorna vazio para attendance sem membros
//! - Duplicate add → `RepositoryError::DuplicateEntry`
//! - remove existente funciona; remove ausente retorna NotFound

use chrono::{NaiveDate, NaiveTime};
use nupevid_api::core::contracts::repository::attendance_members::AttendanceMemberRepository;
use nupevid_api::core::contracts::repository::error::RepositoryError;
use nupevid_api::repositories::attendance_members::PgAttendanceMemberRepository;
use sqlx::PgPool;
use uuid::Uuid;

use crate::common::{db_fixtures, test_helpers};

async fn insert_attendance_victim(
    pool: &PgPool,
    victim_id: Uuid,
    offender_id: Uuid,
    protective_measure_id: Uuid,
) -> Uuid {
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
    .bind(NaiveDate::from_ymd_opt(2025, 1, 1).unwrap())
    .bind(NaiveTime::from_hms_opt(9, 0, 0).unwrap())
    .bind(Option::<String>::None)
    .bind(Option::<f64>::None)
    .bind(Option::<f64>::None)
    .bind(Some(offender_id))
    .bind(protective_measure_id)
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
    .expect("failed to insert attendance_victim");
    id
}

async fn insert_attendance_offender(
    pool: &PgPool,
    offender_id: Uuid,
    victim_id: Uuid,
    protective_measure_id: Uuid,
) -> Uuid {
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
    .bind(protective_measure_id)
    .bind(true)
    .bind(NaiveDate::from_ymd_opt(2025, 1, 2).unwrap())
    .bind(NaiveTime::from_hms_opt(10, 0, 0).unwrap())
    .bind(false)
    .bind(false)
    .bind("AlcoholUse")
    .bind(Option::<String>::None)
    .bind(Option::<String>::None)
    .execute(pool)
    .await
    .expect("failed to insert attendance_offender");
    id
}

async fn seed_attendance_victim(pool: &PgPool, city_prefix: &str) -> (Uuid, Uuid) {
    let city = db_fixtures::insert_city(pool, &format!("{} City", city_prefix)).await;
    let victim = db_fixtures::insert_victim(pool, "Victim", city).await;
    let offender = db_fixtures::insert_offender(pool, "Offender", city).await;
    let pm = db_fixtures::insert_protective_measure(pool, victim, offender, city, "Valid").await;
    let attendance = insert_attendance_victim(pool, victim, offender, pm).await;
    (attendance, city)
}

async fn seed_attendance_offender(pool: &PgPool, city_prefix: &str) -> (Uuid, Uuid) {
    let city = db_fixtures::insert_city(pool, &format!("{} City", city_prefix)).await;
    let victim = db_fixtures::insert_victim(pool, "Victim", city).await;
    let offender = db_fixtures::insert_offender(pool, "Offender", city).await;
    let pm = db_fixtures::insert_protective_measure(pool, victim, offender, city, "Valid").await;
    let attendance = insert_attendance_offender(pool, offender, victim, pm).await;
    (attendance, city)
}

#[actix_rt::test]
async fn get_victim_attendance_members_returns_empty_for_attendance_without_members() {
    let pool = test_helpers::setup_test_db().await;
    test_helpers::clean_database(&pool).await;
    let repo = PgAttendanceMemberRepository::new(pool.clone());

    let (attendance_id, _) = seed_attendance_victim(&pool, "AttM1").await;

    let members = repo
        .get_victim_attendance_members(attendance_id)
        .await
        .expect("query should succeed");

    assert!(
        members.is_empty(),
        "empty attendance should return no members"
    );
}

#[actix_rt::test]
async fn add_member_to_victim_attendance_succeeds_and_is_listed() {
    let pool = test_helpers::setup_test_db().await;
    test_helpers::clean_database(&pool).await;
    let repo = PgAttendanceMemberRepository::new(pool.clone());

    let (attendance_id, city) = seed_attendance_victim(&pool, "AttM2").await;
    let user_id = db_fixtures::insert_user(
        &pool,
        "600001",
        "att-member@test.com",
        "CITY_USER",
        Some(city),
    )
    .await;

    let saved = repo
        .add_member_to_victim_attendance(attendance_id, user_id, None)
        .await
        .expect("add should succeed");
    assert_eq!(saved.attendance_victim_id, attendance_id);
    assert_eq!(saved.user_id, user_id);

    let members = repo
        .get_victim_attendance_members(attendance_id)
        .await
        .expect("query should succeed");
    assert_eq!(members.len(), 1);
    assert_eq!(members[0].user_id, user_id);
}

#[actix_rt::test]
async fn add_member_to_victim_attendance_twice_returns_duplicate_entry() {
    let pool = test_helpers::setup_test_db().await;
    test_helpers::clean_database(&pool).await;
    let repo = PgAttendanceMemberRepository::new(pool.clone());

    let (attendance_id, city) = seed_attendance_victim(&pool, "AttM3").await;
    let user_id =
        db_fixtures::insert_user(&pool, "600002", "att-dup@test.com", "CITY_USER", Some(city))
            .await;

    repo.add_member_to_victim_attendance(attendance_id, user_id, None)
        .await
        .expect("first add should succeed");

    let err = repo
        .add_member_to_victim_attendance(attendance_id, user_id, None)
        .await
        .expect_err("second add should fail");

    assert!(
        matches!(err, RepositoryError::DuplicateEntry(_)),
        "duplicate add should map to DuplicateEntry, got {:?}",
        err
    );
}

#[actix_rt::test]
async fn batch_inserts_preserve_creation_order_on_victim_attendance() {
    let pool = test_helpers::setup_test_db().await;
    test_helpers::clean_database(&pool).await;
    let repo = PgAttendanceMemberRepository::new(pool.clone());

    let (attendance_id, city) = seed_attendance_victim(&pool, "AttM4").await;
    let user_a =
        db_fixtures::insert_user(&pool, "600003", "att-a@test.com", "CITY_USER", Some(city)).await;
    let user_b =
        db_fixtures::insert_user(&pool, "600004", "att-b@test.com", "CITY_USER", Some(city)).await;
    let user_c =
        db_fixtures::insert_user(&pool, "600005", "att-c@test.com", "CITY_USER", Some(city)).await;

    // Insert in order: A, B, C
    for user_id in [user_a, user_b, user_c] {
        repo.add_member_to_victim_attendance(attendance_id, user_id, None)
            .await
            .expect("add should succeed");
    }

    let members = repo
        .get_victim_attendance_members(attendance_id)
        .await
        .expect("query should succeed");

    assert_eq!(members.len(), 3);
    let ordered_ids: Vec<Uuid> = members.iter().map(|m| m.user_id).collect();
    assert_eq!(
        ordered_ids,
        vec![user_a, user_b, user_c],
        "returned members should follow insertion order"
    );
}

#[actix_rt::test]
async fn remove_member_from_victim_attendance_succeeds_for_existing_member() {
    let pool = test_helpers::setup_test_db().await;
    test_helpers::clean_database(&pool).await;
    let repo = PgAttendanceMemberRepository::new(pool.clone());

    let (attendance_id, city) = seed_attendance_victim(&pool, "AttM5").await;
    let user_id =
        db_fixtures::insert_user(&pool, "600006", "att-rm@test.com", "CITY_USER", Some(city)).await;

    repo.add_member_to_victim_attendance(attendance_id, user_id, None)
        .await
        .expect("add should succeed");

    let removed = repo
        .remove_member_from_victim_attendance(attendance_id, user_id)
        .await
        .expect("remove should succeed");
    assert_eq!(removed.user_id, user_id);

    let members = repo
        .get_victim_attendance_members(attendance_id)
        .await
        .expect("query should succeed");
    assert!(members.is_empty(), "member should be gone after remove");
}

#[actix_rt::test]
async fn remove_member_from_victim_attendance_returns_not_found_for_absent_member() {
    let pool = test_helpers::setup_test_db().await;
    test_helpers::clean_database(&pool).await;
    let repo = PgAttendanceMemberRepository::new(pool.clone());

    let (attendance_id, city) = seed_attendance_victim(&pool, "AttM6").await;
    let user_id = db_fixtures::insert_user(
        &pool,
        "600007",
        "att-absent@test.com",
        "CITY_USER",
        Some(city),
    )
    .await;

    let err = repo
        .remove_member_from_victim_attendance(attendance_id, user_id)
        .await
        .expect_err("remove of absent member must fail");

    assert!(
        matches!(err, RepositoryError::NotFound),
        "expected NotFound, got {:?}",
        err
    );
}

#[actix_rt::test]
async fn offender_attendance_members_behave_analogously() {
    let pool = test_helpers::setup_test_db().await;
    test_helpers::clean_database(&pool).await;
    let repo = PgAttendanceMemberRepository::new(pool.clone());

    let (attendance_id, city) = seed_attendance_offender(&pool, "AttM7").await;

    // Empty initially
    let members = repo
        .get_offender_attendance_members(attendance_id)
        .await
        .expect("query should succeed");
    assert!(members.is_empty());

    let user_id =
        db_fixtures::insert_user(&pool, "600008", "att-off@test.com", "CITY_USER", Some(city))
            .await;

    let saved = repo
        .add_member_to_offender_attendance(attendance_id, user_id, None)
        .await
        .expect("add should succeed");
    assert_eq!(saved.user_id, user_id);

    let err = repo
        .add_member_to_offender_attendance(attendance_id, user_id, None)
        .await
        .expect_err("duplicate add must fail");
    assert!(matches!(err, RepositoryError::DuplicateEntry(_)));

    let removed = repo
        .remove_member_from_offender_attendance(attendance_id, user_id)
        .await
        .expect("remove should succeed");
    assert_eq!(removed.user_id, user_id);
}
