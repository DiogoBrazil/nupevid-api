use log::info;

use crate::core::application_error::ApplicationError as AppError;
use crate::core::contracts::repository::error::RepositoryError;
use crate::core::entities::auth::UserClaims;
use crate::core::entities::work_sessions::WorkSession;
use crate::usecases::work_sessions::deps::WorkSessionUseCaseDependencies;
use crate::usecases::work_sessions::helpers::claims_user_id;

pub struct GetActiveSessionUseCase {
    deps: WorkSessionUseCaseDependencies,
}

impl GetActiveSessionUseCase {
    pub fn new(deps: WorkSessionUseCaseDependencies) -> Self {
        Self { deps }
    }

    pub async fn execute(&self, claims: &UserClaims) -> Result<WorkSession, AppError> {
        info!("[GetActiveSessionUseCase] Getting active session");

        let user_id = claims_user_id(claims)?;
        match self
            .deps
            .work_session_read_repository
            .get_active_session_by_user(user_id)
            .await
        {
            Ok(session) => Ok(session),
            Err(RepositoryError::NotFound) => Err(AppError::NotFound(
                "No active work session found".to_string(),
            )),
            Err(_) => Err(AppError::InternalServerError),
        }
    }
}
