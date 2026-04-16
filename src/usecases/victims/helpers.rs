use uuid::Uuid;

use crate::core::contracts::repository::error::RepositoryError;
use crate::core::contracts::repository::victims::VictimReadRepository;
use crate::core::entities::auth::UserClaims;
use crate::core::read_models::victims::VictimWithDetails;
use crate::core::value_objects::policies::Policy;
use crate::core::application_error::ApplicationError as AppError;
use crate::core::auth_context::AuthContext;

pub async fn get_victim_or_not_found(
    victim_read_repository: &dyn VictimReadRepository,
    id: Uuid,
) -> Result<VictimWithDetails, AppError> {
    match victim_read_repository.get_victim_by_id(id).await {
        Ok(victim) => Ok(victim),
        Err(RepositoryError::NotFound) => Err(AppError::NotFound(format!(
            "Victim with id '{}' not found",
            id
        ))),
        Err(_) => Err(AppError::InternalServerError),
    }
}

pub async fn authorize_victim_access(
    auth: &AuthContext,
    victim_read_repository: &dyn VictimReadRepository,
    victim_id: Uuid,
    policy: &Policy,
) -> Result<VictimWithDetails, AppError> {
    let victim = get_victim_or_not_found(victim_read_repository, victim_id).await?;
    auth.check_policy(policy, victim.city_id)?;
    Ok(victim)
}

pub async fn load_auth_and_check_victim(
    user_repository: &dyn crate::core::contracts::repository::users::UserRepository,
    claims: &UserClaims,
    victim_read_repository: &dyn VictimReadRepository,
    victim_id: Uuid,
    policy: &Policy,
) -> Result<VictimWithDetails, AppError> {
    let auth = AuthContext::load(user_repository, claims).await?;
    authorize_victim_access(&auth, victim_read_repository, victim_id, policy).await
}
