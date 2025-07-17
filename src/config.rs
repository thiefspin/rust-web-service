use once_cell::sync::Lazy;
use serde::Deserialize;
use std::env;

#[derive(Debug, Clone, Deserialize)]
pub struct Config {
    pub database_url: String,
    pub jwt_secret: String,
    pub jwt_expiration: i64, // in seconds
    pub server_host: String,
    pub server_port: u16,
    pub bcrypt_cost: u32,
    pub log_level: String,
}

impl Config {
    pub fn from_env() -> Result<Self, env::VarError> {
        Ok(Config {
            database_url: env::var("DATABASE_URL")?,
            jwt_secret: env::var("JWT_SECRET")?,
            jwt_expiration: env::var("JWT_EXPIRATION")
                .unwrap_or_else(|_| "3600".to_string()) // 1 hour default
                .parse()
                .unwrap_or(3600),
            server_host: env::var("SERVER_HOST").unwrap_or_else(|_| "127.0.0.1".to_string()),
            server_port: env::var("SERVER_PORT")
                .unwrap_or_else(|_| "8080".to_string())
                .parse()
                .unwrap_or(8080),
            bcrypt_cost: env::var("BCRYPT_COST")
                .unwrap_or_else(|_| "12".to_string())
                .parse()
                .unwrap_or(12),
            log_level: env::var("LOG_LEVEL").unwrap_or_else(|_| "info".to_string()),
        })
    }

    pub fn jwt_secret_bytes(&self) -> &[u8] {
        self.jwt_secret.as_bytes()
    }

    pub fn server_address(&self) -> String {
        format!("{}:{}", self.server_host, self.server_port)
    }
}

pub static CONFIG: Lazy<Config> = Lazy::new(|| {
    dotenv::dotenv().ok();
    Config::from_env().expect("Failed to load configuration from environment variables")
});

// Validation functions
impl Config {
    pub fn validate(&self) -> Result<(), String> {
        if self.database_url.is_empty() {
            return Err("DATABASE_URL cannot be empty".to_string());
        }

        if self.jwt_secret.len() < 32 {
            return Err("JWT_SECRET must be at least 32 characters long".to_string());
        }

        if self.jwt_expiration <= 0 {
            return Err("JWT_EXPIRATION must be positive".to_string());
        }

        if self.bcrypt_cost < 4 || self.bcrypt_cost > 31 {
            return Err("BCRYPT_COST must be between 4 and 31".to_string());
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_config_validation() {
        let valid_config = Config {
            database_url: "postgresql://user:pass@localhost/db".to_string(),
            jwt_secret: "this_is_a_very_long_secret_key_for_jwt_tokens".to_string(),
            jwt_expiration: 3600,
            server_host: "127.0.0.1".to_string(),
            server_port: 8080,
            bcrypt_cost: 12,
            log_level: "info".to_string(),
        };

        assert!(valid_config.validate().is_ok());

        let invalid_config = Config {
            database_url: "".to_string(),
            jwt_secret: "short".to_string(),
            jwt_expiration: -1,
            server_host: "127.0.0.1".to_string(),
            server_port: 8080,
            bcrypt_cost: 2,
            log_level: "info".to_string(),
        };

        assert!(invalid_config.validate().is_err());
    }

    #[test]
    fn test_server_address() {
        let config = Config {
            database_url: "postgresql://user:pass@localhost/db".to_string(),
            jwt_secret: "this_is_a_very_long_secret_key_for_jwt_tokens".to_string(),
            jwt_expiration: 3600,
            server_host: "0.0.0.0".to_string(),
            server_port: 3000,
            bcrypt_cost: 12,
            log_level: "info".to_string(),
        };

        assert_eq!(config.server_address(), "0.0.0.0:3000");
    }
}
