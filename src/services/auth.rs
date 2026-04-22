use actix_web::{HttpResponse, web};
use chrono::Utc;
use log::info;
use uuid::Uuid;

use crate::adapters::password_hasher::PasswordHasherPort;
use crate::adapters::token_generator::TokenGeneratorPort;
use crate::config::config_env::Config;
use crate::core::contracts::adapters::token_generator::TokenClaimsInput;
use crate::core::contracts::repository::auth::AuthRepository;
use crate::core::contracts::repository::work_sessions::WorkSessionRepository;
use crate::core::entities::auth::{Login, LoginResponse};
use crate::repositories::auth::PgAuthRepository;
use crate::repositories::work_sessions::PgWorkSessionRepository;
use crate::utils::errors::AppError;
use crate::utils::responses::ApiResponse;
use crate::validators::common::PROFILE_ROOT;
pub struct AuthService {
    auth_repository: web::Data<PgAuthRepository>,
    work_session_repository: web::Data<PgWorkSessionRepository>,
    config: web::Data<Config>,
    password_hasher: Box<dyn PasswordHasherPort>,
    token_generator: Box<dyn TokenGeneratorPort>,
}

impl AuthService {
    pub fn new(
        auth_repository: web::Data<PgAuthRepository>,
        work_session_repository: web::Data<PgWorkSessionRepository>,
        config: web::Data<Config>,
        password_hasher: Box<dyn PasswordHasherPort>,
        token_generator: Box<dyn TokenGeneratorPort>,
    ) -> Self {
        Self {
            auth_repository,
            work_session_repository,
            config,
            password_hasher,
            token_generator,
        }
    }

    pub async fn login(&self, data: Login) -> Result<HttpResponse, AppError> {
        let normalized_email = data.email.trim().to_lowercase();
        info!(
            "[Service] Starting login process with email: {}",
            normalized_email
        );

        info!(
            "[Service] Checking if user exists with email: {}",
            normalized_email
        );
        let user = match self
            .auth_repository
            .get_complete_user_data_by_email(&normalized_email)
            .await
        {
            Ok(user) => {
                info!("[Service] User found with email: {}", normalized_email);
                user
            }
            Err(sqlx::Error::RowNotFound) => {
                info!("[Service] User not found with email: {}", normalized_email);
                return Err(AppError::Unauthorized("Invalid credentials".into()));
            }
            Err(e) => {
                info!("[Service] Database error while finding user: {:?}", e);
                return Err(AppError::InternalServerError);
            }
        };

        info!(
            "[Service] Verifying password for user with email: {}",
            normalized_email
        );
        if !self
            .password_hasher
            .verify_password(&user.password, &data.password)
            .map_err(|_| AppError::InternalServerError)?
        {
            info!(
                "[Service] Incorrect password for user with email: {}",
                normalized_email
            );
            return Err(AppError::Unauthorized("Invalid credentials".into()));
        }
        info!(
            "[Service] Password verified successfully for user with email: {}",
            normalized_email
        );

        if user.is_temporary_password {
            match user.temporary_password_expires_at {
                Some(expires_at) => {
                    if Utc::now() > expires_at {
                        info!(
                            "[Service] Temporary password expired for user with email: {}",
                            normalized_email
                        );
                        return Err(AppError::Unauthorized("Temporary password expired".into()));
                    }
                }
                None => {
                    info!(
                        "[Service] Temporary password missing expiration for user with email: {}",
                        normalized_email
                    );
                    return Err(AppError::Unauthorized("Temporary password expired".into()));
                }
            }
        }

        info!(
            "[Service] Generating token for user with email: {}",
            normalized_email
        );
        let user_id_str = user.id.to_string();
        let email_str = user.email.to_string();
        let city_id_str = user.city_id.map(|id| id.to_string());
        let token = self
            .token_generator
            .generate_token(
                TokenClaimsInput {
                    id: &user_id_str,
                    rank: &user.rank,
                    registration: &user.registration,
                    full_name: &user.full_name,
                    profile: &user.profile,
                    email: &email_str,
                    city_id: city_id_str.as_deref(),
                },
                &self.config.jwt_secret,
            )
            .map_err(|_| AppError::InternalServerError)?;

        info!(
            "[Service] Token generated successfully for user with email: {}",
            data.email
        );

        let work_session = if data.auto_create_session {
            info!("[Service] Auto-creating work session for user: {}", user.id);

            if user.profile != PROFILE_ROOT && user.city_id.is_none() {
                return Err(AppError::Forbidden(
                    "User must be associated with a city".to_string(),
                ));
            }

            match self
                .work_session_repository
                .is_user_in_active_session(user.id)
                .await
            {
                Ok(true) => {
                    match self
                        .work_session_repository
                        .get_active_session_by_user(user.id)
                        .await
                    {
                        Ok(session) => {
                            match self
                                .work_session_repository
                                .get_session_by_id(session.id)
                                .await
                            {
                                Ok(session_with_members) => {
                                    info!(
                                        "[Service] Returning existing active session: {}",
                                        session_with_members.id
                                    );
                                    Some(session_with_members)
                                }
                                Err(e) => {
                                    log::error!("[Service] Failed to get session details: {:?}", e);
                                    None
                                }
                            }
                        }
                        Err(e) => {
                            log::error!("[Service] Failed to get active session: {:?}", e);
                            None
                        }
                    }
                }
                Ok(false) => {
                    let session_id = Uuid::new_v4();
                    let session_member_registration_id = Uuid::new_v4();
                    match self
                        .work_session_repository
                        .create_auto_login_session(
                            session_id,
                            session_member_registration_id,
                            user.id,
                        )
                        .await
                    {
                        Ok(session) => {
                            info!("[Service] Work session created on login: {}", session.id);
                            Some(session)
                        }
                        Err(e) => {
                            log::error!(
                                "[Service] Failed to create work session on login: {:?}",
                                e
                            );
                            None
                        }
                    }
                }
                Err(e) => {
                    log::error!("[Service] Failed to check active session: {:?}", e);
                    None
                }
            }
        } else {
            None
        };

        let response = LoginResponse {
            token,
            id: user.id,
            full_name: user.full_name,
            email: user.email,
            rank: user.rank,
            registration: user.registration,
            profile: user.profile,
            work_session,
        };

        Ok(ApiResponse::success(response).into_response())
    }
}
