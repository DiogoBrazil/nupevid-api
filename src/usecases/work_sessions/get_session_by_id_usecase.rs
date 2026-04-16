use log::info;
use uuid::Uuid;

use crate::core::entities::auth::UserClaims;
use crate::core::entities::work_sessions::WorkSession;
use crate::core::value_objects::policies::Policy;
use crate::core::value_objects::profiles::Profile;
use crate::core::application_error::ApplicationError as AppError;
use crate::core::auth_context::AuthContext;
use crate::core::auth_helpers::extract_city_id_from_claims;
use crate::usecases::work_sessions::deps::WorkSessionUseCaseDependencies;
use crate::usecases::work_sessions::helpers::{
    claims_user_id, get_session_by_id_base_or_not_found,
};

pub struct GetSessionByIdUseCase {
    deps: WorkSessionUseCaseDependencies,
}

impl GetSessionByIdUseCase {
    pub fn new(deps: WorkSessionUseCaseDependencies) -> Self {
        Self { deps }
    }

    pub async fn execute(
        &self,
        session_id: Uuid,
        claims: &UserClaims,
    ) -> Result<WorkSession, AppError> {
        info!(
            "[GetSessionByIdUseCase] Getting session by id: {}",
            session_id
        );

        let auth = AuthContext::load(self.deps.user_repository.as_ref(), claims).await?;
        let user_id = claims_user_id(claims)?;
        let session = get_session_by_id_base_or_not_found(
            self.deps.work_session_read_repository.as_ref(),
            session_id,
        )
        .await?;

        if session.created_by_user_id != user_id && claims.profile != Profile::Root {
            let user_city_id = extract_city_id_from_claims(claims)?;
            auth.check_policy(&Policy::ViewOtherWorkSessions, user_city_id)?;
        }

        Ok(session)
    }
}
