use uuid::Uuid;

use crate::core::contracts::repository::error::RepositoryError;
use crate::core::contracts::repository::work_sessions::WorkSessionReadRepository;
use crate::core::entities::auth::UserClaims;
use crate::core::entities::work_session_members::WorkSessionMember;
use crate::core::entities::work_sessions::WorkSession;
use crate::core::application_error::ApplicationError as AppError;
use crate::core::auth_context::AuthContext;
use crate::core::auth_helpers::extract_city_id_from_claims;
use crate::core::contracts::repository::users::UserRepository;
use crate::core::value_objects::policies::Policy;
use crate::core::value_objects::profiles::Profile;

pub fn claims_user_id(claims: &UserClaims) -> Result<Uuid, AppError> {
    Uuid::parse_str(&claims.id)
        .map_err(|_| AppError::Unauthorized("Invalid user id in token".to_string()))
}

pub async fn authorize_non_root_for_policy(
    user_repository: &dyn UserRepository,
    claims: &UserClaims,
    policy: &Policy,
) -> Result<(), AppError> {
    if claims.profile == Profile::Root {
        return Ok(());
    }

    let auth = AuthContext::load(user_repository, claims).await?;
    let user_city_id = extract_city_id_from_claims(claims)?;
    auth.check_policy(policy, user_city_id)
}

pub async fn get_session_by_id_base_or_not_found(
    read_repository: &dyn WorkSessionReadRepository,
    session_id: Uuid,
) -> Result<WorkSession, AppError> {
    read_repository
        .get_session_by_id_base(session_id)
        .await
        .map_err(|error| match error {
            RepositoryError::NotFound => AppError::NotFound("Work session not found".to_string()),
            _ => AppError::InternalServerError,
        })
}

pub async fn get_session_members_or_not_found(
    read_repository: &dyn WorkSessionReadRepository,
    session_id: Uuid,
) -> Result<Vec<WorkSessionMember>, AppError> {
    read_repository
        .get_session_members(session_id)
        .await
        .map_err(|error| match error {
            RepositoryError::NotFound => AppError::NotFound("Session members not found".to_string()),
            _ => AppError::InternalServerError,
        })
}

