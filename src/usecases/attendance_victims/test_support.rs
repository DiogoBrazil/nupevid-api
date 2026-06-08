use std::sync::{Arc, Mutex};

use async_trait::async_trait;
use chrono::{DateTime, NaiveDate, NaiveTime, Utc};
use uuid::Uuid;

use crate::core::commands::attendance_victims::{CreateAttendanceVictim, UpdateAttendanceVictim};
use crate::core::contracts::repository::attendance_members::AttendanceMemberRepository;
use crate::core::contracts::repository::attendance_victims::{
    AttendanceVictimReadRepository, AttendanceVictimWriteRepository,
};
use crate::core::contracts::repository::error::RepositoryError;
use crate::core::contracts::repository::protective_measures::ProtectiveMeasureReadRepository;
use crate::core::contracts::repository::users::MockUserRepository;
use crate::core::contracts::repository::victims::MockVictimReadRepository;
use crate::core::contracts::repository::work_sessions::WorkSessionReadRepository;
use crate::core::entities::attendance_members::{AttendanceOffenderMember, AttendanceVictimMember};
use crate::core::entities::attendance_victims::{AttendanceVictim, AttendanceVictimWriteResult};
use crate::core::entities::auth::UserClaims;
use crate::core::entities::protective_measures::{
    ProtectiveMeasure, ProtectiveMeasureStatus, RelationshipToVictim, ViolenceType,
};
use crate::core::entities::users::User;
use crate::core::entities::work_session_members::WorkSessionMember;
use crate::core::entities::work_sessions::WorkSession;
use crate::core::read_models::attendance_members::AttendanceMemberWithDetails;
use crate::core::read_models::attendance_victims::AttendanceVictimWithAddress;
use crate::core::read_models::victims::{VictimSummary, VictimWithDetails};
use crate::core::read_models::work_sessions::{
    WorkSessionMemberWithDetails, WorkSessionMemberWithUser, WorkSessionWithMemberDetails,
};
use crate::core::value_objects::policies::{PermissionPolicies, Policy};
use crate::core::value_objects::profiles::Profile;
use crate::core::value_objects::ranks::Rank;
use crate::usecases::attendance_victims::deps::AttendanceVictimUseCaseDependencies;

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

pub fn victim_with_details(id: Uuid, city_id: Uuid) -> VictimWithDetails {
    let now = Utc::now();
    VictimWithDetails {
        summary: VictimSummary {
            id,
            full_name: "VICTIM".to_string(),
            cpf: None,
            birth_date: None,
            city_id,
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
        },
        created_at: now,
        updated_at: now,
    }
}

pub fn attendance_victim(id: Uuid, victim_id: Uuid) -> AttendanceVictimWithAddress {
    let now = Utc::now();
    AttendanceVictimWithAddress {
        id,
        victim_id,
        was_victim_present: true,
        attendance_date: NaiveDate::from_ymd_opt(2025, 6, 1).unwrap(),
        attendance_time: NaiveTime::from_hms_opt(10, 0, 0).unwrap(),
        description: None,
        latitude: None,
        longitude: None,
        created_at: now,
        updated_at: now,
        is_deleted: false,
        address: None,
        offender_id: None,
        protective_measure_id: None,
        is_remote: true,
        risk_level: None,
        offender_freedom_status: None,
        offender_has_firearm_access: None,
        needs_legal_assistance: false,
        needs_psychological_support: false,
        was_instructed_about_protective_measure_procedures: false,
        offender_violated_protective_measure: false,
    }
}

pub fn attendance_victim_entity(id: Uuid, victim_id: Uuid) -> AttendanceVictim {
    let now = Utc::now();
    AttendanceVictim {
        id,
        victim_id,
        was_victim_present: true,
        attendance_date: NaiveDate::from_ymd_opt(2025, 6, 1).unwrap(),
        attendance_time: NaiveTime::from_hms_opt(10, 0, 0).unwrap(),
        description: None,
        latitude: None,
        longitude: None,
        created_at: now,
        updated_at: now,
        is_deleted: false,
        offender_id: None,
        protective_measure_id: None,
        is_remote: true,
        risk_level: None,
        offender_freedom_status: None,
        offender_has_firearm_access: None,
        needs_legal_assistance: false,
        needs_psychological_support: false,
        was_instructed_about_protective_measure_procedures: false,
        offender_violated_protective_measure: false,
    }
}

pub fn create_command(protective_measure_id: Uuid) -> CreateAttendanceVictim {
    CreateAttendanceVictim {
        protective_measure_id,
        was_victim_present: true,
        attendance_date: NaiveDate::from_ymd_opt(2025, 6, 1).unwrap(),
        attendance_time: NaiveTime::from_hms_opt(10, 0, 0).unwrap(),
        description: None,
        latitude: None,
        longitude: None,
        address: None,
        is_remote: true,
        risk_level: None,
        offender_freedom_status: None,
        offender_has_firearm_access: None,
        needs_legal_assistance: false,
        needs_psychological_support: false,
        was_instructed_about_protective_measure_procedures: false,
        offender_violated_protective_measure: false,
    }
}

pub fn update_command(protective_measure_id: Uuid) -> UpdateAttendanceVictim {
    UpdateAttendanceVictim {
        protective_measure_id,
        was_victim_present: true,
        attendance_date: NaiveDate::from_ymd_opt(2025, 6, 1).unwrap(),
        attendance_time: NaiveTime::from_hms_opt(10, 0, 0).unwrap(),
        description: None,
        latitude: None,
        longitude: None,
        address: None,
        is_remote: true,
        risk_level: None,
        offender_freedom_status: None,
        offender_has_firearm_access: None,
        needs_legal_assistance: false,
        needs_psychological_support: false,
        was_instructed_about_protective_measure_procedures: false,
        offender_violated_protective_measure: false,
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

pub struct FakeAttendanceVictimReadRepo {
    pub attendance: Option<AttendanceVictimWithAddress>,
}

#[async_trait]
impl AttendanceVictimReadRepository for FakeAttendanceVictimReadRepo {
    async fn get_attendance_victim_by_id(
        &self,
        _id: Uuid,
    ) -> Result<AttendanceVictimWithAddress, RepositoryError> {
        self.attendance.clone().ok_or(RepositoryError::NotFound)
    }

    async fn get_all_attendance_victims(
        &self,
    ) -> Result<Vec<AttendanceVictimWithAddress>, RepositoryError> {
        unimplemented!()
    }

    async fn get_attendance_victims_paginated(
        &self,
        _allowed_cities: Option<&[Uuid]>,
        _limit: i64,
        _offset: i64,
    ) -> Result<Vec<AttendanceVictimWithAddress>, RepositoryError> {
        unimplemented!()
    }

    async fn count_attendance_victims(
        &self,
        _allowed_cities: Option<&[Uuid]>,
    ) -> Result<i64, RepositoryError> {
        unimplemented!()
    }

    async fn get_attendance_victims_by_victim(
        &self,
        _victim_id: Uuid,
    ) -> Result<Vec<AttendanceVictimWithAddress>, RepositoryError> {
        unimplemented!()
    }

    async fn get_attendance_victims_by_protective_measure(
        &self,
        _protective_measure_id: Uuid,
    ) -> Result<Vec<AttendanceVictimWithAddress>, RepositoryError> {
        unimplemented!()
    }
}

pub enum WriteOutcome {
    Success(Box<AttendanceVictim>),
    Referenced(String),
    NotFound,
    Internal,
}

pub struct FakeAttendanceVictimWriteRepo {
    pub outcome: Mutex<Option<WriteOutcome>>,
    pub session_members_captured: Mutex<Option<Vec<(Uuid, Option<Uuid>)>>>,
}

impl FakeAttendanceVictimWriteRepo {
    pub fn success(entity: AttendanceVictim) -> Self {
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

    pub fn not_found() -> Self {
        Self {
            outcome: Mutex::new(Some(WriteOutcome::NotFound)),
            session_members_captured: Mutex::new(None),
        }
    }

    pub fn internal() -> Self {
        Self {
            outcome: Mutex::new(Some(WriteOutcome::Internal)),
            session_members_captured: Mutex::new(None),
        }
    }

    fn produce(&self) -> Result<AttendanceVictimWriteResult, RepositoryError> {
        match self.outcome.lock().unwrap().take() {
            Some(WriteOutcome::Success(entity)) => Ok(AttendanceVictimWriteResult {
                attendance: *entity,
                address: None,
            }),
            Some(WriteOutcome::Referenced(msg)) => {
                Err(RepositoryError::ReferencedEntityNotFound(msg))
            }
            Some(WriteOutcome::NotFound) => Err(RepositoryError::NotFound),
            Some(WriteOutcome::Internal) => Err(RepositoryError::DatabaseError("boom".to_string())),
            None => panic!("repository invoked without outcome"),
        }
    }
}

#[async_trait]
impl AttendanceVictimWriteRepository for FakeAttendanceVictimWriteRepo {
    async fn create_attendance_victim(
        &self,
        _attendance: CreateAttendanceVictim,
        _victim_id: Uuid,
        _offender_id: Option<Uuid>,
        session_members: Vec<(Uuid, Option<Uuid>)>,
    ) -> Result<AttendanceVictimWriteResult, RepositoryError> {
        *self.session_members_captured.lock().unwrap() = Some(session_members);
        self.produce()
    }

    async fn update_attendance_victim_by_id(
        &self,
        _data: UpdateAttendanceVictim,
        _id: Uuid,
        _victim_id: Uuid,
        _offender_id: Option<Uuid>,
    ) -> Result<AttendanceVictimWriteResult, RepositoryError> {
        self.produce()
    }

    async fn delete_attendance_victim_by_id(
        &self,
        _id: Uuid,
    ) -> Result<AttendanceVictimWriteResult, RepositoryError> {
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
        match self.add_outcome.lock().unwrap().take() {
            Some(MemberAddOutcome::Success) => Ok(victim_member()),
            Some(MemberAddOutcome::Duplicate(msg)) => Err(RepositoryError::DuplicateEntry(msg)),
            Some(MemberAddOutcome::Internal) => {
                Err(RepositoryError::DatabaseError("boom".to_string()))
            }
            None => panic!("add_member invoked without outcome"),
        }
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
        match self.remove_outcome.lock().unwrap().take() {
            Some(MemberRemoveOutcome::Success) => Ok(victim_member()),
            Some(MemberRemoveOutcome::NotFound) => Err(RepositoryError::NotFound),
            Some(MemberRemoveOutcome::Internal) => {
                Err(RepositoryError::DatabaseError("boom".to_string()))
            }
            None => panic!("remove_member invoked without outcome"),
        }
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

pub fn victim_repo_ok(victim_id: Uuid, city_id: Uuid) -> MockVictimReadRepository {
    let mut repo = MockVictimReadRepository::new();
    repo.expect_get_victim_by_id()
        .returning(move |_| Ok(victim_with_details(victim_id, city_id)));
    repo
}

pub fn victim_repo_multi(
    v1_id: Uuid,
    v1_city: Uuid,
    v2_id: Uuid,
    v2_city: Uuid,
) -> MockVictimReadRepository {
    let mut repo = MockVictimReadRepository::new();
    repo.expect_get_victim_by_id().returning(move |id| {
        if id == v1_id {
            Ok(victim_with_details(v1_id, v1_city))
        } else if id == v2_id {
            Ok(victim_with_details(v2_id, v2_city))
        } else {
            Err(RepositoryError::NotFound)
        }
    });
    repo
}

#[allow(clippy::too_many_arguments)]
pub fn deps(
    read_repo: FakeAttendanceVictimReadRepo,
    write_repo: FakeAttendanceVictimWriteRepo,
    victim_repo: MockVictimReadRepository,
    pm_repo: FakePmReadRepo,
    user_repo: MockUserRepository,
    work_session_repo: FakeWorkSessionReadRepo,
    member_repo: FakeMemberRepo,
) -> (
    AttendanceVictimUseCaseDependencies,
    Arc<FakeAttendanceVictimWriteRepo>,
) {
    let write_arc = Arc::new(write_repo);
    let deps = AttendanceVictimUseCaseDependencies::new(
        Arc::new(read_repo),
        write_arc.clone(),
        Arc::new(victim_repo),
        Arc::new(pm_repo),
        Arc::new(user_repo),
        Arc::new(work_session_repo),
        Arc::new(member_repo),
    );
    (deps, write_arc)
}
