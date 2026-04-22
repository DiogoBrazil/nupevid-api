use std::sync::Arc;

use crate::core::contracts::repository::attendance_members::AttendanceMemberRepository;
use crate::core::contracts::repository::attendance_victims::{
    AttendanceVictimReadRepository, AttendanceVictimWriteRepository,
};
use crate::core::contracts::repository::protective_measures::ProtectiveMeasureReadRepository;
use crate::core::contracts::repository::users::UserRepository;
use crate::core::contracts::repository::victims::VictimReadRepository;
use crate::core::contracts::repository::work_sessions::WorkSessionReadRepository;

#[derive(Clone)]
pub struct AttendanceVictimUseCaseDependencies {
    pub attendance_victim_read_repository: Arc<dyn AttendanceVictimReadRepository>,
    pub attendance_victim_write_repository: Arc<dyn AttendanceVictimWriteRepository>,
    pub victim_repository: Arc<dyn VictimReadRepository>,
    pub protective_measure_repository: Arc<dyn ProtectiveMeasureReadRepository>,
    pub user_repository: Arc<dyn UserRepository>,
    pub work_session_repository: Arc<dyn WorkSessionReadRepository>,
    pub attendance_member_repository: Arc<dyn AttendanceMemberRepository>,
}

impl AttendanceVictimUseCaseDependencies {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        attendance_victim_read_repository: Arc<dyn AttendanceVictimReadRepository>,
        attendance_victim_write_repository: Arc<dyn AttendanceVictimWriteRepository>,
        victim_repository: Arc<dyn VictimReadRepository>,
        protective_measure_repository: Arc<dyn ProtectiveMeasureReadRepository>,
        user_repository: Arc<dyn UserRepository>,
        work_session_repository: Arc<dyn WorkSessionReadRepository>,
        attendance_member_repository: Arc<dyn AttendanceMemberRepository>,
    ) -> Self {
        Self {
            attendance_victim_read_repository,
            attendance_victim_write_repository,
            victim_repository,
            protective_measure_repository,
            user_repository,
            work_session_repository,
            attendance_member_repository,
        }
    }
}
