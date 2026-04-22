use log::{error, info};
use uuid::Uuid;

use crate::core::application_error::ApplicationError as AppError;
use crate::core::auth_context::AuthContext;
use crate::core::entities::auth::UserClaims;
use crate::core::read_models::offenders::OffenderWithDetails;
use crate::core::value_objects::policies::Policy;
use crate::usecases::offenders::deps::OffenderUseCaseDependencies;

pub struct GetOffendersByVictimUseCase {
    deps: OffenderUseCaseDependencies,
}

impl GetOffendersByVictimUseCase {
    pub fn new(deps: OffenderUseCaseDependencies) -> Self {
        Self { deps }
    }

    pub async fn execute(
        &self,
        victim_id: Uuid,
        claims: &UserClaims,
    ) -> Result<Vec<OffenderWithDetails>, AppError> {
        info!(
            "[GetOffendersByVictimUseCase] Starting process to get offenders for victim: {}",
            victim_id
        );

        let auth = AuthContext::load(&*self.deps.user_repository, claims).await?;

        let offenders = if let Some(allowed_cities) = auth.allowed_cities(&Policy::ReadOffenders) {
            match self
                .deps
                .offender_read_repository
                .get_offenders_by_victim_id(victim_id)
                .await
            {
                Ok(all) => {
                    let filtered: Vec<_> = all
                        .into_iter()
                        .filter(|o| allowed_cities.contains(&o.city_id))
                        .collect();
                    Ok(filtered)
                }
                Err(e) => Err(e),
            }
        } else {
            self.deps
                .offender_read_repository
                .get_offenders_by_victim_id(victim_id)
                .await
        };

        match offenders {
            Ok(offenders_list) => {
                info!(
                    "[GetOffendersByVictimUseCase] Successfully retrieved {} offenders for victim: {}",
                    offenders_list.len(),
                    victim_id
                );
                Ok(offenders_list)
            }
            Err(e) => {
                error!(
                    "[GetOffendersByVictimUseCase] Failed to retrieve offenders for victim: {:?}",
                    e
                );
                Err(AppError::InternalServerError)
            }
        }
    }
}
