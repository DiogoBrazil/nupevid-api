use uuid::Uuid;

use crate::core::application_error::ApplicationError as AppError;
use crate::core::entities::attendance_members::AddAttendanceMember;
use crate::core::value_objects::policies::Policy;
use crate::core::value_objects::profiles::Profile;
use crate::usecases::attendance_victims::AddAttendanceMemberUseCase;
use crate::usecases::attendance_victims::test_support::{
    FakeAttendanceVictimReadRepo, FakeAttendanceVictimWriteRepo, FakeMemberRepo, FakePmReadRepo,
    FakeWorkSessionReadRepo, MemberAddOutcome, attendance_victim, claims, deps, user_record,
    user_repo_with_policy, victim_with_details,
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
async fn add_member_succeeds_when_same_city_and_policy_present() {
    let city_id = Uuid::new_v4();
    let requester_id = Uuid::new_v4();
    let victim_id = Uuid::new_v4();
    let member_id = Uuid::new_v4();
    let att_id = Uuid::new_v4();

    let mut victim_repo =
        crate::core::contracts::repository::victims::MockVictimReadRepository::new();
    victim_repo
        .expect_get_victim_by_id()
        .returning(move |_| Ok(victim_with_details(victim_id, city_id)));

    let (d, _) = deps(
        FakeAttendanceVictimReadRepo {
            attendance: Some(attendance_victim(att_id, victim_id)),
        },
        FakeAttendanceVictimWriteRepo::internal(),
        victim_repo,
        FakePmReadRepo { measure: None },
        setup_user_by_id(city_id, Some(city_id), member_id),
        FakeWorkSessionReadRepo::without_session(),
        FakeMemberRepo::with_add(MemberAddOutcome::Success),
    );
    let usecase = AddAttendanceMemberUseCase::new(d);

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
    let victim_id = Uuid::new_v4();
    let member_id = Uuid::new_v4();
    let att_id = Uuid::new_v4();

    let mut victim_repo =
        crate::core::contracts::repository::victims::MockVictimReadRepository::new();
    victim_repo
        .expect_get_victim_by_id()
        .returning(move |_| Ok(victim_with_details(victim_id, city_id)));

    let (d, _) = deps(
        FakeAttendanceVictimReadRepo {
            attendance: Some(attendance_victim(att_id, victim_id)),
        },
        FakeAttendanceVictimWriteRepo::internal(),
        victim_repo,
        FakePmReadRepo { measure: None },
        setup_user_by_id(city_id, Some(other_city), member_id),
        FakeWorkSessionReadRepo::without_session(),
        FakeMemberRepo::idle(),
    );
    let usecase = AddAttendanceMemberUseCase::new(d);

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
    let victim_id = Uuid::new_v4();
    let member_id = Uuid::new_v4();
    let att_id = Uuid::new_v4();

    let mut victim_repo =
        crate::core::contracts::repository::victims::MockVictimReadRepository::new();
    victim_repo
        .expect_get_victim_by_id()
        .returning(move |_| Ok(victim_with_details(victim_id, city_id)));

    let (d, _) = deps(
        FakeAttendanceVictimReadRepo {
            attendance: Some(attendance_victim(att_id, victim_id)),
        },
        FakeAttendanceVictimWriteRepo::internal(),
        victim_repo,
        FakePmReadRepo { measure: None },
        setup_user_by_id(city_id, Some(city_id), member_id),
        FakeWorkSessionReadRepo::without_session(),
        FakeMemberRepo::with_add(MemberAddOutcome::Duplicate("already member".to_string())),
    );
    let usecase = AddAttendanceMemberUseCase::new(d);

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
    let usecase = AddAttendanceMemberUseCase::new(d);

    let result = usecase
        .execute(
            att_id,
            AddAttendanceMember { user_id: member_id },
            &claims(Profile::CityAdmin, requester_id, Some(city_id)),
        )
        .await;

    assert!(matches!(result.unwrap_err(), AppError::NotFound(_)));
}
