//! Phase 5 — protective_measures repository tests.
//!
//! Foco: `check_active_measure_exists_for_victim(victim_id, offender_id, exclude_measure_id)`
//! em todos os ângulos do plano (§7): par ativo existente/inexistente, exclusão
//! do próprio id, soft-deleted não conta, par diferente não conta.

use nupevid_api::core::contracts::repository::protective_measures::ProtectiveMeasureReadRepository;
use nupevid_api::repositories::protective_measures::PgProtectiveMeasureRepository;
use sqlx::PgPool;
use uuid::Uuid;

use crate::common::db_fixtures;

async fn soft_delete_measure(pool: &PgPool, id: Uuid) {
    sqlx::query("UPDATE protective_measures SET is_deleted = true WHERE id = $1")
        .bind(id)
        .execute(pool)
        .await
        .expect("failed to soft-delete protective measure");
}

#[sqlx::test]
async fn check_active_returns_true_when_valid_measure_exists_for_pair(pool: PgPool) {
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

#[sqlx::test]
async fn check_active_returns_false_when_no_measure_for_pair(pool: PgPool) {
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

#[sqlx::test]
async fn check_active_returns_false_when_measure_is_revoked(pool: PgPool) {
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

#[sqlx::test]
async fn check_active_returns_false_when_measure_is_soft_deleted(pool: PgPool) {
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

#[sqlx::test]
async fn check_active_excludes_measure_by_id_when_provided(pool: PgPool) {
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

#[sqlx::test]
async fn check_active_returns_false_for_different_offender_same_victim(pool: PgPool) {
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

#[sqlx::test]
async fn check_active_returns_false_for_different_victim_same_offender(pool: PgPool) {
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

#[sqlx::test]
async fn get_protective_measures_by_victim_omits_soft_deleted(pool: PgPool) {
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

/// Seeds three measures in a single city:
/// - m1: (victim_a, offender_x, Valid)
/// - m2: (victim_a, offender_y, Revoked) — same victim, distinct offender (non-active to respect
///   the "one active measure per victim" unique index)
/// - m3: (victim_b, offender_x, Valid) — distinct victim, shares offender_x with m1
///
/// Returns (victim_a, victim_b, offender_x, offender_y, m1, m2, m3).
async fn seed_filter_fixture(
    pool: &PgPool,
    label: &str,
) -> (Uuid, Uuid, Uuid, Uuid, Uuid, Uuid, Uuid) {
    let city = db_fixtures::insert_city(pool, &format!("PM Filter City {label}")).await;
    let victim_a = db_fixtures::insert_victim(pool, &format!("Victim {label} A"), city).await;
    let victim_b = db_fixtures::insert_victim(pool, &format!("Victim {label} B"), city).await;
    let offender_x = db_fixtures::insert_offender(pool, &format!("Offender {label} X"), city).await;
    let offender_y = db_fixtures::insert_offender(pool, &format!("Offender {label} Y"), city).await;

    let m1 = db_fixtures::insert_protective_measure(pool, victim_a, offender_x, city, "Valid").await;
    let m2 =
        db_fixtures::insert_protective_measure(pool, victim_a, offender_y, city, "Revoked").await;
    let m3 = db_fixtures::insert_protective_measure(pool, victim_b, offender_x, city, "Valid").await;

    (victim_a, victim_b, offender_x, offender_y, m1, m2, m3)
}

#[sqlx::test]
async fn paginated_filters_by_victim_id(pool: PgPool) {
    let repo = PgProtectiveMeasureRepository::new(pool.clone());
    let (victim_a, _victim_b, _offender_x, _offender_y, m1, m2, m3) =
        seed_filter_fixture(&pool, "Victim").await;

    let measures = repo
        .get_protective_measures_paginated(None, Some(victim_a), None, 50, 0)
        .await
        .expect("query should succeed");
    let ids: Vec<Uuid> = measures.iter().map(|m| m.id).collect();

    assert_eq!(ids.len(), 2, "only victim_a's two measures should be returned");
    assert!(ids.contains(&m1));
    assert!(ids.contains(&m2));
    assert!(!ids.contains(&m3), "other victim's measure must be excluded");

    let count = repo
        .count_protective_measures(None, Some(victim_a), None)
        .await
        .expect("count should succeed");
    assert_eq!(count, 2);
}

#[sqlx::test]
async fn paginated_filters_by_offender_id(pool: PgPool) {
    let repo = PgProtectiveMeasureRepository::new(pool.clone());
    let (_victim_a, _victim_b, offender_x, _offender_y, m1, m2, m3) =
        seed_filter_fixture(&pool, "Offender").await;

    let measures = repo
        .get_protective_measures_paginated(None, None, Some(offender_x), 50, 0)
        .await
        .expect("query should succeed");
    let ids: Vec<Uuid> = measures.iter().map(|m| m.id).collect();

    assert_eq!(ids.len(), 2, "only offender_x's two measures should be returned");
    assert!(ids.contains(&m1));
    assert!(ids.contains(&m3));
    assert!(!ids.contains(&m2), "other offender's measure must be excluded");

    let count = repo
        .count_protective_measures(None, None, Some(offender_x))
        .await
        .expect("count should succeed");
    assert_eq!(count, 2);
}

#[sqlx::test]
async fn paginated_filters_by_victim_and_offender_combined(pool: PgPool) {
    let repo = PgProtectiveMeasureRepository::new(pool.clone());
    let (victim_a, _victim_b, offender_x, _offender_y, m1, _m2, _m3) =
        seed_filter_fixture(&pool, "Combined").await;

    let measures = repo
        .get_protective_measures_paginated(None, Some(victim_a), Some(offender_x), 50, 0)
        .await
        .expect("query should succeed");
    let ids: Vec<Uuid> = measures.iter().map(|m| m.id).collect();

    assert_eq!(ids, vec![m1], "only the (victim_a, offender_x) measure should match");

    let count = repo
        .count_protective_measures(None, Some(victim_a), Some(offender_x))
        .await
        .expect("count should succeed");
    assert_eq!(count, 1);
}

#[sqlx::test]
async fn paginated_without_filters_returns_all(pool: PgPool) {
    let repo = PgProtectiveMeasureRepository::new(pool.clone());
    let (_victim_a, _victim_b, _offender_x, _offender_y, m1, m2, m3) =
        seed_filter_fixture(&pool, "NoFilter").await;

    let measures = repo
        .get_protective_measures_paginated(None, None, None, 50, 0)
        .await
        .expect("query should succeed");
    let ids: Vec<Uuid> = measures.iter().map(|m| m.id).collect();

    assert!(ids.contains(&m1));
    assert!(ids.contains(&m2));
    assert!(ids.contains(&m3));

    let count = repo
        .count_protective_measures(None, None, None)
        .await
        .expect("count should succeed");
    assert_eq!(count, 3);
}

#[sqlx::test]
async fn paginated_filter_combines_with_city_scope(pool: PgPool) {
    let repo = PgProtectiveMeasureRepository::new(pool.clone());

    let city = db_fixtures::insert_city(&pool, "PM Filter City Scope").await;
    let victim_a = db_fixtures::insert_victim(&pool, "Scope Victim A", city).await;
    let victim_b = db_fixtures::insert_victim(&pool, "Scope Victim B", city).await;
    let offender = db_fixtures::insert_offender(&pool, "Scope Offender", city).await;

    let m_a = db_fixtures::insert_protective_measure(&pool, victim_a, offender, city, "Valid").await;
    let _m_b =
        db_fixtures::insert_protective_measure(&pool, victim_b, offender, city, "Valid").await;

    // City scope active (BY_CITIES branch) + victim filter applied on top.
    let measures = repo
        .get_protective_measures_paginated(Some(&[city]), Some(victim_a), None, 50, 0)
        .await
        .expect("query should succeed");
    let ids: Vec<Uuid> = measures.iter().map(|m| m.id).collect();
    assert_eq!(ids, vec![m_a], "only victim_a's measure within the allowed city");

    let count = repo
        .count_protective_measures(Some(&[city]), Some(victim_a), None)
        .await
        .expect("count should succeed");
    assert_eq!(count, 1);
}
