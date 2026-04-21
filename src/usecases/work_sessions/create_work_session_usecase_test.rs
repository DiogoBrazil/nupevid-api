use uuid::Uuid;

use crate::core::application_error::ApplicationError as AppError;
use crate::core::commands::work_sessions::CreateWorkSession;
use crate::core::contracts::repository::users::MockUserRepository;
use crate::core::entities::work_session_members::TeamMemberFunction;
use crate::core::value_objects::policies::Policy;
use crate::core::value_objects::profiles::Profile;
use crate::usecases::work_sessions::CreateWorkSessionUseCase;
use crate::usecases::work_sessions::test_support::{
    FakeReadRepo, FakeWriteRepo, add_member, claims, deps, user_record, user_repo_with_policy,
    work_session,
};

fn create_cmd(
    members: Vec<crate::core::commands::work_session_members::AddWorkSessionMember>,
) -> CreateWorkSession {
    CreateWorkSession {
        description: Some("patrol".to_string()),
        members,
    }
}

#[tokio::test]
async fn create_fails_when_requester_not_in_members() {
    let city_id = Uuid::new_v4();
    let requester_id = Uuid::new_v4();
    let other_id = Uuid::new_v4();

    let cmd = create_cmd(vec![add_member(
        other_id,
        Some(TeamMemberFunction::Commander),
    )]);

    let mut user_repo = user_repo_with_policy(city_id, Policy::CreateWorkSessions);
    user_repo
        .expect_get_user_by_id()
        .returning(move |id| Ok(user_record(id, Some(city_id))));

    let usecase = CreateWorkSessionUseCase::new(deps(
        FakeReadRepo::not_in_active(),
        FakeWriteRepo::new(),
        user_repo,
    ));

    let result = usecase
        .execute(
            cmd,
            &claims(Profile::CityAdmin, requester_id, Some(city_id)),
        )
        .await;

    assert!(
        matches!(result.unwrap_err(), AppError::BadRequest(msg) if msg.contains("Requesting user"))
    );
}

#[tokio::test]
async fn create_fails_when_member_from_different_city() {
    let city_id = Uuid::new_v4();
    let other_city = Uuid::new_v4();
    let requester_id = Uuid::new_v4();
    let second_id = Uuid::new_v4();

    let cmd = create_cmd(vec![
        add_member(requester_id, Some(TeamMemberFunction::Commander)),
        add_member(second_id, Some(TeamMemberFunction::Driver)),
    ]);

    let mut user_repo = user_repo_with_policy(city_id, Policy::CreateWorkSessions);
    user_repo.expect_get_user_by_id().returning(move |id| {
        if id == requester_id {
            Ok(user_record(id, Some(city_id)))
        } else {
            Ok(user_record(id, Some(other_city)))
        }
    });

    let usecase = CreateWorkSessionUseCase::new(deps(
        FakeReadRepo::not_in_active(),
        FakeWriteRepo::new(),
        user_repo,
    ));

    let result = usecase
        .execute(
            cmd,
            &claims(Profile::CityAdmin, requester_id, Some(city_id)),
        )
        .await;

    assert!(matches!(result.unwrap_err(), AppError::BadRequest(msg) if msg.contains("same city")));
}

#[tokio::test]
async fn create_fails_when_no_commander() {
    let city_id = Uuid::new_v4();
    let requester_id = Uuid::new_v4();

    let cmd = create_cmd(vec![add_member(
        requester_id,
        Some(TeamMemberFunction::Driver),
    )]);

    let user_repo = user_repo_with_policy(city_id, Policy::CreateWorkSessions);

    let usecase = CreateWorkSessionUseCase::new(deps(
        FakeReadRepo::not_in_active(),
        FakeWriteRepo::new(),
        user_repo,
    ));

    let result = usecase
        .execute(
            cmd,
            &claims(Profile::CityAdmin, requester_id, Some(city_id)),
        )
        .await;

    assert!(matches!(result.unwrap_err(), AppError::BadRequest(msg) if msg.contains("Commander")));
}

#[tokio::test]
async fn create_fails_when_more_than_one_commander() {
    let city_id = Uuid::new_v4();
    let a = Uuid::new_v4();
    let b = Uuid::new_v4();

    let cmd = create_cmd(vec![
        add_member(a, Some(TeamMemberFunction::Commander)),
        add_member(b, Some(TeamMemberFunction::Commander)),
    ]);

    let user_repo = user_repo_with_policy(city_id, Policy::CreateWorkSessions);

    let usecase = CreateWorkSessionUseCase::new(deps(
        FakeReadRepo::not_in_active(),
        FakeWriteRepo::new(),
        user_repo,
    ));

    let result = usecase
        .execute(cmd, &claims(Profile::CityAdmin, a, Some(city_id)))
        .await;

    assert!(
        matches!(result.unwrap_err(), AppError::BadRequest(msg) if msg.contains("one Commander"))
    );
}

#[tokio::test]
async fn create_fails_when_more_than_one_driver() {
    let city_id = Uuid::new_v4();
    let a = Uuid::new_v4();
    let b = Uuid::new_v4();
    let c = Uuid::new_v4();

    let cmd = create_cmd(vec![
        add_member(a, Some(TeamMemberFunction::Commander)),
        add_member(b, Some(TeamMemberFunction::Driver)),
        add_member(c, Some(TeamMemberFunction::Driver)),
    ]);

    let user_repo = user_repo_with_policy(city_id, Policy::CreateWorkSessions);

    let usecase = CreateWorkSessionUseCase::new(deps(
        FakeReadRepo::not_in_active(),
        FakeWriteRepo::new(),
        user_repo,
    ));

    let result = usecase
        .execute(cmd, &claims(Profile::CityAdmin, a, Some(city_id)))
        .await;

    assert!(matches!(result.unwrap_err(), AppError::BadRequest(msg) if msg.contains("one Driver")));
}

#[tokio::test]
async fn create_fails_when_user_already_has_active_session() {
    let city_id = Uuid::new_v4();
    let requester_id = Uuid::new_v4();

    let cmd = create_cmd(vec![add_member(
        requester_id,
        Some(TeamMemberFunction::Commander),
    )]);

    let mut user_repo = user_repo_with_policy(city_id, Policy::CreateWorkSessions);
    user_repo
        .expect_get_user_by_id()
        .returning(move |id| Ok(user_record(id, Some(city_id))));

    let usecase = CreateWorkSessionUseCase::new(deps(
        FakeReadRepo::in_active(),
        FakeWriteRepo::new(),
        user_repo,
    ));

    let result = usecase
        .execute(
            cmd,
            &claims(Profile::CityAdmin, requester_id, Some(city_id)),
        )
        .await;

    assert!(
        matches!(result.unwrap_err(), AppError::Conflict(msg) if msg.contains("active work session"))
    );
}

#[tokio::test]
async fn create_maps_repository_conflict_to_app_conflict() {
    let city_id = Uuid::new_v4();
    let requester_id = Uuid::new_v4();

    let cmd = create_cmd(vec![add_member(
        requester_id,
        Some(TeamMemberFunction::Commander),
    )]);

    let mut user_repo = user_repo_with_policy(city_id, Policy::CreateWorkSessions);
    user_repo
        .expect_get_user_by_id()
        .returning(move |id| Ok(user_record(id, Some(city_id))));

    let usecase = CreateWorkSessionUseCase::new(deps(
        FakeReadRepo::not_in_active(),
        FakeWriteRepo::with_create_conflict("conflict at insert"),
        user_repo,
    ));

    let result = usecase
        .execute(
            cmd,
            &claims(Profile::CityAdmin, requester_id, Some(city_id)),
        )
        .await;

    assert!(matches!(result.unwrap_err(), AppError::Conflict(_)));
}

#[tokio::test]
async fn create_maps_duplicate_to_app_conflict() {
    let city_id = Uuid::new_v4();
    let requester_id = Uuid::new_v4();

    let cmd = create_cmd(vec![add_member(
        requester_id,
        Some(TeamMemberFunction::Commander),
    )]);

    let mut user_repo = user_repo_with_policy(city_id, Policy::CreateWorkSessions);
    user_repo
        .expect_get_user_by_id()
        .returning(move |id| Ok(user_record(id, Some(city_id))));

    let usecase = CreateWorkSessionUseCase::new(deps(
        FakeReadRepo::not_in_active(),
        FakeWriteRepo::with_create_duplicate("duplicate insert"),
        user_repo,
    ));

    let result = usecase
        .execute(
            cmd,
            &claims(Profile::CityAdmin, requester_id, Some(city_id)),
        )
        .await;

    assert!(matches!(result.unwrap_err(), AppError::Conflict(_)));
}

#[tokio::test]
async fn create_maps_generic_repository_error_to_internal() {
    let city_id = Uuid::new_v4();
    let requester_id = Uuid::new_v4();

    let cmd = create_cmd(vec![add_member(
        requester_id,
        Some(TeamMemberFunction::Commander),
    )]);

    let mut user_repo = user_repo_with_policy(city_id, Policy::CreateWorkSessions);
    user_repo
        .expect_get_user_by_id()
        .returning(move |id| Ok(user_record(id, Some(city_id))));

    let usecase = CreateWorkSessionUseCase::new(deps(
        FakeReadRepo::not_in_active(),
        FakeWriteRepo::with_create_internal(),
        user_repo,
    ));

    let result = usecase
        .execute(
            cmd,
            &claims(Profile::CityAdmin, requester_id, Some(city_id)),
        )
        .await;

    assert!(matches!(result.unwrap_err(), AppError::InternalServerError));
}

#[tokio::test]
async fn create_succeeds_for_valid_single_commander_self() {
    let city_id = Uuid::new_v4();
    let requester_id = Uuid::new_v4();
    let session_id = Uuid::new_v4();

    let cmd = create_cmd(vec![add_member(
        requester_id,
        Some(TeamMemberFunction::Commander),
    )]);

    let mut user_repo = user_repo_with_policy(city_id, Policy::CreateWorkSessions);
    user_repo
        .expect_get_user_by_id()
        .returning(move |id| Ok(user_record(id, Some(city_id))));

    let usecase = CreateWorkSessionUseCase::new(deps(
        FakeReadRepo::not_in_active(),
        FakeWriteRepo::with_create_success(work_session(session_id, requester_id, true)),
        user_repo,
    ));

    let session = usecase
        .execute(
            cmd,
            &claims(Profile::CityAdmin, requester_id, Some(city_id)),
        )
        .await
        .unwrap();

    assert_eq!(session.id, session_id);
    assert!(session.is_active);
    assert_eq!(session.created_by_user_id, requester_id);
}

#[tokio::test]
async fn create_fails_when_policy_missing_for_non_root() {
    let city_id = Uuid::new_v4();
    let requester_id = Uuid::new_v4();

    let cmd = create_cmd(vec![add_member(
        requester_id,
        Some(TeamMemberFunction::Commander),
    )]);

    let mut user_repo = MockUserRepository::new();
    // non-root with no policies
    user_repo
        .expect_get_user_policies_by_id()
        .returning(|_| Ok(std::collections::HashMap::new()));

    let usecase = CreateWorkSessionUseCase::new(deps(
        FakeReadRepo::not_in_active(),
        FakeWriteRepo::new(),
        user_repo,
    ));

    let result = usecase
        .execute(
            cmd,
            &claims(Profile::CityAdmin, requester_id, Some(city_id)),
        )
        .await;

    assert!(matches!(result.unwrap_err(), AppError::Forbidden(_)));
}
