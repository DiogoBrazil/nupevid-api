//! Phase 5 — users repository tests.
//!
//! Foco (plano §7):
//! - `update_user_policies_by_id` + `get_user_policies_by_id` round-trip
//!   (append + remove policy city).
//! - `get_users_paginated` / `count_users` contratam o filtro por cidades.
//! - `check_user_exists_by_email` e `check_email_exists_for_other_user`.

use nupevid_api::core::contracts::repository::users::UserRepository;
use nupevid_api::core::value_objects::policies::{PermissionPolicies, Policy};
use nupevid_api::repositories::users::PgUserRepository;
use uuid::Uuid;

use crate::common::{db_fixtures, test_helpers};

#[actix_rt::test]
async fn update_user_policies_append_then_remove_round_trip() {
    let pool = test_helpers::setup_test_db().await;
    test_helpers::clean_database(&pool).await;
    let repo = PgUserRepository::new(pool.clone());

    let city_a = db_fixtures::insert_city(&pool, "Users Repo City A").await;
    let city_b = db_fixtures::insert_city(&pool, "Users Repo City B").await;
    let user_id = db_fixtures::insert_user(
        &pool,
        "700001",
        "users-repo-rt@test.com",
        "CITY_USER",
        Some(city_a),
    )
    .await;

    // Append: policy with [city_a, city_b]
    let mut policies: PermissionPolicies = PermissionPolicies::new();
    policies.insert(Policy::ReadVictims, vec![city_a, city_b]);
    repo.update_user_policies_by_id(user_id, policies.clone())
        .await
        .expect("update should succeed");

    let after_append = repo
        .get_user_policies_by_id(user_id)
        .await
        .expect("get should succeed");
    let cities = after_append
        .get(&Policy::ReadVictims)
        .expect("policy should be present");
    assert!(cities.contains(&city_a));
    assert!(cities.contains(&city_b));
    assert_eq!(cities.len(), 2);

    // Remove city_b from the policy
    let mut reduced: PermissionPolicies = PermissionPolicies::new();
    reduced.insert(Policy::ReadVictims, vec![city_a]);
    repo.update_user_policies_by_id(user_id, reduced)
        .await
        .expect("update should succeed");

    let after_remove = repo
        .get_user_policies_by_id(user_id)
        .await
        .expect("get should succeed");
    let cities = after_remove
        .get(&Policy::ReadVictims)
        .expect("policy should still be present");
    assert_eq!(cities, &vec![city_a]);
}

#[actix_rt::test]
async fn get_user_policies_by_id_returns_empty_map_for_user_without_policies() {
    let pool = test_helpers::setup_test_db().await;
    test_helpers::clean_database(&pool).await;
    let repo = PgUserRepository::new(pool.clone());

    let city = db_fixtures::insert_city(&pool, "Users Repo City Empty").await;
    let user_id = db_fixtures::insert_user(
        &pool,
        "700002",
        "users-repo-empty@test.com",
        "CITY_USER",
        Some(city),
    )
    .await;

    // Overwrite any defaults with an empty map
    repo.update_user_policies_by_id(user_id, PermissionPolicies::new())
        .await
        .expect("update should succeed");

    let policies = repo
        .get_user_policies_by_id(user_id)
        .await
        .expect("get should succeed");
    assert!(policies.is_empty());
}

#[actix_rt::test]
async fn get_users_paginated_filters_by_allowed_cities() {
    let pool = test_helpers::setup_test_db().await;
    test_helpers::clean_database(&pool).await;
    let repo = PgUserRepository::new(pool.clone());

    let city_a = db_fixtures::insert_city(&pool, "Users Paginated City A").await;
    let city_b = db_fixtures::insert_city(&pool, "Users Paginated City B").await;

    let user_a = db_fixtures::insert_user(
        &pool,
        "700101",
        "users-a@test.com",
        "CITY_USER",
        Some(city_a),
    )
    .await;
    let _user_b = db_fixtures::insert_user(
        &pool,
        "700102",
        "users-b@test.com",
        "CITY_USER",
        Some(city_b),
    )
    .await;

    let allowed = [city_a];
    let page = repo
        .get_users_paginated(Some(&allowed), true, 20, 0)
        .await
        .expect("query should succeed");

    let ids: Vec<Uuid> = page.iter().map(|u| u.id).collect();
    assert!(ids.contains(&user_a), "user in allowed city should appear");
    assert!(
        page.iter().all(|u| u.city_id == Some(city_a)),
        "only users from city_a should be returned"
    );

    let count = repo
        .count_users(Some(&allowed), true)
        .await
        .expect("count should succeed");
    assert_eq!(count, page.len() as i64);
}

#[actix_rt::test]
async fn count_users_exclude_root_filters_root_profile() {
    let pool = test_helpers::setup_test_db().await;
    test_helpers::clean_database(&pool).await;
    let repo = PgUserRepository::new(pool.clone());

    let city = db_fixtures::insert_city(&pool, "Users Count City").await;
    // Root users have no city
    db_fixtures::insert_user(&pool, "700201", "root-repo@test.com", "ROOT", None).await;
    db_fixtures::insert_user(
        &pool,
        "700202",
        "city-user-repo@test.com",
        "CITY_USER",
        Some(city),
    )
    .await;

    let count_all = repo
        .count_users(None, false)
        .await
        .expect("count should succeed");
    assert_eq!(count_all, 2);

    let count_no_root = repo
        .count_users(None, true)
        .await
        .expect("count should succeed");
    assert_eq!(count_no_root, 1, "exclude_root=true must drop ROOT profile");
}

#[actix_rt::test]
async fn check_user_exists_by_email_is_case_sensitive_and_precise() {
    let pool = test_helpers::setup_test_db().await;
    test_helpers::clean_database(&pool).await;
    let repo = PgUserRepository::new(pool.clone());

    let city = db_fixtures::insert_city(&pool, "Users Email City").await;
    let email = "present-user@test.com";
    db_fixtures::insert_user(&pool, "700301", email, "CITY_USER", Some(city)).await;

    let exists = repo
        .check_user_exists_by_email(email)
        .await
        .expect("query should succeed");
    assert!(exists);

    let missing = repo
        .check_user_exists_by_email("absent-user@test.com")
        .await
        .expect("query should succeed");
    assert!(!missing);
}

#[actix_rt::test]
async fn check_email_exists_for_other_user_excludes_the_queried_id() {
    let pool = test_helpers::setup_test_db().await;
    test_helpers::clean_database(&pool).await;
    let repo = PgUserRepository::new(pool.clone());

    let city = db_fixtures::insert_city(&pool, "Users Email Other City").await;
    let email = "shared@test.com";
    let user_id = db_fixtures::insert_user(&pool, "700401", email, "CITY_USER", Some(city)).await;

    // Same user with same email must not be considered a "other user" conflict
    let conflict_self = repo
        .check_email_exists_for_other_user(email, user_id)
        .await
        .expect("query should succeed");
    assert!(
        !conflict_self,
        "own user id must be excluded from uniqueness check"
    );

    // Different id, same email → conflict detected
    let conflict_other = repo
        .check_email_exists_for_other_user(email, Uuid::new_v4())
        .await
        .expect("query should succeed");
    assert!(
        conflict_other,
        "email already in use by a different user must be flagged"
    );
}
