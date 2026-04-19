use std::sync::Arc;

use chrono::Utc;
use uuid::Uuid;

use crate::core::commands::victims::CreateVictim;
use crate::core::contracts::repository::error::RepositoryError;
use crate::core::contracts::repository::users::MockUserRepository;
use crate::core::contracts::repository::victims::{
    MockVictimReadRepository, MockVictimWriteRepository,
};
use crate::core::entities::auth::UserClaims;
use crate::core::entities::victims::{Victim, VictimWriteResult};
use crate::core::value_objects::policies::Policy;
use crate::core::value_objects::profiles::Profile;
use crate::core::value_objects::ranks::Rank;
use crate::usecases::victims::{CreateVictimUseCase, VictimUseCaseDependencies};

fn make_claims(profile: Profile, city_id: Uuid) -> UserClaims {
    UserClaims {
        id: Uuid::new_v4().to_string(),
        exp: 9999999999,
        iss: "nupevid-api".to_string(),
        aud: "nupevid-api".to_string(),
        rank: Rank::SdPm,
        registration: "100012345".to_string(),
        full_name: "Test User".to_string(),
        profile,
        email: "test@test.com".to_string(),
        city_id: Some(city_id.to_string()),
    }
}

fn make_create_victim(city_id: Uuid) -> CreateVictim {
    CreateVictim {
        full_name: "Maria Silva".to_string(),
        cpf: None,
        birth_date: None,
        city_id: Some(city_id),
        phones: None,
        addresses: None,
        education_level: None,
        occupation: None,
        has_children: false,
        children_count: None,
        is_pregnant: None,
        has_special_needs: false,
        special_needs_type: None,
        uses_alcohol: false,
        uses_drugs: false,
        has_psychiatric_issues: false,
        psychiatric_issues_type: None,
    }
}

fn make_victim(city_id: Uuid) -> Victim {
    let now = Utc::now();
    Victim {
        id: Uuid::new_v4(),
        full_name: "Maria Silva".to_string(),
        cpf: None,
        birth_date: None,
        city_id,
        created_at: now,
        updated_at: now,
        is_deleted: false,
        education_level: None,
        occupation: None,
        has_children: false,
        children_count: None,
        is_pregnant: None,
        has_special_needs: false,
        special_needs_type: None,
        uses_alcohol: false,
        uses_drugs: false,
        has_psychiatric_issues: false,
        psychiatric_issues_type: None,
    }
}

fn make_deps(
    user_repo: MockUserRepository,
    write_repo: MockVictimWriteRepository,
) -> VictimUseCaseDependencies {
    VictimUseCaseDependencies::new(
        Arc::new(MockVictimReadRepository::new()),
        Arc::new(write_repo),
        Arc::new(user_repo),
    )
}

fn mock_user_repo_with_policies(city_id: Uuid) -> MockUserRepository {
    let mut user_repo = MockUserRepository::new();
    let mut policies = std::collections::HashMap::new();
    policies.insert(Policy::CreateVictims, vec![city_id]);
    policies.insert(Policy::ReadVictims, vec![city_id]);
    user_repo
        .expect_get_user_policies_by_id()
        .returning(move |_| Ok(policies.clone()));
    user_repo
}

#[tokio::test]
async fn test_create_victim_success() {
    let city_id = Uuid::new_v4();
    let claims = make_claims(Profile::CityAdmin, city_id);
    let victim_data = make_create_victim(city_id);

    let user_repo = mock_user_repo_with_policies(city_id);

    let mut write_repo = MockVictimWriteRepository::new();
    write_repo.expect_create_victim().returning(move |_| {
        Ok(VictimWriteResult {
            victim: make_victim(city_id),
            phones: vec![],
            addresses: vec![],
        })
    });

    let usecase = CreateVictimUseCase::new(make_deps(user_repo, write_repo));
    let result = usecase.execute(victim_data, &claims).await;

    assert!(result.is_ok());
    let victim_response = result.unwrap();
    assert_eq!(victim_response.full_name, "Maria Silva");
    assert_eq!(victim_response.city_id, city_id);
}

#[tokio::test]
async fn test_create_victim_authorization_failure() {
    let city_id = Uuid::new_v4();
    let other_city_id = Uuid::new_v4();
    let claims = make_claims(Profile::CityAdmin, city_id);
    let victim_data = make_create_victim(other_city_id);

    let user_repo = mock_user_repo_with_policies(city_id);
    let write_repo = MockVictimWriteRepository::new();

    let usecase = CreateVictimUseCase::new(make_deps(user_repo, write_repo));
    let result = usecase.execute(victim_data, &claims).await;

    assert!(result.is_err());
    let err = result.unwrap_err();
    assert!(
        err.to_string().contains("permission"),
        "Expected authorization error, got: {}",
        err
    );
}

#[tokio::test]
async fn test_create_victim_duplicate_cpf() {
    let city_id = Uuid::new_v4();
    let claims = make_claims(Profile::CityAdmin, city_id);
    let victim_data = make_create_victim(city_id);

    let user_repo = mock_user_repo_with_policies(city_id);

    let mut write_repo = MockVictimWriteRepository::new();
    write_repo.expect_create_victim().returning(|_| {
        Err(RepositoryError::DuplicateEntry(
            "A victim with this CPF already exists".into(),
        ))
    });

    let usecase = CreateVictimUseCase::new(make_deps(user_repo, write_repo));
    let result = usecase.execute(victim_data, &claims).await;

    assert!(result.is_err());
    let err = result.unwrap_err();
    assert!(
        err.to_string().contains("CPF already exists"),
        "Expected conflict error, got: {}",
        err
    );
}

#[tokio::test]
async fn test_create_victim_root_bypasses_policy() {
    let city_id = Uuid::new_v4();
    let claims = make_claims(Profile::Root, city_id);
    let victim_data = make_create_victim(city_id);

    let user_repo = MockUserRepository::new();
    // Root profile skips policy loading entirely
    // No expectations needed on get_user_policies_json_by_id

    let mut write_repo = MockVictimWriteRepository::new();
    write_repo.expect_create_victim().returning(move |_| {
        Ok(VictimWriteResult {
            victim: make_victim(city_id),
            phones: vec![],
            addresses: vec![],
        })
    });

    let usecase = CreateVictimUseCase::new(make_deps(user_repo, write_repo));
    let result = usecase.execute(victim_data, &claims).await;

    assert!(result.is_ok());
}
