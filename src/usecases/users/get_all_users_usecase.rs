use log::info;

use crate::core::application_error::ApplicationError as AppError;
use crate::core::entities::auth::UserClaims;
use crate::core::entities::users::User;
use crate::core::pagination::PaginatedResult;
use crate::usecases::users::deps::UserUseCaseDependencies;
use crate::usecases::users::helpers::build_user_read_scope;
use crate::utils::pagination::Pagination;

pub struct GetAllUsersUseCase {
    deps: UserUseCaseDependencies,
}

impl GetAllUsersUseCase {
    pub fn new(deps: UserUseCaseDependencies) -> Self {
        Self { deps }
    }

    pub async fn execute(
        &self,
        pagination: Pagination,
        claims: &UserClaims,
    ) -> Result<PaginatedResult<User>, AppError> {
        let scope = build_user_read_scope(self.deps.user_repository.as_ref(), claims).await?;

        let total_items = self
            .deps
            .user_repository
            .count_users(scope.allowed_cities.as_deref(), scope.exclude_root)
            .await
            .map_err(|_| AppError::InternalServerError)?;

        let users = self
            .deps
            .user_repository
            .get_users_paginated(
                scope.allowed_cities.as_deref(),
                scope.exclude_root,
                pagination.page_size,
                pagination.offset,
            )
            .await
            .map_err(|_| AppError::InternalServerError)?;

        info!(
            "[GetAllUsersUseCase] Successfully retrieved {} users",
            users.len()
        );

        Ok(PaginatedResult {
            items: users,
            page: pagination.page,
            page_size: pagination.page_size,
            total_items,
        })
    }
}
