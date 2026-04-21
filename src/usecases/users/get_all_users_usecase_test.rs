use std::collections::HashMap;

use mockall::predicate::eq;
use uuid::Uuid;

use crate::core::contracts::repository::users::MockUserRepository;
use crate::core::value_objects::policies::{PermissionPolicies, Policy};
use crate::core::value_objects::profiles::Profile;
use crate::usecases::users::GetAllUsersUseCase;
use crate::usecases::users::test_support::{
    FakePasswordHasher, claims, deps, empty_policies, user_record,
};
use crate::utils::pagination::Pagination;

fn pagination() -> Pagination {
    Pagination {
        page: 1,
        page_size: 20,
        offset: 0,
    }
}

#[tokio::test]
async fn root_lists_users_without_city_filter() {
    let root_id = Uuid::new_v4();

    let mut repo = MockUserRepository::new();
    repo.expect_count_users()
        .withf(|cities, exclude_root| cities.is_none() && !*exclude_root)
        .returning(|_, _| Ok(3));
    repo.expect_get_users_paginated()
        .withf(|cities, exclude_root, _, _| cities.is_none() && !*exclude_root)
        .returning(move |_, _, _, _| {
            Ok(vec![
                user_record(Uuid::new_v4(), Profile::Root, None, empty_policies()),
                user_record(
                    Uuid::new_v4(),
                    Profile::CityAdmin,
                    Some(Uuid::new_v4()),
                    empty_policies(),
                ),
                user_record(
                    Uuid::new_v4(),
                    Profile::CityUser,
                    Some(Uuid::new_v4()),
                    empty_policies(),
                ),
            ])
        });

    let usecase = GetAllUsersUseCase::new(deps(repo, FakePasswordHasher::ok()));
    let result = usecase
        .execute(pagination(), &claims(Profile::Root, root_id, None))
        .await
        .unwrap();
    assert_eq!(result.total_items, 3);
    assert_eq!(result.items.len(), 3);
}

#[tokio::test]
async fn city_admin_scope_filters_by_allowed_cities() {
    let admin_city = Uuid::new_v4();
    let admin_id = Uuid::new_v4();

    let mut repo = MockUserRepository::new();
    repo.expect_get_user_policies_by_id()
        .with(eq(admin_id))
        .returning(move |_| {
            let mut p: PermissionPolicies = HashMap::new();
            p.insert(Policy::ReadUsers, vec![admin_city]);
            Ok(p)
        });
    repo.expect_count_users()
        .withf(move |cities, exclude_root| {
            cities.map(|c| c == [admin_city]).unwrap_or(false) && *exclude_root
        })
        .returning(|_, _| Ok(1));
    repo.expect_get_users_paginated()
        .withf(move |cities, exclude_root, _, _| {
            cities.map(|c| c == [admin_city]).unwrap_or(false) && *exclude_root
        })
        .returning(move |_, _, _, _| {
            Ok(vec![user_record(
                Uuid::new_v4(),
                Profile::CityUser,
                Some(admin_city),
                empty_policies(),
            )])
        });

    let usecase = GetAllUsersUseCase::new(deps(repo, FakePasswordHasher::ok()));
    let result = usecase
        .execute(
            pagination(),
            &claims(Profile::CityAdmin, admin_id, Some(admin_city)),
        )
        .await
        .unwrap();
    assert_eq!(result.total_items, 1);
}

#[tokio::test]
async fn city_user_scope_excludes_root_and_uses_no_city_filter() {
    let city_id = Uuid::new_v4();
    let user_id = Uuid::new_v4();

    let mut repo = MockUserRepository::new();
    repo.expect_count_users()
        .withf(|cities, exclude_root| cities.is_none() && *exclude_root)
        .returning(|_, _| Ok(5));
    repo.expect_get_users_paginated()
        .withf(|cities, exclude_root, _, _| cities.is_none() && *exclude_root)
        .returning(|_, _, _, _| Ok(vec![]));

    let usecase = GetAllUsersUseCase::new(deps(repo, FakePasswordHasher::ok()));
    let result = usecase
        .execute(
            pagination(),
            &claims(Profile::CityUser, user_id, Some(city_id)),
        )
        .await
        .unwrap();
    assert_eq!(result.total_items, 5);
}
