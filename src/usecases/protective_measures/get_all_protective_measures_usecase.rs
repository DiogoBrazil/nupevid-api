use log::info;
use uuid::Uuid;

use crate::core::application_error::ApplicationError as AppError;
use crate::core::auth_context::AuthContext;
use crate::core::entities::auth::UserClaims;
use crate::core::entities::protective_measures::ProtectiveMeasure;
use crate::core::pagination::PaginatedResult;
use crate::core::value_objects::policies::Policy;
use crate::usecases::protective_measures::deps::ProtectiveMeasureUseCaseDependencies;
use crate::utils::pagination::Pagination;

pub struct GetAllProtectiveMeasuresUseCase {
    deps: ProtectiveMeasureUseCaseDependencies,
}

impl GetAllProtectiveMeasuresUseCase {
    pub fn new(deps: ProtectiveMeasureUseCaseDependencies) -> Self {
        Self { deps }
    }

    pub async fn execute(
        &self,
        pagination: Pagination,
        victim_id: Option<Uuid>,
        offender_id: Option<Uuid>,
        claims: &UserClaims,
    ) -> Result<PaginatedResult<ProtectiveMeasure>, AppError> {
        info!("[GetAllProtectiveMeasuresUseCase] Getting all protective measures");

        let auth = AuthContext::load(&*self.deps.user_repository, claims).await?;
        let allowed_cities = auth.allowed_cities(&Policy::ReadProtectiveMeasures);

        let total_items = self
            .deps
            .measure_read_repository
            .count_protective_measures(allowed_cities.as_deref(), victim_id, offender_id)
            .await
            .map_err(|_| AppError::InternalServerError)?;

        let measures = self
            .deps
            .measure_read_repository
            .get_protective_measures_paginated(
                allowed_cities.as_deref(),
                victim_id,
                offender_id,
                pagination.page_size,
                pagination.offset,
            )
            .await
            .map_err(|_| AppError::InternalServerError)?;

        info!(
            "[GetAllProtectiveMeasuresUseCase] Successfully retrieved {} measures",
            measures.len()
        );
        Ok(PaginatedResult {
            items: measures,
            page: pagination.page,
            page_size: pagination.page_size,
            total_items,
        })
    }
}
