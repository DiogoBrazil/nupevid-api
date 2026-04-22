use uuid::Uuid;

use crate::core::application_error::ApplicationError as AppError;
use crate::core::entities::work_session_members::TeamMemberFunction;
use crate::usecases::work_sessions::guards::ensure_creator_or_commander;
use crate::usecases::work_sessions::test_support::session_member;

#[test]
fn creator_is_allowed() {
    let creator = Uuid::new_v4();
    let session_id = Uuid::new_v4();
    let members = vec![session_member(
        session_id,
        creator,
        Some(TeamMemberFunction::Commander),
    )];

    let result = ensure_creator_or_commander(creator, creator, &members, "forbidden");
    assert!(result.is_ok());
}

#[test]
fn commander_non_creator_is_allowed() {
    let creator = Uuid::new_v4();
    let commander = Uuid::new_v4();
    let session_id = Uuid::new_v4();
    let members = vec![
        session_member(session_id, creator, Some(TeamMemberFunction::Driver)),
        session_member(session_id, commander, Some(TeamMemberFunction::Commander)),
    ];

    let result = ensure_creator_or_commander(creator, commander, &members, "forbidden");
    assert!(result.is_ok());
}

#[test]
fn patroller_non_creator_is_forbidden() {
    let creator = Uuid::new_v4();
    let patroller = Uuid::new_v4();
    let session_id = Uuid::new_v4();
    let members = vec![
        session_member(session_id, creator, Some(TeamMemberFunction::Commander)),
        session_member(session_id, patroller, Some(TeamMemberFunction::Patroller)),
    ];

    let result = ensure_creator_or_commander(creator, patroller, &members, "forbidden by test");
    assert!(matches!(result.unwrap_err(), AppError::Forbidden(msg) if msg == "forbidden by test"));
}

#[test]
fn driver_non_creator_is_forbidden() {
    let creator = Uuid::new_v4();
    let driver = Uuid::new_v4();
    let session_id = Uuid::new_v4();
    let members = vec![
        session_member(session_id, creator, Some(TeamMemberFunction::Commander)),
        session_member(session_id, driver, Some(TeamMemberFunction::Driver)),
    ];

    let result = ensure_creator_or_commander(creator, driver, &members, "forbidden");
    assert!(matches!(result.unwrap_err(), AppError::Forbidden(_)));
}

#[test]
fn non_member_non_creator_is_forbidden() {
    let creator = Uuid::new_v4();
    let outsider = Uuid::new_v4();
    let members = vec![session_member(
        Uuid::new_v4(),
        creator,
        Some(TeamMemberFunction::Commander),
    )];

    let result = ensure_creator_or_commander(creator, outsider, &members, "forbidden");
    assert!(matches!(result.unwrap_err(), AppError::Forbidden(_)));
}
