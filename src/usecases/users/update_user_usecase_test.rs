use std::collections::HashMap;

use mockall::predicate::eq;
use uuid::Uuid;

use crate::core::application_error::ApplicationError as AppError;
use crate::core::commands::users::UpdateUser;
use crate::core::contracts::repository::error::RepositoryError;
use crate::core::contracts::repository::users::MockUserRepository;
use crate::core::value_objects::policies::{PermissionPolicies, Policy};
use crate::core::value_objects::profiles::Profile;
use crate::core::value_objects::ranks::Rank;
use crate::usecases::users::UpdateUserUseCase;
use crate::usecases::users::test_support::{
    FakePasswordHasher, claims, deps, empty_policies, user_record,
};

fn update_cmd(profile: Profile, city_id: Option<Uuid>) -> UpdateUser {
    UpdateUser {
        rank: Rank::CapPm,
        registration: "100012345".to_string(),
        full_name: "Nome Atualizado".to_string(),
        profile,
        email: "  Updated@Test.COM  ".to_string(),
        city_id,
        permission_policies: None,
    }
}

#[tokio::test]
async fn root_updates_city_admin_successfully() {
    let root_id = Uuid::new_v4();
    let target_id = Uuid::new_v4();
    let city_id = Uuid::new_v4();

    let mut repo = MockUserRepository::new();
    repo.expect_get_user_by_id()
        .with(eq(target_id))
        .returning(move |_| {
            Ok(user_record(
                target_id,
                Profile::CityAdmin,
                Some(city_id),
                empty_policies(),
            ))
        });
    repo.expect_check_city_admin_exists_for_city()
        .with(eq(city_id), eq(Some(target_id)))
        .returning(|_, _| Ok(false));
    repo.expect_check_email_exists_for_other_user()
        .with(eq("updated@test.com"), eq(target_id))
        .returning(|_, _| Ok(false));
    repo.expect_update_user_by_id().returning(move |data, id| {
        assert_eq!(data.email, "updated@test.com");
        Ok(user_record(
            id,
            data.profile,
            data.city_id,
            empty_policies(),
        ))
    });

    let usecase = UpdateUserUseCase::new(deps(repo, FakePasswordHasher::ok()));
    let result = usecase
        .execute(
            update_cmd(Profile::CityAdmin, Some(city_id)),
            target_id,
            &claims(Profile::Root, root_id, None),
        )
        .await;

    assert!(result.is_ok());
}

#[tokio::test]
async fn city_admin_cannot_promote_user_to_root() {
    let admin_city = Uuid::new_v4();
    let admin_id = Uuid::new_v4();
    let target_id = Uuid::new_v4();

    let repo = MockUserRepository::new();
    let usecase = UpdateUserUseCase::new(deps(repo, FakePasswordHasher::ok()));

    let result = usecase
        .execute(
            update_cmd(Profile::Root, None),
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
async fn non_root_cannot_update_root_user() {
    let admin_city = Uuid::new_v4();
    let admin_id = Uuid::new_v4();
    let target_id = Uuid::new_v4();

    let mut repo = MockUserRepository::new();
    repo.expect_get_user_by_id()
        .with(eq(target_id))
        .returning(move |_| {
            Ok(user_record(
                target_id,
                Profile::Root,
                None,
                empty_policies(),
            ))
        });

    let usecase = UpdateUserUseCase::new(deps(repo, FakePasswordHasher::ok()));
    let result = usecase
        .execute(
            update_cmd(Profile::CityUser, Some(admin_city)),
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
async fn city_admin_without_update_users_policy_is_forbidden() {
    let admin_city = Uuid::new_v4();
    let admin_id = Uuid::new_v4();
    let target_id = Uuid::new_v4();

    let mut repo = MockUserRepository::new();
    repo.expect_get_user_by_id()
        .with(eq(target_id))
        .returning(move |_| {
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

    let usecase = UpdateUserUseCase::new(deps(repo, FakePasswordHasher::ok()));
    let result = usecase
        .execute(
            update_cmd(Profile::CityUser, Some(admin_city)),
            target_id,
            &claims(Profile::CityAdmin, admin_id, Some(admin_city)),
        )
        .await;

    assert!(matches!(result.unwrap_err(), AppError::Forbidden(_)));
}

#[tokio::test]
async fn not_found_when_user_does_not_exist() {
    let root_id = Uuid::new_v4();
    let target_id = Uuid::new_v4();

    let mut repo = MockUserRepository::new();
    repo.expect_get_user_by_id()
        .with(eq(target_id))
        .returning(|_| Err(RepositoryError::NotFound));

    let usecase = UpdateUserUseCase::new(deps(repo, FakePasswordHasher::ok()));
    let result = usecase
        .execute(
            update_cmd(Profile::CityUser, Some(Uuid::new_v4())),
            target_id,
            &claims(Profile::Root, root_id, None),
        )
        .await;

    assert!(matches!(result.unwrap_err(), AppError::NotFound(_)));
}

#[tokio::test]
async fn update_promotes_to_city_admin_when_no_existing_admin() {
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
    repo.expect_check_city_admin_exists_for_city()
        .with(eq(city_id), eq(Some(target_id)))
        .returning(|_, _| Ok(false));
    repo.expect_check_email_exists_for_other_user()
        .returning(|_, _| Ok(false));
    repo.expect_update_user_by_id().returning(move |data, id| {
        Ok(user_record(
            id,
            data.profile,
            data.city_id,
            empty_policies(),
        ))
    });

    let usecase = UpdateUserUseCase::new(deps(repo, FakePasswordHasher::ok()));
    let result = usecase
        .execute(
            update_cmd(Profile::CityAdmin, Some(city_id)),
            target_id,
            &claims(Profile::Root, root_id, None),
        )
        .await;

    assert!(result.is_ok());
}

#[tokio::test]
async fn promoting_to_city_admin_when_one_exists_fails() {
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
    repo.expect_check_city_admin_exists_for_city()
        .returning(|_, _| Ok(true));

    let usecase = UpdateUserUseCase::new(deps(repo, FakePasswordHasher::ok()));
    let result = usecase
        .execute(
            update_cmd(Profile::CityAdmin, Some(city_id)),
            target_id,
            &claims(Profile::Root, root_id, None),
        )
        .await;

    assert!(matches!(
        result.unwrap_err(),
        AppError::BadRequest(msg) if msg.contains("CITY_ADMIN already exists")
    ));
}

#[tokio::test]
async fn email_taken_by_other_user_is_rejected() {
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
    repo.expect_check_email_exists_for_other_user()
        .returning(|_, _| Ok(true));

    let usecase = UpdateUserUseCase::new(deps(repo, FakePasswordHasher::ok()));
    let result = usecase
        .execute(
            update_cmd(Profile::CityUser, Some(city_id)),
            target_id,
            &claims(Profile::Root, root_id, None),
        )
        .await;

    assert!(matches!(
        result.unwrap_err(),
        AppError::BadRequest(msg) if msg.contains("already in use")
    ));
}

#[tokio::test]
async fn city_admin_cannot_assign_policy_it_does_not_own() {
    let admin_city = Uuid::new_v4();
    let admin_id = Uuid::new_v4();
    let target_id = Uuid::new_v4();

    let mut repo = MockUserRepository::new();
    repo.expect_get_user_by_id()
        .with(eq(target_id))
        .returning(move |_| {
            let mut p: PermissionPolicies = HashMap::new();
            p.insert(Policy::UpdateUsers, vec![admin_city]);
            Ok(user_record(
                target_id,
                Profile::CityUser,
                Some(admin_city),
                p,
            ))
        });
    // AuthContext load + policy-assignment load
    repo.expect_get_user_policies_by_id()
        .with(eq(admin_id))
        .returning(move |_| {
            let mut p: PermissionPolicies = HashMap::new();
            p.insert(Policy::UpdateUsers, vec![admin_city]);
            Ok(p)
        });

    let mut cmd = update_cmd(Profile::CityUser, Some(admin_city));
    let mut policies: PermissionPolicies = HashMap::new();
    policies.insert(Policy::ReadVictims, vec![admin_city]);
    cmd.permission_policies = Some(policies);

    let usecase = UpdateUserUseCase::new(deps(repo, FakePasswordHasher::ok()));
    let result = usecase
        .execute(
            cmd,
            target_id,
            &claims(Profile::CityAdmin, admin_id, Some(admin_city)),
        )
        .await;

    assert!(matches!(result.unwrap_err(), AppError::Forbidden(_)));
}
