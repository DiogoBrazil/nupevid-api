use uuid::Uuid;

use crate::core::application_error::ApplicationError as AppError;
use crate::core::entities::attendance_members::AddAttendanceMember;
use crate::core::value_objects::policies::Policy;
use crate::core::value_objects::profiles::Profile;
use crate::usecases::attendance_offenders::AddAttendanceOffenderMemberUseCase;
use crate::usecases::attendance_offenders::test_support::{
    FakeAttendanceOffenderReadRepo, FakeAttendanceOffenderWriteRepo, FakeMemberRepo,
    FakeOffenderReadRepo, FakePmReadRepo, FakeWorkSessionReadRepo, MemberAddOutcome,
    attendance_offender, claims, deps, offender_with_details, user_record, user_repo_with_policy,
    victim_repo_ok,
};

fn setup_user_by_id(
    city_id: Uuid,
    member_city: Option<Uuid>,
    member_id: Uuid,
) -> crate::core::contracts::repository::users::MockUserRepository {
    let mut repo = user_repo_with_policy(city_id, Policy::ManageAttendanceMembers);
    repo.expect_get_user_by_id()
        .withf(move |id| *id == member_id)
        .returning(move |id| Ok(user_record(id, member_city)));
    repo
}

#[tokio::test]
async fn add_member_succeeds_when_same_city() {
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
        setup_user_by_id(city_id, Some(city_id), member_id),
        FakeWorkSessionReadRepo::without_session(),
        FakeMemberRepo::with_add(MemberAddOutcome::Success),
    );
    let usecase = AddAttendanceOffenderMemberUseCase::new(d);

    let result = usecase
        .execute(
            att_id,
            AddAttendanceMember { user_id: member_id },
            &claims(Profile::CityAdmin, requester_id, Some(city_id)),
        )
        .await;

    assert!(result.is_ok());
}

#[tokio::test]
async fn add_member_rejects_user_from_different_city() {
    let city_id = Uuid::new_v4();
    let other_city = Uuid::new_v4();
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
        setup_user_by_id(city_id, Some(other_city), member_id),
        FakeWorkSessionReadRepo::without_session(),
        FakeMemberRepo::idle(),
    );
    let usecase = AddAttendanceOffenderMemberUseCase::new(d);

    let result = usecase
        .execute(
            att_id,
            AddAttendanceMember { user_id: member_id },
            &claims(Profile::CityAdmin, requester_id, Some(city_id)),
        )
        .await;

    assert!(matches!(result.unwrap_err(), AppError::BadRequest(msg) if msg.contains("same city")));
}

#[tokio::test]
async fn add_member_maps_duplicate_to_conflict() {
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
        setup_user_by_id(city_id, Some(city_id), member_id),
        FakeWorkSessionReadRepo::without_session(),
        FakeMemberRepo::with_add(MemberAddOutcome::Duplicate("already".to_string())),
    );
    let usecase = AddAttendanceOffenderMemberUseCase::new(d);

    let result = usecase
        .execute(
            att_id,
            AddAttendanceMember { user_id: member_id },
            &claims(Profile::CityAdmin, requester_id, Some(city_id)),
        )
        .await;

    assert!(matches!(result.unwrap_err(), AppError::Conflict(_)));
}

#[tokio::test]
async fn add_member_fails_when_attendance_not_found() {
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
    let usecase = AddAttendanceOffenderMemberUseCase::new(d);

    let result = usecase
        .execute(
            att_id,
            AddAttendanceMember { user_id: member_id },
            &claims(Profile::CityAdmin, requester_id, Some(city_id)),
        )
        .await;

    assert!(matches!(result.unwrap_err(), AppError::NotFound(_)));
}
