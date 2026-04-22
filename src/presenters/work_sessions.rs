use std::sync::Arc;

use serde::{Deserialize, Serialize};

use crate::core::application_error::ApplicationError as AppError;
use crate::core::contracts::repository::work_sessions::WorkSessionReadRepository;
use crate::core::entities::work_sessions::WorkSession;
use crate::core::pagination::PaginatedResult;
use crate::core::read_models::work_sessions::{
    WorkSessionWithMemberDetails, WorkSessionWithMemberUsers,
};
use crate::utils::pagination::Pagination;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum WorkSessionResponse {
    Simple(WorkSessionWithMemberDetails),
    WithEntities(WorkSessionWithMemberUsers),
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
                WorkSessionWithMemberUsers::from_entity(session, members),
            ))
        } else {
            let members = self
                .read_repository
                .get_session_members_with_details(session.id)
                .await
                .map_err(|_| AppError::InternalServerError)?;
            Ok(WorkSessionResponse::Simple(
                WorkSessionWithMemberDetails::from_entity(session, members),
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

#[cfg(test)]
mod tests {
    use std::{collections::HashMap, sync::Arc};

    use async_trait::async_trait;
    use chrono::{NaiveDate, Utc};
    use uuid::Uuid;

    use super::*;
    use crate::core::contracts::repository::error::RepositoryError;
    use crate::core::entities::work_session_members::{TeamMemberFunction, WorkSessionMember};
    use crate::core::read_models::users::UserSummary;
    use crate::core::read_models::work_sessions::{
        WorkSessionMemberWithDetails, WorkSessionMemberWithUser,
    };
    use crate::core::value_objects::profiles::Profile;
    use crate::core::value_objects::ranks::Rank;

    struct FakeWorkSessionReadRepo {
        member_details: Vec<WorkSessionMemberWithDetails>,
        member_users: Vec<WorkSessionMemberWithUser>,
    }

    #[async_trait]
    impl WorkSessionReadRepository for FakeWorkSessionReadRepo {
        async fn get_active_session_by_user(
            &self,
            _user_id: Uuid,
        ) -> Result<WorkSession, RepositoryError> {
            unimplemented!()
        }

        async fn get_user_active_session(
            &self,
            _user_id: Uuid,
        ) -> Result<WorkSession, RepositoryError> {
            unimplemented!()
        }

        async fn is_user_in_active_session(&self, _user_id: Uuid) -> Result<bool, RepositoryError> {
            unimplemented!()
        }

        async fn get_session_by_id(
            &self,
            _session_id: Uuid,
        ) -> Result<WorkSessionWithMemberDetails, RepositoryError> {
            unimplemented!()
        }

        async fn get_session_by_id_base(
            &self,
            _session_id: Uuid,
        ) -> Result<WorkSession, RepositoryError> {
            unimplemented!()
        }

        async fn get_sessions_by_user(
            &self,
            _user_id: Uuid,
        ) -> Result<Vec<WorkSessionWithMemberDetails>, RepositoryError> {
            unimplemented!()
        }

        async fn list_sessions_filtered(
            &self,
            _user_id: Option<Uuid>,
            _start_date: Option<NaiveDate>,
            _end_date: Option<NaiveDate>,
            _city_id: Option<Uuid>,
            _limit: i64,
            _offset: i64,
        ) -> Result<Vec<WorkSession>, RepositoryError> {
            unimplemented!()
        }

        async fn count_sessions_filtered(
            &self,
            _user_id: Option<Uuid>,
            _start_date: Option<NaiveDate>,
            _end_date: Option<NaiveDate>,
            _city_id: Option<Uuid>,
        ) -> Result<i64, RepositoryError> {
            unimplemented!()
        }

        async fn get_session_members(
            &self,
            _session_id: Uuid,
        ) -> Result<Vec<WorkSessionMember>, RepositoryError> {
            unimplemented!()
        }

        async fn get_session_members_with_details(
            &self,
            _session_id: Uuid,
        ) -> Result<Vec<WorkSessionMemberWithDetails>, RepositoryError> {
            Ok(self.member_details.clone())
        }

        async fn get_session_members_with_user_details(
            &self,
            _session_id: Uuid,
        ) -> Result<Vec<WorkSessionMemberWithUser>, RepositoryError> {
            Ok(self.member_users.clone())
        }
    }

    fn presenter(
        member_details: Vec<WorkSessionMemberWithDetails>,
        member_users: Vec<WorkSessionMemberWithUser>,
    ) -> WorkSessionPresenter {
        WorkSessionPresenter::new(Arc::new(FakeWorkSessionReadRepo {
            member_details,
            member_users,
        }))
    }

    fn session() -> WorkSession {
        let now = Utc::now();
        WorkSession {
            id: Uuid::new_v4(),
            created_by_user_id: Uuid::new_v4(),
            started_at: now,
            ended_at: None,
            is_active: true,
            description: Some("Turno de teste".to_string()),
            created_at: now,
            updated_at: now,
        }
    }

    fn member_details(user_id: Uuid) -> WorkSessionMemberWithDetails {
        WorkSessionMemberWithDetails {
            id: Uuid::new_v4(),
            user_id,
            user_name: "Operador Teste".to_string(),
            function: Some(TeamMemberFunction::Commander),
            created_at: Utc::now(),
        }
    }

    fn member_user(user_id: Uuid) -> WorkSessionMemberWithUser {
        WorkSessionMemberWithUser {
            id: Uuid::new_v4(),
            function: Some(TeamMemberFunction::Commander),
            user: UserSummary {
                id: user_id,
                rank: Rank::SdPm,
                registration: "123456".to_string(),
                full_name: "Operador Teste".to_string(),
                profile: Profile::CityUser,
                email: "operador@test.com".to_string(),
                city_id: Some(Uuid::new_v4()),
                permission_policies: HashMap::new(),
                is_deleted: false,
            },
        }
    }

    #[actix_rt::test]
    async fn build_response_without_include_related_populates_member_details() {
        let user_id = Uuid::new_v4();
        let presenter = presenter(vec![member_details(user_id)], vec![]);

        let response = presenter.build_response(session(), false).await.unwrap();

        match response {
            WorkSessionResponse::Simple(simple) => {
                assert_eq!(simple.members.len(), 1);
                assert_eq!(simple.members[0].user_id, user_id);
                assert_eq!(simple.members[0].user_name, "Operador Teste");
            }
            WorkSessionResponse::WithEntities(_) => panic!("unexpected related response"),
        }
    }

    #[actix_rt::test]
    async fn build_response_with_include_related_populates_member_user_details() {
        let user_id = Uuid::new_v4();
        let presenter = presenter(vec![], vec![member_user(user_id)]);

        let response = presenter.build_response(session(), true).await.unwrap();

        match response {
            WorkSessionResponse::WithEntities(with_entities) => {
                assert_eq!(with_entities.members.len(), 1);
                assert_eq!(with_entities.members[0].user.id, user_id);
                assert_eq!(with_entities.members[0].user.full_name, "Operador Teste");
            }
            WorkSessionResponse::Simple(_) => panic!("expected related response"),
        }
    }

    #[actix_rt::test]
    async fn build_response_without_members_does_not_panic() {
        let presenter = presenter(vec![], vec![]);

        let response = presenter.build_response(session(), false).await.unwrap();

        match response {
            WorkSessionResponse::Simple(simple) => assert!(simple.members.is_empty()),
            WorkSessionResponse::WithEntities(_) => panic!("unexpected related response"),
        }
    }
}
