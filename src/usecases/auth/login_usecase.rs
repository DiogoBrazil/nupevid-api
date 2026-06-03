use chrono::Utc;
use log::{error, info};
use uuid::Uuid;

use crate::core::application_error::ApplicationError as AppError;
use crate::core::commands::auth::Login;
use crate::core::contracts::repository::error::RepositoryError;
use crate::core::entities::auth::ClientMetadata;
use crate::core::read_models::work_sessions::WorkSessionWithMemberDetails;
use crate::core::responses::auth::LoginResponse;
use crate::core::value_objects::profiles::Profile;
use crate::usecases::auth::deps::AuthUseCaseDependencies;
use crate::usecases::auth::helpers::{
    AccessTokenSubject, build_new_refresh_token, issue_access_token,
};

pub struct LoginUseCase {
    deps: AuthUseCaseDependencies,
}

impl LoginUseCase {
    pub fn new(deps: AuthUseCaseDependencies) -> Self {
        Self { deps }
    }

    pub async fn execute(
        &self,
        data: Login,
        metadata: ClientMetadata,
    ) -> Result<LoginResponse, AppError> {
        let normalized_email = data.email.trim().to_lowercase();
        info!(
            "[LoginUseCase] Starting login process with email: {}",
            normalized_email
        );

        let user = match self
            .deps
            .auth_repository
            .get_complete_user_data_by_email(&normalized_email)
            .await
        {
            Ok(user) => user,
            Err(RepositoryError::NotFound) => {
                return Err(AppError::Unauthorized("Invalid credentials".into()));
            }
            Err(error) => {
                error!(
                    "[LoginUseCase] Database error while finding user {}: {:?}",
                    normalized_email, error
                );
                return Err(AppError::InternalServerError);
            }
        };

        let password_matches = self
            .deps
            .password_hasher
            .verify_password(&user.password, &data.password)
            .map_err(|_| AppError::InternalServerError)?;

        if !password_matches {
            return Err(AppError::Unauthorized("Invalid credentials".into()));
        }

        if user.is_temporary_password {
            match user.temporary_password_expires_at {
                Some(expires_at) if Utc::now() <= expires_at => {}
                _ => return Err(AppError::Unauthorized("Temporary password expired".into())),
            }
        }

        let (access_token, expires_in) = issue_access_token(
            &*self.deps.token_generator,
            &self.deps.config,
            AccessTokenSubject {
                id: user.id,
                rank: &user.rank,
                registration: &user.registration,
                full_name: &user.full_name,
                profile: &user.profile,
                email: &user.email,
                city_id: user.city_id,
            },
        )?;

        let (new_refresh, refresh_token) = build_new_refresh_token(
            &*self.deps.password_hasher,
            user.id,
            self.deps.config.refresh_token_ttl_seconds,
            &metadata,
        )?;

        self.deps
            .refresh_token_repository
            .create_refresh_token(new_refresh)
            .await
            .map_err(|error| {
                error!(
                    "[LoginUseCase] Failed to persist refresh token for {}: {:?}",
                    user.id, error
                );
                AppError::InternalServerError
            })?;

        let work_session = if data.auto_create_session {
            self.resolve_login_work_session(user.id, &user.profile, user.city_id)
                .await?
        } else {
            None
        };

        Ok(LoginResponse {
            access_token,
            refresh_token,
            token_type: "Bearer".to_string(),
            expires_in,
            id: user.id,
            full_name: user.full_name,
            email: user.email,
            rank: user.rank,
            registration: user.registration,
            profile: user.profile,
            work_session,
        })
    }

    async fn resolve_login_work_session(
        &self,
        user_id: Uuid,
        profile: &Profile,
        city_id: Option<Uuid>,
    ) -> Result<Option<WorkSessionWithMemberDetails>, AppError> {
        info!(
            "[LoginUseCase] Resolving login work session for user: {}",
            user_id
        );

        if *profile != Profile::Root && city_id.is_none() {
            return Err(AppError::Forbidden(
                "User must be associated with a city".to_string(),
            ));
        }

        match self
            .deps
            .work_session_read_repository
            .is_user_in_active_session(user_id)
            .await
        {
            Ok(true) => match self
                .deps
                .work_session_read_repository
                .get_active_session_by_user(user_id)
                .await
            {
                Ok(session) => self
                    .deps
                    .work_session_read_repository
                    .get_session_by_id(session.id)
                    .await
                    .map(Some)
                    .or_else(|error| {
                        error!(
                            "[LoginUseCase] Failed to get active session details for {}: {:?}",
                            user_id, error
                        );
                        Ok(None)
                    }),
                Err(error) => {
                    error!(
                        "[LoginUseCase] Failed to get active session for {}: {:?}",
                        user_id, error
                    );
                    Ok(None)
                }
            },
            Ok(false) => {
                let session_id = Uuid::new_v4();
                let session_member_registration_id = Uuid::new_v4();
                match self
                    .deps
                    .work_session_write_repository
                    .create_auto_login_session(session_id, session_member_registration_id, user_id)
                    .await
                {
                    Ok(session) => self
                        .deps
                        .work_session_read_repository
                        .get_session_by_id(session.id)
                        .await
                        .map(Some)
                        .or_else(|error| {
                            error!(
                                "[LoginUseCase] Failed to get created session details for {}: {:?}",
                                user_id, error
                            );
                            Ok(None)
                        }),
                    Err(RepositoryError::Conflict(_)) => {
                        error!(
                            "[LoginUseCase] User already has an active work session: {}",
                            user_id
                        );
                        Err(AppError::Conflict(
                            "User already has an active work session".to_string(),
                        ))
                    }
                    Err(error) => {
                        error!(
                            "[LoginUseCase] Failed to create work session for {}: {:?}",
                            user_id, error
                        );
                        Err(AppError::InternalServerError)
                    }
                }
            }
            Err(error) => {
                error!(
                    "[LoginUseCase] Failed to check active session for {}: {:?}",
                    user_id, error
                );
                Ok(None)
            }
        }
    }
}
