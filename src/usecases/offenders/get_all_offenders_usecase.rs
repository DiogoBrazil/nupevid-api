use log::{error, info};

use crate::core::application_error::ApplicationError as AppError;
use crate::core::auth_context::AuthContext;
use crate::core::entities::auth::UserClaims;
use crate::core::pagination::PaginatedResult;
use crate::core::read_models::offenders::OffenderWithDetails;
use crate::core::value_objects::policies::Policy;
use crate::usecases::offenders::deps::OffenderUseCaseDependencies;
use crate::utils::pagination::Pagination;

pub struct GetAllOffendersUseCase {
    deps: OffenderUseCaseDependencies,
}

impl GetAllOffendersUseCase {
    pub fn new(deps: OffenderUseCaseDependencies) -> Self {
        Self { deps }
    }

    pub async fn execute(
        &self,
        pagination: Pagination,
        claims: &UserClaims,
    ) -> Result<PaginatedResult<OffenderWithDetails>, AppError> {
        info!("[GetAllOffendersUseCase] Starting process to get offenders");

        let auth = AuthContext::load(&*self.deps.user_repository, claims).await?;
        let allowed_cities = auth.allowed_cities(&Policy::ReadOffenders);

        let total_items = self
            .deps
            .offender_read_repository
            .count_offenders(allowed_cities.as_deref())
            .await
            .map_err(|e| {
                error!(
                    "[GetAllOffendersUseCase] Failed to count offenders: {:?}",
                    e
                );
                AppError::InternalServerError
            })?;

        let offenders_list = self
            .deps
            .offender_read_repository
            .get_offenders_paginated(
                allowed_cities.as_deref(),
                pagination.page_size,
                pagination.offset,
            )
            .await
            .map_err(|e| {
                error!(
                    "[GetAllOffendersUseCase] Failed to retrieve offenders: {:?}",
                    e
                );
                AppError::InternalServerError
            })?;

        info!(
            "[GetAllOffendersUseCase] Successfully retrieved {} offenders (paged)",
            offenders_list.len()
        );
        Ok(PaginatedResult {
            items: offenders_list,
            page: pagination.page,
            page_size: pagination.page_size,
            total_items,
        })
    }
}
