use std::sync::{Arc, Mutex};

use async_trait::async_trait;
use chrono::{NaiveDate, Utc};
use uuid::Uuid;

use crate::core::commands::work_session_members::AddWorkSessionMember;
use crate::core::commands::work_sessions::CreateWorkSession;
use crate::core::contracts::repository::error::RepositoryError;
use crate::core::contracts::repository::users::MockUserRepository;
use crate::core::contracts::repository::work_sessions::{
    WorkSessionReadRepository, WorkSessionWriteRepository,
};
use crate::core::entities::auth::UserClaims;
use crate::core::entities::users::User;
use crate::core::entities::work_session_members::{TeamMemberFunction, WorkSessionMember};
use crate::core::entities::work_sessions::WorkSession;
use crate::core::read_models::work_sessions::{
    WorkSessionMemberWithDetails, WorkSessionMemberWithUser, WorkSessionWithMemberDetails,
};
use crate::core::value_objects::policies::{PermissionPolicies, Policy};
use crate::core::value_objects::profiles::Profile;
use crate::core::value_objects::ranks::Rank;
use crate::usecases::work_sessions::deps::WorkSessionUseCaseDependencies;

pub fn claims(profile: Profile, user_id: Uuid, city_id: Option<Uuid>) -> UserClaims {
    UserClaims {
        id: user_id.to_string(),
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

pub fn user_record(id: Uuid, city_id: Option<Uuid>) -> User {
    let now = Utc::now();
    User {
        id,
        rank: Rank::SdPm,
        registration: "200000000".to_string(),
        full_name: "MEMBRO".to_string(),
        profile: Profile::CityUser,
        email: "m@test.com".to_string(),
        city_id,
        permission_policies: PermissionPolicies::new(),
        created_at: now,
        updated_at: now,
        is_deleted: false,
    }
}

pub fn work_session(id: Uuid, created_by: Uuid, is_active: bool) -> WorkSession {
    let now = Utc::now();
    WorkSession {
        id,
        created_by_user_id: created_by,
        started_at: now,
        ended_at: if is_active { None } else { Some(now) },
        is_active,
        description: None,
        created_at: now,
        updated_at: now,
    }
}

pub fn session_member(
    session_id: Uuid,
    user_id: Uuid,
    function: Option<TeamMemberFunction>,
) -> WorkSessionMember {
    WorkSessionMember {
        id: Uuid::new_v4(),
        work_session_id: session_id,
        user_id,
        function,
        created_at: Utc::now(),
    }
}

pub fn add_member(user_id: Uuid, function: Option<TeamMemberFunction>) -> AddWorkSessionMember {
    AddWorkSessionMember { user_id, function }
}

pub fn user_repo_with_policy(city_id: Uuid, policy: Policy) -> MockUserRepository {
    let mut repo = MockUserRepository::new();
    let mut p = PermissionPolicies::new();
    p.insert(policy, vec![city_id]);
    repo.expect_get_user_policies_by_id()
        .returning(move |_| Ok(p.clone()));
    repo
}

pub fn user_repo_without_policy() -> MockUserRepository {
    let mut repo = MockUserRepository::new();
    repo.expect_get_user_policies_by_id()
        .returning(|_| Ok(PermissionPolicies::new()));
    repo
}

pub struct FakeReadRepo {
    pub is_user_in_active: bool,
    pub is_user_in_active_error: bool,
    pub active_session: Option<WorkSession>,
    pub session_members: Mutex<Vec<WorkSessionMember>>,
    pub session_members_error: bool,
}

impl FakeReadRepo {
    pub fn not_in_active() -> Self {
        Self {
            is_user_in_active: false,
            is_user_in_active_error: false,
            active_session: None,
            session_members: Mutex::new(vec![]),
            session_members_error: false,
        }
    }

    pub fn in_active() -> Self {
        Self {
            is_user_in_active: true,
            is_user_in_active_error: false,
            active_session: None,
            session_members: Mutex::new(vec![]),
            session_members_error: false,
        }
    }

    pub fn with_active_session(session: WorkSession, members: Vec<WorkSessionMember>) -> Self {
        Self {
            is_user_in_active: true,
            is_user_in_active_error: false,
            active_session: Some(session),
            session_members: Mutex::new(members),
            session_members_error: false,
        }
    }

    pub fn without_active_session() -> Self {
        Self {
            is_user_in_active: false,
            is_user_in_active_error: false,
            active_session: None,
            session_members: Mutex::new(vec![]),
            session_members_error: false,
        }
    }
}

#[async_trait]
impl WorkSessionReadRepository for FakeReadRepo {
    async fn get_active_session_by_user(
        &self,
        _user_id: Uuid,
    ) -> Result<WorkSession, RepositoryError> {
        self.active_session.clone().ok_or(RepositoryError::NotFound)
    }

    async fn get_user_active_session(
        &self,
        _user_id: Uuid,
    ) -> Result<WorkSession, RepositoryError> {
        self.active_session.clone().ok_or(RepositoryError::NotFound)
    }

    async fn is_user_in_active_session(&self, _user_id: Uuid) -> Result<bool, RepositoryError> {
        if self.is_user_in_active_error {
            return Err(RepositoryError::DatabaseError("boom".to_string()));
        }
        Ok(self.is_user_in_active)
    }

    async fn get_session_by_id(
        &self,
        _session_id: Uuid,
    ) -> Result<WorkSessionWithMemberDetails, RepositoryError> {
        unimplemented!()
    }

    async fn get_session_by_id_base(
        &self,
        _session_id: Uuid,
    ) -> Result<WorkSession, RepositoryError> {
        self.active_session.clone().ok_or(RepositoryError::NotFound)
    }

    async fn get_sessions_by_user(
        &self,
        _user_id: Uuid,
    ) -> Result<Vec<WorkSessionWithMemberDetails>, RepositoryError> {
        unimplemented!()
    }

    async fn list_sessions_filtered(
        &self,
        _user_id: Option<Uuid>,
        _start_date: Option<NaiveDate>,
        _end_date: Option<NaiveDate>,
        _city_id: Option<Uuid>,
        _limit: i64,
        _offset: i64,
    ) -> Result<Vec<WorkSession>, RepositoryError> {
        unimplemented!()
    }

    async fn count_sessions_filtered(
        &self,
        _user_id: Option<Uuid>,
        _start_date: Option<NaiveDate>,
        _end_date: Option<NaiveDate>,
        _city_id: Option<Uuid>,
    ) -> Result<i64, RepositoryError> {
        unimplemented!()
    }

    async fn get_session_members(
        &self,
        _session_id: Uuid,
    ) -> Result<Vec<WorkSessionMember>, RepositoryError> {
        if self.session_members_error {
            return Err(RepositoryError::DatabaseError("boom".to_string()));
        }
        Ok(self.session_members.lock().unwrap().clone())
    }

    async fn get_session_members_with_details(
        &self,
        _session_id: Uuid,
    ) -> Result<Vec<WorkSessionMemberWithDetails>, RepositoryError> {
        unimplemented!()
    }

    async fn get_session_members_with_user_details(
        &self,
        _session_id: Uuid,
    ) -> Result<Vec<WorkSessionMemberWithUser>, RepositoryError> {
        unimplemented!()
    }
}

pub enum CreateOutcome {
    Success(Box<WorkSession>),
    Conflict(String),
    Duplicate(String),
    Internal,
}

pub enum EndOutcome {
    Success,
    Error,
}

pub struct FakeWriteRepo {
    pub create: Mutex<Option<CreateOutcome>>,
    pub end: Mutex<Option<EndOutcome>>,
}

impl FakeWriteRepo {
    pub fn new() -> Self {
        Self {
            create: Mutex::new(None),
            end: Mutex::new(None),
        }
    }

    pub fn with_create_success(session: WorkSession) -> Self {
        Self {
            create: Mutex::new(Some(CreateOutcome::Success(Box::new(session)))),
            end: Mutex::new(None),
        }
    }

    pub fn with_create_conflict(msg: &str) -> Self {
        Self {
            create: Mutex::new(Some(CreateOutcome::Conflict(msg.to_string()))),
            end: Mutex::new(None),
        }
    }

    pub fn with_create_duplicate(msg: &str) -> Self {
        Self {
            create: Mutex::new(Some(CreateOutcome::Duplicate(msg.to_string()))),
            end: Mutex::new(None),
        }
    }

    pub fn with_create_internal() -> Self {
        Self {
            create: Mutex::new(Some(CreateOutcome::Internal)),
            end: Mutex::new(None),
        }
    }

    pub fn with_end(outcome: EndOutcome) -> Self {
        Self {
            create: Mutex::new(None),
            end: Mutex::new(Some(outcome)),
        }
    }
}

#[async_trait]
impl WorkSessionWriteRepository for FakeWriteRepo {
    async fn create_work_session(
        &self,
        _data: CreateWorkSession,
        _created_by_user_id: Uuid,
    ) -> Result<WorkSession, RepositoryError> {
        match self.create.lock().unwrap().take() {
            Some(CreateOutcome::Success(session)) => Ok(*session),
            Some(CreateOutcome::Conflict(msg)) => Err(RepositoryError::Conflict(msg)),
            Some(CreateOutcome::Duplicate(msg)) => Err(RepositoryError::DuplicateEntry(msg)),
            Some(CreateOutcome::Internal) => {
                Err(RepositoryError::DatabaseError("boom".to_string()))
            }
            None => panic!("create_work_session called without outcome configured"),
        }
    }

    async fn create_auto_login_session(
        &self,
        _session_id: Uuid,
        _session_member_registration_id: Uuid,
        _user_id: Uuid,
    ) -> Result<WorkSession, RepositoryError> {
        unimplemented!()
    }

    async fn end_session(&self, _session_id: Uuid) -> Result<WorkSession, RepositoryError> {
        match self.end.lock().unwrap().take() {
            Some(EndOutcome::Success) => Ok(work_session(Uuid::new_v4(), Uuid::new_v4(), false)),
            Some(EndOutcome::Error) => Err(RepositoryError::DatabaseError("boom".to_string())),
            None => panic!("end_session called without outcome configured"),
        }
    }

    async fn add_member_to_session(
        &self,
        _session_member_registration_id: Uuid,
        _session_id: Uuid,
        _user_id: Uuid,
        _function: Option<TeamMemberFunction>,
    ) -> Result<WorkSessionMember, RepositoryError> {
        unimplemented!()
    }

    async fn remove_member_from_session(
        &self,
        _session_id: Uuid,
        _user_id: Uuid,
    ) -> Result<WorkSessionMember, RepositoryError> {
        unimplemented!()
    }

    async fn update_member_function(
        &self,
        _session_id: Uuid,
        _user_id: Uuid,
        _function: Option<TeamMemberFunction>,
    ) -> Result<WorkSessionMember, RepositoryError> {
        unimplemented!()
    }

    async fn update_session_members(
        &self,
        _session_id: Uuid,
        _members: Vec<AddWorkSessionMember>,
    ) -> Result<Vec<WorkSessionMember>, RepositoryError> {
        unimplemented!()
    }

    async fn update_work_session_with_members(
        &self,
        _session_id: Uuid,
        _description: Option<String>,
        _members: Vec<AddWorkSessionMember>,
    ) -> Result<WorkSession, RepositoryError> {
        unimplemented!()
    }
}

pub fn deps(
    read_repo: FakeReadRepo,
    write_repo: FakeWriteRepo,
    user_repo: MockUserRepository,
) -> WorkSessionUseCaseDependencies {
    WorkSessionUseCaseDependencies::new(
        Arc::new(read_repo),
        Arc::new(write_repo),
        Arc::new(user_repo),
    )
}
