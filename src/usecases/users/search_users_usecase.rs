use log::{error, info};

use crate::core::application_error::ApplicationError as AppError;
use crate::core::entities::auth::UserClaims;
use crate::core::entities::users::UserRecord;
use crate::usecases::users::deps::UserUseCaseDependencies;
use crate::usecases::users::helpers::{build_user_read_scope, filter_users_by_scope};
use crate::usecases::users::search_criteria::UserSearchCriteria;

pub struct SearchUsersUseCase {
    deps: UserUseCaseDependencies,
}

impl SearchUsersUseCase {
    pub fn new(deps: UserUseCaseDependencies) -> Self {
        Self { deps }
    }

    pub async fn execute(
        &self,
        name: Option<String>,
        registration: Option<String>,
        claims: &UserClaims,
    ) -> Result<Vec<UserRecord>, AppError> {
        info!("[SearchUsersUseCase] Starting user search");

        let search = UserSearchCriteria::parse(name, registration)?;
        let scope = build_user_read_scope(self.deps.user_repository.as_ref(), claims).await?;

        let users = match search {
            UserSearchCriteria::ByName(name) => {
                self.deps.user_repository.get_users_by_name(&name).await
            }
            UserSearchCriteria::ByRegistration(registration) => {
                self.deps
                    .user_repository
                    .get_users_by_registration(&registration)
                    .await
            }
        };

        match users {
            Ok(users) => Ok(filter_users_by_scope(users, &scope)),
            Err(error) => {
                error!("[SearchUsersUseCase] Failed to search users: {:?}", error);
                Err(AppError::InternalServerError)
            }
        }
    }
}
