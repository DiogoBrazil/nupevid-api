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
use crate::usecases::work_sessions::guards::{
    ensure_creator_or_commander, validate_members_same_city,
};
use crate::usecases::work_sessions::deps::WorkSessionUseCaseDependencies;
use crate::usecases::work_sessions::helpers::{
    authorize_non_root_for_policy, claims_user_id,
    get_session_by_id_base_or_not_found, get_session_members_or_not_found,
};
use crate::validators::work_session_validator::WorkSessionValidator;

pub struct UpdateWorkSessionUseCase {
    deps: WorkSessionUseCaseDependencies,
}

impl UpdateWorkSessionUseCase {
    pub fn new(deps: WorkSessionUseCaseDependencies) -> Self {
        Self { deps }
    }

    pub async fn execute(
        &self,
        session_id: Uuid,
        data: CreateWorkSession,
        claims: &UserClaims,
    ) -> Result<WorkSession, AppError> {
        info!(
            "[UpdateWorkSessionUseCase] Updating work session: {}",
            session_id
        );

        let requesting_user_id = claims_user_id(claims)?;
        authorize_non_root_for_policy(
            self.deps.user_repository.as_ref(),
            claims,
            &Policy::UpdateWorkSessions,
        )
        .await?;

        let session = get_session_by_id_base_or_not_found(
            self.deps.work_session_read_repository.as_ref(),
            session_id,
        )
        .await?;

        if !session.is_active {
            return Err(AppError::BadRequest(
                "Cannot update inactive session".to_string(),
            ));
        }

        let current_members = get_session_members_or_not_found(
            self.deps.work_session_read_repository.as_ref(),
            session_id,
        )
        .await?;

        ensure_creator_or_commander(
            session.created_by_user_id,
            requesting_user_id,
            &current_members,
            "Only the session creator or commander can update the session",
        )?;

        if !data
            .members
            .iter()
            .any(|member| member.user_id == requesting_user_id)
        {
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

        match self
            .deps
            .work_session_write_repository
            .update_work_session_with_members(session_id, data.description, data.members)
            .await
        {
            Ok(updated_session) => Ok(updated_session),
            Err(RepositoryError::Conflict(msg)) => Err(AppError::Conflict(msg)),
            Err(RepositoryError::DuplicateEntry(msg)) => Err(AppError::Conflict(msg)),
            Err(error) => {
                error!(
                    "[UpdateWorkSessionUseCase] Failed to update work session: {:?}",
                    error
                );
                Err(AppError::InternalServerError)
            }
        }
    }
}
