use log::{error, info};

use crate::core::application_error::ApplicationError as AppError;
use crate::core::entities::auth::UserClaims;
use crate::core::value_objects::policies::Policy;
use crate::usecases::work_sessions::deps::WorkSessionUseCaseDependencies;
use crate::usecases::work_sessions::guards::ensure_creator_or_commander;
use crate::usecases::work_sessions::helpers::{
    authorize_non_root_for_policy, claims_user_id, get_session_members_or_not_found,
};

pub struct EndSessionUseCase {
    deps: WorkSessionUseCaseDependencies,
}

impl EndSessionUseCase {
    pub fn new(deps: WorkSessionUseCaseDependencies) -> Self {
        Self { deps }
    }

    pub async fn execute(&self, claims: &UserClaims) -> Result<(), AppError> {
        info!("[EndSessionUseCase] Ending session");

        let user_id = claims_user_id(claims)?;
        authorize_non_root_for_policy(
            self.deps.user_repository.as_ref(),
            claims,
            &Policy::EndWorkSessions,
        )
        .await?;

        let session = self
            .deps
            .work_session_read_repository
            .get_user_active_session(user_id)
            .await
            .map_err(|_| AppError::NotFound("No active work session found".to_string()))?;

        let current_members = get_session_members_or_not_found(
            self.deps.work_session_read_repository.as_ref(),
            session.id,
        )
        .await?;

        ensure_creator_or_commander(
            session.created_by_user_id,
            user_id,
            &current_members,
            "Only the session creator or commander can end the session",
        )?;

        self.deps
            .work_session_write_repository
            .end_session(session.id)
            .await
            .map_err(|error| {
                error!("[EndSessionUseCase] Failed to end session: {:?}", error);
                AppError::InternalServerError
            })?;

        Ok(())
    }
}
