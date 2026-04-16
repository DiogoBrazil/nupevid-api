use log::{error, info};
use uuid::Uuid;

use crate::core::contracts::repository::error::RepositoryError;
use crate::core::entities::auth::UserClaims;
use crate::core::entities::work_session_members::{TeamMemberFunction, WorkSessionMember};
use crate::core::value_objects::policies::Policy;
use crate::core::value_objects::profiles::Profile;
use crate::core::application_error::ApplicationError as AppError;
use crate::usecases::work_sessions::guards::{
    ensure_creator_or_commander, validate_members_same_city,
};
use crate::usecases::work_sessions::deps::WorkSessionUseCaseDependencies;
use crate::usecases::work_sessions::helpers::{
    authorize_non_root_for_policy, claims_user_id, get_session_by_id_base_or_not_found,
    get_session_members_or_not_found,
};
use crate::validators::work_session_validator::WorkSessionValidator;

pub struct AddMemberToSessionUseCase {
    deps: WorkSessionUseCaseDependencies,
}

impl AddMemberToSessionUseCase {
    pub fn new(deps: WorkSessionUseCaseDependencies) -> Self {
        Self { deps }
    }

    pub async fn execute(
        &self,
        session_id: Uuid,
        user_id: Uuid,
        function: Option<TeamMemberFunction>,
        claims: &UserClaims,
    ) -> Result<WorkSessionMember, AppError> {
        info!(
            "[AddMemberToSessionUseCase] Adding member to session: {}",
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
                "Cannot add members to inactive session".to_string(),
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
            "Only the session creator or commander can add members",
        )?;

        if let Ok(true) = self
            .deps
            .work_session_read_repository
            .is_user_in_active_session(user_id)
            .await
        {
            return Err(AppError::Conflict(
                "User is already in an active session".to_string(),
            ));
        }

        let mut all_user_ids: Vec<Uuid> = current_members.iter().map(|member| member.user_id).collect();
        all_user_ids.push(user_id);
        validate_members_same_city(
            self.deps.user_repository.as_ref(),
            &all_user_ids,
            claims.profile == Profile::Root,
        )
        .await?;

        let members_with_functions: Vec<(Uuid, Option<TeamMemberFunction>)> = current_members
            .iter()
            .map(|member| (member.user_id, member.function.clone()))
            .collect();

        WorkSessionValidator::can_add_member_with_function(&members_with_functions, &function)
            .map_err(AppError::BadRequest)?;

        let session_member_registration_id = Uuid::new_v4();
        match self
            .deps
            .work_session_write_repository
            .add_member_to_session(session_member_registration_id, session_id, user_id, function)
            .await
        {
            Ok(member) => Ok(member),
            Err(RepositoryError::DuplicateEntry(msg)) => Err(AppError::Conflict(msg)),
            Err(error) => {
                error!(
                    "[AddMemberToSessionUseCase] Failed to add member to session: {:?}",
                    error
                );
                Err(AppError::InternalServerError)
            }
        }
    }
}
