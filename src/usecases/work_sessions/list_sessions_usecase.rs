use log::info;
use uuid::Uuid;

use crate::core::entities::auth::UserClaims;
use crate::core::entities::work_sessions::WorkSession;
use crate::core::pagination::PaginatedResult;
use crate::core::filters::work_sessions::ListWorkSessionsQuery;
use crate::core::value_objects::policies::Policy;
use crate::core::value_objects::profiles::Profile;
use crate::core::application_error::ApplicationError as AppError;
use crate::core::auth_context::AuthContext;
use crate::usecases::work_sessions::deps::WorkSessionUseCaseDependencies;
use crate::usecases::work_sessions::helpers::claims_user_id;
use crate::utils::pagination::Pagination;

pub struct ListSessionsUseCase {
    deps: WorkSessionUseCaseDependencies,
}

impl ListSessionsUseCase {
    pub fn new(deps: WorkSessionUseCaseDependencies) -> Self {
        Self { deps }
    }

    pub async fn execute(
        &self,
        mut query: ListWorkSessionsQuery,
        pagination: Pagination,
        claims: &UserClaims,
    ) -> Result<PaginatedResult<WorkSession>, AppError> {
        info!("[ListSessionsUseCase] Listing sessions with filters");

        let auth = AuthContext::load(self.deps.user_repository.as_ref(), claims).await?;
        let user_id = claims_user_id(claims)?;

        let can_view_others = if claims.profile == Profile::Root {
            true
        } else if let Some(city_id_str) = &claims.city_id {
            let user_city_id = Uuid::parse_str(city_id_str)
                .map_err(|_| AppError::Unauthorized("Invalid city id in token".to_string()))?;

            auth.check_policy(&Policy::ViewOtherWorkSessions, user_city_id)
                .is_ok()
        } else {
            false
        };

        if !can_view_others {
            query.user_id = Some(user_id);
        } else if claims.profile != Profile::Root
            && let Some(city_id_str) = &claims.city_id
        {
            let user_city_id = Uuid::parse_str(city_id_str)
                .map_err(|_| AppError::Unauthorized("Invalid city id in token".to_string()))?;
            query.city_id = Some(user_city_id);
        }

        let total_items = self
            .deps
            .work_session_read_repository
            .count_sessions_filtered(
                query.user_id,
                query.start_date,
                query.end_date,
                query.city_id,
            )
            .await
            .map_err(|_| AppError::InternalServerError)?;

        let sessions = self
            .deps
            .work_session_read_repository
            .list_sessions_filtered(
                query.user_id,
                query.start_date,
                query.end_date,
                query.city_id,
                pagination.page_size,
                pagination.offset,
            )
            .await
            .map_err(|_| AppError::InternalServerError)?;

        Ok(PaginatedResult {
            items: sessions,
            page: pagination.page,
            page_size: pagination.page_size,
            total_items,
        })
    }
}
