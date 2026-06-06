use std::env;
use std::fmt;

#[derive(Clone)]
pub struct Config {
    pub database_url: String,
    pub server_addr: String,
    pub jwt_secret: String,
    pub jwt_issuer: String,
    pub jwt_audience: String,
    pub api_key: String,
    pub db_max_connections: u32,
    pub enable_bootstrap_root: bool,
    pub run_migrations_on_startup: bool,
    pub access_token_ttl_seconds: i64,
    pub refresh_token_ttl_seconds: i64,
}

#[derive(Debug)]
pub struct ConfigError {
    pub missing_vars: Vec<String>,
}

impl fmt::Display for ConfigError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Missing required environment variables: {}",
            self.missing_vars.join(", ")
        )
    }
}

impl Config {
    pub fn from_env() -> Result<Self, ConfigError> {
        let mut missing = Vec::new();

        let database_url = env::var("DATABASE_URL").unwrap_or_else(|_| {
            missing.push("DATABASE_URL".to_string());
            String::new()
        });
        let server_addr = env::var("SERVER_ADDR").unwrap_or_else(|_| {
            missing.push("SERVER_ADDR".to_string());
            String::new()
        });
        let jwt_secret = env::var("JWT_SECRET").unwrap_or_else(|_| {
            missing.push("JWT_SECRET".to_string());
            String::new()
        });
        let jwt_issuer = env::var("JWT_ISSUER").unwrap_or_else(|_| {
            missing.push("JWT_ISSUER".to_string());
            String::new()
        });
        let jwt_audience = env::var("JWT_AUDIENCE").unwrap_or_else(|_| {
            missing.push("JWT_AUDIENCE".to_string());
            String::new()
        });
        let api_key = env::var("API_KEY").unwrap_or_else(|_| {
            missing.push("API_KEY".to_string());
            String::new()
        });

        if !missing.is_empty() {
            return Err(ConfigError {
                missing_vars: missing,
            });
        }

        let db_max_connections = env::var("DB_MAX_CONNECTIONS")
            .unwrap_or_else(|_| "20".to_string())
            .parse::<u32>()
            .unwrap_or(20);
        let enable_bootstrap_root = env::var("ENABLE_BOOTSTRAP_ROOT")
            .map(|value| matches!(value.to_lowercase().as_str(), "1" | "true" | "yes" | "on"))
            .unwrap_or(false);
        let run_migrations_on_startup = env::var("RUN_MIGRATIONS_ON_STARTUP")
            .map(|value| matches!(value.to_lowercase().as_str(), "1" | "true" | "yes" | "on"))
            .unwrap_or(true);

        let access_token_ttl_seconds = env::var("ACCESS_TOKEN_TTL_SECONDS")
            .unwrap_or_else(|_| "900".to_string())
            .parse::<i64>()
            .unwrap_or(900);
        let refresh_token_ttl_seconds = env::var("REFRESH_TOKEN_TTL_SECONDS")
            .unwrap_or_else(|_| "604800".to_string())
            .parse::<i64>()
            .unwrap_or(604800);

        Ok(Self {
            database_url,
            server_addr,
            jwt_secret,
            jwt_issuer,
            jwt_audience,
            api_key,
            db_max_connections,
            enable_bootstrap_root,
            run_migrations_on_startup,
            access_token_ttl_seconds,
            refresh_token_ttl_seconds,
        })
    }
}
