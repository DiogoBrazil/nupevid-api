use log::info;

use crate::core::entities::auth::UserClaims;
use crate::core::pagination::PaginatedResult;
use crate::core::read_models::attendance_victims::AttendanceVictimWithAddress;
use crate::core::value_objects::policies::Policy;
use crate::core::application_error::ApplicationError as AppError;
use crate::core::auth_context::AuthContext;
use crate::usecases::attendance_victims::deps::AttendanceVictimUseCaseDependencies;
use crate::utils::pagination::Pagination;

pub struct GetAllAttendanceVictimsUseCase {
    deps: AttendanceVictimUseCaseDependencies,
}

impl GetAllAttendanceVictimsUseCase {
    pub fn new(deps: AttendanceVictimUseCaseDependencies) -> Self {
        Self { deps }
    }

    pub async fn execute(
        &self,
        pagination: Pagination,
        claims: &UserClaims,
    ) -> Result<PaginatedResult<AttendanceVictimWithAddress>, AppError> {
        info!("[GetAllAttendanceVictimsUseCase] Getting all attendance victims");

        let auth = AuthContext::load(&*self.deps.user_repository, claims).await?;
        let allowed_cities = auth.allowed_cities(&Policy::ReadAttendances);

        let total_items = self
            .deps
            .attendance_victim_read_repository
            .count_attendance_victims(allowed_cities.as_deref())
            .await
            .map_err(|_| AppError::InternalServerError)?;

        let attendances_list = self
            .deps
            .attendance_victim_read_repository
            .get_attendance_victims_paginated(
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
