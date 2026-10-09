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
    /// Max requests per minute per client IP on the /auth endpoints.
    /// 0 disables rate limiting (used by tests).
    pub login_rate_limit_per_minute: u64,
    pub json_payload_limit_bytes: usize,
    pub client_request_timeout_seconds: u64,
    pub keep_alive_seconds: u64,
    pub lgpd_retention_policy_days: Option<u32>,
    pub audit_log_retention_days: Option<u32>,
    pub retention_enforcement_enabled: bool,
}

#[derive(Debug)]
pub struct ConfigError {
    pub missing_vars: Vec<String>,
    pub invalid_vars: Vec<String>,
}

impl fmt::Display for ConfigError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut parts = Vec::new();
        if !self.missing_vars.is_empty() {
            parts.push(format!(
                "missing required environment variables: {}",
                self.missing_vars.join(", ")
            ));
        }
        if !self.invalid_vars.is_empty() {
            parts.push(format!(
                "invalid environment variables: {}",
                self.invalid_vars.join(", ")
            ));
        }
        write!(f, "Configuration error: {}", parts.join("; "))
    }
}

impl Config {
    pub fn from_env() -> Result<Self, ConfigError> {
        let mut missing = Vec::new();
        let mut invalid = Vec::new();

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

        if jwt_secret.len() < 32 {
            invalid.push("JWT_SECRET must be at least 32 bytes".to_string());
        }
        if api_key.len() < 32 {
            invalid.push("API_KEY must be at least 32 bytes".to_string());
        }

        if !missing.is_empty() || !invalid.is_empty() {
            return Err(ConfigError {
                missing_vars: missing,
                invalid_vars: invalid,
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

        let login_rate_limit_per_minute = env::var("LOGIN_RATE_LIMIT_PER_MINUTE")
            .unwrap_or_else(|_| "5".to_string())
            .parse::<u64>()
            .unwrap_or(5);
        let json_payload_limit_bytes = env::var("JSON_PAYLOAD_LIMIT_BYTES")
            .unwrap_or_else(|_| "1048576".to_string())
            .parse::<usize>()
            .unwrap_or(1048576);
        let client_request_timeout_seconds = env::var("CLIENT_REQUEST_TIMEOUT_SECONDS")
            .unwrap_or_else(|_| "15".to_string())
            .parse::<u64>()
            .unwrap_or(15);
        let keep_alive_seconds = env::var("KEEP_ALIVE_SECONDS")
            .unwrap_or_else(|_| "75".to_string())
            .parse::<u64>()
            .unwrap_or(75);
        let lgpd_retention_policy_days = parse_optional_u32("LGPD_RETENTION_POLICY_DAYS");
        let audit_log_retention_days = parse_optional_u32("AUDIT_LOG_RETENTION_DAYS");
        let retention_enforcement_enabled = env::var("RETENTION_ENFORCEMENT_ENABLED")
            .map(|value| matches!(value.to_lowercase().as_str(), "1" | "true" | "yes" | "on"))
            .unwrap_or(false);

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
            login_rate_limit_per_minute,
            json_payload_limit_bytes,
            client_request_timeout_seconds,
            keep_alive_seconds,
            lgpd_retention_policy_days,
            audit_log_retention_days,
            retention_enforcement_enabled,
        })
    }
}

fn parse_optional_u32(name: &str) -> Option<u32> {
    env::var(name)
        .ok()
        .and_then(|value| value.trim().parse::<u32>().ok())
}
