use crate::error::{AppError, AppResult};
use clap::Parser;
use serde::{Deserialize, Serialize};
use std::{env, path::PathBuf};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub server: ServerConfig,
    pub git: GitConfig,
    pub auth: AuthConfig,
    pub database: DatabaseConfig,
    pub pass: PassConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerConfig {
    pub port: u16,
    pub host: String,
    pub log_level: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GitConfig {
    pub repo_url: String,
    pub access_token: String,
    pub sync_interval_minutes: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthConfig {
    pub master_password_path: String,
    pub totp_path: String,
    pub session_timeout_hours: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DatabaseConfig {
    pub url: String,
    pub encryption_key: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PassConfig {
    pub store_dir: PathBuf,
    pub gpg_key_id: Option<String>,
}

impl Config {
    pub fn load(config_path: Option<&str>) -> AppResult<Self> {
        // Load from environment variables first
        dotenvy::dotenv().ok();
        
        let config = Config {
            server: ServerConfig {
                port: env::var("PORT")
                    .unwrap_or_else(|_| "8080".to_string())
                    .parse()
                    .map_err(|e| AppError::ConfigError(format!("Invalid PORT: {}", e)))?,
                host: env::var("HOST").unwrap_or_else(|_| "0.0.0.0".to_string()),
                log_level: env::var("LOG_LEVEL").unwrap_or_else(|_| "info".to_string()),
            },
            git: GitConfig {
                repo_url: env::var("GIT_REPO_URL")
                    .map_err(|_| AppError::ConfigError("GIT_REPO_URL is required".to_string()))?,
                access_token: env::var("GIT_ACCESS_TOKEN")
                    .map_err(|_| AppError::ConfigError("GIT_ACCESS_TOKEN is required".to_string()))?,
                sync_interval_minutes: env::var("SYNC_INTERVAL_MINUTES")
                    .unwrap_or_else(|_| "5".to_string())
                    .parse()
                    .map_err(|e| AppError::ConfigError(format!("Invalid SYNC_INTERVAL_MINUTES: {}", e)))?,
            },
            auth: AuthConfig {
                master_password_path: env::var("MASTER_PASSWORD_PATH")
                    .unwrap_or_else(|_| "kagikanri/master-password".to_string()),
                totp_path: env::var("TOTP_PATH")
                    .unwrap_or_else(|_| "kagikanri/totp".to_string()),
                session_timeout_hours: env::var("SESSION_TIMEOUT_HOURS")
                    .unwrap_or_else(|_| "24".to_string())
                    .parse()
                    .map_err(|e| AppError::ConfigError(format!("Invalid SESSION_TIMEOUT_HOURS: {}", e)))?,
            },
            database: DatabaseConfig {
                url: env::var("DATABASE_URL")
                    .unwrap_or_else(|_| "sqlite:///data/passkeys.db".to_string()),
                encryption_key: env::var("DATABASE_ENCRYPTION_KEY")
                    .map_err(|_| AppError::ConfigError("DATABASE_ENCRYPTION_KEY is required".to_string()))?,
            },
            pass: PassConfig {
                store_dir: env::var("PASSWORD_STORE_DIR")
                    .unwrap_or_else(|_| "/data/password-store".to_string())
                    .into(),
                gpg_key_id: env::var("GPG_KEY_ID").ok(),
            },
        };

        // If a config file path is provided, try to load and merge it
        if let Some(path) = config_path {
            // TODO: Implement config file loading
            tracing::warn!("Config file loading not yet implemented, using environment variables only");
        }

        // Validate configuration
        config.validate()?;

        Ok(config)
    }

    fn validate(&self) -> AppResult<()> {
        // Validate Git repo URL
        if !self.git.repo_url.starts_with("http") && !self.git.repo_url.starts_with("git@") {
            return Err(AppError::ConfigError(
                "GIT_REPO_URL must be a valid HTTP or SSH URL".to_string(),
            ));
        }

        // Validate database encryption key length (should be 32 bytes in hex = 64 chars)
        if self.database.encryption_key.len() != 64 {
            return Err(AppError::ConfigError(
                "DATABASE_ENCRYPTION_KEY must be 32 bytes in hexadecimal format (64 characters)".to_string(),
            ));
        }

        // Ensure password store directory is absolute
        if !self.pass.store_dir.is_absolute() {
            return Err(AppError::ConfigError(
                "PASSWORD_STORE_DIR must be an absolute path".to_string(),
            ));
        }

        Ok(())
    }
}

impl Default for Config {
    fn default() -> Self {
        Config {
            server: ServerConfig {
                port: 8080,
                host: "0.0.0.0".to_string(),
                log_level: "info".to_string(),
            },
            git: GitConfig {
                repo_url: "".to_string(),
                access_token: "".to_string(),
                sync_interval_minutes: 5,
            },
            auth: AuthConfig {
                master_password_path: "kagikanri/master-password".to_string(),
                totp_path: "kagikanri/totp".to_string(),
                session_timeout_hours: 24,
            },
            database: DatabaseConfig {
                url: "sqlite:///data/passkeys.db".to_string(),
                encryption_key: "".to_string(),
            },
            pass: PassConfig {
                store_dir: "/data/password-store".into(),
                gpg_key_id: None,
            },
        }
    }
}