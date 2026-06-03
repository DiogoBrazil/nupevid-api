use crate::core::errors::DomainError;
use crate::core::value_objects::profiles::Profile;
use crate::core::value_objects::ranks::Rank;

pub struct TokenClaimsInput<'a> {
    pub id: &'a str,
    pub rank: &'a Rank,
    pub registration: &'a str,
    pub full_name: &'a str,
    pub profile: &'a Profile,
    pub email: &'a str,
    pub city_id: Option<&'a str>,
    pub issuer: &'a str,
    pub audience: &'a str,
    pub expires_in_seconds: i64,
}

pub trait TokenGeneratorPort: Send + Sync {
    fn generate_token(
        &self,
        claims: TokenClaimsInput<'_>,
        secret: &str,
    ) -> Result<String, DomainError>;
}
