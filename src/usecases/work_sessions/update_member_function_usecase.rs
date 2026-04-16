use log::{error, info};
use uuid::Uuid;

use crate::core::application_error::ApplicationError as AppError;
use crate::core::entities::auth::UserClaims;
use crate::core::entities::work_session_members::TeamMemberFunction;
use crate::core::value_objects::policies::Policy;
use crate::usecases::work_sessions::deps::WorkSessionUseCaseDependencies;
use crate::usecases::work_sessions::guards::ensure_creator_or_commander;
use crate::usecases::work_sessions::helpers::{
    authorize_non_root_for_policy, claims_user_id, get_session_by_id_base_or_not_found,
    get_session_members_or_not_found,
};
use crate::validators::work_session_validator::WorkSessionValidator;

pub struct UpdateMemberFunctionUseCase {
    deps: WorkSessionUseCaseDependencies,
}

impl UpdateMemberFunctionUseCase {
    pub fn new(deps: WorkSessionUseCaseDependencies) -> Self {
        Self { deps }
    }

    pub async fn execute(
        &self,
        session_id: Uuid,
        user_id: Uuid,
        new_function: Option<TeamMemberFunction>,
        claims: &UserClaims,
    ) -> Result<String, AppError> {
        info!(
            "[UpdateMemberFunctionUseCase] Updating member function for session: {}",
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
                "Cannot update members of inactive session".to_string(),
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
            "Only the session creator or commander can update member functions",
        )?;

        if !current_members
            .iter()
            .any(|member| member.user_id == user_id)
        {
            return Err(AppError::NotFound(
                "User is not a member of this session".to_string(),
            ));
        }

        let members_with_functions: Vec<(Uuid, Option<TeamMemberFunction>)> = current_members
            .iter()
            .map(|member| {
                if member.user_id == user_id {
                    (member.user_id, new_function.clone())
                } else {
                    (member.user_id, member.function.clone())
                }
            })
            .collect();

        WorkSessionValidator::validate_team_functions(&members_with_functions)
            .map_err(AppError::BadRequest)?;

        self.deps
            .work_session_write_repository
            .update_member_function(session_id, user_id, new_function)
            .await
            .map(|_| "Member function updated successfully".to_string())
            .map_err(|error| {
                error!(
                    "[UpdateMemberFunctionUseCase] Failed to update member function: {:?}",
                    error
                );
                AppError::InternalServerError
            })
    }
}
