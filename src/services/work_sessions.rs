use log::{error, info};
use std::sync::Arc;
use uuid::Uuid;

use crate::core::commands::work_sessions::{CreateWorkSession, UpdateWorkSessionMembers};
use crate::core::contracts::repository::users::UserRepository;
use crate::core::contracts::repository::work_sessions::{
    WorkSessionReadRepository, WorkSessionWriteRepository,
};
use crate::core::entities::auth::ClaimsToUserToken;
use crate::core::entities::common::PaginatedResult;
use crate::core::entities::work_session_members::{TeamMemberFunction, WorkSessionMember};
use crate::utils::errors::AppError;
use crate::core::contracts::repository::error::RepositoryError;
use crate::core::queries::work_sessions::ListWorkSessionsQuery;
use crate::core::read_models::work_sessions::WorkSessionWithMembers;
use crate::core::responses::work_sessions::WorkSessionResponse;
use crate::services::auth_context::AuthContext;
use crate::services::error_mapping::map_unique_constraint;
use crate::services::helpers::extract_city_id_from_claims;
use crate::utils::pagination::Pagination;
use crate::core::value_objects::policies::Policy;
use crate::core::value_objects::profiles::Profile;
use crate::validators::work_session_validator::WorkSessionValidator;

pub struct WorkSessionService {
    work_session_read_repository: Arc<dyn WorkSessionReadRepository>,
    work_session_write_repository: Arc<dyn WorkSessionWriteRepository>,
    user_repository: Arc<dyn UserRepository>,
}

impl WorkSessionService {
    pub fn new(
        work_session_read_repository: Arc<dyn WorkSessionReadRepository>,
        work_session_write_repository: Arc<dyn WorkSessionWriteRepository>,
        user_repository: Arc<dyn UserRepository>,
    ) -> Self {
        Self {
            work_session_read_repository,
            work_session_write_repository,
            user_repository,
        }
    }

    pub async fn create_work_session(
        &self,
        data: CreateWorkSession,
        claims: &ClaimsToUserToken,
        include_complement_for_entities: bool,
    ) -> Result<WorkSessionResponse, AppError> {
        info!("[WorkSessionService] Starting work session creation");

        let auth = AuthContext::load(self.user_repository.as_ref(), claims).await?;
        let user_id = Uuid::parse_str(&claims.id)
            .map_err(|_| AppError::Unauthorized("Invalid user id in token".to_string()))?;
        if claims.profile != Profile::Root {
            let user_city_id = extract_city_id_from_claims(claims)?;
            auth.check_policy(&Policy::CreateWorkSessions, user_city_id)?;
        }

        if !data.members.iter().any(|member| member.user_id == user_id) {
            return Err(AppError::BadRequest(
                "Requesting user must be included in session members".to_string(),
            ));
        }

        let members_with_functions: Vec<(Uuid, Option<TeamMemberFunction>)> = data
            .members
            .iter()
            .map(|m| (m.user_id, m.function.clone()))
            .collect();

        WorkSessionValidator::validate_team_functions(&members_with_functions)
            .map_err(AppError::BadRequest)?;

        self.validate_members_same_city(
            &data.members.iter().map(|m| m.user_id).collect::<Vec<_>>(),
            claims.profile == Profile::Root,
        )
        .await?;

        if let Ok(true) = self
            .work_session_read_repository
            .is_user_in_active_session(user_id)
            .await
        {
            return Err(AppError::Conflict(
                "User already has an active work session".to_string(),
            ));
        }

        match self
            .work_session_write_repository
            .create_work_session(data, user_id)
            .await
        {
            Ok(session) => {
                info!("[WorkSessionService] Work session created: {}", session.id);
                if include_complement_for_entities {
                    let members = self
                        .work_session_read_repository
                        .get_session_members_with_user_details(session.id)
                        .await
                        .map_err(|_| AppError::InternalServerError)?;
                    Ok(WorkSessionResponse::WithEntities(
                        session.with_members_complement(members),
                    ))
                } else {
                    let members = self
                        .work_session_read_repository
                        .get_session_members_with_details(session.id)
                        .await
                        .map_err(|_| AppError::InternalServerError)?;
                    Ok(WorkSessionResponse::Simple(session.with_members(members)))
                }
            }
            Err(e) => {
                if let RepositoryError::UniqueViolation { constraint } = &e
                    && let Some(app_err) = map_unique_constraint(
                        constraint.as_deref(),
                        &[
                            (
                                "unique_active_session_per_user",
                                "User already has an active work session",
                            ),
                            (
                                "work_session_members_work_session_id_user_id_key",
                                "User already added to session",
                            ),
                        ],
                    )
                {
                    return Err(app_err);
                }
                error!(
                    "[WorkSessionService] Failed to create work session: {:?}",
                    e
                );
                Err(AppError::InternalServerError)
            }
        }
    }

    pub async fn get_active_session(
        &self,
        claims: &ClaimsToUserToken,
        include_complement_for_entities: bool,
    ) -> Result<WorkSessionResponse, AppError> {
        info!("[WorkSessionService] Getting active session");

        let user_id = Uuid::parse_str(&claims.id)
            .map_err(|_| AppError::Unauthorized("Invalid user id in token".to_string()))?;

        match self
            .work_session_read_repository
            .get_active_session_by_user(user_id)
            .await
        {
            Ok(session) => {
                if include_complement_for_entities {
                    let members = self
                        .work_session_read_repository
                        .get_session_members_with_user_details(session.id)
                        .await
                        .map_err(|_| AppError::InternalServerError)?;
                    Ok(WorkSessionResponse::WithEntities(
                        session.with_members_complement(members),
                    ))
                } else {
                    let with_members = self
                        .work_session_read_repository
                        .get_session_by_id(session.id)
                        .await
                        .map_err(|e| match e {
                            RepositoryError::NotFound => {
                                AppError::NotFound("Work session not found".to_string())
                            }
                            _ => AppError::InternalServerError,
                        })?;
                    Ok(WorkSessionResponse::Simple(with_members))
                }
            }
            Err(RepositoryError::NotFound) => Err(AppError::NotFound(
                "No active work session found".to_string(),
            )),
            Err(_) => Err(AppError::InternalServerError),
        }
    }

    pub async fn get_session_by_id(
        &self,
        session_id: Uuid,
        claims: &ClaimsToUserToken,
        include_complement_for_entities: bool,
    ) -> Result<WorkSessionResponse, AppError> {
        info!("[WorkSessionService] Getting session by id: {}", session_id);

        let auth = AuthContext::load(self.user_repository.as_ref(), claims).await?;
        let user_id = Uuid::parse_str(&claims.id)
            .map_err(|_| AppError::Unauthorized("Invalid user id in token".to_string()))?;

        if include_complement_for_entities {
            let session = self
                .work_session_read_repository
                .get_session_by_id_base(session_id)
                .await
                .map_err(|e| match e {
                    RepositoryError::NotFound => {
                        AppError::NotFound("Work session not found".to_string())
                    }
                    _ => AppError::InternalServerError,
                })?;

            if session.created_by_user_id != user_id && claims.profile != Profile::Root {
                let user_city_id = extract_city_id_from_claims(claims)?;
                auth.check_policy(&Policy::ViewOtherWorkSessions, user_city_id)?;
            }

            let members = self
                .work_session_read_repository
                .get_session_members_with_user_details(session_id)
                .await
                .map_err(|_| AppError::InternalServerError)?;
            Ok(WorkSessionResponse::WithEntities(
                session.with_members_complement(members),
            ))
        } else {
            let session = self
                .work_session_read_repository
                .get_session_by_id(session_id)
                .await
                .map_err(|e| match e {
                    RepositoryError::NotFound => {
                        AppError::NotFound("Work session not found".to_string())
                    }
                    _ => AppError::InternalServerError,
                })?;

            if session.created_by_user_id != user_id && claims.profile != Profile::Root {
                let user_city_id = extract_city_id_from_claims(claims)?;
                auth.check_policy(&Policy::ViewOtherWorkSessions, user_city_id)?;
            }

            Ok(WorkSessionResponse::Simple(session))
        }
    }

    pub async fn list_sessions(
        &self,
        mut query: ListWorkSessionsQuery,
        pagination: Pagination,
        claims: &ClaimsToUserToken,
    ) -> Result<PaginatedResult<WorkSessionResponse>, AppError> {
        info!("[WorkSessionService] Listing sessions with filters");

        let auth = AuthContext::load(&*self.user_repository, claims).await?;
        let user_id = Uuid::parse_str(&claims.id)
            .map_err(|_| AppError::Unauthorized("Invalid user id in token".to_string()))?;

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
        } else if claims.profile == Profile::Root {
        } else if let Some(city_id_str) = &claims.city_id {
            let user_city_id = Uuid::parse_str(city_id_str)
                .map_err(|_| AppError::Unauthorized("Invalid city id in token".to_string()))?;
            query.city_id = Some(user_city_id);
        }

        let total_items = self
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

        let include_complement_for_entities =
            query.include_complement_for_entities.unwrap_or(false);

        if include_complement_for_entities {
            let mut items = Vec::with_capacity(sessions.len());
            for session in sessions {
                let members =
                    self.work_session_read_repository
                        .get_session_members_with_user_details(session.id)
                        .await
                        .map_err(|e| match e {
                            RepositoryError::NotFound => AppError::NotFound(
                                format!("Session members not found for session '{}'", session.id),
                            ),
                            _ => AppError::InternalServerError,
                        })?;
                items.push(WorkSessionResponse::WithEntities(
                    session.with_members_complement(members),
                ));
            }

            info!("[WorkSessionService] Found {} sessions", items.len());
            Ok(PaginatedResult {
                items,
                page: pagination.page,
                page_size: pagination.page_size,
                total_items,
            })
        } else {
            let mut items = Vec::with_capacity(sessions.len());
            for session in sessions {
                let members =
                    self.work_session_read_repository
                        .get_session_members_with_details(session.id)
                        .await
                        .map_err(|e| match e {
                            RepositoryError::NotFound => AppError::NotFound(
                                format!("Session members not found for session '{}'", session.id),
                            ),
                            _ => AppError::InternalServerError,
                        })?;
                items.push(WorkSessionResponse::Simple(session.with_members(members)));
            }

            info!("[WorkSessionService] Found {} sessions", items.len());
            Ok(PaginatedResult {
                items,
                page: pagination.page,
                page_size: pagination.page_size,
                total_items,
            })
        }
    }

    pub async fn end_session(&self, claims: &ClaimsToUserToken) -> Result<String, AppError> {
        info!("[WorkSessionService] Ending session");

        let auth = AuthContext::load(&*self.user_repository, claims).await?;
        let user_id = Uuid::parse_str(&claims.id)
            .map_err(|_| AppError::Unauthorized("Invalid user id in token".to_string()))?;
        if claims.profile != Profile::Root {
            let user_city_id = extract_city_id_from_claims(claims)?;
            auth.check_policy(&Policy::EndWorkSessions, user_city_id)?;
        }

        let session = self
            .work_session_read_repository
            .get_user_active_session(user_id)
            .await
            .map_err(|_| AppError::NotFound("No active work session found".to_string()))?;

        let current_members = self
            .work_session_read_repository
            .get_session_members(session.id)
            .await
            .map_err(|e| match e {
                RepositoryError::NotFound => {
                    AppError::NotFound("Session members not found".to_string())
                }
                _ => AppError::InternalServerError,
            })?;

        self.ensure_creator_or_commander(
            session.created_by_user_id,
            user_id,
            &current_members,
            "Only the session creator or commander can end the session",
        )?;

        match self
            .work_session_write_repository
            .end_session(session.id)
            .await
        {
            Ok(_) => {
                info!("[WorkSessionService] Work session ended: {}", session.id);
                Ok("Work session ended successfully".to_string())
            }
            Err(_) => Err(AppError::InternalServerError),
        }
    }

    pub async fn add_member_to_session(
        &self,
        session_id: Uuid,
        user_id: Uuid,
        function: Option<TeamMemberFunction>,
        claims: &ClaimsToUserToken,
    ) -> Result<WorkSessionMember, AppError> {
        info!(
            "[WorkSessionService] Adding member to session: {}",
            session_id
        );

        let auth = AuthContext::load(&*self.user_repository, claims).await?;
        let requesting_user_id = Uuid::parse_str(&claims.id)
            .map_err(|_| AppError::Unauthorized("Invalid user id in token".to_string()))?;
        if claims.profile != Profile::Root {
            let user_city_id = extract_city_id_from_claims(claims)?;
            auth.check_policy(&Policy::UpdateWorkSessions, user_city_id)?;
        }

        let session = self
            .work_session_read_repository
            .get_session_by_id(session_id)
            .await
            .map_err(|_| AppError::NotFound("Work session not found".to_string()))?;

        if !session.is_active {
            return Err(AppError::BadRequest(
                "Cannot add members to inactive session".to_string(),
            ));
        }

        let current_members = self
            .work_session_read_repository
            .get_session_members(session_id)
            .await
            .map_err(|e| match e {
                RepositoryError::NotFound => {
                    AppError::NotFound("Session members not found".to_string())
                }
                _ => AppError::InternalServerError,
            })?;

        self.ensure_creator_or_commander(
            session.created_by_user_id,
            requesting_user_id,
            &current_members,
            "Only the session creator or commander can add members",
        )?;

        if let Ok(true) = self
            .work_session_read_repository
            .is_user_in_active_session(user_id)
            .await
        {
            return Err(AppError::Conflict(
                "User is already in an active session".to_string(),
            ));
        }

        let mut all_user_ids: Vec<Uuid> = current_members.iter().map(|m| m.user_id).collect();
        all_user_ids.push(user_id);
        self.validate_members_same_city(&all_user_ids, claims.profile == Profile::Root)
            .await?;

        let members_with_functions: Vec<(Uuid, Option<TeamMemberFunction>)> = current_members
            .iter()
            .map(|m| (m.user_id, m.function.clone()))
            .collect();

        WorkSessionValidator::can_add_member_with_function(&members_with_functions, &function)
            .map_err(AppError::BadRequest)?;

        let session_member_registration_id = Uuid::new_v4();
        match self
            .work_session_write_repository
            .add_member_to_session(
                session_member_registration_id,
                session_id,
                user_id,
                function,
            )
            .await
        {
            Ok(member) => {
                info!("[WorkSessionService] Member added to session");
                Ok(member)
            }
            Err(e) => {
                if let RepositoryError::UniqueViolation { constraint } = &e
                    && let Some(app_err) = map_unique_constraint(
                        constraint.as_deref(),
                        &[(
                            "work_session_members_work_session_id_user_id_key",
                            "User already added to session",
                        )],
                    )
                {
                    return Err(app_err);
                }
                Err(AppError::InternalServerError)
            }
        }
    }

    pub async fn remove_member_from_session(
        &self,
        session_id: Uuid,
        member_id: Uuid,
        claims: &ClaimsToUserToken,
    ) -> Result<String, AppError> {
        info!(
            "[WorkSessionService] Removing member from session: {}",
            session_id
        );

        let auth = AuthContext::load(&*self.user_repository, claims).await?;
        let requesting_user_id = Uuid::parse_str(&claims.id)
            .map_err(|_| AppError::Unauthorized("Invalid user id in token".to_string()))?;
        if claims.profile != Profile::Root {
            let user_city_id = extract_city_id_from_claims(claims)?;
            auth.check_policy(&Policy::UpdateWorkSessions, user_city_id)?;
        }

        let session = self
            .work_session_read_repository
            .get_session_by_id(session_id)
            .await
            .map_err(|_| AppError::NotFound("Work session not found".to_string()))?;

        if !session.is_active {
            return Err(AppError::BadRequest(
                "Cannot remove members from inactive session".to_string(),
            ));
        }

        let current_members = self
            .work_session_read_repository
            .get_session_members(session_id)
            .await
            .map_err(|e| match e {
                RepositoryError::NotFound => {
                    AppError::NotFound("Session members not found".to_string())
                }
                _ => AppError::InternalServerError,
            })?;

        self.ensure_creator_or_commander(
            session.created_by_user_id,
            requesting_user_id,
            &current_members,
            "Only the session creator or commander can remove members",
        )?;

        let members_with_functions: Vec<(Uuid, Option<TeamMemberFunction>)> = current_members
            .iter()
            .map(|m| (m.user_id, m.function.clone()))
            .collect();
        WorkSessionValidator::can_remove_member(&members_with_functions, member_id)
            .map_err(AppError::BadRequest)?;

        match self
            .work_session_write_repository
            .remove_member_from_session(session_id, member_id)
            .await
        {
            Ok(_) => {
                info!("[WorkSessionService] Member removed from session");
                Ok("Member removed successfully".to_string())
            }
            Err(RepositoryError::NotFound) => Err(AppError::NotFound(
                "User is not a member of this session".to_string(),
            )),
            Err(_) => Err(AppError::InternalServerError),
        }
    }

    pub async fn update_members(
        &self,
        session_id: Uuid,
        data: UpdateWorkSessionMembers,
        claims: &ClaimsToUserToken,
    ) -> Result<WorkSessionWithMembers, AppError> {
        info!(
            "[WorkSessionService] Updating session members: {}",
            session_id
        );

        let auth = AuthContext::load(&*self.user_repository, claims).await?;
        let requesting_user_id = Uuid::parse_str(&claims.id)
            .map_err(|_| AppError::Unauthorized("Invalid user id in token".to_string()))?;
        if claims.profile != Profile::Root {
            let user_city_id = extract_city_id_from_claims(claims)?;
            auth.check_policy(&Policy::UpdateWorkSessions, user_city_id)?;
        }

        let session = self
            .work_session_read_repository
            .get_session_by_id(session_id)
            .await
            .map_err(|_| AppError::NotFound("Work session not found".to_string()))?;

        if !session.is_active {
            return Err(AppError::BadRequest(
                "Cannot update members of inactive session".to_string(),
            ));
        }

        let current_members = self
            .work_session_read_repository
            .get_session_members(session_id)
            .await
            .map_err(|e| match e {
                RepositoryError::NotFound => {
                    AppError::NotFound("Session members not found".to_string())
                }
                _ => AppError::InternalServerError,
            })?;

        self.ensure_creator_or_commander(
            session.created_by_user_id,
            requesting_user_id,
            &current_members,
            "Only the session creator or commander can update members",
        )?;

        let members_with_functions: Vec<(Uuid, Option<TeamMemberFunction>)> = data
            .members
            .iter()
            .map(|m| (m.user_id, m.function.clone()))
            .collect();

        WorkSessionValidator::validate_team_functions(&members_with_functions)
            .map_err(AppError::BadRequest)?;

        self.validate_members_same_city(
            &data.members.iter().map(|m| m.user_id).collect::<Vec<_>>(),
            claims.profile == Profile::Root,
        )
        .await?;

        match self
            .work_session_write_repository
            .update_session_members(session_id, data.members)
            .await
        {
            Ok(_) => {
                let updated_session = self
                    .work_session_read_repository
                    .get_session_by_id(session_id)
                    .await
                    .map_err(|_| AppError::InternalServerError)?;

                info!("[WorkSessionService] Session members updated");
                Ok(updated_session)
            }
            Err(e) => {
                if let RepositoryError::UniqueViolation { constraint } = &e
                    && let Some(app_err) = map_unique_constraint(
                        constraint.as_deref(),
                        &[(
                            "work_session_members_work_session_id_user_id_key",
                            "User already added to session",
                        )],
                    )
                {
                    return Err(app_err);
                }
                Err(AppError::InternalServerError)
            }
        }
    }

    pub async fn update_member_function(
        &self,
        session_id: Uuid,
        user_id: Uuid,
        new_function: Option<TeamMemberFunction>,
        claims: &ClaimsToUserToken,
    ) -> Result<String, AppError> {
        info!(
            "[WorkSessionService] Updating member function for session: {}",
            session_id
        );

        let auth = AuthContext::load(&*self.user_repository, claims).await?;

        let requesting_user_id = Uuid::parse_str(&claims.id)
            .map_err(|_| AppError::Unauthorized("Invalid user id in token".to_string()))?;

        if claims.profile != Profile::Root {
            let user_city_id = extract_city_id_from_claims(claims)?;
            auth.check_policy(&Policy::UpdateWorkSessions, user_city_id)?;
        }

        let session = self
            .work_session_read_repository
            .get_session_by_id(session_id)
            .await
            .map_err(|_| AppError::NotFound("Work session not found".to_string()))?;

        if !session.is_active {
            return Err(AppError::BadRequest(
                "Cannot update members of inactive session".to_string(),
            ));
        }

        let current_members = self
            .work_session_read_repository
            .get_session_members(session_id)
            .await
            .map_err(|e| match e {
                RepositoryError::NotFound => {
                    AppError::NotFound("Session members not found".to_string())
                }
                _ => AppError::InternalServerError,
            })?;

        self.ensure_creator_or_commander(
            session.created_by_user_id,
            requesting_user_id,
            &current_members,
            "Only the session creator or commander can update member functions",
        )?;

        if !current_members.iter().any(|m| m.user_id == user_id) {
            return Err(AppError::NotFound(
                "User is not a member of this session".to_string(),
            ));
        }

        let members_with_functions: Vec<(Uuid, Option<TeamMemberFunction>)> = current_members
            .iter()
            .map(|m| {
                if m.user_id == user_id {
                    (m.user_id, new_function.clone())
                } else {
                    (m.user_id, m.function.clone())
                }
            })
            .collect();

        WorkSessionValidator::validate_team_functions(&members_with_functions)
            .map_err(AppError::BadRequest)?;

        match self
            .work_session_write_repository
            .update_member_function(session_id, user_id, new_function.clone())
            .await
        {
            Ok(_) => {
                info!("[WorkSessionService] Member function updated successfully");
                Ok("Member function updated successfully".to_string())
            }
            Err(e) => {
                error!(
                    "[WorkSessionService] Failed to update member function: {:?}",
                    e
                );
                Err(AppError::InternalServerError)
            }
        }
    }

    pub async fn update_work_session(
        &self,
        session_id: Uuid,
        data: CreateWorkSession,
        claims: &ClaimsToUserToken,
        include_complement_for_entities: bool,
    ) -> Result<WorkSessionResponse, AppError> {
        info!("[WorkSessionService] Updating work session: {}", session_id);

        let auth = AuthContext::load(&*self.user_repository, claims).await?;
        let requesting_user_id = Uuid::parse_str(&claims.id)
            .map_err(|_| AppError::Unauthorized("Invalid user id in token".to_string()))?;
        if claims.profile != Profile::Root {
            let user_city_id = extract_city_id_from_claims(claims)?;
            auth.check_policy(&Policy::UpdateWorkSessions, user_city_id)?;
        }

        let session = self
            .work_session_read_repository
            .get_session_by_id_base(session_id)
            .await
            .map_err(|_| AppError::NotFound("Work session not found".to_string()))?;

        if !session.is_active {
            return Err(AppError::BadRequest(
                "Cannot update inactive session".to_string(),
            ));
        }

        let current_members = self
            .work_session_read_repository
            .get_session_members(session_id)
            .await
            .map_err(|e| match e {
                RepositoryError::NotFound => {
                    AppError::NotFound("Session members not found".to_string())
                }
                _ => AppError::InternalServerError,
            })?;

        self.ensure_creator_or_commander(
            session.created_by_user_id,
            requesting_user_id,
            &current_members,
            "Only the session creator or commander can update the session",
        )?;

        if !data
            .members
            .iter()
            .any(|member| member.user_id == requesting_user_id)
        {
            return Err(AppError::BadRequest(
                "Requesting user must be included in session members".to_string(),
            ));
        }

        let members_with_functions: Vec<(Uuid, Option<TeamMemberFunction>)> = data
            .members
            .iter()
            .map(|m| (m.user_id, m.function.clone()))
            .collect();

        WorkSessionValidator::validate_team_functions(&members_with_functions)
            .map_err(AppError::BadRequest)?;

        self.validate_members_same_city(
            &data.members.iter().map(|m| m.user_id).collect::<Vec<_>>(),
            claims.profile == Profile::Root,
        )
        .await?;

        match self
            .work_session_write_repository
            .update_work_session_with_members(session_id, data.description, data.members)
            .await
        {
            Ok(updated_session) => {
                if include_complement_for_entities {
                    let members = self
                        .work_session_read_repository
                        .get_session_members_with_user_details(session_id)
                        .await
                        .map_err(|_| AppError::InternalServerError)?;
                    Ok(WorkSessionResponse::WithEntities(
                        updated_session.with_members_complement(members),
                    ))
                } else {
                    let members = self
                        .work_session_read_repository
                        .get_session_members_with_details(session_id)
                        .await
                        .map_err(|_| AppError::InternalServerError)?;
                    Ok(WorkSessionResponse::Simple(
                        updated_session.with_members(members),
                    ))
                }
            }
            Err(e) => {
                if let RepositoryError::UniqueViolation { constraint } = &e
                    && let Some(app_err) = map_unique_constraint(
                        constraint.as_deref(),
                        &[(
                            "work_session_members_work_session_id_user_id_key",
                            "User already added to session",
                        )],
                    )
                {
                    return Err(app_err);
                }
                error!(
                    "[WorkSessionService] Failed to update work session: {:?}",
                    e
                );
                Err(AppError::InternalServerError)
            }
        }
    }

    async fn validate_members_same_city(
        &self,
        user_ids: &[Uuid],
        allow_cityless: bool,
    ) -> Result<(), AppError> {
        if user_ids.is_empty() {
            return Ok(());
        }

        let mut city_id: Option<Uuid> = None;

        for user_id in user_ids {
            let user = self
                .user_repository
                .get_user_by_id(*user_id)
                .await
                .map_err(|e| match e {
                    RepositoryError::NotFound => {
                        AppError::NotFound(format!("User {} not found", user_id))
                    }
                    _ => AppError::InternalServerError,
                })?;

            let user_city_id = match user.city_id {
                Some(city_id) => city_id,
                None => {
                    if allow_cityless {
                        continue;
                    }
                    return Err(AppError::BadRequest(format!(
                        "User {} is not associated with a city",
                        user_id
                    )));
                }
            };

            match city_id {
                None => city_id = Some(user_city_id),
                Some(expected_city_id) => {
                    if user_city_id != expected_city_id {
                        return Err(AppError::BadRequest(
                            "All team members must be from the same city".to_string(),
                        ));
                    }
                }
            }
        }

        Ok(())
    }

    fn ensure_creator_or_commander(
        &self,
        created_by_user_id: Uuid,
        requesting_user_id: Uuid,
        current_members: &[WorkSessionMember],
        error_message: &str,
    ) -> Result<(), AppError> {
        if created_by_user_id == requesting_user_id {
            return Ok(());
        }

        let is_commander = current_members.iter().any(|member| {
            member.user_id == requesting_user_id
                && matches!(member.function, Some(TeamMemberFunction::Commander))
        });

        if is_commander {
            return Ok(());
        }

        Err(AppError::Forbidden(error_message.to_string()))
    }
}
