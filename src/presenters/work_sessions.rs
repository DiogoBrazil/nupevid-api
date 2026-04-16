use std::sync::Arc;

use serde::{Deserialize, Serialize};

use crate::core::contracts::repository::work_sessions::WorkSessionReadRepository;
use crate::core::entities::work_sessions::WorkSession;
use crate::core::pagination::PaginatedResult;
use crate::core::read_models::work_sessions::{
    WorkSessionWithMembers, WorkSessionWithMembersSummary,
};
use crate::core::application_error::ApplicationError as AppError;
use crate::utils::pagination::Pagination;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum WorkSessionResponse {
    Simple(WorkSessionWithMembers),
    WithEntities(WorkSessionWithMembersSummary),
}

pub struct WorkSessionPresenter {
    read_repository: Arc<dyn WorkSessionReadRepository>,
}

impl WorkSessionPresenter {
    pub fn new(read_repository: Arc<dyn WorkSessionReadRepository>) -> Self {
        Self { read_repository }
    }

    pub async fn build_response(
        &self,
        session: WorkSession,
        include_related_entities: bool,
    ) -> Result<WorkSessionResponse, AppError> {
        if include_related_entities {
            let members = self
                .read_repository
                .get_session_members_with_user_details(session.id)
                .await
                .map_err(|_| AppError::InternalServerError)?;
            Ok(WorkSessionResponse::WithEntities(
                WorkSessionWithMembersSummary::from_entity(session, members),
            ))
        } else {
            let members = self
                .read_repository
                .get_session_members_with_details(session.id)
                .await
                .map_err(|_| AppError::InternalServerError)?;
            Ok(WorkSessionResponse::Simple(
                WorkSessionWithMembers::from_entity(session, members),
            ))
        }
    }

    pub async fn build_responses(
        &self,
        sessions: Vec<WorkSession>,
        include_related_entities: bool,
        pagination: Pagination,
        total_items: i64,
    ) -> Result<PaginatedResult<WorkSessionResponse>, AppError> {
        let mut items = Vec::with_capacity(sessions.len());

        for session in sessions {
            items.push(
                self.build_response(session, include_related_entities)
                    .await?,
            );
        }

        Ok(PaginatedResult {
            items,
            page: pagination.page,
            page_size: pagination.page_size,
            total_items,
        })
    }
}
