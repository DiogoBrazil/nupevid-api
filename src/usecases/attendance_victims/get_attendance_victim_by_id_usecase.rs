use log::info;
use uuid::Uuid;

use crate::core::application_error::ApplicationError as AppError;
use crate::core::auth_context::AuthContext;
use crate::core::entities::auth::UserClaims;
use crate::core::read_models::attendance_victims::AttendanceVictimWithAddress;
use crate::core::value_objects::policies::Policy;
use crate::usecases::attendance_victims::deps::AttendanceVictimUseCaseDependencies;
use crate::usecases::helpers_common::{
    get_attendance_victim_or_not_found, get_victim_or_not_found,
};

pub struct GetAttendanceVictimByIdUseCase {
    deps: AttendanceVictimUseCaseDependencies,
}

impl GetAttendanceVictimByIdUseCase {
    pub fn new(deps: AttendanceVictimUseCaseDependencies) -> Self {
        Self { deps }
    }

    pub async fn execute(
        &self,
        id: Uuid,
        claims: &UserClaims,
    ) -> Result<AttendanceVictimWithAddress, AppError> {
        info!(
            "[GetAttendanceVictimByIdUseCase] Getting attendance victim with ID: {}",
            id
        );

        let attendance_with_address =
            get_attendance_victim_or_not_found(&*self.deps.attendance_victim_read_repository, id)
                .await?;
        let victim = get_victim_or_not_found(
            &*self.deps.victim_repository,
            attendance_with_address.victim_id,
        )
        .await?;
        let auth = AuthContext::load(&*self.deps.user_repository, claims).await?;
        auth.check_policy(&Policy::ReadAttendances, victim.summary.city_id)?;
        Ok(attendance_with_address)
    }
}
