use log::error;
use uuid::Uuid;

use crate::core::application_error::ApplicationError as AppError;
use crate::core::auth_context::AuthContext;
use crate::core::contracts::repository::error::RepositoryError;
use crate::core::contracts::repository::protective_measures::ProtectiveMeasureReadRepository;
use crate::core::contracts::repository::users::UserRepository;
use crate::core::contracts::repository::victims::VictimReadRepository;
use crate::core::entities::auth::UserClaims;
use crate::core::entities::protective_measures::ProtectiveMeasureExtension;
use crate::core::value_objects::policies::Policy;
use crate::usecases::helpers_common::get_extension_or_not_found;
use crate::usecases::extensions::deps::ExtensionUseCaseDependencies;

pub async fn authorize_extension_access(
    protective_measure_repository: &dyn ProtectiveMeasureReadRepository,
    victim_repository: &dyn VictimReadRepository,
    user_repository: &dyn UserRepository,
    protective_measure_id: Uuid,
    policy: &Policy,
    claims: &UserClaims,
    label: &str,
) -> Result<(), AppError> {
    let protective_measure = match protective_measure_repository
        .get_protective_measure_by_id(protective_measure_id)
        .await
    {
        Ok(pm) => pm,
        Err(RepositoryError::NotFound) => {
            error!(
                "[{}] Protective measure not found: {}",
                label, protective_measure_id
            );
            return Err(AppError::NotFound(
                "Protective measure not found".to_string(),
            ));
        }
        Err(e) => {
            error!(
                "[{}] Database error checking protective measure: {:?}",
                label, e
            );
            return Err(AppError::InternalServerError);
        }
    };

    let victim = match victim_repository
        .get_victim_by_id(protective_measure.victim_id)
        .await
    {
        Ok(v) => v,
        Err(RepositoryError::NotFound) => {
            error!(
                "[{}] Victim not found: {}",
                label, protective_measure.victim_id
            );
            return Err(AppError::NotFound(format!(
                "Victim with id '{}' not found",
                protective_measure.victim_id
            )));
        }
        Err(e) => {
            error!("[{}] Error checking victim: {:?}", label, e);
            return Err(AppError::InternalServerError);
        }
    };

    let auth = AuthContext::load(user_repository, claims).await?;
    auth.check_policy(policy, victim.city_id)?;

    Ok(())
}

pub async fn load_auth_and_check_extension(
    deps: &ExtensionUseCaseDependencies,
    extension_id: Uuid,
    policy: &Policy,
    claims: &UserClaims,
    label: &str,
) -> Result<ProtectiveMeasureExtension, AppError> {
    let extension = get_extension_or_not_found(&*deps.extension_repository, extension_id).await?;

    let protective_measure = deps
        .protective_measure_repository
        .get_protective_measure_by_id(extension.protective_measure_id)
        .await
        .map_err(|e| {
            error!("[{}] Error fetching protective measure: {:?}", label, e);
            AppError::InternalServerError
        })?;

    let victim = deps
        .victim_repository
        .get_victim_by_id(protective_measure.victim_id)
        .await
        .map_err(|e| {
            error!("[{}] Error fetching victim: {:?}", label, e);
            AppError::InternalServerError
        })?;

    let auth = AuthContext::load(&*deps.user_repository, claims).await?;
    auth.check_policy(policy, victim.city_id)?;

    Ok(extension)
}
