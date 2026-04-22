use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use chrono::Utc;
use uuid::Uuid;

use crate::core::contracts::adapters::password_hasher::PasswordHasherPort;
use crate::core::contracts::repository::users::MockUserRepository;
use crate::core::entities::auth::UserClaims;
use crate::core::entities::users::User;
use crate::core::errors::DomainError;
use crate::core::value_objects::policies::PermissionPolicies;
use crate::core::value_objects::profiles::Profile;
use crate::core::value_objects::ranks::Rank;
use crate::usecases::users::deps::UserUseCaseDependencies;

pub fn claims(profile: Profile, user_id: Uuid, city_id: Option<Uuid>) -> UserClaims {
    UserClaims {
        id: user_id.to_string(),
        exp: 9999999999,
        iss: "nupevid-api".to_string(),
        aud: "nupevid-api".to_string(),
        rank: Rank::CapPm,
        registration: "100099999".to_string(),
        full_name: "Operador Teste".to_string(),
        profile,
        email: "operador@test.com".to_string(),
        city_id: city_id.map(|c| c.to_string()),
    }
}

pub fn user_record(
    id: Uuid,
    profile: Profile,
    city_id: Option<Uuid>,
    policies: PermissionPolicies,
) -> User {
    let now = Utc::now();
    User {
        id,
        rank: Rank::CapPm,
        registration: "100000000".to_string(),
        full_name: "USUARIO TESTE".to_string(),
        profile,
        email: "teste@nupevid.dev".to_string(),
        city_id,
        permission_policies: policies,
        created_at: now,
        updated_at: now,
        is_deleted: false,
    }
}

pub struct FakePasswordHasher {
    pub fail_on_hash: bool,
    pub verify_result: Option<bool>,
    pub verify_error: bool,
    pub hashed: Mutex<Vec<String>>,
}

impl FakePasswordHasher {
    pub fn ok() -> Self {
        Self {
            fail_on_hash: false,
            verify_result: Some(true),
            verify_error: false,
            hashed: Mutex::new(Vec::new()),
        }
    }

    pub fn failing() -> Self {
        Self {
            fail_on_hash: true,
            verify_result: None,
            verify_error: false,
            hashed: Mutex::new(Vec::new()),
        }
    }
}

impl PasswordHasherPort for FakePasswordHasher {
    fn hash_password(&self, password: &str) -> Result<String, DomainError> {
        if self.fail_on_hash {
            return Err(DomainError::Internal("hash failed".to_string()));
        }
        let hashed = format!("hashed:{}", password);
        self.hashed.lock().unwrap().push(hashed.clone());
        Ok(hashed)
    }

    fn verify_password(&self, _hash: &str, _password: &str) -> Result<bool, DomainError> {
        if self.verify_error {
            return Err(DomainError::Internal("verify failed".to_string()));
        }
        Ok(self.verify_result.unwrap_or(false))
    }
}

pub fn deps(user_repo: MockUserRepository, hasher: FakePasswordHasher) -> UserUseCaseDependencies {
    UserUseCaseDependencies::new(Arc::new(user_repo), Arc::new(hasher))
}

pub fn empty_policies() -> PermissionPolicies {
    HashMap::new()
}
