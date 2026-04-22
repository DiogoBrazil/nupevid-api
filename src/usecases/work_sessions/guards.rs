use uuid::Uuid;

use crate::core::application_error::ApplicationError;
use crate::core::contracts::repository::error::RepositoryError;
use crate::core::contracts::repository::users::UserRepository;
use crate::core::entities::work_session_members::{TeamMemberFunction, WorkSessionMember};

pub async fn validate_members_same_city(
    user_repository: &dyn UserRepository,
    user_ids: &[Uuid],
    allow_cityless: bool,
) -> Result<(), ApplicationError> {
    if user_ids.is_empty() {
        return Ok(());
    }

    let mut city_id: Option<Uuid> = None;

    for user_id in user_ids {
        let user = user_repository
            .get_user_by_id(*user_id)
            .await
            .map_err(|error| match error {
                RepositoryError::NotFound => {
                    ApplicationError::NotFound(format!("User {} not found", user_id))
                }
                _ => ApplicationError::InternalServerError,
            })?;

        let user_city_id = match user.city_id {
            Some(city_id) => city_id,
            None => {
                if allow_cityless {
                    continue;
                }
                return Err(ApplicationError::BadRequest(format!(
                    "User {} is not associated with a city",
                    user_id
                )));
            }
        };

        match city_id {
            None => city_id = Some(user_city_id),
            Some(expected_city_id) if user_city_id != expected_city_id => {
                return Err(ApplicationError::BadRequest(
                    "All team members must be from the same city".to_string(),
                ));
            }
            Some(_) => {}
        }
    }

    Ok(())
}

pub fn ensure_creator_or_commander(
    created_by_user_id: Uuid,
    requesting_user_id: Uuid,
    current_members: &[WorkSessionMember],
    error_message: &str,
) -> Result<(), ApplicationError> {
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

    Err(ApplicationError::Forbidden(error_message.to_string()))
}
