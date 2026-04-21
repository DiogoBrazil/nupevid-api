//! Phase 5 — protective_measures repository tests.
//!
//! Foco: `check_active_measure_exists_for_victim(victim_id, offender_id, exclude_measure_id)`
//! em todos os ângulos do plano (§7): par ativo existente/inexistente, exclusão
//! do próprio id, soft-deleted não conta, par diferente não conta.

use nupevid_api::core::contracts::repository::protective_measures::ProtectiveMeasureReadRepository;
use nupevid_api::repositories::protective_measures::PgProtectiveMeasureRepository;
use sqlx::PgPool;
use uuid::Uuid;

use crate::common::{db_fixtures, test_helpers};

async fn soft_delete_measure(pool: &PgPool, id: Uuid) {
    sqlx::query("UPDATE protective_measures SET is_deleted = true WHERE id = $1")
        .bind(id)
        .execute(pool)
        .await
        .expect("failed to soft-delete protective measure");
}

#[actix_rt::test]
async fn check_active_returns_true_when_valid_measure_exists_for_pair() {
    let pool = test_helpers::setup_test_db().await;
    test_helpers::clean_database(&pool).await;
    let repo = PgProtectiveMeasureRepository::new(pool.clone());

    let city = db_fixtures::insert_city(&pool, "PM Repo City 1").await;
    let victim = db_fixtures::insert_victim(&pool, "Victim 1", city).await;
    let offender = db_fixtures::insert_offender(&pool, "Offender 1", city).await;
    db_fixtures::insert_protective_measure(&pool, victim, offender, city, "Valid").await;

    let exists = repo
        .check_active_measure_exists_for_victim(victim, offender, None)
        .await
        .expect("query should succeed");

    assert!(exists);
}

#[actix_rt::test]
async fn check_active_returns_false_when_no_measure_for_pair() {
    let pool = test_helpers::setup_test_db().await;
    test_helpers::clean_database(&pool).await;
    let repo = PgProtectiveMeasureRepository::new(pool.clone());

    let city = db_fixtures::insert_city(&pool, "PM Repo City 2").await;
    let victim = db_fixtures::insert_victim(&pool, "Victim 2", city).await;
    let offender = db_fixtures::insert_offender(&pool, "Offender 2", city).await;

    let exists = repo
        .check_active_measure_exists_for_victim(victim, offender, None)
        .await
        .expect("query should succeed");

    assert!(!exists);
}

#[actix_rt::test]
async fn check_active_returns_false_when_measure_is_revoked() {
    let pool = test_helpers::setup_test_db().await;
    test_helpers::clean_database(&pool).await;
    let repo = PgProtectiveMeasureRepository::new(pool.clone());

    let city = db_fixtures::insert_city(&pool, "PM Repo City 3").await;
    let victim = db_fixtures::insert_victim(&pool, "Victim 3", city).await;
    let offender = db_fixtures::insert_offender(&pool, "Offender 3", city).await;
    db_fixtures::insert_protective_measure(&pool, victim, offender, city, "Revoked").await;

    let exists = repo
        .check_active_measure_exists_for_victim(victim, offender, None)
        .await
        .expect("query should succeed");

    assert!(!exists, "Revoked status should not be considered active");
}

#[actix_rt::test]
async fn check_active_returns_false_when_measure_is_soft_deleted() {
    let pool = test_helpers::setup_test_db().await;
    test_helpers::clean_database(&pool).await;
    let repo = PgProtectiveMeasureRepository::new(pool.clone());

    let city = db_fixtures::insert_city(&pool, "PM Repo City 4").await;
    let victim = db_fixtures::insert_victim(&pool, "Victim 4", city).await;
    let offender = db_fixtures::insert_offender(&pool, "Offender 4", city).await;
    let pm_id =
        db_fixtures::insert_protective_measure(&pool, victim, offender, city, "Valid").await;
    soft_delete_measure(&pool, pm_id).await;

    let exists = repo
        .check_active_measure_exists_for_victim(victim, offender, None)
        .await
        .expect("query should succeed");

    assert!(!exists, "Soft-deleted measure should not count as active");
}

#[actix_rt::test]
async fn check_active_excludes_measure_by_id_when_provided() {
    let pool = test_helpers::setup_test_db().await;
    test_helpers::clean_database(&pool).await;
    let repo = PgProtectiveMeasureRepository::new(pool.clone());

    let city = db_fixtures::insert_city(&pool, "PM Repo City 5").await;
    let victim = db_fixtures::insert_victim(&pool, "Victim 5", city).await;
    let offender = db_fixtures::insert_offender(&pool, "Offender 5", city).await;
    let pm_id =
        db_fixtures::insert_protective_measure(&pool, victim, offender, city, "Valid").await;

    // Without exclude → exists
    let exists = repo
        .check_active_measure_exists_for_victim(victim, offender, None)
        .await
        .expect("query should succeed");
    assert!(exists);

    // With exclude of the same measure → reported as not existing
    let exists_excluded = repo
        .check_active_measure_exists_for_victim(victim, offender, Some(pm_id))
        .await
        .expect("query should succeed");
    assert!(
        !exists_excluded,
        "Excluding the only active measure should report no active measure for the pair"
    );
}

#[actix_rt::test]
async fn check_active_returns_false_for_different_offender_same_victim() {
    let pool = test_helpers::setup_test_db().await;
    test_helpers::clean_database(&pool).await;
    let repo = PgProtectiveMeasureRepository::new(pool.clone());

    let city = db_fixtures::insert_city(&pool, "PM Repo City 6").await;
    let victim = db_fixtures::insert_victim(&pool, "Victim 6", city).await;
    let offender_a = db_fixtures::insert_offender(&pool, "Offender 6A", city).await;
    let offender_b = db_fixtures::insert_offender(&pool, "Offender 6B", city).await;
    db_fixtures::insert_protective_measure(&pool, victim, offender_a, city, "Valid").await;

    let exists = repo
        .check_active_measure_exists_for_victim(victim, offender_b, None)
        .await
        .expect("query should succeed");

    assert!(
        !exists,
        "Active measure for (victim, offender_a) must NOT count for (victim, offender_b)"
    );
}

#[actix_rt::test]
async fn check_active_returns_false_for_different_victim_same_offender() {
    let pool = test_helpers::setup_test_db().await;
    test_helpers::clean_database(&pool).await;
    let repo = PgProtectiveMeasureRepository::new(pool.clone());

    let city = db_fixtures::insert_city(&pool, "PM Repo City 7").await;
    let victim_a = db_fixtures::insert_victim(&pool, "Victim 7A", city).await;
    let victim_b = db_fixtures::insert_victim(&pool, "Victim 7B", city).await;
    let offender = db_fixtures::insert_offender(&pool, "Offender 7", city).await;
    db_fixtures::insert_protective_measure(&pool, victim_a, offender, city, "Valid").await;

    let exists = repo
        .check_active_measure_exists_for_victim(victim_b, offender, None)
        .await
        .expect("query should succeed");

    assert!(
        !exists,
        "Active measure for (victim_a, offender) must NOT count for (victim_b, offender)"
    );
}

#[actix_rt::test]
async fn get_protective_measures_by_victim_omits_soft_deleted() {
    let pool = test_helpers::setup_test_db().await;
    test_helpers::clean_database(&pool).await;
    let repo = PgProtectiveMeasureRepository::new(pool.clone());

    let city = db_fixtures::insert_city(&pool, "PM Repo City 8").await;
    let victim = db_fixtures::insert_victim(&pool, "Victim 8", city).await;
    let offender_a = db_fixtures::insert_offender(&pool, "Offender 8A", city).await;
    let offender_b = db_fixtures::insert_offender(&pool, "Offender 8B", city).await;

    let pm_keep =
        db_fixtures::insert_protective_measure(&pool, victim, offender_a, city, "Revoked").await;
    let pm_drop =
        db_fixtures::insert_protective_measure(&pool, victim, offender_b, city, "Revoked").await;
    soft_delete_measure(&pool, pm_drop).await;

    let measures = repo
        .get_protective_measures_by_victim(victim)
        .await
        .expect("query should succeed");

    let ids: Vec<Uuid> = measures.iter().map(|m| m.id).collect();
    assert!(
        ids.contains(&pm_keep),
        "non-deleted measure should be returned"
    );
    assert!(
        !ids.contains(&pm_drop),
        "soft-deleted measure must not be returned"
    );
}
