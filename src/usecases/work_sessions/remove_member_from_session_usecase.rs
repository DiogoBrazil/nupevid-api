use log::{error, info};
use uuid::Uuid;

use crate::core::application_error::ApplicationError as AppError;
use crate::core::entities::auth::UserClaims;
use crate::core::value_objects::policies::Policy;
use crate::usecases::work_sessions::deps::WorkSessionUseCaseDependencies;
use crate::usecases::work_sessions::guards::ensure_creator_or_commander;
use crate::usecases::work_sessions::helpers::{
    authorize_non_root_for_policy, claims_user_id, get_session_by_id_base_or_not_found,
    get_session_members_or_not_found,
};
use crate::validators::work_session_validator::WorkSessionValidator;

pub struct RemoveMemberFromSessionUseCase {
    deps: WorkSessionUseCaseDependencies,
}

impl RemoveMemberFromSessionUseCase {
    pub fn new(deps: WorkSessionUseCaseDependencies) -> Self {
        Self { deps }
    }

    pub async fn execute(
        &self,
        session_id: Uuid,
        member_id: Uuid,
        claims: &UserClaims,
    ) -> Result<String, AppError> {
        info!(
            "[RemoveMemberFromSessionUseCase] Removing member from session: {}",
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
                "Cannot remove members from inactive session".to_string(),
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
            "Only the session creator or commander can remove members",
        )?;

        let members_with_functions: Vec<(Uuid, Option<_>)> = current_members
            .iter()
            .map(|member| (member.user_id, member.function.clone()))
            .collect();

        WorkSessionValidator::can_remove_member(&members_with_functions, member_id)
            .map_err(AppError::BadRequest)?;

        self.deps
            .work_session_write_repository
            .remove_member_from_session(session_id, member_id)
            .await
            .map(|_| "Member removed successfully".to_string())
            .map_err(|error| match error {
                crate::core::contracts::repository::error::RepositoryError::NotFound => {
                    AppError::NotFound("User is not a member of this session".to_string())
                }
                other => {
                    error!(
                        "[RemoveMemberFromSessionUseCase] Failed to remove member: {:?}",
                        other
                    );
                    AppError::InternalServerError
                }
            })
    }
}
