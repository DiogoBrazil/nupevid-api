use uuid::Uuid;

use crate::core::application_error::ApplicationError as AppError;
use crate::core::contracts::repository::users::UserRepository;
use crate::core::entities::auth::UserClaims;
use crate::core::value_objects::policies::Policy;

use crate::core::auth_helpers::get_user_policies_with_defaults;
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
        let policies = get_user_policies_with_defaults(user_repository, claims).await?;
        Ok(Self {
            claims: claims.clone(),
            policies,
        })
    }

    pub fn check_policy(&self, policy: &Policy, city_id: Uuid) -> Result<(), AppError> {
        check_policy(&self.claims, policy, city_id, &self.policies)
    }

    pub fn allowed_cities(&self, policy: &Policy) -> Option<Vec<Uuid>> {
        get_allowed_cities_for_policy(&self.claims, policy, &self.policies)
    }
}
