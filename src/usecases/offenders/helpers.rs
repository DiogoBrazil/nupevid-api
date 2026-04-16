use uuid::Uuid;

use crate::core::contracts::repository::error::RepositoryError;
use crate::core::contracts::repository::offenders::OffenderReadRepository;
use crate::core::entities::auth::UserClaims;
use crate::core::read_models::offenders::OffenderWithDetails;
use crate::core::value_objects::policies::Policy;
use crate::core::application_error::ApplicationError as AppError;
use crate::core::auth_context::AuthContext;

pub async fn get_offender_or_not_found(
    offender_read_repository: &dyn OffenderReadRepository,
    id: Uuid,
) -> Result<OffenderWithDetails, AppError> {
    match offender_read_repository.get_offender_by_id(id).await {
        Ok(offender) => Ok(offender),
        Err(RepositoryError::NotFound) => Err(AppError::NotFound(format!(
            "Offender with id '{}' not found",
            id
        ))),
        Err(_) => Err(AppError::InternalServerError),
    }
}

pub async fn authorize_offender_access(
    auth: &AuthContext,
    offender_read_repository: &dyn OffenderReadRepository,
    offender_id: Uuid,
    policy: &Policy,
) -> Result<OffenderWithDetails, AppError> {
    let offender = get_offender_or_not_found(offender_read_repository, offender_id).await?;
    auth.check_policy(policy, offender.city_id)?;
    Ok(offender)
}

pub async fn load_auth_and_check_offender(
    user_repository: &dyn crate::core::contracts::repository::users::UserRepository,
    claims: &UserClaims,
    offender_read_repository: &dyn OffenderReadRepository,
    offender_id: Uuid,
    policy: &Policy,
) -> Result<OffenderWithDetails, AppError> {
    let auth = AuthContext::load(user_repository, claims).await?;
    authorize_offender_access(&auth, offender_read_repository, offender_id, policy).await
}
