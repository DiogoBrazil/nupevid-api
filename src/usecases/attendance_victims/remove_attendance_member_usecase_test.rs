use uuid::Uuid;

use crate::core::application_error::ApplicationError as AppError;
use crate::core::value_objects::policies::Policy;
use crate::core::value_objects::profiles::Profile;
use crate::usecases::attendance_victims::RemoveAttendanceMemberUseCase;
use crate::usecases::attendance_victims::test_support::{
    FakeAttendanceVictimReadRepo, FakeAttendanceVictimWriteRepo, FakeMemberRepo, FakePmReadRepo,
    FakeWorkSessionReadRepo, MemberRemoveOutcome, attendance_victim, claims, deps,
    user_repo_with_policy, victim_repo_ok,
};

#[tokio::test]
async fn remove_member_succeeds_for_existing() {
    let city_id = Uuid::new_v4();
    let requester_id = Uuid::new_v4();
    let member_id = Uuid::new_v4();
    let victim_id = Uuid::new_v4();
    let att_id = Uuid::new_v4();

    let (d, _) = deps(
        FakeAttendanceVictimReadRepo {
            attendance: Some(attendance_victim(att_id, victim_id)),
        },
        FakeAttendanceVictimWriteRepo::internal(),
        victim_repo_ok(victim_id, city_id),
        FakePmReadRepo { measure: None },
        user_repo_with_policy(city_id, Policy::ManageAttendanceMembers),
        FakeWorkSessionReadRepo::without_session(),
        FakeMemberRepo::with_remove(MemberRemoveOutcome::Success),
    );
    let usecase = RemoveAttendanceMemberUseCase::new(d);

    let result = usecase
        .execute(
            att_id,
            member_id,
            &claims(Profile::CityAdmin, requester_id, Some(city_id)),
        )
        .await;

    assert!(result.is_ok());
}

#[tokio::test]
async fn remove_member_maps_not_found_to_internal_server_error() {
    // Note: current usecase maps all errors (including NotFound) from remove_member
    // to InternalServerError; this test documents the existing mapping.
    let city_id = Uuid::new_v4();
    let requester_id = Uuid::new_v4();
    let member_id = Uuid::new_v4();
    let victim_id = Uuid::new_v4();
    let att_id = Uuid::new_v4();

    let (d, _) = deps(
        FakeAttendanceVictimReadRepo {
            attendance: Some(attendance_victim(att_id, victim_id)),
        },
        FakeAttendanceVictimWriteRepo::internal(),
        victim_repo_ok(victim_id, city_id),
        FakePmReadRepo { measure: None },
        user_repo_with_policy(city_id, Policy::ManageAttendanceMembers),
        FakeWorkSessionReadRepo::without_session(),
        FakeMemberRepo::with_remove(MemberRemoveOutcome::NotFound),
    );
    let usecase = RemoveAttendanceMemberUseCase::new(d);

    let result = usecase
        .execute(
            att_id,
            member_id,
            &claims(Profile::CityAdmin, requester_id, Some(city_id)),
        )
        .await;

    assert!(matches!(result.unwrap_err(), AppError::InternalServerError));
}

#[tokio::test]
async fn remove_member_fails_when_attendance_not_found() {
    let city_id = Uuid::new_v4();
    let requester_id = Uuid::new_v4();
    let member_id = Uuid::new_v4();
    let att_id = Uuid::new_v4();

    let victim_repo = crate::core::contracts::repository::victims::MockVictimReadRepository::new();

    let (d, _) = deps(
        FakeAttendanceVictimReadRepo { attendance: None },
        FakeAttendanceVictimWriteRepo::internal(),
        victim_repo,
        FakePmReadRepo { measure: None },
        user_repo_with_policy(city_id, Policy::ManageAttendanceMembers),
        FakeWorkSessionReadRepo::without_session(),
        FakeMemberRepo::idle(),
    );
    let usecase = RemoveAttendanceMemberUseCase::new(d);

    let result = usecase
        .execute(
            att_id,
            member_id,
            &claims(Profile::CityAdmin, requester_id, Some(city_id)),
        )
        .await;

    assert!(matches!(result.unwrap_err(), AppError::NotFound(_)));
}

#[tokio::test]
async fn remove_member_fails_when_policy_missing() {
    let city_id = Uuid::new_v4();
    let requester_id = Uuid::new_v4();
    let member_id = Uuid::new_v4();
    let victim_id = Uuid::new_v4();
    let att_id = Uuid::new_v4();

    let mut user_repo = crate::core::contracts::repository::users::MockUserRepository::new();
    user_repo
        .expect_get_user_policies_by_id()
        .returning(|_| Ok(std::collections::HashMap::new()));

    let (d, _) = deps(
        FakeAttendanceVictimReadRepo {
            attendance: Some(attendance_victim(att_id, victim_id)),
        },
        FakeAttendanceVictimWriteRepo::internal(),
        victim_repo_ok(victim_id, city_id),
        FakePmReadRepo { measure: None },
        user_repo,
        FakeWorkSessionReadRepo::without_session(),
        FakeMemberRepo::idle(),
    );
    let usecase = RemoveAttendanceMemberUseCase::new(d);

    let result = usecase
        .execute(
            att_id,
            member_id,
            &claims(Profile::CityAdmin, requester_id, Some(city_id)),
        )
        .await;

    assert!(matches!(result.unwrap_err(), AppError::Forbidden(_)));
}
