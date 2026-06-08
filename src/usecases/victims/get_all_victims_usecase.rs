use log::{error, info};

use crate::core::application_error::ApplicationError as AppError;
use crate::core::auth_context::AuthContext;
use crate::core::entities::auth::UserClaims;
use crate::core::pagination::PaginatedResult;
use crate::core::read_models::victims::VictimWithDetails;
use crate::core::value_objects::policies::Policy;
use crate::usecases::victims::deps::VictimUseCaseDependencies;
use crate::utils::pagination::Pagination;

pub struct GetAllVictimsUseCase {
    deps: VictimUseCaseDependencies,
}

impl GetAllVictimsUseCase {
    pub fn new(deps: VictimUseCaseDependencies) -> Self {
        Self { deps }
    }

    pub async fn execute(
        &self,
        pagination: Pagination,
        claims: &UserClaims,
    ) -> Result<PaginatedResult<VictimWithDetails>, AppError> {
        info!("[GetAllVictimsUseCase] Starting process to get victims");

        let auth = AuthContext::load(&*self.deps.user_repository, claims).await?;
        let allowed_cities = auth.allowed_cities(&Policy::ReadVictims);

        let total_items = self
            .deps
            .victim_read_repository
            .count_victims(allowed_cities.as_deref())
            .await
            .map_err(|e| {
                error!("[GetAllVictimsUseCase] Failed to count victims: {:?}", e);
                AppError::InternalServerError
            })?;

        let victims_list = self
            .deps
            .victim_read_repository
            .get_victims_paginated(
                allowed_cities.as_deref(),
                pagination.page_size,
                pagination.offset,
            )
            .await
            .map_err(|e| {
                error!("[GetAllVictimsUseCase] Failed to retrieve victims: {:?}", e);
                AppError::InternalServerError
            })?;

        info!(
            "[GetAllVictimsUseCase] Successfully retrieved {} victims (paged)",
            victims_list.len()
        );
        Ok(PaginatedResult {
            items: victims_list,
            page: pagination.page,
            page_size: pagination.page_size,
            total_items,
        })
    }
}
