use std::collections::HashMap;

use uuid::Uuid;

use crate::core::application_error::ApplicationError as AppError;
use crate::core::contracts::repository::users::MockUserRepository;
use crate::core::value_objects::policies::Policy;
use crate::core::value_objects::profiles::Profile;
use crate::usecases::offenders::UpdateOffenderUseCase;
use crate::usecases::offenders::test_support::{
    FakeOffenderReadRepo, FakeOffenderWriteRepo, base_save_offender, claims, deps, offender_entity,
    offender_with_details,
};

fn user_repo_with_policy_on(city_ids: &[Uuid]) -> MockUserRepository {
    let mut repo = MockUserRepository::new();
    let mut policies = HashMap::new();
    policies.insert(Policy::UpdateOffenders, city_ids.to_vec());
    repo.expect_get_user_policies_by_id()
        .returning(move |_| Ok(policies.clone()));
    repo
}

#[tokio::test]
async fn update_succeeds_when_policy_present_for_same_city() {
    let city_id = Uuid::new_v4();
    let offender_id = Uuid::new_v4();
    let existing = offender_with_details(offender_id, city_id, "EXISTING");
    let cmd = base_save_offender(Some(city_id), "UPDATED");

    let usecase = UpdateOffenderUseCase::new(deps(
        FakeOffenderReadRepo::found(existing),
        FakeOffenderWriteRepo::success(offender_entity(offender_id, city_id, "UPDATED")),
        user_repo_with_policy_on(&[city_id]),
    ));

    let result = usecase
        .execute(cmd, offender_id, &claims(Profile::CityAdmin, Some(city_id)))
        .await;

    assert_eq!(result.unwrap().summary.full_name, "UPDATED");
}

#[tokio::test]
async fn update_change_city_requires_policy_on_new_city() {
    let old_city = Uuid::new_v4();
    let new_city = Uuid::new_v4();
    let offender_id = Uuid::new_v4();
    let existing = offender_with_details(offender_id, old_city, "EXISTING");
    // User only has policy on old_city; tries to move to new_city
    let cmd = base_save_offender(Some(new_city), "UPDATED");

    let usecase = UpdateOffenderUseCase::new(deps(
        FakeOffenderReadRepo::found(existing),
        FakeOffenderWriteRepo::internal(),
        user_repo_with_policy_on(&[old_city]),
    ));

    let result = usecase
        .execute(
            cmd,
            offender_id,
            &claims(Profile::CityAdmin, Some(old_city)),
        )
        .await;

    assert!(matches!(result.unwrap_err(), AppError::Forbidden(_)));
}

#[tokio::test]
async fn update_change_city_requires_policy_on_old_city() {
    let old_city = Uuid::new_v4();
    let new_city = Uuid::new_v4();
    let offender_id = Uuid::new_v4();
    let existing = offender_with_details(offender_id, old_city, "EXISTING");
    // User only has policy on new_city, cannot modify an offender sitting in old_city
    let cmd = base_save_offender(Some(new_city), "UPDATED");

    let usecase = UpdateOffenderUseCase::new(deps(
        FakeOffenderReadRepo::found(existing),
        FakeOffenderWriteRepo::internal(),
        user_repo_with_policy_on(&[new_city]),
    ));

    let result = usecase
        .execute(
            cmd,
            offender_id,
            &claims(Profile::CityAdmin, Some(new_city)),
        )
        .await;

    assert!(matches!(result.unwrap_err(), AppError::Forbidden(_)));
}

#[tokio::test]
async fn update_succeeds_when_policy_present_on_both_cities() {
    let old_city = Uuid::new_v4();
    let new_city = Uuid::new_v4();
    let offender_id = Uuid::new_v4();
    let existing = offender_with_details(offender_id, old_city, "EXISTING");
    let cmd = base_save_offender(Some(new_city), "UPDATED");

    let usecase = UpdateOffenderUseCase::new(deps(
        FakeOffenderReadRepo::found(existing),
        FakeOffenderWriteRepo::success(offender_entity(offender_id, new_city, "UPDATED")),
        user_repo_with_policy_on(&[old_city, new_city]),
    ));

    let result = usecase
        .execute(
            cmd,
            offender_id,
            &claims(Profile::CityAdmin, Some(old_city)),
        )
        .await;

    assert_eq!(result.unwrap().summary.city_id, new_city);
}

#[tokio::test]
async fn update_returns_not_found_when_offender_missing() {
    let city_id = Uuid::new_v4();
    let offender_id = Uuid::new_v4();
    let cmd = base_save_offender(Some(city_id), "UPDATED");

    let usecase = UpdateOffenderUseCase::new(deps(
        FakeOffenderReadRepo::not_found(),
        FakeOffenderWriteRepo::internal(),
        user_repo_with_policy_on(&[city_id]),
    ));

    let result = usecase
        .execute(cmd, offender_id, &claims(Profile::CityAdmin, Some(city_id)))
        .await;

    assert!(matches!(result.unwrap_err(), AppError::NotFound(_)));
}

#[tokio::test]
async fn update_maps_duplicate_entry_to_conflict() {
    let city_id = Uuid::new_v4();
    let offender_id = Uuid::new_v4();
    let existing = offender_with_details(offender_id, city_id, "EXISTING");
    let cmd = base_save_offender(Some(city_id), "UPDATED");

    let usecase = UpdateOffenderUseCase::new(deps(
        FakeOffenderReadRepo::found(existing),
        FakeOffenderWriteRepo::duplicate("cpf already in use"),
        user_repo_with_policy_on(&[city_id]),
    ));

    let result = usecase
        .execute(cmd, offender_id, &claims(Profile::CityAdmin, Some(city_id)))
        .await;

    assert!(matches!(result.unwrap_err(), AppError::Conflict(_)));
}

#[tokio::test]
async fn update_maps_referenced_entity_not_found_to_bad_request() {
    let city_id = Uuid::new_v4();
    let offender_id = Uuid::new_v4();
    let existing = offender_with_details(offender_id, city_id, "EXISTING");
    let cmd = base_save_offender(Some(city_id), "UPDATED");

    let usecase = UpdateOffenderUseCase::new(deps(
        FakeOffenderReadRepo::found(existing),
        FakeOffenderWriteRepo::referenced("city not found"),
        user_repo_with_policy_on(&[city_id]),
    ));

    let result = usecase
        .execute(cmd, offender_id, &claims(Profile::CityAdmin, Some(city_id)))
        .await;

    assert!(matches!(result.unwrap_err(), AppError::BadRequest(_)));
}

#[tokio::test]
async fn update_root_bypasses_policy_check() {
    let old_city = Uuid::new_v4();
    let new_city = Uuid::new_v4();
    let offender_id = Uuid::new_v4();
    let existing = offender_with_details(offender_id, old_city, "EXISTING");
    let cmd = base_save_offender(Some(new_city), "UPDATED");

    let usecase = UpdateOffenderUseCase::new(deps(
        FakeOffenderReadRepo::found(existing),
        FakeOffenderWriteRepo::success(offender_entity(offender_id, new_city, "UPDATED")),
        MockUserRepository::new(),
    ));

    let result = usecase
        .execute(cmd, offender_id, &claims(Profile::Root, None))
        .await;

    assert!(result.is_ok());
}
