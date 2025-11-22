use jsonwebtoken::errors::Error as JwtError;

pub trait TokenGeneratorPort: Send + Sync {
    fn generate_token(
        &self,
        id: String,
        rank: String,
        registration: String,
        full_name: String,
        profile: String,
        email: String,
        city_id: Option<String>,
        secret: &str,
    ) -> Result<String, JwtError>;
}
