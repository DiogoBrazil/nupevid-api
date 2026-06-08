use log::{error, info};

use crate::core::application_error::ApplicationError as AppError;
use crate::core::auth_context::AuthContext;
use crate::core::entities::auth::UserClaims;
use crate::core::read_models::offenders::OffenderWithDetails;
use crate::core::value_objects::policies::Policy;
use crate::core::value_objects::search::SearchCriteria;
use crate::usecases::offenders::deps::OffenderUseCaseDependencies;

pub struct SearchOffendersUseCase {
    deps: OffenderUseCaseDependencies,
}

impl SearchOffendersUseCase {
    pub fn new(deps: OffenderUseCaseDependencies) -> Self {
        Self { deps }
    }

    pub async fn execute(
        &self,
        name: Option<String>,
        cpf: Option<String>,
        claims: &UserClaims,
    ) -> Result<Vec<OffenderWithDetails>, AppError> {
        info!("[SearchOffendersUseCase] Starting offender search");

        let search = SearchCriteria::parse(name, cpf)
            .map_err(|e| AppError::BadRequest(format!("Error searching offenders: {}", e)))?;

        let auth = AuthContext::load(&*self.deps.user_repository, claims).await?;

        let offenders = match search {
            SearchCriteria::ByName(name) => {
                self.deps
                    .offender_read_repository
                    .get_offenders_by_name(&name)
                    .await
            }
            SearchCriteria::ByCpf(cpf) => {
                self.deps
                    .offender_read_repository
                    .get_offenders_by_cpf(&cpf)
                    .await
            }
        };

        let offenders = if let Some(allowed_cities) = auth.allowed_cities(&Policy::ReadOffenders) {
            match offenders {
                Ok(list) => {
                    let filtered: Vec<_> = list
                        .into_iter()
                        .filter(|o| allowed_cities.contains(&o.summary.city_id))
                        .collect();
                    Ok(filtered)
                }
                Err(e) => Err(e),
            }
        } else {
            offenders
        };

        match offenders {
            Ok(offenders_list) => {
                info!(
                    "[SearchOffendersUseCase] Successfully retrieved {} offenders from search",
                    offenders_list.len()
                );
                Ok(offenders_list)
            }
            Err(e) => {
                error!(
                    "[SearchOffendersUseCase] Failed to search offenders: {:?}",
                    e
                );
                Err(AppError::InternalServerError)
            }
        }
    }
}
