use uuid::Uuid;

use crate::core::application_error::ApplicationError as AppError;
use crate::core::auth_context::AuthContext;
use crate::core::contracts::repository::users::UserRepository;
use crate::core::contracts::repository::victims::VictimReadRepository;
use crate::core::entities::auth::UserClaims;
use crate::core::read_models::victims::VictimWithDetails;
use crate::core::value_objects::policies::Policy;
use crate::usecases::helpers_common::get_victim_or_not_found;

pub async fn authorize_victim_access(
    auth: &AuthContext,
    victim_read_repository: &dyn VictimReadRepository,
    victim_id: Uuid,
    policy: &Policy,
) -> Result<VictimWithDetails, AppError> {
    let victim = get_victim_or_not_found(victim_read_repository, victim_id).await?;
    auth.check_policy(policy, victim.summary.city_id)?;
    Ok(victim)
}

pub async fn load_auth_and_check_victim(
    user_repository: &dyn UserRepository,
    claims: &UserClaims,
    victim_read_repository: &dyn VictimReadRepository,
    victim_id: Uuid,
    policy: &Policy,
) -> Result<VictimWithDetails, AppError> {
    let auth = AuthContext::load(user_repository, claims).await?;
    authorize_victim_access(&auth, victim_read_repository, victim_id, policy).await
}
