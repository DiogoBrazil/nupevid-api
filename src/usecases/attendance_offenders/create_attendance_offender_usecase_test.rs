use chrono::NaiveDate;
use uuid::Uuid;

use crate::core::application_error::ApplicationError as AppError;
use crate::core::entities::attendance_offenders::ViolenceAggravator;
use crate::core::value_objects::policies::Policy;
use crate::core::value_objects::profiles::Profile;
use crate::usecases::attendance_offenders::CreateAttendanceOffenderUseCase;
use crate::usecases::attendance_offenders::test_support::{
    FakeAttendanceOffenderReadRepo, FakeAttendanceOffenderWriteRepo, FakeMemberRepo,
    FakeOffenderReadRepo, FakePmReadRepo, FakeWorkSessionReadRepo, attendance_offender_entity,
    claims, create_command, deps, offender_with_details, protective_measure, session_member_entity,
    user_repo_with_policy, user_repo_without_policy, victim_repo_ok, work_session_active,
};

#[tokio::test]
async fn create_fails_when_no_active_session() {
    let city_id = Uuid::new_v4();
    let requester_id = Uuid::new_v4();
    let victim_id = Uuid::new_v4();
    let offender_id = Uuid::new_v4();
    let pm_id = Uuid::new_v4();

    let (d, _) = deps(
        FakeAttendanceOffenderReadRepo { attendance: None },
        FakeAttendanceOffenderWriteRepo::internal(),
        FakeOffenderReadRepo::found(offender_with_details(offender_id, city_id)),
        victim_repo_ok(victim_id, city_id),
        FakePmReadRepo {
            measure: Some(protective_measure(pm_id, victim_id, offender_id)),
        },
        user_repo_with_policy(city_id, Policy::CreateAttendances),
        FakeWorkSessionReadRepo::without_session(),
        FakeMemberRepo::idle(),
    );
    let usecase = CreateAttendanceOffenderUseCase::new(d);

    let result = usecase
        .execute(
            create_command(pm_id),
            &claims(Profile::CityAdmin, requester_id, Some(city_id)),
        )
        .await;

    assert!(
        matches!(result.unwrap_err(), AppError::BadRequest(msg) if msg.contains("active work session"))
    );
}

#[tokio::test]
async fn create_snapshots_session_members_at_creation_time() {
    let city_id = Uuid::new_v4();
    let requester_id = Uuid::new_v4();
    let member_id = Uuid::new_v4();
    let victim_id = Uuid::new_v4();
    let offender_id = Uuid::new_v4();
    let pm_id = Uuid::new_v4();
    let att_id = Uuid::new_v4();
    let session_id = Uuid::new_v4();

    let session = work_session_active(session_id, requester_id);
    let members = vec![
        session_member_entity(session_id, requester_id),
        session_member_entity(session_id, member_id),
    ];

    let (d, write_arc) = deps(
        FakeAttendanceOffenderReadRepo { attendance: None },
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
        user_repo_with_policy(city_id, Policy::CreateAttendances),
        FakeWorkSessionReadRepo::with_session(session, members),
        FakeMemberRepo::idle(),
    );
    let usecase = CreateAttendanceOffenderUseCase::new(d);

    let result = usecase
        .execute(
            create_command(pm_id),
            &claims(Profile::CityAdmin, requester_id, Some(city_id)),
        )
        .await;

    assert!(result.is_ok());

    let captured = write_arc
        .session_members_captured
        .lock()
        .unwrap()
        .clone()
        .unwrap();
    assert_eq!(captured.len(), 2);
    let captured_users: Vec<Uuid> = captured.iter().map(|(u, _)| *u).collect();
    assert!(captured_users.contains(&requester_id));
    assert!(captured_users.contains(&member_id));
}

#[tokio::test]
async fn create_maps_referenced_entity_not_found_to_bad_request() {
    let city_id = Uuid::new_v4();
    let requester_id = Uuid::new_v4();
    let victim_id = Uuid::new_v4();
    let offender_id = Uuid::new_v4();
    let pm_id = Uuid::new_v4();
    let session_id = Uuid::new_v4();

    let (d, _) = deps(
        FakeAttendanceOffenderReadRepo { attendance: None },
        FakeAttendanceOffenderWriteRepo::referenced("city missing"),
        FakeOffenderReadRepo::found(offender_with_details(offender_id, city_id)),
        victim_repo_ok(victim_id, city_id),
        FakePmReadRepo {
            measure: Some(protective_measure(pm_id, victim_id, offender_id)),
        },
        user_repo_with_policy(city_id, Policy::CreateAttendances),
        FakeWorkSessionReadRepo::with_session(
            work_session_active(session_id, requester_id),
            vec![session_member_entity(session_id, requester_id)],
        ),
        FakeMemberRepo::idle(),
    );
    let usecase = CreateAttendanceOffenderUseCase::new(d);

    let result = usecase
        .execute(
            create_command(pm_id),
            &claims(Profile::CityAdmin, requester_id, Some(city_id)),
        )
        .await;

    assert!(matches!(result.unwrap_err(), AppError::BadRequest(_)));
}

#[tokio::test]
async fn create_fails_when_policy_missing() {
    let city_id = Uuid::new_v4();
    let requester_id = Uuid::new_v4();
    let victim_id = Uuid::new_v4();
    let offender_id = Uuid::new_v4();
    let pm_id = Uuid::new_v4();

    let (d, _) = deps(
        FakeAttendanceOffenderReadRepo { attendance: None },
        FakeAttendanceOffenderWriteRepo::internal(),
        FakeOffenderReadRepo::found(offender_with_details(offender_id, city_id)),
        victim_repo_ok(victim_id, city_id),
        FakePmReadRepo {
            measure: Some(protective_measure(pm_id, victim_id, offender_id)),
        },
        user_repo_without_policy(),
        FakeWorkSessionReadRepo::without_session(),
        FakeMemberRepo::idle(),
    );
    let usecase = CreateAttendanceOffenderUseCase::new(d);

    let result = usecase
        .execute(
            create_command(pm_id),
            &claims(Profile::CityAdmin, requester_id, Some(city_id)),
        )
        .await;

    assert!(matches!(result.unwrap_err(), AppError::Forbidden(_)));
}

#[tokio::test]
async fn create_rejects_future_attendance_date() {
    let city_id = Uuid::new_v4();
    let requester_id = Uuid::new_v4();
    let victim_id = Uuid::new_v4();
    let offender_id = Uuid::new_v4();
    let pm_id = Uuid::new_v4();

    let (d, _) = deps(
        FakeAttendanceOffenderReadRepo { attendance: None },
        FakeAttendanceOffenderWriteRepo::internal(),
        FakeOffenderReadRepo::found(offender_with_details(offender_id, city_id)),
        victim_repo_ok(victim_id, city_id),
        FakePmReadRepo {
            measure: Some(protective_measure(pm_id, victim_id, offender_id)),
        },
        user_repo_with_policy(city_id, Policy::CreateAttendances),
        FakeWorkSessionReadRepo::without_session(),
        FakeMemberRepo::idle(),
    );
    let usecase = CreateAttendanceOffenderUseCase::new(d);

    let mut command = create_command(pm_id);
    command.attendance_date = NaiveDate::from_ymd_opt(2999, 1, 1).unwrap();

    let result = usecase
        .execute(
            command,
            &claims(Profile::CityAdmin, requester_id, Some(city_id)),
        )
        .await;

    assert!(
        matches!(result.unwrap_err(), AppError::BadRequest(msg) if msg.contains("attendance_date"))
    );
}

#[tokio::test]
async fn create_rejects_other_aggravator_without_text() {
    let city_id = Uuid::new_v4();
    let requester_id = Uuid::new_v4();
    let victim_id = Uuid::new_v4();
    let offender_id = Uuid::new_v4();
    let pm_id = Uuid::new_v4();

    let (d, _) = deps(
        FakeAttendanceOffenderReadRepo { attendance: None },
        FakeAttendanceOffenderWriteRepo::internal(),
        FakeOffenderReadRepo::found(offender_with_details(offender_id, city_id)),
        victim_repo_ok(victim_id, city_id),
        FakePmReadRepo {
            measure: Some(protective_measure(pm_id, victim_id, offender_id)),
        },
        user_repo_with_policy(city_id, Policy::CreateAttendances),
        FakeWorkSessionReadRepo::without_session(),
        FakeMemberRepo::idle(),
    );
    let usecase = CreateAttendanceOffenderUseCase::new(d);

    let mut command = create_command(pm_id);
    command.violence_aggravator = Some(ViolenceAggravator::Other);
    command.violence_aggravator_other = None;

    let result = usecase
        .execute(
            command,
            &claims(Profile::CityAdmin, requester_id, Some(city_id)),
        )
        .await;

    assert!(
        matches!(result.unwrap_err(), AppError::BadRequest(msg) if msg.contains("is required"))
    );
}

#[tokio::test]
async fn create_rejects_other_text_without_other_aggravator() {
    let city_id = Uuid::new_v4();
    let requester_id = Uuid::new_v4();
    let victim_id = Uuid::new_v4();
    let offender_id = Uuid::new_v4();
    let pm_id = Uuid::new_v4();

    let (d, _) = deps(
        FakeAttendanceOffenderReadRepo { attendance: None },
        FakeAttendanceOffenderWriteRepo::internal(),
        FakeOffenderReadRepo::found(offender_with_details(offender_id, city_id)),
        victim_repo_ok(victim_id, city_id),
        FakePmReadRepo {
            measure: Some(protective_measure(pm_id, victim_id, offender_id)),
        },
        user_repo_with_policy(city_id, Policy::CreateAttendances),
        FakeWorkSessionReadRepo::without_session(),
        FakeMemberRepo::idle(),
    );
    let usecase = CreateAttendanceOffenderUseCase::new(d);

    let mut command = create_command(pm_id);
    command.violence_aggravator = Some(ViolenceAggravator::AlcoholUse);
    command.violence_aggravator_other = Some("texto indevido".to_string());

    let result = usecase
        .execute(
            command,
            &claims(Profile::CityAdmin, requester_id, Some(city_id)),
        )
        .await;

    assert!(
        matches!(result.unwrap_err(), AppError::BadRequest(msg) if msg.contains("is only allowed"))
    );
}

#[tokio::test]
async fn create_succeeds_with_no_aggravator() {
    let city_id = Uuid::new_v4();
    let requester_id = Uuid::new_v4();
    let victim_id = Uuid::new_v4();
    let offender_id = Uuid::new_v4();
    let pm_id = Uuid::new_v4();
    let att_id = Uuid::new_v4();
    let session_id = Uuid::new_v4();

    let (d, _) = deps(
        FakeAttendanceOffenderReadRepo { attendance: None },
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
        user_repo_with_policy(city_id, Policy::CreateAttendances),
        FakeWorkSessionReadRepo::with_session(
            work_session_active(session_id, requester_id),
            vec![session_member_entity(session_id, requester_id)],
        ),
        FakeMemberRepo::idle(),
    );
    let usecase = CreateAttendanceOffenderUseCase::new(d);

    let mut command = create_command(pm_id);
    command.violence_aggravator = None;
    command.violence_aggravator_other = None;

    let result = usecase
        .execute(
            command,
            &claims(Profile::CityAdmin, requester_id, Some(city_id)),
        )
        .await;

    assert!(result.is_ok());
}
