use uuid::Uuid;

use crate::core::application_error::ApplicationError as AppError;
use crate::core::value_objects::policies::Policy;
use crate::core::value_objects::profiles::Profile;
use crate::usecases::attendance_offenders::UpdateAttendanceOffenderUseCase;
use crate::usecases::attendance_offenders::test_support::{
    FakeAttendanceOffenderReadRepo, FakeAttendanceOffenderWriteRepo, FakeMemberRepo,
    FakeOffenderReadRepo, FakePmReadRepo, FakeWorkSessionReadRepo, attendance_offender,
    attendance_offender_entity, claims, deps, offender_with_details, protective_measure,
    update_command, user_repo_with_policy, victim_repo_ok,
};

#[tokio::test]
async fn update_succeeds_when_policy_present_for_same_city() {
    let city_id = Uuid::new_v4();
    let requester_id = Uuid::new_v4();
    let offender_id = Uuid::new_v4();
    let victim_id = Uuid::new_v4();
    let pm_id = Uuid::new_v4();
    let att_id = Uuid::new_v4();

    let (d, _) = deps(
        FakeAttendanceOffenderReadRepo {
            attendance: Some(attendance_offender(att_id, offender_id, victim_id)),
        },
        FakeAttendanceOffenderWriteRepo::success(attendance_offender_entity(
            att_id,
            offender_id,
            victim_id,
        )),
        FakeOffenderReadRepo::found(offender_with_details(offender_id, city_id)),
        victim_repo_ok(victim_id, city_id),
        FakePmReadRepo {
            measure: Some(protective_measure(pm_id, victim_id, offender_id)),
        },
        user_repo_with_policy(city_id, Policy::UpdateAttendances),
        FakeWorkSessionReadRepo::without_session(),
        FakeMemberRepo::idle(),
    );
    let usecase = UpdateAttendanceOffenderUseCase::new(d);

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
async fn update_change_offender_requires_policy_on_new_city() {
    let old_city = Uuid::new_v4();
    let new_city = Uuid::new_v4();
    let requester_id = Uuid::new_v4();
    let old_offender_id = Uuid::new_v4();
    let new_offender_id = Uuid::new_v4();
    let victim_id = Uuid::new_v4();
    let pm_id = Uuid::new_v4();
    let att_id = Uuid::new_v4();

    let (d, _) = deps(
        FakeAttendanceOffenderReadRepo {
            attendance: Some(attendance_offender(att_id, old_offender_id, victim_id)),
        },
        FakeAttendanceOffenderWriteRepo::internal(),
        FakeOffenderReadRepo::with_two(
            offender_with_details(old_offender_id, old_city),
            new_offender_id,
            offender_with_details(new_offender_id, new_city),
        ),
        victim_repo_ok(victim_id, old_city),
        FakePmReadRepo {
            measure: Some(protective_measure(pm_id, victim_id, new_offender_id)),
        },
        user_repo_with_policy(old_city, Policy::UpdateAttendances),
        FakeWorkSessionReadRepo::without_session(),
        FakeMemberRepo::idle(),
    );
    let usecase = UpdateAttendanceOffenderUseCase::new(d);

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
        FakeAttendanceOffenderReadRepo { attendance: None },
        FakeAttendanceOffenderWriteRepo::internal(),
        FakeOffenderReadRepo::found(offender_with_details(Uuid::new_v4(), city_id)),
        victim_repo_ok(Uuid::new_v4(), city_id),
        FakePmReadRepo { measure: None },
        user_repo_with_policy(city_id, Policy::UpdateAttendances),
        FakeWorkSessionReadRepo::without_session(),
        FakeMemberRepo::idle(),
    );
    let usecase = UpdateAttendanceOffenderUseCase::new(d);

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
    let offender_id = Uuid::new_v4();
    let victim_id = Uuid::new_v4();
    let pm_id = Uuid::new_v4();
    let att_id = Uuid::new_v4();

    let (d, _) = deps(
        FakeAttendanceOffenderReadRepo {
            attendance: Some(attendance_offender(att_id, offender_id, victim_id)),
        },
        FakeAttendanceOffenderWriteRepo::referenced("city not found"),
        FakeOffenderReadRepo::found(offender_with_details(offender_id, city_id)),
        victim_repo_ok(victim_id, city_id),
        FakePmReadRepo {
            measure: Some(protective_measure(pm_id, victim_id, offender_id)),
        },
        user_repo_with_policy(city_id, Policy::UpdateAttendances),
        FakeWorkSessionReadRepo::without_session(),
        FakeMemberRepo::idle(),
    );
    let usecase = UpdateAttendanceOffenderUseCase::new(d);

    let result = usecase
        .execute(
            update_command(pm_id),
            att_id,
            &claims(Profile::CityAdmin, requester_id, Some(city_id)),
        )
        .await;

    assert!(matches!(result.unwrap_err(), AppError::BadRequest(_)));
}
