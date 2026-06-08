use std::collections::HashMap;

use uuid::Uuid;

use crate::core::application_error::ApplicationError as AppError;
use crate::core::contracts::repository::users::MockUserRepository;
use crate::core::entities::common::{AddressData, AddressType};
use crate::core::value_objects::policies::Policy;
use crate::core::value_objects::profiles::Profile;
use crate::usecases::offenders::CreateOffenderUseCase;
use crate::usecases::offenders::test_support::{
    FakeOffenderReadRepo, FakeOffenderWriteRepo, base_save_offender, claims, deps, offender_entity,
};

fn user_repo_with_policy(city_id: Uuid) -> MockUserRepository {
    let mut repo = MockUserRepository::new();
    let mut policies = HashMap::new();
    policies.insert(Policy::CreateOffenders, vec![city_id]);
    repo.expect_get_user_policies_by_id()
        .returning(move |_| Ok(policies.clone()));
    repo
}

fn user_repo_without_policy() -> MockUserRepository {
    let mut repo = MockUserRepository::new();
    repo.expect_get_user_policies_by_id()
        .returning(|_| Ok(HashMap::new()));
    repo
}

#[tokio::test]
async fn create_succeeds_when_valid_cpf_and_policy_present() {
    let city_id = Uuid::new_v4();
    let offender_id = Uuid::new_v4();
    let mut cmd = base_save_offender(Some(city_id), "JOAO TESTE");
    cmd.cpf = Some("529.982.247-25".to_string());

    let usecase = CreateOffenderUseCase::new(deps(
        FakeOffenderReadRepo::not_expected(),
        FakeOffenderWriteRepo::success(offender_entity(offender_id, city_id, "JOAO TESTE")),
        user_repo_with_policy(city_id),
    ));

    let result = usecase
        .execute(cmd, &claims(Profile::CityAdmin, Some(city_id)))
        .await;

    let offender = result.unwrap();
    assert_eq!(offender.summary.full_name, "JOAO TESTE");
    assert_eq!(offender.summary.city_id, city_id);
}

#[tokio::test]
async fn create_fails_on_duplicate_cpf() {
    let city_id = Uuid::new_v4();
    let cmd = base_save_offender(Some(city_id), "JOAO TESTE");

    let usecase = CreateOffenderUseCase::new(deps(
        FakeOffenderReadRepo::not_expected(),
        FakeOffenderWriteRepo::duplicate("offender with this cpf already exists"),
        user_repo_with_policy(city_id),
    ));

    let result = usecase
        .execute(cmd, &claims(Profile::CityAdmin, Some(city_id)))
        .await;

    assert!(matches!(result.unwrap_err(), AppError::Conflict(_)));
}

#[tokio::test]
async fn create_fails_on_policy_missing() {
    let city_id = Uuid::new_v4();
    let cmd = base_save_offender(Some(city_id), "JOAO TESTE");

    let usecase = CreateOffenderUseCase::new(deps(
        FakeOffenderReadRepo::not_expected(),
        FakeOffenderWriteRepo::internal(),
        user_repo_without_policy(),
    ));

    let result = usecase
        .execute(cmd, &claims(Profile::CityAdmin, Some(city_id)))
        .await;

    assert!(matches!(result.unwrap_err(), AppError::Forbidden(_)));
}

#[tokio::test]
async fn create_derives_city_from_residential_address_when_city_id_absent() {
    let residential_city = Uuid::new_v4();
    let offender_id = Uuid::new_v4();
    let mut cmd = base_save_offender(None, "USER");
    cmd.addresses = Some(vec![AddressData {
        street: Some("Rua A".to_string()),
        number: Some("1".to_string()),
        district: None,
        city_id: residential_city,
        zip_code: None,
        complement: None,
        address_type: AddressType::Residential,
    }]);

    let usecase = CreateOffenderUseCase::new(deps(
        FakeOffenderReadRepo::not_expected(),
        FakeOffenderWriteRepo::success(offender_entity(offender_id, residential_city, "USER")),
        user_repo_with_policy(residential_city),
    ));

    let result = usecase
        .execute(cmd, &claims(Profile::CityAdmin, Some(residential_city)))
        .await;

    assert_eq!(result.unwrap().summary.city_id, residential_city);
}

#[tokio::test]
async fn create_fails_when_city_id_and_addresses_absent() {
    let city_id = Uuid::new_v4();
    let cmd = base_save_offender(None, "USER");

    let usecase = CreateOffenderUseCase::new(deps(
        FakeOffenderReadRepo::not_expected(),
        FakeOffenderWriteRepo::internal(),
        MockUserRepository::new(),
    ));

    let result = usecase
        .execute(cmd, &claims(Profile::CityAdmin, Some(city_id)))
        .await;

    assert!(matches!(result.unwrap_err(), AppError::BadRequest(msg) if msg.contains("city_id")));
}

#[tokio::test]
async fn create_maps_referenced_entity_not_found_to_bad_request() {
    let city_id = Uuid::new_v4();
    let cmd = base_save_offender(Some(city_id), "USER");

    let usecase = CreateOffenderUseCase::new(deps(
        FakeOffenderReadRepo::not_expected(),
        FakeOffenderWriteRepo::referenced("City not found"),
        user_repo_with_policy(city_id),
    ));

    let result = usecase
        .execute(cmd, &claims(Profile::CityAdmin, Some(city_id)))
        .await;

    assert!(matches!(result.unwrap_err(), AppError::BadRequest(_)));
}

#[tokio::test]
async fn create_maps_generic_repository_error_to_internal() {
    let city_id = Uuid::new_v4();
    let cmd = base_save_offender(Some(city_id), "USER");

    let usecase = CreateOffenderUseCase::new(deps(
        FakeOffenderReadRepo::not_expected(),
        FakeOffenderWriteRepo::internal(),
        user_repo_with_policy(city_id),
    ));

    let result = usecase
        .execute(cmd, &claims(Profile::CityAdmin, Some(city_id)))
        .await;

    assert!(matches!(result.unwrap_err(), AppError::InternalServerError));
}

#[tokio::test]
async fn create_root_bypasses_policy_lookup() {
    let city_id = Uuid::new_v4();
    let offender_id = Uuid::new_v4();
    let cmd = base_save_offender(Some(city_id), "USER");

    let usecase = CreateOffenderUseCase::new(deps(
        FakeOffenderReadRepo::not_expected(),
        FakeOffenderWriteRepo::success(offender_entity(offender_id, city_id, "USER")),
        MockUserRepository::new(),
    ));

    let result = usecase.execute(cmd, &claims(Profile::Root, None)).await;

    assert!(result.is_ok());
}

#[tokio::test]
async fn create_fails_with_empty_full_name() {
    let city_id = Uuid::new_v4();
    let cmd = base_save_offender(Some(city_id), "   ");

    let usecase = CreateOffenderUseCase::new(deps(
        FakeOffenderReadRepo::not_expected(),
        FakeOffenderWriteRepo::internal(),
        user_repo_with_policy(city_id),
    ));

    let result = usecase
        .execute(cmd, &claims(Profile::CityAdmin, Some(city_id)))
        .await;

    assert!(matches!(result.unwrap_err(), AppError::BadRequest(msg) if msg.contains("full_name")));
}
