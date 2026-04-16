use log::{error, info};
use uuid::Uuid;

use crate::core::commands::work_sessions::CreateWorkSession;
use crate::core::contracts::repository::error::RepositoryError;
use crate::core::entities::auth::UserClaims;
use crate::core::entities::work_session_members::TeamMemberFunction;
use crate::core::entities::work_sessions::WorkSession;
use crate::core::value_objects::policies::Policy;
use crate::core::value_objects::profiles::Profile;
use crate::core::application_error::ApplicationError as AppError;
use crate::usecases::work_sessions::guards::validate_members_same_city;
use crate::usecases::work_sessions::deps::WorkSessionUseCaseDependencies;
use crate::usecases::work_sessions::helpers::{
    authorize_non_root_for_policy, claims_user_id,
};
use crate::validators::work_session_validator::WorkSessionValidator;

pub struct CreateWorkSessionUseCase {
    deps: WorkSessionUseCaseDependencies,
}

impl CreateWorkSessionUseCase {
    pub fn new(deps: WorkSessionUseCaseDependencies) -> Self {
        Self { deps }
    }

    pub async fn execute(
        &self,
        data: CreateWorkSession,
        claims: &UserClaims,
    ) -> Result<WorkSession, AppError> {
        info!("[CreateWorkSessionUseCase] Starting work session creation");

        let user_id = claims_user_id(claims)?;
        authorize_non_root_for_policy(
            self.deps.user_repository.as_ref(),
            claims,
            &Policy::CreateWorkSessions,
        )
        .await?;

        if !data.members.iter().any(|member| member.user_id == user_id) {
            return Err(AppError::BadRequest(
                "Requesting user must be included in session members".to_string(),
            ));
        }

        let members_with_functions: Vec<(Uuid, Option<TeamMemberFunction>)> = data
            .members
            .iter()
            .map(|member| (member.user_id, member.function.clone()))
            .collect();

        WorkSessionValidator::validate_team_functions(&members_with_functions)
            .map_err(AppError::BadRequest)?;

        validate_members_same_city(
            self.deps.user_repository.as_ref(),
            &data.members.iter().map(|member| member.user_id).collect::<Vec<_>>(),
            claims.profile == Profile::Root,
        )
        .await?;

        if let Ok(true) = self
            .deps
            .work_session_read_repository
            .is_user_in_active_session(user_id)
            .await
        {
            return Err(AppError::Conflict(
                "User already has an active work session".to_string(),
            ));
        }

        match self
            .deps
            .work_session_write_repository
            .create_work_session(data, user_id)
            .await
        {
            Ok(session) => Ok(session),
            Err(RepositoryError::Conflict(msg)) => Err(AppError::Conflict(msg)),
            Err(RepositoryError::DuplicateEntry(msg)) => Err(AppError::Conflict(msg)),
            Err(error) => {
                error!(
                    "[CreateWorkSessionUseCase] Failed to create work session: {:?}",
                    error
                );
                Err(AppError::InternalServerError)
            }
        }
    }
}
