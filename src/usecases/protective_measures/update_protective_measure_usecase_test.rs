use std::{collections::HashMap, sync::Arc};

use async_trait::async_trait;
use chrono::{Duration, Local, NaiveDate, Utc};
use uuid::Uuid;

use crate::core::application_error::ApplicationError as AppError;
use crate::core::commands::cities::{CreateCity, UpdateCity};
use crate::core::commands::protective_measures::UpdateProtectiveMeasure;
use crate::core::contracts::repository::cities::CityRepository;
use crate::core::contracts::repository::error::RepositoryError;
use crate::core::contracts::repository::offenders::OffenderReadRepository;
use crate::core::contracts::repository::protective_measures::{
    ProtectiveMeasureReadRepository, ProtectiveMeasureWriteRepository,
};
use crate::core::contracts::repository::users::MockUserRepository;
use crate::core::contracts::repository::victims::MockVictimReadRepository;
use crate::core::entities::auth::UserClaims;
use crate::core::entities::cities::City;
use crate::core::entities::protective_measures::{
    ProtectiveMeasure, ProtectiveMeasureStatus, RelationshipToVictim, ViolenceType,
};
use crate::core::read_models::offenders::OffenderWithDetails;
use crate::core::read_models::victims::VictimWithDetails;
use crate::core::value_objects::policies::Policy;
use crate::core::value_objects::profiles::Profile;
use crate::core::value_objects::ranks::Rank;
use crate::usecases::protective_measures::{
    ProtectiveMeasureUseCaseDependencies, UpdateProtectiveMeasureUseCase,
};

fn claims(profile: Profile, city_id: Uuid) -> UserClaims {
    UserClaims {
        id: Uuid::new_v4().to_string(),
        exp: 9999999999,
        iss: "nupevid-api".to_string(),
        aud: "nupevid-api".to_string(),
        rank: Rank::SdPm,
        registration: "100000001".to_string(),
        full_name: "Operador Teste".to_string(),
        profile,
        email: "operador@test.com".to_string(),
        city_id: Some(city_id.to_string()),
    }
}

fn update_command(
    victim_id: Uuid,
    offender_id: Uuid,
    city_id: Uuid,
    status: ProtectiveMeasureStatus,
) -> UpdateProtectiveMeasure {
    UpdateProtectiveMeasure {
        process_number: "12345-67.2025.8.26.0000".to_string(),
        sei_process_number: None,
        occurrence_report_number: None,
        issued_at: NaiveDate::from_ymd_opt(2025, 1, 1).unwrap(),
        judicial_authority: "Juiz A".to_string(),
        court_district_id: city_id,
        distance_meters: None,
        status,
        violence_types: vec![ViolenceType::Physical],
        relationship_to_victim: RelationshipToVictim::Spouse,
        assaults_children: false,
        was_drunk_during_assault: false,
        victim_id,
        offender_id,
    }
}

fn measure(
    id: Uuid,
    victim_id: Uuid,
    offender_id: Uuid,
    city_id: Uuid,
    status: ProtectiveMeasureStatus,
) -> ProtectiveMeasure {
    let now = Utc::now();
    ProtectiveMeasure {
        id,
        process_number: "12345-67.2025.8.26.0000".to_string(),
        sei_process_number: None,
        occurrence_report_number: None,
        issued_at: NaiveDate::from_ymd_opt(2025, 1, 1).unwrap(),
        judicial_authority: "Juiz A".to_string(),
        court_district_id: city_id,
        distance_meters: None,
        status,
        violence_types: vec![ViolenceType::Physical],
        relationship_to_victim: RelationshipToVictim::Spouse,
        assaults_children: false,
        was_drunk_during_assault: false,
        victim_id,
        offender_id,
        created_at: now,
        updated_at: now,
        is_deleted: false,
    }
}

fn victim(id: Uuid, city_id: Uuid) -> VictimWithDetails {
    let now = Utc::now();
    VictimWithDetails {
        id,
        full_name: "MARIA TESTE".to_string(),
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
        phones: vec![],
        addresses: vec![],
    }
}

struct FakeMeasureReadRepo {
    existing: Option<ProtectiveMeasure>,
    active_exists: bool,
    expected_check: Option<(Uuid, Uuid, Option<Uuid>)>,
}

impl FakeMeasureReadRepo {
    fn existing(existing: ProtectiveMeasure) -> Self {
        Self {
            existing: Some(existing),
            active_exists: false,
            expected_check: None,
        }
    }

    fn missing() -> Self {
        Self {
            existing: None,
            active_exists: false,
            expected_check: None,
        }
    }

    fn with_active_check(
        existing: ProtectiveMeasure,
        active_exists: bool,
        expected_check: Option<(Uuid, Uuid, Option<Uuid>)>,
    ) -> Self {
        Self {
            existing: Some(existing),
            active_exists,
            expected_check,
        }
    }
}

#[async_trait]
impl ProtectiveMeasureReadRepository for FakeMeasureReadRepo {
    async fn get_protective_measure_by_id(
        &self,
        _id: Uuid,
    ) -> Result<ProtectiveMeasure, RepositoryError> {
        self.existing.clone().ok_or(RepositoryError::NotFound)
    }

    async fn get_all_protective_measures(&self) -> Result<Vec<ProtectiveMeasure>, RepositoryError> {
        unimplemented!()
    }

    async fn get_protective_measures_paginated(
        &self,
        _allowed_cities: Option<&[Uuid]>,
        _victim_id: Option<Uuid>,
        _offender_id: Option<Uuid>,
        _limit: i64,
        _offset: i64,
    ) -> Result<Vec<ProtectiveMeasure>, RepositoryError> {
        unimplemented!()
    }

    async fn count_protective_measures(
        &self,
        _allowed_cities: Option<&[Uuid]>,
        _victim_id: Option<Uuid>,
        _offender_id: Option<Uuid>,
    ) -> Result<i64, RepositoryError> {
        unimplemented!()
    }

    async fn get_protective_measures_by_victim(
        &self,
        _victim_id: Uuid,
    ) -> Result<Vec<ProtectiveMeasure>, RepositoryError> {
        unimplemented!()
    }

    async fn check_active_measure_exists_for_victim(
        &self,
        victim_id: Uuid,
        offender_id: Uuid,
        exclude_measure_id: Option<Uuid>,
    ) -> Result<bool, RepositoryError> {
        if let Some((expected_victim, expected_offender, expected_exclude)) = self.expected_check {
            assert_eq!(victim_id, expected_victim);
            assert_eq!(offender_id, expected_offender);
            assert_eq!(exclude_measure_id, expected_exclude);
        }
        Ok(self.active_exists)
    }
}

enum FakeUpdateResult {
    Success(Box<ProtectiveMeasure>),
    Conflict(String),
    Internal,
}

struct FakeMeasureWriteRepo {
    update_result: FakeUpdateResult,
}

impl FakeMeasureWriteRepo {
    fn success(measure: ProtectiveMeasure) -> Self {
        Self {
            update_result: FakeUpdateResult::Success(Box::new(measure)),
        }
    }

    fn conflict(message: &str) -> Self {
        Self {
            update_result: FakeUpdateResult::Conflict(message.to_string()),
        }
    }

    fn internal() -> Self {
        Self {
            update_result: FakeUpdateResult::Internal,
        }
    }
}

#[async_trait]
impl ProtectiveMeasureWriteRepository for FakeMeasureWriteRepo {
    async fn create_protective_measure(
        &self,
        _measure: crate::core::commands::protective_measures::CreateProtectiveMeasure,
    ) -> Result<ProtectiveMeasure, RepositoryError> {
        unimplemented!()
    }

    async fn update_protective_measure_by_id(
        &self,
        _data: UpdateProtectiveMeasure,
        _id: Uuid,
    ) -> Result<ProtectiveMeasure, RepositoryError> {
        match &self.update_result {
            FakeUpdateResult::Success(measure) => Ok(measure.as_ref().clone()),
            FakeUpdateResult::Conflict(message) => Err(RepositoryError::Conflict(message.clone())),
            FakeUpdateResult::Internal => Err(RepositoryError::DatabaseError(
                "unexpected write".to_string(),
            )),
        }
    }

    async fn delete_protective_measure_by_id(
        &self,
        _id: Uuid,
    ) -> Result<ProtectiveMeasure, RepositoryError> {
        unimplemented!()
    }
}

struct FakeOffenderReadRepo {
    offender: Option<OffenderWithDetails>,
}

#[async_trait]
impl OffenderReadRepository for FakeOffenderReadRepo {
    async fn get_offender_by_id(&self, _id: Uuid) -> Result<OffenderWithDetails, RepositoryError> {
        self.offender.clone().ok_or(RepositoryError::NotFound)
    }

    async fn get_all_offenders(&self) -> Result<Vec<OffenderWithDetails>, RepositoryError> {
        unimplemented!()
    }

    async fn get_offenders_paginated(
        &self,
        _allowed_cities: Option<&[Uuid]>,
        _limit: i64,
        _offset: i64,
    ) -> Result<Vec<OffenderWithDetails>, RepositoryError> {
        unimplemented!()
    }

    async fn count_offenders(
        &self,
        _allowed_cities: Option<&[Uuid]>,
    ) -> Result<i64, RepositoryError> {
        unimplemented!()
    }

    async fn get_offenders_by_city(
        &self,
        _city_id: Uuid,
    ) -> Result<Vec<OffenderWithDetails>, RepositoryError> {
        unimplemented!()
    }

    async fn get_offenders_by_name(
        &self,
        _name: &str,
    ) -> Result<Vec<OffenderWithDetails>, RepositoryError> {
        unimplemented!()
    }

    async fn get_offenders_by_cpf(
        &self,
        _cpf: &str,
    ) -> Result<Vec<OffenderWithDetails>, RepositoryError> {
        unimplemented!()
    }

    async fn get_offenders_by_victim_id(
        &self,
        _victim_id: Uuid,
    ) -> Result<Vec<OffenderWithDetails>, RepositoryError> {
        unimplemented!()
    }
}

struct FakeCityRepo;

#[async_trait]
impl CityRepository for FakeCityRepo {
    async fn create_city(&self, _city: CreateCity) -> Result<City, RepositoryError> {
        unimplemented!()
    }

    async fn get_city_by_id(&self, _id: Uuid) -> Result<City, RepositoryError> {
        unimplemented!()
    }

    async fn get_all_cities(&self) -> Result<Vec<City>, RepositoryError> {
        unimplemented!()
    }

    async fn get_cities_paginated(
        &self,
        _allowed_cities: Option<&[Uuid]>,
        _limit: i64,
        _offset: i64,
    ) -> Result<Vec<City>, RepositoryError> {
        unimplemented!()
    }

    async fn count_cities(&self, _allowed_cities: Option<&[Uuid]>) -> Result<i64, RepositoryError> {
        unimplemented!()
    }

    async fn update_city_by_id(
        &self,
        _data: UpdateCity,
        _id: Uuid,
    ) -> Result<City, RepositoryError> {
        unimplemented!()
    }

    async fn delete_city_by_id(&self, _id: Uuid) -> Result<City, RepositoryError> {
        unimplemented!()
    }

    async fn get_city_by_name_and_battalion(
        &self,
        _name: &str,
        _battalion: &str,
    ) -> Result<Option<City>, RepositoryError> {
        unimplemented!()
    }
}

fn user_repo_with_update_policy(city_ids: Vec<Uuid>) -> MockUserRepository {
    let mut user_repo = MockUserRepository::new();
    let mut policies = HashMap::new();
    policies.insert(Policy::UpdateProtectiveMeasures, city_ids);
    user_repo
        .expect_get_user_policies_by_id()
        .returning(move |_| Ok(policies.clone()));
    user_repo
}

fn user_repo_without_policy() -> MockUserRepository {
    let mut user_repo = MockUserRepository::new();
    user_repo
        .expect_get_user_policies_by_id()
        .returning(|_| Ok(HashMap::new()));
    user_repo
}

fn deps(
    read_repo: FakeMeasureReadRepo,
    write_repo: FakeMeasureWriteRepo,
    victim_repo: MockVictimReadRepository,
    offender_repo: FakeOffenderReadRepo,
    user_repo: MockUserRepository,
) -> ProtectiveMeasureUseCaseDependencies {
    ProtectiveMeasureUseCaseDependencies::new(
        Arc::new(read_repo),
        Arc::new(write_repo),
        Arc::new(victim_repo),
        Arc::new(offender_repo),
        Arc::new(user_repo),
        Arc::new(FakeCityRepo),
    )
}

fn victim_repo_for(ids: Vec<(Uuid, Uuid)>) -> MockVictimReadRepository {
    let mut victim_repo = MockVictimReadRepository::new();
    victim_repo.expect_get_victim_by_id().returning(move |id| {
        ids.iter()
            .find(|(victim_id, _)| *victim_id == id)
            .map(|(victim_id, city_id)| victim(*victim_id, *city_id))
            .ok_or(RepositoryError::NotFound)
    });
    victim_repo
}

fn offender_repo_missing() -> FakeOffenderReadRepo {
    FakeOffenderReadRepo { offender: None }
}

#[tokio::test]
async fn update_valid_measure_succeeds_and_excludes_current_measure_from_active_check() {
    let city_id = Uuid::new_v4();
    let measure_id = Uuid::new_v4();
    let victim_id = Uuid::new_v4();
    let offender_id = Uuid::new_v4();

    let read_repo = FakeMeasureReadRepo::with_active_check(
        measure(
            measure_id,
            victim_id,
            offender_id,
            city_id,
            ProtectiveMeasureStatus::Revoked,
        ),
        false,
        Some((victim_id, offender_id, Some(measure_id))),
    );
    let write_repo = FakeMeasureWriteRepo::success(measure(
        measure_id,
        victim_id,
        offender_id,
        city_id,
        ProtectiveMeasureStatus::Valid,
    ));

    let usecase = UpdateProtectiveMeasureUseCase::new(deps(
        read_repo,
        write_repo,
        victim_repo_for(vec![(victim_id, city_id)]),
        offender_repo_missing(),
        user_repo_with_update_policy(vec![city_id]),
    ));

    let result = usecase
        .execute(
            update_command(
                victim_id,
                offender_id,
                city_id,
                ProtectiveMeasureStatus::Valid,
            ),
            measure_id,
            &claims(Profile::CityAdmin, city_id),
        )
        .await;

    assert!(result.is_ok());
    assert_eq!(result.unwrap().status, ProtectiveMeasureStatus::Valid);
}

#[tokio::test]
async fn update_revoked_measure_skips_active_pair_check() {
    let city_id = Uuid::new_v4();
    let measure_id = Uuid::new_v4();
    let victim_id = Uuid::new_v4();
    let offender_id = Uuid::new_v4();
    let read_repo = FakeMeasureReadRepo::existing(measure(
        measure_id,
        victim_id,
        offender_id,
        city_id,
        ProtectiveMeasureStatus::Valid,
    ));
    let write_repo = FakeMeasureWriteRepo::success(measure(
        measure_id,
        victim_id,
        offender_id,
        city_id,
        ProtectiveMeasureStatus::Revoked,
    ));

    let usecase = UpdateProtectiveMeasureUseCase::new(deps(
        read_repo,
        write_repo,
        victim_repo_for(vec![(victim_id, city_id)]),
        offender_repo_missing(),
        user_repo_with_update_policy(vec![city_id]),
    ));

    let result = usecase
        .execute(
            update_command(
                victim_id,
                offender_id,
                city_id,
                ProtectiveMeasureStatus::Revoked,
            ),
            measure_id,
            &claims(Profile::CityAdmin, city_id),
        )
        .await;

    assert!(result.is_ok());
}

#[tokio::test]
async fn update_fails_when_existing_measure_does_not_exist() {
    let city_id = Uuid::new_v4();
    let victim_id = Uuid::new_v4();
    let offender_id = Uuid::new_v4();
    let usecase = UpdateProtectiveMeasureUseCase::new(deps(
        FakeMeasureReadRepo::missing(),
        FakeMeasureWriteRepo::internal(),
        MockVictimReadRepository::new(),
        offender_repo_missing(),
        MockUserRepository::new(),
    ));

    let result = usecase
        .execute(
            update_command(
                victim_id,
                offender_id,
                city_id,
                ProtectiveMeasureStatus::Revoked,
            ),
            Uuid::new_v4(),
            &claims(Profile::CityAdmin, city_id),
        )
        .await;

    assert!(
        matches!(result.unwrap_err(), AppError::NotFound(message) if message.contains("Protective measure"))
    );
}

#[tokio::test]
async fn update_fails_when_issued_at_is_in_the_future() {
    let city_id = Uuid::new_v4();
    let measure_id = Uuid::new_v4();
    let victim_id = Uuid::new_v4();
    let offender_id = Uuid::new_v4();
    let mut command = update_command(
        victim_id,
        offender_id,
        city_id,
        ProtectiveMeasureStatus::Revoked,
    );
    command.issued_at = Local::now().date_naive() + Duration::days(1);

    let read_repo = FakeMeasureReadRepo::existing(measure(
        measure_id,
        victim_id,
        offender_id,
        city_id,
        ProtectiveMeasureStatus::Valid,
    ));

    let usecase = UpdateProtectiveMeasureUseCase::new(deps(
        read_repo,
        FakeMeasureWriteRepo::internal(),
        victim_repo_for(vec![(victim_id, city_id)]),
        offender_repo_missing(),
        user_repo_with_update_policy(vec![city_id]),
    ));

    let result = usecase
        .execute(command, measure_id, &claims(Profile::CityAdmin, city_id))
        .await;

    assert!(
        matches!(result.unwrap_err(), AppError::BadRequest(message) if message.contains("issued_at"))
    );
}

#[tokio::test]
async fn update_fails_with_conflict_when_pair_already_has_active_measure() {
    let city_id = Uuid::new_v4();
    let measure_id = Uuid::new_v4();
    let victim_id = Uuid::new_v4();
    let offender_id = Uuid::new_v4();
    let read_repo = FakeMeasureReadRepo::with_active_check(
        measure(
            measure_id,
            victim_id,
            offender_id,
            city_id,
            ProtectiveMeasureStatus::Revoked,
        ),
        true,
        None,
    );

    let usecase = UpdateProtectiveMeasureUseCase::new(deps(
        read_repo,
        FakeMeasureWriteRepo::internal(),
        victim_repo_for(vec![(victim_id, city_id)]),
        offender_repo_missing(),
        user_repo_with_update_policy(vec![city_id]),
    ));

    let result = usecase
        .execute(
            update_command(
                victim_id,
                offender_id,
                city_id,
                ProtectiveMeasureStatus::Valid,
            ),
            measure_id,
            &claims(Profile::CityAdmin, city_id),
        )
        .await;

    assert!(
        matches!(result.unwrap_err(), AppError::Conflict(message) if message.contains("active protective measure"))
    );
}

#[tokio::test]
async fn update_fails_without_update_policy_for_existing_victim_city() {
    let city_id = Uuid::new_v4();
    let measure_id = Uuid::new_v4();
    let victim_id = Uuid::new_v4();
    let offender_id = Uuid::new_v4();
    let read_repo = FakeMeasureReadRepo::existing(measure(
        measure_id,
        victim_id,
        offender_id,
        city_id,
        ProtectiveMeasureStatus::Revoked,
    ));

    let usecase = UpdateProtectiveMeasureUseCase::new(deps(
        read_repo,
        FakeMeasureWriteRepo::internal(),
        victim_repo_for(vec![(victim_id, city_id)]),
        offender_repo_missing(),
        user_repo_without_policy(),
    ));

    let result = usecase
        .execute(
            update_command(
                victim_id,
                offender_id,
                city_id,
                ProtectiveMeasureStatus::Revoked,
            ),
            measure_id,
            &claims(Profile::CityAdmin, city_id),
        )
        .await;

    assert!(matches!(result.unwrap_err(), AppError::Forbidden(_)));
}

#[tokio::test]
async fn update_change_victim_requires_policy_for_new_victim_city() {
    let old_city_id = Uuid::new_v4();
    let new_city_id = Uuid::new_v4();
    let measure_id = Uuid::new_v4();
    let old_victim_id = Uuid::new_v4();
    let new_victim_id = Uuid::new_v4();
    let offender_id = Uuid::new_v4();
    let read_repo = FakeMeasureReadRepo::existing(measure(
        measure_id,
        old_victim_id,
        offender_id,
        old_city_id,
        ProtectiveMeasureStatus::Revoked,
    ));

    let usecase = UpdateProtectiveMeasureUseCase::new(deps(
        read_repo,
        FakeMeasureWriteRepo::internal(),
        victim_repo_for(vec![
            (old_victim_id, old_city_id),
            (new_victim_id, new_city_id),
        ]),
        offender_repo_missing(),
        user_repo_with_update_policy(vec![old_city_id]),
    ));

    let result = usecase
        .execute(
            update_command(
                new_victim_id,
                offender_id,
                new_city_id,
                ProtectiveMeasureStatus::Revoked,
            ),
            measure_id,
            &claims(Profile::CityAdmin, old_city_id),
        )
        .await;

    assert!(matches!(result.unwrap_err(), AppError::Forbidden(_)));
}

#[tokio::test]
async fn update_change_offender_fails_when_new_offender_does_not_exist() {
    let city_id = Uuid::new_v4();
    let measure_id = Uuid::new_v4();
    let victim_id = Uuid::new_v4();
    let old_offender_id = Uuid::new_v4();
    let new_offender_id = Uuid::new_v4();
    let read_repo = FakeMeasureReadRepo::existing(measure(
        measure_id,
        victim_id,
        old_offender_id,
        city_id,
        ProtectiveMeasureStatus::Revoked,
    ));

    let usecase = UpdateProtectiveMeasureUseCase::new(deps(
        read_repo,
        FakeMeasureWriteRepo::internal(),
        victim_repo_for(vec![(victim_id, city_id)]),
        offender_repo_missing(),
        user_repo_with_update_policy(vec![city_id]),
    ));

    let result = usecase
        .execute(
            update_command(
                victim_id,
                new_offender_id,
                city_id,
                ProtectiveMeasureStatus::Revoked,
            ),
            measure_id,
            &claims(Profile::CityAdmin, city_id),
        )
        .await;

    assert!(
        matches!(result.unwrap_err(), AppError::NotFound(message) if message.contains("Offender"))
    );
}

#[tokio::test]
async fn update_maps_repository_conflict_to_conflict() {
    let city_id = Uuid::new_v4();
    let measure_id = Uuid::new_v4();
    let victim_id = Uuid::new_v4();
    let offender_id = Uuid::new_v4();
    let read_repo = FakeMeasureReadRepo::with_active_check(
        measure(
            measure_id,
            victim_id,
            offender_id,
            city_id,
            ProtectiveMeasureStatus::Revoked,
        ),
        false,
        None,
    );
    let write_repo = FakeMeasureWriteRepo::conflict("duplicate active measure");

    let usecase = UpdateProtectiveMeasureUseCase::new(deps(
        read_repo,
        write_repo,
        victim_repo_for(vec![(victim_id, city_id)]),
        offender_repo_missing(),
        user_repo_with_update_policy(vec![city_id]),
    ));

    let result = usecase
        .execute(
            update_command(
                victim_id,
                offender_id,
                city_id,
                ProtectiveMeasureStatus::Valid,
            ),
            measure_id,
            &claims(Profile::CityAdmin, city_id),
        )
        .await;

    assert!(
        matches!(result.unwrap_err(), AppError::Conflict(message) if message.contains("duplicate"))
    );
}

#[tokio::test]
async fn update_fails_with_empty_judicial_authority() {
    let city_id = Uuid::new_v4();
    let measure_id = Uuid::new_v4();
    let victim_id = Uuid::new_v4();
    let offender_id = Uuid::new_v4();
    let mut command = update_command(
        victim_id,
        offender_id,
        city_id,
        ProtectiveMeasureStatus::Revoked,
    );
    command.judicial_authority = String::new();

    let read_repo = FakeMeasureReadRepo::existing(measure(
        measure_id,
        victim_id,
        offender_id,
        city_id,
        ProtectiveMeasureStatus::Valid,
    ));

    let usecase = UpdateProtectiveMeasureUseCase::new(deps(
        read_repo,
        FakeMeasureWriteRepo::internal(),
        victim_repo_for(vec![(victim_id, city_id)]),
        offender_repo_missing(),
        user_repo_with_update_policy(vec![city_id]),
    ));

    let result = usecase
        .execute(command, measure_id, &claims(Profile::CityAdmin, city_id))
        .await;

    assert!(
        matches!(result.unwrap_err(), AppError::BadRequest(message) if message.contains("judicial_authority"))
    );
}

#[tokio::test]
async fn update_root_bypasses_policy_lookup() {
    let city_id = Uuid::new_v4();
    let measure_id = Uuid::new_v4();
    let victim_id = Uuid::new_v4();
    let offender_id = Uuid::new_v4();
    let read_repo = FakeMeasureReadRepo::existing(measure(
        measure_id,
        victim_id,
        offender_id,
        city_id,
        ProtectiveMeasureStatus::Valid,
    ));
    let write_repo = FakeMeasureWriteRepo::success(measure(
        measure_id,
        victim_id,
        offender_id,
        city_id,
        ProtectiveMeasureStatus::Revoked,
    ));

    let usecase = UpdateProtectiveMeasureUseCase::new(deps(
        read_repo,
        write_repo,
        victim_repo_for(vec![(victim_id, city_id)]),
        offender_repo_missing(),
        MockUserRepository::new(),
    ));

    let result = usecase
        .execute(
            update_command(
                victim_id,
                offender_id,
                city_id,
                ProtectiveMeasureStatus::Revoked,
            ),
            measure_id,
            &claims(Profile::Root, city_id),
        )
        .await;

    assert!(result.is_ok());
}
