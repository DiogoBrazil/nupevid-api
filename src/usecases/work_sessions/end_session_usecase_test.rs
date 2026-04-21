use uuid::Uuid;

use crate::core::application_error::ApplicationError as AppError;
use crate::core::entities::work_session_members::TeamMemberFunction;
use crate::core::value_objects::policies::Policy;
use crate::core::value_objects::profiles::Profile;
use crate::usecases::work_sessions::EndSessionUseCase;
use crate::usecases::work_sessions::test_support::{
    EndOutcome, FakeReadRepo, FakeWriteRepo, claims, deps, session_member, user_repo_with_policy,
    user_repo_without_policy, work_session,
};

#[tokio::test]
async fn end_succeeds_when_caller_is_creator() {
    let city_id = Uuid::new_v4();
    let user_id = Uuid::new_v4();
    let session_id = Uuid::new_v4();
    let session = work_session(session_id, user_id, true);

    let usecase = EndSessionUseCase::new(deps(
        FakeReadRepo::with_active_session(
            session,
            vec![session_member(
                session_id,
                user_id,
                Some(TeamMemberFunction::Commander),
            )],
        ),
        FakeWriteRepo::with_end(EndOutcome::Success),
        user_repo_with_policy(city_id, Policy::EndWorkSessions),
    ));

    let msg = usecase
        .execute(&claims(Profile::CityAdmin, user_id, Some(city_id)))
        .await
        .unwrap();

    assert!(msg.contains("ended"));
}

#[tokio::test]
async fn end_succeeds_when_caller_is_commander() {
    let city_id = Uuid::new_v4();
    let creator_id = Uuid::new_v4();
    let commander_id = Uuid::new_v4();
    let session_id = Uuid::new_v4();
    let session = work_session(session_id, creator_id, true);

    let usecase = EndSessionUseCase::new(deps(
        FakeReadRepo::with_active_session(
            session,
            vec![
                session_member(session_id, creator_id, Some(TeamMemberFunction::Driver)),
                session_member(
                    session_id,
                    commander_id,
                    Some(TeamMemberFunction::Commander),
                ),
            ],
        ),
        FakeWriteRepo::with_end(EndOutcome::Success),
        user_repo_with_policy(city_id, Policy::EndWorkSessions),
    ));

    let msg = usecase
        .execute(&claims(Profile::CityAdmin, commander_id, Some(city_id)))
        .await
        .unwrap();

    assert!(msg.contains("ended"));
}

#[tokio::test]
async fn end_forbids_patroller_non_creator() {
    let city_id = Uuid::new_v4();
    let creator_id = Uuid::new_v4();
    let patroller_id = Uuid::new_v4();
    let session_id = Uuid::new_v4();
    let session = work_session(session_id, creator_id, true);

    let usecase = EndSessionUseCase::new(deps(
        FakeReadRepo::with_active_session(
            session,
            vec![
                session_member(session_id, creator_id, Some(TeamMemberFunction::Commander)),
                session_member(
                    session_id,
                    patroller_id,
                    Some(TeamMemberFunction::Patroller),
                ),
            ],
        ),
        FakeWriteRepo::new(),
        user_repo_with_policy(city_id, Policy::EndWorkSessions),
    ));

    let result = usecase
        .execute(&claims(Profile::CityAdmin, patroller_id, Some(city_id)))
        .await;

    assert!(matches!(result.unwrap_err(), AppError::Forbidden(_)));
}

#[tokio::test]
async fn end_returns_not_found_when_no_active_session() {
    let city_id = Uuid::new_v4();
    let user_id = Uuid::new_v4();

    let usecase = EndSessionUseCase::new(deps(
        FakeReadRepo::without_active_session(),
        FakeWriteRepo::new(),
        user_repo_with_policy(city_id, Policy::EndWorkSessions),
    ));

    let result = usecase
        .execute(&claims(Profile::CityAdmin, user_id, Some(city_id)))
        .await;

    assert!(matches!(result.unwrap_err(), AppError::NotFound(_)));
}

#[tokio::test]
async fn end_fails_when_policy_missing() {
    let city_id = Uuid::new_v4();
    let user_id = Uuid::new_v4();

    let usecase = EndSessionUseCase::new(deps(
        FakeReadRepo::without_active_session(),
        FakeWriteRepo::new(),
        user_repo_without_policy(),
    ));

    let result = usecase
        .execute(&claims(Profile::CityAdmin, user_id, Some(city_id)))
        .await;

    assert!(matches!(result.unwrap_err(), AppError::Forbidden(_)));
}

#[tokio::test]
async fn end_maps_write_repo_error_to_internal() {
    let city_id = Uuid::new_v4();
    let user_id = Uuid::new_v4();
    let session_id = Uuid::new_v4();
    let session = work_session(session_id, user_id, true);

    let usecase = EndSessionUseCase::new(deps(
        FakeReadRepo::with_active_session(
            session,
            vec![session_member(
                session_id,
                user_id,
                Some(TeamMemberFunction::Commander),
            )],
        ),
        FakeWriteRepo::with_end(EndOutcome::Error),
        user_repo_with_policy(city_id, Policy::EndWorkSessions),
    ));

    let result = usecase
        .execute(&claims(Profile::CityAdmin, user_id, Some(city_id)))
        .await;

    assert!(matches!(result.unwrap_err(), AppError::InternalServerError));
}

#[tokio::test]
async fn end_root_bypasses_policy() {
    let user_id = Uuid::new_v4();
    let session_id = Uuid::new_v4();
    let session = work_session(session_id, user_id, true);

    let usecase = EndSessionUseCase::new(deps(
        FakeReadRepo::with_active_session(
            session,
            vec![session_member(
                session_id,
                user_id,
                Some(TeamMemberFunction::Commander),
            )],
        ),
        FakeWriteRepo::with_end(EndOutcome::Success),
        crate::core::contracts::repository::users::MockUserRepository::new(),
    ));

    let result = usecase.execute(&claims(Profile::Root, user_id, None)).await;

    assert!(result.is_ok());
}
