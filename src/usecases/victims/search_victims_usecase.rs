use log::{error, info};

use crate::core::application_error::ApplicationError as AppError;
use crate::core::auth_context::AuthContext;
use crate::core::entities::auth::UserClaims;
use crate::core::read_models::victims::VictimWithDetails;
use crate::core::value_objects::policies::Policy;
use crate::core::value_objects::search::SearchCriteria;
use crate::usecases::victims::deps::VictimUseCaseDependencies;

pub struct SearchVictimsUseCase {
    deps: VictimUseCaseDependencies,
}

impl SearchVictimsUseCase {
    pub fn new(deps: VictimUseCaseDependencies) -> Self {
        Self { deps }
    }

    pub async fn execute(
        &self,
        name: Option<String>,
        cpf: Option<String>,
        claims: &UserClaims,
    ) -> Result<Vec<VictimWithDetails>, AppError> {
        info!("[SearchVictimsUseCase] Starting victim search");

        let search = SearchCriteria::parse(name, cpf)
            .map_err(|e| AppError::BadRequest(format!("Error searching victims: {}", e)))?;

        let auth = AuthContext::load(&*self.deps.user_repository, claims).await?;

        let victims = match search {
            SearchCriteria::ByName(name) => {
                self.deps
                    .victim_read_repository
                    .get_victims_by_name(&name)
                    .await
            }
            SearchCriteria::ByCpf(cpf) => {
                self.deps
                    .victim_read_repository
                    .get_victims_by_cpf(&cpf)
                    .await
            }
        };

        let victims = if let Some(allowed_cities) = auth.allowed_cities(&Policy::ReadVictims) {
            match victims {
                Ok(list) => {
                    let filtered: Vec<_> = list
                        .into_iter()
                        .filter(|v| allowed_cities.contains(&v.city_id))
                        .collect();
                    Ok(filtered)
                }
                Err(e) => Err(e),
            }
        } else {
            victims
        };

        match victims {
            Ok(victims_list) => {
                info!(
                    "[SearchVictimsUseCase] Successfully retrieved {} victims from search",
                    victims_list.len()
                );
                Ok(victims_list)
            }
            Err(e) => {
                error!("[SearchVictimsUseCase] Failed to search victims: {:?}", e);
                Err(AppError::InternalServerError)
            }
        }
    }
}
