//! Phase 5 — work_sessions repository tests.
//!
//! Foco: `get_active_session_by_user` / `is_user_in_active_session` / `end_session`,
//! incluindo os ângulos do plano (§7): retorna NotFound quando não há sessão
//! ativa; retorna a sessão correta quando há histórico + sessão ativa.

use nupevid_api::core::contracts::repository::error::RepositoryError;
use nupevid_api::core::contracts::repository::work_sessions::{
    WorkSessionReadRepository, WorkSessionWriteRepository,
};
use nupevid_api::repositories::work_sessions::PgWorkSessionRepository;

use crate::common::{db_fixtures, test_helpers};

#[actix_rt::test]
async fn get_active_session_by_user_returns_not_found_when_user_has_no_session() {
    let pool = test_helpers::setup_test_db().await;
    test_helpers::clean_database(&pool).await;
    let repo = PgWorkSessionRepository::new(pool.clone());

    let city = db_fixtures::insert_city(&pool, "WS Repo City 1").await;
    let user_id = db_fixtures::insert_user(
        &pool,
        "500001",
        "ws-repo-1@test.com",
        "CITY_USER",
        Some(city),
    )
    .await;

    let result = repo.get_active_session_by_user(user_id).await;

    assert!(
        matches!(result.unwrap_err(), RepositoryError::NotFound),
        "expected NotFound for user with no session at all"
    );
}

#[actix_rt::test]
async fn get_active_session_by_user_returns_not_found_when_all_sessions_are_ended() {
    let pool = test_helpers::setup_test_db().await;
    test_helpers::clean_database(&pool).await;
    let repo = PgWorkSessionRepository::new(pool.clone());

    let city = db_fixtures::insert_city(&pool, "WS Repo City 2").await;
    let user_id = db_fixtures::insert_user(
        &pool,
        "500002",
        "ws-repo-2@test.com",
        "CITY_USER",
        Some(city),
    )
    .await;

    // Create a session and end it
    let session_id = test_helpers::create_work_session_for_user(&pool, user_id).await;
    repo.end_session(session_id)
        .await
        .expect("should be able to end session");

    let result = repo.get_active_session_by_user(user_id).await;

    assert!(
        matches!(result.unwrap_err(), RepositoryError::NotFound),
        "expected NotFound once the session is ended"
    );

    let is_active = repo
        .is_user_in_active_session(user_id)
        .await
        .expect("query should succeed");
    assert!(!is_active);
}

#[actix_rt::test]
async fn get_active_session_by_user_returns_the_active_one_when_history_exists() {
    let pool = test_helpers::setup_test_db().await;
    test_helpers::clean_database(&pool).await;
    let repo = PgWorkSessionRepository::new(pool.clone());

    let city = db_fixtures::insert_city(&pool, "WS Repo City 3").await;
    let user_id = db_fixtures::insert_user(
        &pool,
        "500003",
        "ws-repo-3@test.com",
        "CITY_USER",
        Some(city),
    )
    .await;

    // First session (ended)
    let old_session = test_helpers::create_work_session_for_user(&pool, user_id).await;
    repo.end_session(old_session)
        .await
        .expect("should end old session");

    // Second session (active)
    let active_session = test_helpers::create_work_session_for_user(&pool, user_id).await;

    let session = repo
        .get_active_session_by_user(user_id)
        .await
        .expect("should find active session");

    assert_eq!(session.id, active_session);
    assert!(session.is_active, "returned session must be active");
    assert_eq!(
        session.created_by_user_id, user_id,
        "returned session must belong to the queried user"
    );

    let is_active = repo
        .is_user_in_active_session(user_id)
        .await
        .expect("query should succeed");
    assert!(is_active);
}

#[actix_rt::test]
async fn end_session_flips_is_active_flag_and_sets_ended_at() {
    let pool = test_helpers::setup_test_db().await;
    test_helpers::clean_database(&pool).await;
    let repo = PgWorkSessionRepository::new(pool.clone());

    let city = db_fixtures::insert_city(&pool, "WS Repo City 4").await;
    let user_id = db_fixtures::insert_user(
        &pool,
        "500004",
        "ws-repo-4@test.com",
        "CITY_USER",
        Some(city),
    )
    .await;

    let session_id = test_helpers::create_work_session_for_user(&pool, user_id).await;
    let ended = repo
        .end_session(session_id)
        .await
        .expect("should be able to end session");

    assert_eq!(ended.id, session_id);
    assert!(
        !ended.is_active,
        "ended session should have is_active=false"
    );
    assert!(
        ended.ended_at.is_some(),
        "ended session should have ended_at set"
    );
}

#[actix_rt::test]
async fn get_session_members_returns_members_for_session_with_members() {
    let pool = test_helpers::setup_test_db().await;
    test_helpers::clean_database(&pool).await;
    let repo = PgWorkSessionRepository::new(pool.clone());

    let city = db_fixtures::insert_city(&pool, "WS Repo City 5").await;
    let user_id = db_fixtures::insert_user(
        &pool,
        "500005",
        "ws-repo-5@test.com",
        "CITY_USER",
        Some(city),
    )
    .await;

    let session_id = test_helpers::create_work_session_for_user(&pool, user_id).await;

    let members = repo
        .get_session_members(session_id)
        .await
        .expect("query should succeed");

    assert_eq!(members.len(), 1, "session should contain the creator");
    assert_eq!(members[0].user_id, user_id);
}
