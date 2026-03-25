use crate::core::errors::DomainError;

pub struct TokenClaimsInput<'a> {
    pub id: &'a str,
    pub rank: &'a str,
    pub registration: &'a str,
    pub full_name: &'a str,
    pub profile: &'a str,
    pub email: &'a str,
    pub city_id: Option<&'a str>,
}

pub trait TokenGeneratorPort: Send + Sync {
    fn generate_token(
        &self,
        claims: TokenClaimsInput<'_>,
        secret: &str,
    ) -> Result<String, DomainError>;
}
