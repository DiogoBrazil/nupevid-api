use std::collections::HashMap;

use mockall::predicate::eq;
use uuid::Uuid;

use crate::core::application_error::ApplicationError as AppError;
use crate::core::contracts::repository::error::RepositoryError;
use crate::core::contracts::repository::users::MockUserRepository;
use crate::core::value_objects::policies::{PermissionPolicies, Policy};
use crate::core::value_objects::profiles::Profile;
use crate::usecases::users::DeleteUserByIdUseCase;
use crate::usecases::users::test_support::{
    FakePasswordHasher, claims, deps, empty_policies, user_record,
};

#[tokio::test]
async fn root_deletes_user_successfully() {
    let root_id = Uuid::new_v4();
    let target_id = Uuid::new_v4();
    let city_id = Uuid::new_v4();

    let mut repo = MockUserRepository::new();
    repo.expect_get_user_by_id()
        .with(eq(target_id))
        .returning(move |_| {
            Ok(user_record(
                target_id,
                Profile::CityUser,
                Some(city_id),
                empty_policies(),
            ))
        });
    repo.expect_delete_user_by_id()
        .with(eq(target_id))
        .returning(move |_| {
            let mut u = user_record(
                target_id,
                Profile::CityUser,
                Some(city_id),
                empty_policies(),
            );
            u.is_deleted = true;
            Ok(u)
        });

    let usecase = DeleteUserByIdUseCase::new(deps(repo, FakePasswordHasher::ok()));
    let result = usecase
        .execute(target_id, &claims(Profile::Root, root_id, None))
        .await;

    assert!(result.unwrap().is_deleted);
}

#[tokio::test]
async fn non_root_cannot_delete_root_user() {
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

    let usecase = DeleteUserByIdUseCase::new(deps(repo, FakePasswordHasher::ok()));
    let result = usecase
        .execute(
            target_id,
            &claims(Profile::CityAdmin, admin_id, Some(admin_city)),
        )
        .await;

    assert!(matches!(
        result.unwrap_err(),
        AppError::Forbidden(msg) if msg.contains("ROOT")
    ));
}

#[tokio::test]
async fn city_admin_without_delete_policy_is_forbidden() {
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
        .returning(|_| Ok(HashMap::new()));

    let usecase = DeleteUserByIdUseCase::new(deps(repo, FakePasswordHasher::ok()));
    let result = usecase
        .execute(
            target_id,
            &claims(Profile::CityAdmin, admin_id, Some(admin_city)),
        )
        .await;

    assert!(matches!(result.unwrap_err(), AppError::Forbidden(_)));
}

#[tokio::test]
async fn city_admin_with_delete_policy_can_delete_city_user() {
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
            p.insert(Policy::DeleteUsers, vec![admin_city]);
            Ok(p)
        });
    repo.expect_delete_user_by_id().returning(move |_| {
        Ok(user_record(
            target_id,
            Profile::CityUser,
            Some(admin_city),
            empty_policies(),
        ))
    });

    let usecase = DeleteUserByIdUseCase::new(deps(repo, FakePasswordHasher::ok()));
    let result = usecase
        .execute(
            target_id,
            &claims(Profile::CityAdmin, admin_id, Some(admin_city)),
        )
        .await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn not_found_when_user_missing() {
    let root_id = Uuid::new_v4();
    let target_id = Uuid::new_v4();

    let mut repo = MockUserRepository::new();
    repo.expect_get_user_by_id()
        .returning(|_| Err(RepositoryError::NotFound));

    let usecase = DeleteUserByIdUseCase::new(deps(repo, FakePasswordHasher::ok()));
    let result = usecase
        .execute(target_id, &claims(Profile::Root, root_id, None))
        .await;

    assert!(matches!(result.unwrap_err(), AppError::NotFound(_)));
}

#[tokio::test]
async fn database_error_maps_to_internal_server_error() {
    let root_id = Uuid::new_v4();
    let target_id = Uuid::new_v4();

    let mut repo = MockUserRepository::new();
    repo.expect_get_user_by_id().returning(move |_| {
        Ok(user_record(
            target_id,
            Profile::CityUser,
            Some(Uuid::new_v4()),
            empty_policies(),
        ))
    });
    repo.expect_delete_user_by_id()
        .returning(|_| Err(RepositoryError::DatabaseError("boom".to_string())));

    let usecase = DeleteUserByIdUseCase::new(deps(repo, FakePasswordHasher::ok()));
    let result = usecase
        .execute(target_id, &claims(Profile::Root, root_id, None))
        .await;

    assert!(matches!(result.unwrap_err(), AppError::InternalServerError));
}
