use actix_web::{HttpRequest, HttpResponse, web};
use log::{error, info};
use uuid::Uuid;

use crate::core::contracts::repository::users::UserRepository;
use crate::core::contracts::repository::work_sessions::WorkSessionRepository;
use crate::core::entities::work_session_members::{TeamMemberFunction, WorkSessionMember};
use crate::core::entities::work_sessions::{
    CreateWorkSession, ListWorkSessionsQuery, UpdateWorkSessionMembers,
};
use crate::repositories::users::PgUserRepository;
use crate::repositories::work_sessions::PgWorkSessionRepository;
use crate::utils::authorization::check_policy;
use crate::utils::db_error_mapper::map_unique_constraint;
use crate::utils::errors::AppError;
use crate::utils::pagination::{PaginationParams, normalize_pagination};
use crate::utils::responses::{ApiResponse, PaginatedResponse};
use crate::utils::service_helpers::{
    extract_city_id_from_claims, extract_claims, get_user_policies_with_defaults,
};
use crate::validators::common::{
    POLICY_CREATE_WORK_SESSIONS, POLICY_END_WORK_SESSIONS, POLICY_UPDATE_WORK_SESSIONS,
    POLICY_VIEW_OTHER_WORK_SESSIONS, PROFILE_ROOT,
};
use crate::validators::work_session_validator::WorkSessionValidator;

pub struct WorkSessionService {
    work_session_repository: web::Data<PgWorkSessionRepository>,
    user_repository: web::Data<PgUserRepository>,
}

impl WorkSessionService {
    pub fn new(
        work_session_repository: web::Data<PgWorkSessionRepository>,
        user_repository: web::Data<PgUserRepository>,
    ) -> Self {
        Self {
            work_session_repository,
            user_repository,
        }
    }

    pub async fn create_work_session(
        &self,
        data: CreateWorkSession,
        req: HttpRequest,
    ) -> Result<HttpResponse, AppError> {
        info!("[WorkSessionService] Starting work session creation");

        let claims = extract_claims(&req)?;
        let policies =
            get_user_policies_with_defaults(self.user_repository.as_ref(), &claims).await?;
        let user_id = Uuid::parse_str(&claims.id)
            .map_err(|_| AppError::Unauthorized("Invalid user id in token".to_string()))?;
        if claims.profile != PROFILE_ROOT {
            let user_city_id = extract_city_id_from_claims(&claims)?;
            check_policy(
                &claims,
                POLICY_CREATE_WORK_SESSIONS,
                user_city_id,
                &policies,
            )?;
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
            claims.profile == PROFILE_ROOT,
        )
        .await?;

        if let Ok(true) = self
            .work_session_repository
            .is_user_in_active_session(user_id)
            .await
        {
            return Err(AppError::Conflict(
                "User already has an active work session".to_string(),
            ));
        }

        match self
            .work_session_repository
            .create_work_session(data, user_id)
            .await
        {
            Ok(session) => {
                info!("[WorkSessionService] Work session created: {}", session.id);
                Ok(ApiResponse::created(session).into_response())
            }
            Err(e) => {
                if let sqlx::Error::Database(db_err) = &e
                    && db_err.is_unique_violation()
                    && let Some(app_err) = map_unique_constraint(
                        db_err.constraint(),
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

    pub async fn get_active_session(&self, req: HttpRequest) -> Result<HttpResponse, AppError> {
        info!("[WorkSessionService] Getting active session");

        let claims = extract_claims(&req)?;
        let user_id = Uuid::parse_str(&claims.id)
            .map_err(|_| AppError::Unauthorized("Invalid user id in token".to_string()))?;

        match self
            .work_session_repository
            .get_active_session_by_user(user_id)
            .await
        {
            Ok(session) => {
                let with_members = self
                    .work_session_repository
                    .get_session_by_id(session.id)
                    .await
                    .map_err(|e| match e {
                        sqlx::Error::RowNotFound => {
                            AppError::NotFound("Work session not found".to_string())
                        }
                        _ => AppError::InternalServerError,
                    })?;

                Ok(ApiResponse::success(with_members).into_response())
            }
            Err(sqlx::Error::RowNotFound) => Err(AppError::NotFound(
                "No active work session found".to_string(),
            )),
            Err(_) => Err(AppError::InternalServerError),
        }
    }

    pub async fn get_session_by_id(
        &self,
        session_id: Uuid,
        req: HttpRequest,
    ) -> Result<HttpResponse, AppError> {
        info!("[WorkSessionService] Getting session by id: {}", session_id);

        let claims = extract_claims(&req)?;
        let policies = get_user_policies_with_defaults(self.user_repository.as_ref(), &claims).await?;
        let user_id = Uuid::parse_str(&claims.id)
            .map_err(|_| AppError::Unauthorized("Invalid user id in token".to_string()))?;

        let session = self
            .work_session_repository
            .get_session_by_id(session_id)
            .await
            .map_err(|e| match e {
                sqlx::Error::RowNotFound => {
                    AppError::NotFound("Work session not found".to_string())
                }
                _ => AppError::InternalServerError,
            })?;

        if session.created_by_user_id != user_id {
            let user_city_id = extract_city_id_from_claims(&claims)?;
            check_policy(
                &claims,
                POLICY_VIEW_OTHER_WORK_SESSIONS,
                user_city_id,
                &policies,
            )?;
        }

        Ok(ApiResponse::success(session).into_response())
    }

    pub async fn list_sessions(
        &self,
        mut query: ListWorkSessionsQuery,
        req: HttpRequest,
    ) -> Result<HttpResponse, AppError> {
        info!("[WorkSessionService] Listing sessions with filters");

        let claims = extract_claims(&req)?;
        let policies = get_user_policies_with_defaults(&**self.user_repository, &claims).await?;
        let user_id = Uuid::parse_str(&claims.id)
            .map_err(|_| AppError::Unauthorized("Invalid user id in token".to_string()))?;

        let can_view_others = if claims.profile == PROFILE_ROOT {
            true
        } else if let Some(city_id_str) = &claims.city_id {
            let user_city_id = Uuid::parse_str(city_id_str)
                .map_err(|_| AppError::Unauthorized("Invalid city id in token".to_string()))?;

            check_policy(
                &claims,
                POLICY_VIEW_OTHER_WORK_SESSIONS,
                user_city_id,
                &policies,
            )
            .is_ok()
        } else {
            false
        };

        if !can_view_others {
            query.user_id = Some(user_id);
        } else if claims.profile == PROFILE_ROOT {
        } else if let Some(city_id_str) = &claims.city_id {
            let user_city_id = Uuid::parse_str(city_id_str)
                .map_err(|_| AppError::Unauthorized("Invalid city id in token".to_string()))?;
            query.city_id = Some(user_city_id);
        }

        let pagination_params = PaginationParams {
            page: query.page,
            page_size: query.page_size,
        };
        let pagination = normalize_pagination(&pagination_params);

        let total_items = self
            .work_session_repository
            .count_sessions_filtered(
                query.user_id,
                query.start_date,
                query.end_date,
                query.city_id,
            )
            .await
            .map_err(|_| AppError::InternalServerError)?;

        let sessions = self
            .work_session_repository
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

        let mut sessions_with_members = Vec::with_capacity(sessions.len());
        for session in sessions {
            let members = self
                .work_session_repository
                .get_session_members_with_details(session.id)
                .await
                .map_err(|e| match e {
                    sqlx::Error::RowNotFound => AppError::NotFound(format!(
                        "Session members not found for session '{}'",
                        session.id
                    )),
                    _ => AppError::InternalServerError,
                })?;
            sessions_with_members.push(session.with_members(members));
        }

        info!(
            "[WorkSessionService] Found {} sessions",
            sessions_with_members.len()
        );
        Ok(PaginatedResponse::success(
            sessions_with_members,
            pagination.page,
            pagination.page_size,
            total_items,
        )
        .into_response())
    }

    pub async fn end_session(&self, req: HttpRequest) -> Result<HttpResponse, AppError> {
        info!("[WorkSessionService] Ending session");

        let claims = extract_claims(&req)?;
        let policies = get_user_policies_with_defaults(&**self.user_repository, &claims).await?;
        let user_id = Uuid::parse_str(&claims.id)
            .map_err(|_| AppError::Unauthorized("Invalid user id in token".to_string()))?;
        let user_city_id = extract_city_id_from_claims(&claims)?;

        check_policy(&claims, POLICY_END_WORK_SESSIONS, user_city_id, &policies)?;

        let session = self
            .work_session_repository
            .get_user_active_session(user_id)
            .await
            .map_err(|_| AppError::NotFound("No active work session found".to_string()))?;

        let current_members = self
            .work_session_repository
            .get_session_members(session.id)
            .await
            .map_err(|e| match e {
                sqlx::Error::RowNotFound => {
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

        match self.work_session_repository.end_session(session.id).await {
            Ok(_) => {
                info!("[WorkSessionService] Work session ended: {}", session.id);
                Ok(ApiResponse::success("Work session ended successfully").into_response())
            }
            Err(_) => Err(AppError::InternalServerError),
        }
    }

    pub async fn add_member_to_session(
        &self,
        session_id: Uuid,
        user_id: Uuid,
        function: Option<TeamMemberFunction>,
        req: HttpRequest,
    ) -> Result<HttpResponse, AppError> {
        info!(
            "[WorkSessionService] Adding member to session: {}",
            session_id
        );

        let claims = extract_claims(&req)?;
        let policies = get_user_policies_with_defaults(&**self.user_repository, &claims).await?;
        let requesting_user_id = Uuid::parse_str(&claims.id)
            .map_err(|_| AppError::Unauthorized("Invalid user id in token".to_string()))?;
        let user_city_id = extract_city_id_from_claims(&claims)?;

        check_policy(
            &claims,
            POLICY_UPDATE_WORK_SESSIONS,
            user_city_id,
            &policies,
        )?;

        let session = self
            .work_session_repository
            .get_session_by_id(session_id)
            .await
            .map_err(|_| AppError::NotFound("Work session not found".to_string()))?;

        if !session.is_active {
            return Err(AppError::BadRequest(
                "Cannot add members to inactive session".to_string(),
            ));
        }

        let current_members = self
            .work_session_repository
            .get_session_members(session_id)
            .await
            .map_err(|e| match e {
                sqlx::Error::RowNotFound => {
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
            .work_session_repository
            .is_user_in_active_session(user_id)
            .await
        {
            return Err(AppError::Conflict(
                "User is already in an active session".to_string(),
            ));
        }

        self.validate_members_same_city(&[user_id], false).await?;

        let members_with_functions: Vec<(Uuid, Option<TeamMemberFunction>)> = current_members
            .iter()
            .map(|m| (m.user_id, m.function.clone()))
            .collect();

        WorkSessionValidator::can_add_member_with_function(&members_with_functions, &function)
            .map_err(AppError::BadRequest)?;

        let session_member_registration_id = Uuid::new_v4();
        match self
            .work_session_repository
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
                Ok(ApiResponse::success(member).into_response())
            }
            Err(e) => {
                if let sqlx::Error::Database(db_err) = &e
                    && db_err.is_unique_violation()
                    && let Some(app_err) = map_unique_constraint(
                        db_err.constraint(),
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
        req: HttpRequest,
    ) -> Result<HttpResponse, AppError> {
        info!(
            "[WorkSessionService] Removing member from session: {}",
            session_id
        );

        let claims = extract_claims(&req)?;
        let policies = get_user_policies_with_defaults(&**self.user_repository, &claims).await?;
        let requesting_user_id = Uuid::parse_str(&claims.id)
            .map_err(|_| AppError::Unauthorized("Invalid user id in token".to_string()))?;
        let user_city_id = extract_city_id_from_claims(&claims)?;

        check_policy(
            &claims,
            POLICY_UPDATE_WORK_SESSIONS,
            user_city_id,
            &policies,
        )?;

        let session = self
            .work_session_repository
            .get_session_by_id(session_id)
            .await
            .map_err(|_| AppError::NotFound("Work session not found".to_string()))?;

        if !session.is_active {
            return Err(AppError::BadRequest(
                "Cannot remove members from inactive session".to_string(),
            ));
        }

        let current_members = self
            .work_session_repository
            .get_session_members(session_id)
            .await
            .map_err(|e| match e {
                sqlx::Error::RowNotFound => {
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

        WorkSessionValidator::can_remove_member(current_members.len())
            .map_err(AppError::BadRequest)?;

        match self
            .work_session_repository
            .remove_member_from_session(session_id, member_id)
            .await
        {
            Ok(_) => {
                info!("[WorkSessionService] Member removed from session");
                Ok(ApiResponse::success("Member removed successfully").into_response())
            }
            Err(sqlx::Error::RowNotFound) => Err(AppError::NotFound(
                "User is not a member of this session".to_string(),
            )),
            Err(_) => Err(AppError::InternalServerError),
        }
    }

    pub async fn update_members(
        &self,
        session_id: Uuid,
        data: UpdateWorkSessionMembers,
        req: HttpRequest,
    ) -> Result<HttpResponse, AppError> {
        info!(
            "[WorkSessionService] Updating session members: {}",
            session_id
        );

        let claims = extract_claims(&req)?;
        let policies = get_user_policies_with_defaults(&**self.user_repository, &claims).await?;
        let requesting_user_id = Uuid::parse_str(&claims.id)
            .map_err(|_| AppError::Unauthorized("Invalid user id in token".to_string()))?;
        let user_city_id = extract_city_id_from_claims(&claims)?;

        check_policy(
            &claims,
            POLICY_UPDATE_WORK_SESSIONS,
            user_city_id,
            &policies,
        )?;

        let session = self
            .work_session_repository
            .get_session_by_id(session_id)
            .await
            .map_err(|_| AppError::NotFound("Work session not found".to_string()))?;

        if !session.is_active {
            return Err(AppError::BadRequest(
                "Cannot update members of inactive session".to_string(),
            ));
        }

        let current_members = self
            .work_session_repository
            .get_session_members(session_id)
            .await
            .map_err(|e| match e {
                sqlx::Error::RowNotFound => {
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
            false,
        )
        .await?;

        match self
            .work_session_repository
            .update_session_members(session_id, data.members)
            .await
        {
            Ok(_) => {
                let updated_session = self
                    .work_session_repository
                    .get_session_by_id(session_id)
                    .await
                    .map_err(|_| AppError::InternalServerError)?;

                info!("[WorkSessionService] Session members updated");
                Ok(ApiResponse::success(updated_session).into_response())
            }
            Err(e) => {
                if let sqlx::Error::Database(db_err) = &e
                    && db_err.is_unique_violation()
                    && let Some(app_err) = map_unique_constraint(
                        db_err.constraint(),
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
        req: HttpRequest,
    ) -> Result<HttpResponse, AppError> {
        info!(
            "[WorkSessionService] Updating member function for session: {}",
            session_id
        );

        let claims = extract_claims(&req)?;
        let policies = get_user_policies_with_defaults(&**self.user_repository, &claims).await?;

        let requesting_user_id = Uuid::parse_str(&claims.id)
            .map_err(|_| AppError::Unauthorized("Invalid user id in token".to_string()))?;

        let user_city_id = extract_city_id_from_claims(&claims)?;

        check_policy(
            &claims,
            POLICY_UPDATE_WORK_SESSIONS,
            user_city_id,
            &policies,
        )?;

        let session = self
            .work_session_repository
            .get_session_by_id(session_id)
            .await
            .map_err(|_| AppError::NotFound("Work session not found".to_string()))?;

        if !session.is_active {
            return Err(AppError::BadRequest(
                "Cannot update members of inactive session".to_string(),
            ));
        }

        let current_members = self
            .work_session_repository
            .get_session_members(session_id)
            .await
            .map_err(|e| match e {
                sqlx::Error::RowNotFound => {
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
            .work_session_repository
            .update_member_function(session_id, user_id, new_function.clone())
            .await
        {
            Ok(_) => {
                info!("[WorkSessionService] Member function updated successfully");
                Ok(ApiResponse::success("Member function updated successfully").into_response())
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
                    sqlx::Error::RowNotFound => {
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
