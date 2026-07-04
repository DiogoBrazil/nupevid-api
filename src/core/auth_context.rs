use uuid::Uuid;

use crate::core::application_error::ApplicationError as AppError;
use crate::core::contracts::repository::users::UserRepository;
use crate::core::entities::auth::UserClaims;
use crate::core::value_objects::policies::Policy;

use crate::core::auth_helpers::get_user_policies;
use crate::core::authorization::{check_policy, get_allowed_cities_for_policy};
use crate::core::value_objects::policies::PermissionPolicies;

pub struct AuthContext {
    pub claims: UserClaims,
    pub policies: PermissionPolicies,
}

impl AuthContext {
    pub async fn load(
        user_repository: &dyn UserRepository,
        claims: &UserClaims,
    ) -> Result<Self, AppError> {
        let policies = get_user_policies(user_repository, claims).await?;
        Ok(Self {
            claims: claims.clone(),
            policies,
        })
    }

    pub fn check_policy(&self, policy: &Policy, city_id: Uuid) -> Result<(), AppError> {
        check_policy(&self.claims, policy, city_id, &self.policies)
    }

    /// Like `check_policy`, but converts an authorization failure into the
    /// provided "not found" error. Used on by-ID access so callers outside the
    /// resource's city scope cannot learn that the resource exists (the
    /// response is identical to a genuinely missing resource).
    pub fn check_policy_or_not_found(
        &self,
        policy: &Policy,
        city_id: Uuid,
        not_found: impl FnOnce() -> AppError,
    ) -> Result<(), AppError> {
        self.check_policy(policy, city_id).map_err(|err| match err {
            AppError::Forbidden(_) => not_found(),
            other => other,
        })
    }

    pub fn allowed_cities(&self, policy: &Policy) -> Option<Vec<Uuid>> {
        get_allowed_cities_for_policy(&self.claims, policy, &self.policies)
    }
}
