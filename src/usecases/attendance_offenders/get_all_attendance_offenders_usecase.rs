use log::info;

use crate::core::entities::auth::UserClaims;
use crate::core::pagination::PaginatedResult;
use crate::core::read_models::attendance_offenders::AttendanceOffenderWithAddress;
use crate::core::value_objects::policies::Policy;
use crate::core::application_error::ApplicationError as AppError;
use crate::core::auth_context::AuthContext;
use crate::usecases::attendance_offenders::deps::AttendanceOffenderUseCaseDependencies;
use crate::utils::pagination::Pagination;

pub struct GetAllAttendanceOffendersUseCase {
    deps: AttendanceOffenderUseCaseDependencies,
}

impl GetAllAttendanceOffendersUseCase {
    pub fn new(deps: AttendanceOffenderUseCaseDependencies) -> Self {
        Self { deps }
    }

    pub async fn execute(
        &self,
        pagination: Pagination,
        claims: &UserClaims,
    ) -> Result<PaginatedResult<AttendanceOffenderWithAddress>, AppError> {
        info!("[GetAllAttendanceOffendersUseCase] Getting all attendance offenders");

        let auth = AuthContext::load(&*self.deps.user_repository, claims).await?;
        let allowed_cities = auth.allowed_cities(&Policy::ReadAttendances);

        let total_items = self
            .deps
            .attendance_offender_read_repository
            .count_attendance_offenders(allowed_cities.as_deref())
            .await
            .map_err(|_| AppError::InternalServerError)?;

        let attendances_list = self
            .deps
            .attendance_offender_read_repository
            .get_attendance_offenders_paginated(
                allowed_cities.as_deref(),
                pagination.page_size,
                pagination.offset,
            )
            .await
            .map_err(|_| AppError::InternalServerError)?;

        Ok(PaginatedResult {
            items: attendances_list,
            page: pagination.page,
            page_size: pagination.page_size,
            total_items,
        })
    }
}
