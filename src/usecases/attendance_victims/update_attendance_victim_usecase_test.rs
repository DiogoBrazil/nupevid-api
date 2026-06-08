use uuid::Uuid;

use crate::core::application_error::ApplicationError as AppError;
use crate::core::value_objects::policies::Policy;
use crate::core::value_objects::profiles::Profile;
use crate::usecases::attendance_victims::UpdateAttendanceVictimUseCase;
use crate::usecases::attendance_victims::test_support::{
    FakeAttendanceVictimReadRepo, FakeAttendanceVictimWriteRepo, FakeMemberRepo, FakePmReadRepo,
    FakeWorkSessionReadRepo, attendance_victim, attendance_victim_entity, claims, deps,
    protective_measure, update_command, user_repo_with_policy, victim_repo_multi, victim_repo_ok,
};

#[tokio::test]
async fn update_succeeds_when_policy_present_for_same_city() {
    let city_id = Uuid::new_v4();
    let requester_id = Uuid::new_v4();
    let victim_id = Uuid::new_v4();
    let offender_id = Uuid::new_v4();
    let pm_id = Uuid::new_v4();
    let att_id = Uuid::new_v4();

    let (d, _) = deps(
        FakeAttendanceVictimReadRepo {
            attendance: Some(attendance_victim(att_id, victim_id)),
        },
        FakeAttendanceVictimWriteRepo::success(attendance_victim_entity(att_id, victim_id)),
        victim_repo_ok(victim_id, city_id),
        FakePmReadRepo {
            measure: Some(protective_measure(pm_id, victim_id, offender_id)),
        },
        user_repo_with_policy(city_id, Policy::UpdateAttendances),
        FakeWorkSessionReadRepo::without_session(),
        FakeMemberRepo::idle(),
    );
    let usecase = UpdateAttendanceVictimUseCase::new(d);

    let result = usecase
        .execute(
            update_command(pm_id),
            att_id,
            &claims(Profile::CityAdmin, requester_id, Some(city_id)),
        )
        .await;

    assert!(result.is_ok());
}

#[tokio::test]
async fn update_change_victim_requires_policy_on_new_city() {
    let old_city = Uuid::new_v4();
    let new_city = Uuid::new_v4();
    let requester_id = Uuid::new_v4();
    let old_victim_id = Uuid::new_v4();
    let new_victim_id = Uuid::new_v4();
    let offender_id = Uuid::new_v4();
    let pm_id = Uuid::new_v4();
    let att_id = Uuid::new_v4();

    // User has policy only for old_city, cannot update into new_city
    let (d, _) = deps(
        FakeAttendanceVictimReadRepo {
            attendance: Some(attendance_victim(att_id, old_victim_id)),
        },
        FakeAttendanceVictimWriteRepo::internal(),
        victim_repo_multi(old_victim_id, old_city, new_victim_id, new_city),
        FakePmReadRepo {
            measure: Some(protective_measure(pm_id, new_victim_id, offender_id)),
        },
        user_repo_with_policy(old_city, Policy::UpdateAttendances),
        FakeWorkSessionReadRepo::without_session(),
        FakeMemberRepo::idle(),
    );
    let usecase = UpdateAttendanceVictimUseCase::new(d);

    let result = usecase
        .execute(
            update_command(pm_id),
            att_id,
            &claims(Profile::CityAdmin, requester_id, Some(old_city)),
        )
        .await;

    assert!(matches!(result.unwrap_err(), AppError::Forbidden(_)));
}

#[tokio::test]
async fn update_fails_when_attendance_not_found() {
    let city_id = Uuid::new_v4();
    let requester_id = Uuid::new_v4();
    let pm_id = Uuid::new_v4();
    let att_id = Uuid::new_v4();

    let (d, _) = deps(
        FakeAttendanceVictimReadRepo { attendance: None },
        FakeAttendanceVictimWriteRepo::internal(),
        victim_repo_ok(Uuid::new_v4(), city_id),
        FakePmReadRepo { measure: None },
        user_repo_with_policy(city_id, Policy::UpdateAttendances),
        FakeWorkSessionReadRepo::without_session(),
        FakeMemberRepo::idle(),
    );
    let usecase = UpdateAttendanceVictimUseCase::new(d);

    let result = usecase
        .execute(
            update_command(pm_id),
            att_id,
            &claims(Profile::CityAdmin, requester_id, Some(city_id)),
        )
        .await;

    assert!(matches!(result.unwrap_err(), AppError::NotFound(_)));
}

#[tokio::test]
async fn update_maps_referenced_entity_not_found_to_bad_request() {
    let city_id = Uuid::new_v4();
    let requester_id = Uuid::new_v4();
    let victim_id = Uuid::new_v4();
    let offender_id = Uuid::new_v4();
    let pm_id = Uuid::new_v4();
    let att_id = Uuid::new_v4();

    let (d, _) = deps(
        FakeAttendanceVictimReadRepo {
            attendance: Some(attendance_victim(att_id, victim_id)),
        },
        FakeAttendanceVictimWriteRepo::referenced("city not found"),
        victim_repo_ok(victim_id, city_id),
        FakePmReadRepo {
            measure: Some(protective_measure(pm_id, victim_id, offender_id)),
        },
        user_repo_with_policy(city_id, Policy::UpdateAttendances),
        FakeWorkSessionReadRepo::without_session(),
        FakeMemberRepo::idle(),
    );
    let usecase = UpdateAttendanceVictimUseCase::new(d);

    let result = usecase
        .execute(
            update_command(pm_id),
            att_id,
            &claims(Profile::CityAdmin, requester_id, Some(city_id)),
        )
        .await;

    assert!(matches!(result.unwrap_err(), AppError::BadRequest(_)));
}
