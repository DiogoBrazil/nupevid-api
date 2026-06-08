use std::collections::HashMap;

use mockall::predicate::eq;
use uuid::Uuid;

use crate::core::application_error::ApplicationError as AppError;
use crate::core::contracts::repository::error::RepositoryError;
use crate::core::contracts::repository::users::MockUserRepository;
use crate::core::value_objects::policies::{PermissionPolicies, Policy};
use crate::core::value_objects::profiles::Profile;
use crate::usecases::users::GetUserByIdUseCase;
use crate::usecases::users::test_support::{
    FakePasswordHasher, claims, deps, empty_policies, user_record,
};

#[tokio::test]
async fn root_reads_any_user() {
    let root_id = Uuid::new_v4();
    let target_id = Uuid::new_v4();

    let mut repo = MockUserRepository::new();
    repo.expect_get_user_by_id().returning(move |_| {
        Ok(user_record(
            target_id,
            Profile::Root,
            None,
            empty_policies(),
        ))
    });

    let usecase = GetUserByIdUseCase::new(deps(repo, FakePasswordHasher::ok()));
    let result = usecase
        .execute(target_id, &claims(Profile::Root, root_id, None))
        .await;
    assert_eq!(result.unwrap().id, target_id);
}

#[tokio::test]
async fn non_root_cannot_read_root_user() {
    let admin_city = Uuid::new_v4();
    let admin_id = Uuid::new_v4();
    let target_id = Uuid::new_v4();

    let mut repo = MockUserRepository::new();
    repo.expect_get_user_by_id().returning(move |_| {
        Ok(user_record(
            target_id,
            Profile::Root,
            None,
            empty_policies(),
        ))
    });

    let usecase = GetUserByIdUseCase::new(deps(repo, FakePasswordHasher::ok()));
    let result = usecase
        .execute(
            target_id,
            &claims(Profile::CityAdmin, admin_id, Some(admin_city)),
        )
        .await;

    assert!(matches!(result.unwrap_err(), AppError::Forbidden(_)));
}

#[tokio::test]
async fn city_admin_with_read_policy_succeeds() {
    let admin_city = Uuid::new_v4();
    let admin_id = Uuid::new_v4();
    let target_id = Uuid::new_v4();

    let mut repo = MockUserRepository::new();
    repo.expect_get_user_by_id().returning(move |_| {
        Ok(user_record(
            target_id,
            Profile::CityUser,
            Some(admin_city),
            empty_policies(),
        ))
    });
    repo.expect_get_user_policies_by_id()
        .with(eq(admin_id))
        .returning(move |_| {
            let mut p: PermissionPolicies = HashMap::new();
            p.insert(Policy::ReadUsers, vec![admin_city]);
            Ok(p)
        });

    let usecase = GetUserByIdUseCase::new(deps(repo, FakePasswordHasher::ok()));
    let result = usecase
        .execute(
            target_id,
            &claims(Profile::CityAdmin, admin_id, Some(admin_city)),
        )
        .await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn city_admin_without_read_policy_is_forbidden() {
    let admin_city = Uuid::new_v4();
    let admin_id = Uuid::new_v4();
    let target_id = Uuid::new_v4();

    let mut repo = MockUserRepository::new();
    repo.expect_get_user_by_id().returning(move |_| {
        Ok(user_record(
            target_id,
            Profile::CityUser,
            Some(admin_city),
            empty_policies(),
        ))
    });
    repo.expect_get_user_policies_by_id()
        .returning(|_| Ok(HashMap::new()));

    let usecase = GetUserByIdUseCase::new(deps(repo, FakePasswordHasher::ok()));
    let result = usecase
        .execute(
            target_id,
            &claims(Profile::CityAdmin, admin_id, Some(admin_city)),
        )
        .await;

    assert!(matches!(result.unwrap_err(), AppError::Forbidden(_)));
}

#[tokio::test]
async fn not_found_bubbles_up() {
    let root_id = Uuid::new_v4();
    let target_id = Uuid::new_v4();

    let mut repo = MockUserRepository::new();
    repo.expect_get_user_by_id()
        .returning(|_| Err(RepositoryError::NotFound));

    let usecase = GetUserByIdUseCase::new(deps(repo, FakePasswordHasher::ok()));
    let result = usecase
        .execute(target_id, &claims(Profile::Root, root_id, None))
        .await;

    assert!(matches!(result.unwrap_err(), AppError::NotFound(_)));
}
