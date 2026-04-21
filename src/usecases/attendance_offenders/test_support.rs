use std::sync::{Arc, Mutex};

use async_trait::async_trait;
use chrono::{DateTime, NaiveDate, NaiveTime, Utc};
use uuid::Uuid;

use crate::core::commands::attendance_offenders::{
    CreateAttendanceOffender, UpdateAttendanceOffender,
};
use crate::core::contracts::repository::attendance_members::AttendanceMemberRepository;
use crate::core::contracts::repository::attendance_offenders::{
    AttendanceOffenderReadRepository, AttendanceOffenderWriteRepository,
};
use crate::core::contracts::repository::error::RepositoryError;
use crate::core::contracts::repository::offenders::OffenderReadRepository;
use crate::core::contracts::repository::protective_measures::ProtectiveMeasureReadRepository;
use crate::core::contracts::repository::users::MockUserRepository;
use crate::core::contracts::repository::victims::MockVictimReadRepository;
use crate::core::contracts::repository::work_sessions::WorkSessionReadRepository;
use crate::core::entities::attendance_members::{AttendanceOffenderMember, AttendanceVictimMember};
use crate::core::entities::attendance_offenders::{
    AttendanceOffender, AttendanceOffenderWriteResult, ViolenceAggravator,
};
use crate::core::entities::auth::UserClaims;
use crate::core::entities::common::EducationLevel;
use crate::core::entities::protective_measures::{
    ProtectiveMeasure, ProtectiveMeasureStatus, RelationshipToVictim, ViolenceType,
};
use crate::core::entities::users::User;
use crate::core::entities::work_session_members::WorkSessionMember;
use crate::core::entities::work_sessions::WorkSession;
use crate::core::read_models::attendance_members::AttendanceMemberWithDetails;
use crate::core::read_models::attendance_offenders::AttendanceOffenderWithAddress;
use crate::core::read_models::offenders::OffenderWithDetails;
use crate::core::read_models::victims::VictimWithDetails;
use crate::core::read_models::work_sessions::{
    WorkSessionMemberWithDetails, WorkSessionMemberWithUser, WorkSessionWithMemberDetails,
};
use crate::core::value_objects::policies::{PermissionPolicies, Policy};
use crate::core::value_objects::profiles::Profile;
use crate::core::value_objects::ranks::Rank;
use crate::usecases::attendance_offenders::deps::AttendanceOffenderUseCaseDependencies;

pub fn claims(profile: Profile, user_id: Uuid, city_id: Option<Uuid>) -> UserClaims {
    UserClaims {
        id: user_id.to_string(),
        exp: 9999999999,
        iss: "nupevid-api".to_string(),
        aud: "nupevid-api".to_string(),
        rank: Rank::SdPm,
        registration: "100000001".to_string(),
        full_name: "Op".to_string(),
        profile,
        email: "op@test.com".to_string(),
        city_id: city_id.map(|c| c.to_string()),
    }
}

pub fn protective_measure(id: Uuid, victim_id: Uuid, offender_id: Uuid) -> ProtectiveMeasure {
    let now = Utc::now();
    ProtectiveMeasure {
        id,
        process_number: "00000-00.0000.0.00.0000".to_string(),
        sei_process_number: None,
        occurrence_report_number: None,
        issued_at: NaiveDate::from_ymd_opt(2025, 1, 1).unwrap(),
        judicial_authority: "Juiz".to_string(),
        court_district_id: Uuid::new_v4(),
        distance_meters: None,
        status: ProtectiveMeasureStatus::Valid,
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

pub fn offender_with_details(id: Uuid, city_id: Uuid) -> OffenderWithDetails {
    let now = Utc::now();
    OffenderWithDetails {
        id,
        full_name: "OFFENDER".to_string(),
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
        phones: vec![],
        addresses: vec![],
    }
}

pub fn victim_with_details(id: Uuid, city_id: Uuid) -> VictimWithDetails {
    let now = Utc::now();
    VictimWithDetails {
        id,
        full_name: "VICTIM".to_string(),
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

pub fn attendance_offender(
    id: Uuid,
    offender_id: Uuid,
    victim_id: Uuid,
) -> AttendanceOffenderWithAddress {
    let now = Utc::now();
    AttendanceOffenderWithAddress {
        id,
        offender_id,
        victim_id,
        protective_measure_id: None,
        was_offender_present: true,
        attendance_date: NaiveDate::from_ymd_opt(2025, 6, 1).unwrap(),
        attendance_time: NaiveTime::from_hms_opt(10, 0, 0).unwrap(),
        is_remote: true,
        assaults_children: false,
        violence_aggravator: ViolenceAggravator::Other,
        violence_aggravator_other: None,
        description: None,
        created_at: now,
        updated_at: now,
        is_deleted: false,
        address: None,
    }
}

pub fn attendance_offender_entity(
    id: Uuid,
    offender_id: Uuid,
    victim_id: Uuid,
) -> AttendanceOffender {
    let now = Utc::now();
    AttendanceOffender {
        id,
        offender_id,
        victim_id,
        protective_measure_id: None,
        was_offender_present: true,
        attendance_date: NaiveDate::from_ymd_opt(2025, 6, 1).unwrap(),
        attendance_time: NaiveTime::from_hms_opt(10, 0, 0).unwrap(),
        is_remote: true,
        assaults_children: false,
        violence_aggravator: ViolenceAggravator::Other,
        violence_aggravator_other: None,
        description: None,
        created_at: now,
        updated_at: now,
        is_deleted: false,
    }
}

pub fn create_command(protective_measure_id: Uuid) -> CreateAttendanceOffender {
    CreateAttendanceOffender {
        protective_measure_id,
        was_offender_present: true,
        attendance_date: NaiveDate::from_ymd_opt(2025, 6, 1).unwrap(),
        attendance_time: NaiveTime::from_hms_opt(10, 0, 0).unwrap(),
        address: None,
        is_remote: true,
        assaults_children: false,
        violence_aggravator: ViolenceAggravator::Other,
        violence_aggravator_other: None,
        description: None,
    }
}

pub fn update_command(protective_measure_id: Uuid) -> UpdateAttendanceOffender {
    UpdateAttendanceOffender {
        protective_measure_id,
        was_offender_present: true,
        attendance_date: NaiveDate::from_ymd_opt(2025, 6, 1).unwrap(),
        attendance_time: NaiveTime::from_hms_opt(10, 0, 0).unwrap(),
        address: None,
        is_remote: true,
        assaults_children: false,
        violence_aggravator: ViolenceAggravator::Other,
        violence_aggravator_other: None,
        description: None,
    }
}

pub fn user_record(id: Uuid, city_id: Option<Uuid>) -> User {
    let now = Utc::now();
    User {
        id,
        rank: Rank::SdPm,
        registration: "200000000".to_string(),
        full_name: "M".to_string(),
        profile: Profile::CityUser,
        email: "m@t.com".to_string(),
        city_id,
        permission_policies: PermissionPolicies::new(),
        created_at: now,
        updated_at: now,
        is_deleted: false,
    }
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

pub struct FakeAttendanceOffenderReadRepo {
    pub attendance: Option<AttendanceOffenderWithAddress>,
}

#[async_trait]
impl AttendanceOffenderReadRepository for FakeAttendanceOffenderReadRepo {
    async fn get_attendance_offender_by_id(
        &self,
        _id: Uuid,
    ) -> Result<AttendanceOffenderWithAddress, RepositoryError> {
        self.attendance.clone().ok_or(RepositoryError::NotFound)
    }

    async fn get_all_attendance_offenders(
        &self,
    ) -> Result<Vec<AttendanceOffenderWithAddress>, RepositoryError> {
        unimplemented!()
    }

    async fn get_attendance_offenders_paginated(
        &self,
        _allowed_cities: Option<&[Uuid]>,
        _limit: i64,
        _offset: i64,
    ) -> Result<Vec<AttendanceOffenderWithAddress>, RepositoryError> {
        unimplemented!()
    }

    async fn count_attendance_offenders(
        &self,
        _allowed_cities: Option<&[Uuid]>,
    ) -> Result<i64, RepositoryError> {
        unimplemented!()
    }

    async fn get_attendance_offenders_by_offender(
        &self,
        _offender_id: Uuid,
    ) -> Result<Vec<AttendanceOffenderWithAddress>, RepositoryError> {
        unimplemented!()
    }

    async fn get_attendance_offenders_by_victim(
        &self,
        _victim_id: Uuid,
    ) -> Result<Vec<AttendanceOffenderWithAddress>, RepositoryError> {
        unimplemented!()
    }

    async fn get_attendance_offenders_by_protective_measure(
        &self,
        _protective_measure_id: Uuid,
    ) -> Result<Vec<AttendanceOffenderWithAddress>, RepositoryError> {
        unimplemented!()
    }
}

pub enum WriteOutcome {
    Success(Box<AttendanceOffender>),
    Referenced(String),
    Internal,
}

pub struct FakeAttendanceOffenderWriteRepo {
    pub outcome: Mutex<Option<WriteOutcome>>,
    pub session_members_captured: Mutex<Option<Vec<(Uuid, Option<Uuid>)>>>,
}

impl FakeAttendanceOffenderWriteRepo {
    pub fn success(entity: AttendanceOffender) -> Self {
        Self {
            outcome: Mutex::new(Some(WriteOutcome::Success(Box::new(entity)))),
            session_members_captured: Mutex::new(None),
        }
    }

    pub fn referenced(msg: &str) -> Self {
        Self {
            outcome: Mutex::new(Some(WriteOutcome::Referenced(msg.to_string()))),
            session_members_captured: Mutex::new(None),
        }
    }

    pub fn internal() -> Self {
        Self {
            outcome: Mutex::new(Some(WriteOutcome::Internal)),
            session_members_captured: Mutex::new(None),
        }
    }

    fn produce(&self) -> Result<AttendanceOffenderWriteResult, RepositoryError> {
        match self.outcome.lock().unwrap().take() {
            Some(WriteOutcome::Success(entity)) => Ok(AttendanceOffenderWriteResult {
                attendance: *entity,
                address: None,
            }),
            Some(WriteOutcome::Referenced(msg)) => {
                Err(RepositoryError::ReferencedEntityNotFound(msg))
            }
            Some(WriteOutcome::Internal) => Err(RepositoryError::DatabaseError("boom".to_string())),
            None => panic!("repository invoked without outcome"),
        }
    }
}

#[async_trait]
impl AttendanceOffenderWriteRepository for FakeAttendanceOffenderWriteRepo {
    async fn create_attendance_offender(
        &self,
        _attendance: CreateAttendanceOffender,
        _offender_id: Uuid,
        _victim_id: Uuid,
        session_members: Vec<(Uuid, Option<Uuid>)>,
    ) -> Result<AttendanceOffenderWriteResult, RepositoryError> {
        *self.session_members_captured.lock().unwrap() = Some(session_members);
        self.produce()
    }

    async fn update_attendance_offender_by_id(
        &self,
        _data: UpdateAttendanceOffender,
        _id: Uuid,
        _offender_id: Uuid,
        _victim_id: Uuid,
    ) -> Result<AttendanceOffenderWriteResult, RepositoryError> {
        self.produce()
    }

    async fn delete_attendance_offender_by_id(
        &self,
        _id: Uuid,
    ) -> Result<AttendanceOffenderWriteResult, RepositoryError> {
        unimplemented!()
    }
}

pub struct FakePmReadRepo {
    pub measure: Option<ProtectiveMeasure>,
}

#[async_trait]
impl ProtectiveMeasureReadRepository for FakePmReadRepo {
    async fn get_protective_measure_by_id(
        &self,
        _id: Uuid,
    ) -> Result<ProtectiveMeasure, RepositoryError> {
        self.measure.clone().ok_or(RepositoryError::NotFound)
    }

    async fn get_all_protective_measures(&self) -> Result<Vec<ProtectiveMeasure>, RepositoryError> {
        unimplemented!()
    }

    async fn get_protective_measures_paginated(
        &self,
        _allowed_cities: Option<&[Uuid]>,
        _limit: i64,
        _offset: i64,
    ) -> Result<Vec<ProtectiveMeasure>, RepositoryError> {
        unimplemented!()
    }

    async fn count_protective_measures(
        &self,
        _allowed_cities: Option<&[Uuid]>,
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
        _victim_id: Uuid,
        _offender_id: Uuid,
        _exclude_measure_id: Option<Uuid>,
    ) -> Result<bool, RepositoryError> {
        Ok(false)
    }
}

pub struct FakeWorkSessionReadRepo {
    pub active_session: Option<WorkSession>,
    pub session_members: Vec<WorkSessionMember>,
}

impl FakeWorkSessionReadRepo {
    pub fn with_session(session: WorkSession, session_members: Vec<WorkSessionMember>) -> Self {
        Self {
            active_session: Some(session),
            session_members,
        }
    }

    pub fn without_session() -> Self {
        Self {
            active_session: None,
            session_members: vec![],
        }
    }
}

#[async_trait]
impl WorkSessionReadRepository for FakeWorkSessionReadRepo {
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
        Ok(self.active_session.is_some())
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
        Ok(self.session_members.clone())
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

pub enum MemberAddOutcome {
    Success,
    Duplicate(String),
    Internal,
}

pub enum MemberRemoveOutcome {
    Success,
    NotFound,
    Internal,
}

pub struct FakeMemberRepo {
    pub add_outcome: Mutex<Option<MemberAddOutcome>>,
    pub remove_outcome: Mutex<Option<MemberRemoveOutcome>>,
}

impl FakeMemberRepo {
    pub fn with_add(outcome: MemberAddOutcome) -> Self {
        Self {
            add_outcome: Mutex::new(Some(outcome)),
            remove_outcome: Mutex::new(None),
        }
    }

    pub fn with_remove(outcome: MemberRemoveOutcome) -> Self {
        Self {
            add_outcome: Mutex::new(None),
            remove_outcome: Mutex::new(Some(outcome)),
        }
    }

    pub fn idle() -> Self {
        Self {
            add_outcome: Mutex::new(None),
            remove_outcome: Mutex::new(None),
        }
    }
}

fn victim_member() -> AttendanceVictimMember {
    AttendanceVictimMember {
        id: Uuid::new_v4(),
        attendance_victim_id: Uuid::new_v4(),
        user_id: Uuid::new_v4(),
        work_session_id: None,
        created_at: Utc::now(),
    }
}

fn offender_member() -> AttendanceOffenderMember {
    AttendanceOffenderMember {
        id: Uuid::new_v4(),
        attendance_offender_id: Uuid::new_v4(),
        user_id: Uuid::new_v4(),
        work_session_id: None,
        created_at: Utc::now(),
    }
}

#[async_trait]
impl AttendanceMemberRepository for FakeMemberRepo {
    async fn add_member_to_victim_attendance(
        &self,
        _attendance_id: Uuid,
        _user_id: Uuid,
        _work_session_id: Option<Uuid>,
    ) -> Result<AttendanceVictimMember, RepositoryError> {
        Ok(victim_member())
    }

    async fn get_victim_attendance_members(
        &self,
        _attendance_id: Uuid,
    ) -> Result<Vec<AttendanceMemberWithDetails>, RepositoryError> {
        Ok(vec![])
    }

    async fn remove_member_from_victim_attendance(
        &self,
        _attendance_id: Uuid,
        _user_id: Uuid,
    ) -> Result<AttendanceVictimMember, RepositoryError> {
        Ok(victim_member())
    }

    async fn add_member_to_offender_attendance(
        &self,
        _attendance_id: Uuid,
        _user_id: Uuid,
        _work_session_id: Option<Uuid>,
    ) -> Result<AttendanceOffenderMember, RepositoryError> {
        match self.add_outcome.lock().unwrap().take() {
            Some(MemberAddOutcome::Success) => Ok(offender_member()),
            Some(MemberAddOutcome::Duplicate(msg)) => Err(RepositoryError::DuplicateEntry(msg)),
            Some(MemberAddOutcome::Internal) => {
                Err(RepositoryError::DatabaseError("boom".to_string()))
            }
            None => panic!("add_member invoked without outcome"),
        }
    }

    async fn get_offender_attendance_members(
        &self,
        _attendance_id: Uuid,
    ) -> Result<Vec<AttendanceMemberWithDetails>, RepositoryError> {
        Ok(vec![])
    }

    async fn remove_member_from_offender_attendance(
        &self,
        _attendance_id: Uuid,
        _user_id: Uuid,
    ) -> Result<AttendanceOffenderMember, RepositoryError> {
        match self.remove_outcome.lock().unwrap().take() {
            Some(MemberRemoveOutcome::Success) => Ok(offender_member()),
            Some(MemberRemoveOutcome::NotFound) => Err(RepositoryError::NotFound),
            Some(MemberRemoveOutcome::Internal) => {
                Err(RepositoryError::DatabaseError("boom".to_string()))
            }
            None => panic!("remove_member invoked without outcome"),
        }
    }
}

pub fn work_session_active(id: Uuid, created_by: Uuid) -> WorkSession {
    let now: DateTime<Utc> = Utc::now();
    WorkSession {
        id,
        created_by_user_id: created_by,
        started_at: now,
        ended_at: None,
        is_active: true,
        description: None,
        created_at: now,
        updated_at: now,
    }
}

pub fn session_member_entity(session_id: Uuid, user_id: Uuid) -> WorkSessionMember {
    WorkSessionMember {
        id: Uuid::new_v4(),
        work_session_id: session_id,
        user_id,
        function: None,
        created_at: Utc::now(),
    }
}

pub struct FakeOffenderReadRepo {
    pub offender: Option<OffenderWithDetails>,
    pub offender2: Option<(Uuid, OffenderWithDetails)>,
}

impl FakeOffenderReadRepo {
    pub fn found(offender: OffenderWithDetails) -> Self {
        Self {
            offender: Some(offender),
            offender2: None,
        }
    }

    pub fn with_two(
        first: OffenderWithDetails,
        second_id: Uuid,
        second: OffenderWithDetails,
    ) -> Self {
        Self {
            offender: Some(first),
            offender2: Some((second_id, second)),
        }
    }
}

#[async_trait]
impl OffenderReadRepository for FakeOffenderReadRepo {
    async fn get_offender_by_id(&self, id: Uuid) -> Result<OffenderWithDetails, RepositoryError> {
        if let Some((second_id, second)) = self.offender2.as_ref() {
            if id == *second_id {
                return Ok(second.clone());
            }
        }
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

pub fn victim_repo_ok(victim_id: Uuid, city_id: Uuid) -> MockVictimReadRepository {
    let mut repo = MockVictimReadRepository::new();
    repo.expect_get_victim_by_id()
        .returning(move |_| Ok(victim_with_details(victim_id, city_id)));
    repo
}

#[allow(clippy::too_many_arguments)]
pub fn deps(
    read_repo: FakeAttendanceOffenderReadRepo,
    write_repo: FakeAttendanceOffenderWriteRepo,
    offender_repo: FakeOffenderReadRepo,
    victim_repo: MockVictimReadRepository,
    pm_repo: FakePmReadRepo,
    user_repo: MockUserRepository,
    work_session_repo: FakeWorkSessionReadRepo,
    member_repo: FakeMemberRepo,
) -> (
    AttendanceOffenderUseCaseDependencies,
    Arc<FakeAttendanceOffenderWriteRepo>,
) {
    let write_arc = Arc::new(write_repo);
    let deps = AttendanceOffenderUseCaseDependencies::new(
        Arc::new(read_repo),
        write_arc.clone(),
        Arc::new(offender_repo),
        Arc::new(victim_repo),
        Arc::new(pm_repo),
        Arc::new(user_repo),
        Arc::new(work_session_repo),
        Arc::new(member_repo),
    );
    (deps, write_arc)
}
