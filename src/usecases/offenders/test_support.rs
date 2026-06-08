use std::sync::Arc;

use async_trait::async_trait;
use chrono::Utc;
use uuid::Uuid;

use crate::core::commands::offenders::{CreateOffender, UpdateOffender};
use crate::core::contracts::repository::error::RepositoryError;
use crate::core::contracts::repository::offenders::{
    OffenderReadRepository, OffenderWriteRepository,
};
use crate::core::contracts::repository::users::MockUserRepository;
use crate::core::entities::auth::UserClaims;
use crate::core::entities::common::{AddressData, EducationLevel, PhoneData};
use crate::core::entities::offenders::{
    Offender, OffenderAddress, OffenderPhone, OffenderWriteResult,
};
use crate::core::read_models::offenders::{OffenderSummary, OffenderWithDetails};
use crate::core::value_objects::profiles::Profile;
use crate::core::value_objects::ranks::Rank;
use crate::usecases::offenders::deps::OffenderUseCaseDependencies;

pub fn claims(profile: Profile, city_id: Option<Uuid>) -> UserClaims {
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
        city_id: city_id.map(|c| c.to_string()),
    }
}

pub fn offender_with_details(id: Uuid, city_id: Uuid, full_name: &str) -> OffenderWithDetails {
    let now = Utc::now();
    OffenderWithDetails {
        summary: OffenderSummary {
            id,
            full_name: full_name.to_string(),
            cpf: None,
            birth_date: None,
            city_id,
            imprisoned: false,
            occupation: None,
            is_public_security_agent: false,
            security_force: None,
            uses_alcohol: false,
            uses_drugs: false,
            has_psychiatric_issues: false,
            psychiatric_issues_type: None,
            education_level: EducationLevel::HighSchool,
            observation: None,
            is_deleted: false,
            phones: vec![],
            addresses: vec![],
        },
        created_at: now,
        updated_at: now,
    }
}

pub fn offender_entity(id: Uuid, city_id: Uuid, full_name: &str) -> Offender {
    let now = Utc::now();
    Offender {
        id,
        full_name: full_name.to_string(),
        cpf: None,
        birth_date: None,
        city_id,
        imprisoned: false,
        occupation: None,
        is_public_security_agent: false,
        security_force: None,
        uses_alcohol: false,
        uses_drugs: false,
        has_psychiatric_issues: false,
        psychiatric_issues_type: None,
        education_level: EducationLevel::HighSchool,
        observation: None,
        created_at: now,
        updated_at: now,
        is_deleted: false,
    }
}

pub enum FakeWriteOutcome {
    Success(Box<Offender>),
    Duplicate(String),
    Referenced(String),
    Internal,
}

pub struct FakeOffenderWriteRepo {
    pub outcome: FakeWriteOutcome,
}

impl FakeOffenderWriteRepo {
    pub fn success(offender: Offender) -> Self {
        Self {
            outcome: FakeWriteOutcome::Success(Box::new(offender)),
        }
    }

    pub fn duplicate(msg: &str) -> Self {
        Self {
            outcome: FakeWriteOutcome::Duplicate(msg.to_string()),
        }
    }

    pub fn referenced(msg: &str) -> Self {
        Self {
            outcome: FakeWriteOutcome::Referenced(msg.to_string()),
        }
    }

    pub fn internal() -> Self {
        Self {
            outcome: FakeWriteOutcome::Internal,
        }
    }

    fn produce(&self) -> Result<OffenderWriteResult, RepositoryError> {
        match &self.outcome {
            FakeWriteOutcome::Success(offender) => Ok(OffenderWriteResult {
                offender: offender.as_ref().clone(),
                phones: vec![],
                addresses: vec![],
            }),
            FakeWriteOutcome::Duplicate(msg) => Err(RepositoryError::DuplicateEntry(msg.clone())),
            FakeWriteOutcome::Referenced(msg) => {
                Err(RepositoryError::ReferencedEntityNotFound(msg.clone()))
            }
            FakeWriteOutcome::Internal => Err(RepositoryError::DatabaseError("boom".to_string())),
        }
    }
}

#[async_trait]
impl OffenderWriteRepository for FakeOffenderWriteRepo {
    async fn create_offender(
        &self,
        _offender: CreateOffender,
    ) -> Result<OffenderWriteResult, RepositoryError> {
        self.produce()
    }

    async fn update_offender_by_id(
        &self,
        _data: UpdateOffender,
        _id: Uuid,
    ) -> Result<OffenderWriteResult, RepositoryError> {
        self.produce()
    }

    async fn delete_offender_by_id(
        &self,
        _id: Uuid,
    ) -> Result<OffenderWriteResult, RepositoryError> {
        unimplemented!()
    }

    async fn create_phone(
        &self,
        _offender_id: Uuid,
        _phone_data: PhoneData,
    ) -> Result<OffenderPhone, RepositoryError> {
        unimplemented!()
    }

    async fn get_phone_by_id(&self, _phone_id: Uuid) -> Result<OffenderPhone, RepositoryError> {
        unimplemented!()
    }

    async fn update_phone_by_id(
        &self,
        _phone_id: Uuid,
        _phone_data: PhoneData,
    ) -> Result<OffenderPhone, RepositoryError> {
        unimplemented!()
    }

    async fn delete_phone_by_id(&self, _phone_id: Uuid) -> Result<OffenderPhone, RepositoryError> {
        unimplemented!()
    }

    async fn create_address(
        &self,
        _offender_id: Uuid,
        _address_data: AddressData,
    ) -> Result<OffenderAddress, RepositoryError> {
        unimplemented!()
    }

    async fn get_address_by_id(
        &self,
        _address_id: Uuid,
    ) -> Result<OffenderAddress, RepositoryError> {
        unimplemented!()
    }

    async fn update_address_by_id(
        &self,
        _address_id: Uuid,
        _address_data: AddressData,
    ) -> Result<OffenderAddress, RepositoryError> {
        unimplemented!()
    }

    async fn delete_address_by_id(
        &self,
        _address_id: Uuid,
    ) -> Result<OffenderAddress, RepositoryError> {
        unimplemented!()
    }
}

pub struct FakeOffenderReadRepo {
    pub offender: Option<OffenderWithDetails>,
    pub error: Option<RepositoryError>,
}

impl FakeOffenderReadRepo {
    pub fn found(offender: OffenderWithDetails) -> Self {
        Self {
            offender: Some(offender),
            error: None,
        }
    }

    pub fn not_found() -> Self {
        Self {
            offender: None,
            error: Some(RepositoryError::NotFound),
        }
    }

    pub fn not_expected() -> Self {
        Self {
            offender: None,
            error: None,
        }
    }
}

#[async_trait]
impl OffenderReadRepository for FakeOffenderReadRepo {
    async fn get_offender_by_id(&self, _id: Uuid) -> Result<OffenderWithDetails, RepositoryError> {
        if let Some(err) = self.error.as_ref() {
            return Err(clone_repository_error(err));
        }
        self.offender
            .clone()
            .ok_or_else(|| RepositoryError::DatabaseError("not expected".to_string()))
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

fn clone_repository_error(err: &RepositoryError) -> RepositoryError {
    match err {
        RepositoryError::NotFound => RepositoryError::NotFound,
        RepositoryError::DuplicateEntry(msg) => RepositoryError::DuplicateEntry(msg.clone()),
        RepositoryError::Conflict(msg) => RepositoryError::Conflict(msg.clone()),
        RepositoryError::ReferencedEntityNotFound(msg) => {
            RepositoryError::ReferencedEntityNotFound(msg.clone())
        }
        RepositoryError::DatabaseError(msg) => RepositoryError::DatabaseError(msg.clone()),
        RepositoryError::UniqueViolation { constraint } => RepositoryError::UniqueViolation {
            constraint: constraint.clone(),
        },
        RepositoryError::ForeignKeyViolation { constraint } => {
            RepositoryError::ForeignKeyViolation {
                constraint: constraint.clone(),
            }
        }
    }
}

pub fn base_save_offender(city_id: Option<Uuid>, full_name: &str) -> CreateOffender {
    CreateOffender {
        full_name: full_name.to_string(),
        cpf: None,
        birth_date: None,
        city_id,
        imprisoned: false,
        occupation: None,
        is_public_security_agent: false,
        security_force: None,
        uses_alcohol: false,
        uses_drugs: false,
        has_psychiatric_issues: false,
        psychiatric_issues_type: None,
        education_level: EducationLevel::HighSchool,
        observation: None,
        phones: None,
        addresses: None,
    }
}

pub fn deps(
    read_repo: FakeOffenderReadRepo,
    write_repo: FakeOffenderWriteRepo,
    user_repo: MockUserRepository,
) -> OffenderUseCaseDependencies {
    OffenderUseCaseDependencies::new(
        Arc::new(read_repo),
        Arc::new(write_repo),
        Arc::new(user_repo),
    )
}
