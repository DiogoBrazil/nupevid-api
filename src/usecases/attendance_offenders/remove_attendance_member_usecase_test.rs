use uuid::Uuid;

use crate::core::application_error::ApplicationError as AppError;
use crate::core::value_objects::policies::Policy;
use crate::core::value_objects::profiles::Profile;
use crate::usecases::attendance_offenders::RemoveAttendanceOffenderMemberUseCase;
use crate::usecases::attendance_offenders::test_support::{
    FakeAttendanceOffenderReadRepo, FakeAttendanceOffenderWriteRepo, FakeMemberRepo,
    FakeOffenderReadRepo, FakePmReadRepo, FakeWorkSessionReadRepo, MemberRemoveOutcome,
    attendance_offender, claims, deps, offender_with_details, user_repo_with_policy,
    victim_repo_ok,
};

#[tokio::test]
async fn remove_member_succeeds_for_existing() {
    let city_id = Uuid::new_v4();
    let requester_id = Uuid::new_v4();
    let member_id = Uuid::new_v4();
    let offender_id = Uuid::new_v4();
    let victim_id = Uuid::new_v4();
    let att_id = Uuid::new_v4();

    let (d, _) = deps(
        FakeAttendanceOffenderReadRepo {
            attendance: Some(attendance_offender(att_id, offender_id, victim_id)),
        },
        FakeAttendanceOffenderWriteRepo::internal(),
        FakeOffenderReadRepo::found(offender_with_details(offender_id, city_id)),
        victim_repo_ok(victim_id, city_id),
        FakePmReadRepo { measure: None },
        user_repo_with_policy(city_id, Policy::ManageAttendanceMembers),
        FakeWorkSessionReadRepo::without_session(),
        FakeMemberRepo::with_remove(MemberRemoveOutcome::Success),
    );
    let usecase = RemoveAttendanceOffenderMemberUseCase::new(d);

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
async fn remove_member_maps_errors_to_internal() {
    let city_id = Uuid::new_v4();
    let requester_id = Uuid::new_v4();
    let member_id = Uuid::new_v4();
    let offender_id = Uuid::new_v4();
    let victim_id = Uuid::new_v4();
    let att_id = Uuid::new_v4();

    let (d, _) = deps(
        FakeAttendanceOffenderReadRepo {
            attendance: Some(attendance_offender(att_id, offender_id, victim_id)),
        },
        FakeAttendanceOffenderWriteRepo::internal(),
        FakeOffenderReadRepo::found(offender_with_details(offender_id, city_id)),
        victim_repo_ok(victim_id, city_id),
        FakePmReadRepo { measure: None },
        user_repo_with_policy(city_id, Policy::ManageAttendanceMembers),
        FakeWorkSessionReadRepo::without_session(),
        FakeMemberRepo::with_remove(MemberRemoveOutcome::NotFound),
    );
    let usecase = RemoveAttendanceOffenderMemberUseCase::new(d);

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

    let (d, _) = deps(
        FakeAttendanceOffenderReadRepo { attendance: None },
        FakeAttendanceOffenderWriteRepo::internal(),
        FakeOffenderReadRepo::found(offender_with_details(Uuid::new_v4(), city_id)),
        victim_repo_ok(Uuid::new_v4(), city_id),
        FakePmReadRepo { measure: None },
        user_repo_with_policy(city_id, Policy::ManageAttendanceMembers),
        FakeWorkSessionReadRepo::without_session(),
        FakeMemberRepo::idle(),
    );
    let usecase = RemoveAttendanceOffenderMemberUseCase::new(d);

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
    let offender_id = Uuid::new_v4();
    let victim_id = Uuid::new_v4();
    let att_id = Uuid::new_v4();

    let mut user_repo = crate::core::contracts::repository::users::MockUserRepository::new();
    user_repo
        .expect_get_user_policies_by_id()
        .returning(|_| Ok(std::collections::HashMap::new()));

    let (d, _) = deps(
        FakeAttendanceOffenderReadRepo {
            attendance: Some(attendance_offender(att_id, offender_id, victim_id)),
        },
        FakeAttendanceOffenderWriteRepo::internal(),
        FakeOffenderReadRepo::found(offender_with_details(offender_id, city_id)),
        victim_repo_ok(victim_id, city_id),
        FakePmReadRepo { measure: None },
        user_repo,
        FakeWorkSessionReadRepo::without_session(),
        FakeMemberRepo::idle(),
    );
    let usecase = RemoveAttendanceOffenderMemberUseCase::new(d);

    let result = usecase
        .execute(
            att_id,
            member_id,
            &claims(Profile::CityAdmin, requester_id, Some(city_id)),
        )
        .await;

    assert!(matches!(result.unwrap_err(), AppError::Forbidden(_)));
}
