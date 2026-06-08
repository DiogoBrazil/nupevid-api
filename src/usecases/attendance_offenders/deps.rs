use std::sync::Arc;

use crate::core::contracts::repository::attendance_members::AttendanceMemberRepository;
use crate::core::contracts::repository::attendance_offenders::{
    AttendanceOffenderReadRepository, AttendanceOffenderWriteRepository,
};
use crate::core::contracts::repository::offenders::OffenderReadRepository;
use crate::core::contracts::repository::protective_measures::ProtectiveMeasureReadRepository;
use crate::core::contracts::repository::users::UserRepository;
use crate::core::contracts::repository::victims::VictimReadRepository;
use crate::core::contracts::repository::work_sessions::WorkSessionReadRepository;

#[derive(Clone)]
pub struct AttendanceOffenderUseCaseDependencies {
    pub attendance_offender_read_repository: Arc<dyn AttendanceOffenderReadRepository>,
    pub attendance_offender_write_repository: Arc<dyn AttendanceOffenderWriteRepository>,
    pub offender_repository: Arc<dyn OffenderReadRepository>,
    pub victim_repository: Arc<dyn VictimReadRepository>,
    pub protective_measure_repository: Arc<dyn ProtectiveMeasureReadRepository>,
    pub user_repository: Arc<dyn UserRepository>,
    pub work_session_repository: Arc<dyn WorkSessionReadRepository>,
    pub attendance_member_repository: Arc<dyn AttendanceMemberRepository>,
}

impl AttendanceOffenderUseCaseDependencies {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        attendance_offender_read_repository: Arc<dyn AttendanceOffenderReadRepository>,
        attendance_offender_write_repository: Arc<dyn AttendanceOffenderWriteRepository>,
        offender_repository: Arc<dyn OffenderReadRepository>,
        victim_repository: Arc<dyn VictimReadRepository>,
        protective_measure_repository: Arc<dyn ProtectiveMeasureReadRepository>,
        user_repository: Arc<dyn UserRepository>,
        work_session_repository: Arc<dyn WorkSessionReadRepository>,
        attendance_member_repository: Arc<dyn AttendanceMemberRepository>,
    ) -> Self {
        Self {
            attendance_offender_read_repository,
            attendance_offender_write_repository,
            offender_repository,
            victim_repository,
            protective_measure_repository,
            user_repository,
            work_session_repository,
            attendance_member_repository,
        }
    }
}
