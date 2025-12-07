use jsonwebtoken::errors::Error as JwtError;

pub trait TokenGeneratorPort: Send + Sync {
    fn generate_token(
        &self,
        id: &str,
        rank: &str,
        registration: &str,
        full_name: &str,
        profile: &str,
        email: &str,
        city_id: Option<&str>,
        secret: &str,
    ) -> Result<String, JwtError>;
}
