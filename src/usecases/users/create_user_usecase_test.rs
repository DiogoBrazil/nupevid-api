use std::collections::HashMap;

use mockall::predicate::{always, eq};
use uuid::Uuid;

use crate::core::application_error::ApplicationError as AppError;
use crate::core::commands::users::CreateUser;
use crate::core::contracts::repository::error::RepositoryError;
use crate::core::contracts::repository::users::MockUserRepository;
use crate::core::value_objects::policies::{PermissionPolicies, Policy};
use crate::core::value_objects::profiles::Profile;
use crate::core::value_objects::ranks::Rank;
use crate::usecases::users::CreateUserUseCase;
use crate::usecases::users::test_support::{
    FakePasswordHasher, claims, deps, empty_policies, user_record,
};

fn create_cmd(profile: Profile, city_id: Option<Uuid>) -> CreateUser {
    CreateUser {
        rank: Rank::CapPm,
        registration: "100012345".to_string(),
        full_name: "Novo Usuario".to_string(),
        profile,
        email: "  NewUser@Test.COM  ".to_string(),
        password: "senha123".to_string(),
        city_id,
        permission_policies: None,
    }
}

#[tokio::test]
async fn root_creates_city_admin_success_and_applies_default_policies() {
    let root_id = Uuid::new_v4();
    let city_id = Uuid::new_v4();
    let created_id = Uuid::new_v4();

    let mut repo = MockUserRepository::new();
    repo.expect_check_city_admin_exists_for_city()
        .with(eq(city_id), eq(None))
        .returning(|_, _| Ok(false));
    repo.expect_check_user_exists_by_email()
        .with(eq("newuser@test.com"))
        .returning(|_| Ok(false));
    repo.expect_create_user().returning(move |data| {
        // Default CITY_ADMIN policies should be applied when none provided
        let policies = data.permission_policies.clone().unwrap_or_default();
        assert!(policies.contains_key(&Policy::CreateUsers));
        assert!(policies.contains_key(&Policy::ReadVictims));
        assert_eq!(data.email, "newuser@test.com");
        assert_eq!(data.password, "hashed:senha123");
        Ok(user_record(
            created_id,
            Profile::CityAdmin,
            Some(city_id),
            policies,
        ))
    });

    let usecase = CreateUserUseCase::new(deps(repo, FakePasswordHasher::ok()));

    let result = usecase
        .execute(
            create_cmd(Profile::CityAdmin, Some(city_id)),
            &claims(Profile::Root, root_id, None),
        )
        .await;

    let user = result.unwrap();
    assert_eq!(user.profile, Profile::CityAdmin);
    assert_eq!(user.city_id, Some(city_id));
}

#[tokio::test]
async fn city_user_cannot_create_users() {
    let city_id = Uuid::new_v4();
    let caller_id = Uuid::new_v4();

    let repo = MockUserRepository::new();
    let usecase = CreateUserUseCase::new(deps(repo, FakePasswordHasher::ok()));

    let result = usecase
        .execute(
            create_cmd(Profile::CityUser, Some(city_id)),
            &claims(Profile::CityUser, caller_id, Some(city_id)),
        )
        .await;

    assert!(matches!(
        result.unwrap_err(),
        AppError::Forbidden(msg) if msg.contains("CITY_USER")
    ));
}

#[tokio::test]
async fn city_admin_cannot_create_root() {
    let city_id = Uuid::new_v4();
    let caller_id = Uuid::new_v4();

    let repo = MockUserRepository::new();
    let usecase = CreateUserUseCase::new(deps(repo, FakePasswordHasher::ok()));

    let result = usecase
        .execute(
            create_cmd(Profile::Root, None),
            &claims(Profile::CityAdmin, caller_id, Some(city_id)),
        )
        .await;

    assert!(matches!(
        result.unwrap_err(),
        AppError::Forbidden(msg) if msg.contains("ROOT")
    ));
}

#[tokio::test]
async fn city_admin_cannot_create_other_city_admin() {
    let city_id = Uuid::new_v4();
    let caller_id = Uuid::new_v4();

    let repo = MockUserRepository::new();
    let usecase = CreateUserUseCase::new(deps(repo, FakePasswordHasher::ok()));

    let result = usecase
        .execute(
            create_cmd(Profile::CityAdmin, Some(city_id)),
            &claims(Profile::CityAdmin, caller_id, Some(city_id)),
        )
        .await;

    assert!(matches!(
        result.unwrap_err(),
        AppError::Forbidden(msg) if msg.contains("CITY_ADMIN")
    ));
}

#[tokio::test]
async fn city_admin_creates_city_user_with_own_city_forced() {
    let admin_city = Uuid::new_v4();
    let admin_id = Uuid::new_v4();
    let created_id = Uuid::new_v4();

    let mut repo = MockUserRepository::new();
    let admin_id_clone = admin_id;
    repo.expect_get_user_policies_by_id()
        .with(eq(admin_id_clone))
        .returning(|_| Ok(HashMap::new()));
    repo.expect_check_user_exists_by_email()
        .returning(|_| Ok(false));
    repo.expect_create_user().returning(move |data| {
        // CITY_ADMIN creating CITY_USER: city_id must be forced to the admin's city
        assert_eq!(data.city_id, Some(admin_city));
        Ok(user_record(
            created_id,
            Profile::CityUser,
            Some(admin_city),
            data.permission_policies.clone().unwrap_or_default(),
        ))
    });

    let other_city = Uuid::new_v4();
    let usecase = CreateUserUseCase::new(deps(repo, FakePasswordHasher::ok()));

    let result = usecase
        .execute(
            create_cmd(Profile::CityUser, Some(other_city)),
            &claims(Profile::CityAdmin, admin_id, Some(admin_city)),
        )
        .await;

    assert_eq!(result.unwrap().city_id, Some(admin_city));
}

#[tokio::test]
async fn city_admin_cannot_assign_policy_it_does_not_have() {
    let admin_city = Uuid::new_v4();
    let admin_id = Uuid::new_v4();

    let mut repo = MockUserRepository::new();
    repo.expect_get_user_policies_by_id()
        .with(eq(admin_id))
        .returning(|_| Ok(HashMap::new()));

    let mut cmd = create_cmd(Profile::CityUser, Some(admin_city));
    let mut policies: PermissionPolicies = HashMap::new();
    policies.insert(Policy::ReadVictims, vec![admin_city]);
    cmd.permission_policies = Some(policies);

    let usecase = CreateUserUseCase::new(deps(repo, FakePasswordHasher::ok()));

    let result = usecase
        .execute(cmd, &claims(Profile::CityAdmin, admin_id, Some(admin_city)))
        .await;

    assert!(matches!(
        result.unwrap_err(),
        AppError::Forbidden(msg) if msg.contains("cannot assign")
    ));
}

#[tokio::test]
async fn root_creating_city_admin_without_city_id_fails() {
    let root_id = Uuid::new_v4();

    let repo = MockUserRepository::new();
    let usecase = CreateUserUseCase::new(deps(repo, FakePasswordHasher::ok()));

    let result = usecase
        .execute(
            create_cmd(Profile::CityAdmin, None),
            &claims(Profile::Root, root_id, None),
        )
        .await;

    assert!(matches!(
        result.unwrap_err(),
        AppError::BadRequest(msg) if msg.contains("city_id is required")
    ));
}

#[tokio::test]
async fn duplicate_email_returns_bad_request() {
    let root_id = Uuid::new_v4();
    let city_id = Uuid::new_v4();

    let mut repo = MockUserRepository::new();
    repo.expect_check_city_admin_exists_for_city()
        .returning(|_, _| Ok(false));
    repo.expect_check_user_exists_by_email()
        .returning(|_| Ok(true));

    let usecase = CreateUserUseCase::new(deps(repo, FakePasswordHasher::ok()));

    let result = usecase
        .execute(
            create_cmd(Profile::CityAdmin, Some(city_id)),
            &claims(Profile::Root, root_id, None),
        )
        .await;

    assert!(matches!(
        result.unwrap_err(),
        AppError::BadRequest(msg) if msg.contains("already exists")
    ));
}

#[tokio::test]
async fn creating_second_city_admin_for_same_city_fails() {
    let root_id = Uuid::new_v4();
    let city_id = Uuid::new_v4();

    let mut repo = MockUserRepository::new();
    repo.expect_check_city_admin_exists_for_city()
        .with(eq(city_id), eq(None))
        .returning(|_, _| Ok(true));

    let usecase = CreateUserUseCase::new(deps(repo, FakePasswordHasher::ok()));

    let result = usecase
        .execute(
            create_cmd(Profile::CityAdmin, Some(city_id)),
            &claims(Profile::Root, root_id, None),
        )
        .await;

    assert!(matches!(
        result.unwrap_err(),
        AppError::BadRequest(msg) if msg.contains("CITY_ADMIN already exists")
    ));
}

#[tokio::test]
async fn invalid_registration_is_rejected() {
    let root_id = Uuid::new_v4();
    let city_id = Uuid::new_v4();

    let mut repo = MockUserRepository::new();
    repo.expect_check_city_admin_exists_for_city()
        .returning(|_, _| Ok(false));

    let mut cmd = create_cmd(Profile::CityAdmin, Some(city_id));
    cmd.registration = "ABC123".to_string();

    let usecase = CreateUserUseCase::new(deps(repo, FakePasswordHasher::ok()));

    let result = usecase
        .execute(cmd, &claims(Profile::Root, root_id, None))
        .await;

    assert!(matches!(
        result.unwrap_err(),
        AppError::BadRequest(msg) if msg.contains("invalid registration")
    ));
}

#[tokio::test]
async fn password_hash_failure_bubbles_up_as_internal_error() {
    let root_id = Uuid::new_v4();
    let city_id = Uuid::new_v4();

    let mut repo = MockUserRepository::new();
    repo.expect_check_city_admin_exists_for_city()
        .returning(|_, _| Ok(false));
    repo.expect_check_user_exists_by_email()
        .returning(|_| Ok(false));

    let usecase = CreateUserUseCase::new(deps(repo, FakePasswordHasher::failing()));

    let result = usecase
        .execute(
            create_cmd(Profile::CityAdmin, Some(city_id)),
            &claims(Profile::Root, root_id, None),
        )
        .await;

    assert!(matches!(result.unwrap_err(), AppError::InternalServerError));
}

#[tokio::test]
async fn duplicate_entry_from_repo_maps_to_bad_request() {
    let root_id = Uuid::new_v4();
    let city_id = Uuid::new_v4();

    let mut repo = MockUserRepository::new();
    repo.expect_check_city_admin_exists_for_city()
        .returning(|_, _| Ok(false));
    repo.expect_check_user_exists_by_email()
        .returning(|_| Ok(false));
    repo.expect_create_user()
        .returning(|_| Err(RepositoryError::DuplicateEntry("dup".to_string())));

    let usecase = CreateUserUseCase::new(deps(repo, FakePasswordHasher::ok()));

    let result = usecase
        .execute(
            create_cmd(Profile::CityAdmin, Some(city_id)),
            &claims(Profile::Root, root_id, None),
        )
        .await;

    assert!(matches!(
        result.unwrap_err(),
        AppError::BadRequest(msg) if msg == "dup"
    ));
}

#[tokio::test]
async fn referenced_entity_not_found_from_repo_maps_to_bad_request() {
    let root_id = Uuid::new_v4();
    let city_id = Uuid::new_v4();

    let mut repo = MockUserRepository::new();
    repo.expect_check_city_admin_exists_for_city()
        .returning(|_, _| Ok(false));
    repo.expect_check_user_exists_by_email()
        .returning(|_| Ok(false));
    repo.expect_create_user().returning(|_| {
        Err(RepositoryError::ReferencedEntityNotFound(
            "city missing".to_string(),
        ))
    });

    let usecase = CreateUserUseCase::new(deps(repo, FakePasswordHasher::ok()));

    let result = usecase
        .execute(
            create_cmd(Profile::CityAdmin, Some(city_id)),
            &claims(Profile::Root, root_id, None),
        )
        .await;

    assert!(matches!(
        result.unwrap_err(),
        AppError::BadRequest(msg) if msg == "city missing"
    ));
}

#[tokio::test]
async fn database_error_from_repo_maps_to_internal_server_error() {
    let root_id = Uuid::new_v4();
    let city_id = Uuid::new_v4();

    let mut repo = MockUserRepository::new();
    repo.expect_check_city_admin_exists_for_city()
        .returning(|_, _| Ok(false));
    repo.expect_check_user_exists_by_email()
        .returning(|_| Ok(false));
    repo.expect_create_user()
        .returning(|_| Err(RepositoryError::DatabaseError("boom".to_string())));

    let usecase = CreateUserUseCase::new(deps(repo, FakePasswordHasher::ok()));

    let result = usecase
        .execute(
            create_cmd(Profile::CityAdmin, Some(city_id)),
            &claims(Profile::Root, root_id, None),
        )
        .await;

    assert!(matches!(result.unwrap_err(), AppError::InternalServerError));
}

#[tokio::test]
async fn custom_policies_replace_defaults_when_provided() {
    let root_id = Uuid::new_v4();
    let city_id = Uuid::new_v4();
    let created_id = Uuid::new_v4();

    let mut repo = MockUserRepository::new();
    repo.expect_check_city_admin_exists_for_city()
        .returning(|_, _| Ok(false));
    repo.expect_check_user_exists_by_email()
        .returning(|_| Ok(false));
    repo.expect_create_user().returning(move |data| {
        let policies = data.permission_policies.clone().unwrap_or_default();
        // Custom policy list should pass through untouched (no default layer)
        assert_eq!(policies.len(), 1);
        assert!(policies.contains_key(&Policy::ReadVictims));
        Ok(user_record(
            created_id,
            Profile::CityUser,
            Some(city_id),
            policies,
        ))
    });

    let mut custom: PermissionPolicies = HashMap::new();
    custom.insert(Policy::ReadVictims, vec![city_id]);
    let mut cmd = create_cmd(Profile::CityUser, Some(city_id));
    cmd.permission_policies = Some(custom);

    let usecase = CreateUserUseCase::new(deps(repo, FakePasswordHasher::ok()));
    let result = usecase
        .execute(cmd, &claims(Profile::Root, root_id, None))
        .await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn check_email_uniqueness_internal_error_maps_to_internal_server_error() {
    let root_id = Uuid::new_v4();
    let city_id = Uuid::new_v4();

    let mut repo = MockUserRepository::new();
    repo.expect_check_city_admin_exists_for_city()
        .returning(|_, _| Ok(false));
    repo.expect_check_user_exists_by_email()
        .with(always())
        .returning(|_| Err(RepositoryError::DatabaseError("boom".to_string())));

    let usecase = CreateUserUseCase::new(deps(repo, FakePasswordHasher::ok()));
    let result = usecase
        .execute(
            create_cmd(Profile::CityAdmin, Some(city_id)),
            &claims(Profile::Root, root_id, None),
        )
        .await;

    assert!(matches!(result.unwrap_err(), AppError::InternalServerError));
}

// Sanity check: ensure empty_policies helper gives a HashMap we can use as a baseline
#[test]
fn empty_policies_is_empty_hashmap() {
    let p = empty_policies();
    assert!(p.is_empty());
}
