use crate::{
    config::AuthConfig,
    error::{AppError, AppResult},
    pass::PassInterface,
};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use totp_lite::{totp, Sha1};

#[derive(Debug, Clone)]
pub struct AuthService {
    config: AuthConfig,
    pass: Arc<PassInterface>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct LoginRequest {
    pub master_password: String,
    pub totp_code: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct LoginResponse {
    pub session_id: String,
    pub expires_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AuthStatus {
    pub authenticated: bool,
    pub user_id: Option<String>,
    pub expires_at: Option<DateTime<Utc>>,
}

impl AuthService {
    pub fn new(config: AuthConfig, pass: Arc<PassInterface>) -> Self {
        Self { config, pass }
    }

    pub async fn authenticate(&self, request: LoginRequest) -> AppResult<LoginResponse> {
        // Verify master password
        self.verify_master_password(&request.master_password).await?;
        
        // Verify TOTP code
        self.verify_totp_code(&request.totp_code).await?;
        
        // Generate session
        let session_id = uuid::Uuid::new_v4().to_string();
        let expires_at = Utc::now() + chrono::Duration::hours(self.config.session_timeout_hours as i64);
        
        Ok(LoginResponse {
            session_id,
            expires_at,
        })
    }

    async fn verify_master_password(&self, provided_password: &str) -> AppResult<()> {
        // Get the stored master password from pass
        let stored_password = self.pass
            .get_password(&self.config.master_password_path)
            .await?
            .password;
        
        if provided_password != stored_password {
            return Err(AppError::AuthenticationFailed(
                "Invalid master password".to_string(),
            ));
        }
        
        Ok(())
    }

    async fn verify_totp_code(&self, provided_code: &str) -> AppResult<()> {
        // Get the TOTP secret from pass
        let totp_entry = self.pass
            .get_password(&self.config.totp_path)
            .await?;
        
        let secret = totp_entry.password;
        
        // Decode the base32 secret
        let secret_bytes = base32::decode(base32::Alphabet::RFC4648 { padding: true }, &secret)
            .ok_or_else(|| AppError::AuthenticationFailed("Invalid TOTP secret".to_string()))?;
        
        // Generate current TOTP code  
        let current_time = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs();
        
        let current_code = totp::<Sha1>(&secret_bytes, current_time / 30);
        
        // Also check the previous and next time windows for clock skew
        let prev_code = totp::<Sha1>(&secret_bytes, (current_time - 30) / 30);
        let next_code = totp::<Sha1>(&secret_bytes, (current_time + 30) / 30);
        
        if provided_code != current_code && provided_code != prev_code && provided_code != next_code {
            return Err(AppError::AuthenticationFailed(
                "Invalid TOTP code".to_string(),
            ));
        }
        
        Ok(())
    }

    pub fn extract_session_from_header(&self, auth_header: Option<&str>) -> Option<String> {
        auth_header
            .and_then(|header| header.strip_prefix("Bearer "))
            .map(|token| token.to_string())
    }

    pub async fn get_auth_status(&self, session_id: Option<String>) -> AuthStatus {
        match session_id {
            Some(id) if !id.is_empty() => {
                // In a real implementation, you'd check the session store
                // For now, we'll assume any non-empty session ID is valid
                AuthStatus {
                    authenticated: true,
                    user_id: Some("user".to_string()),
                    expires_at: Some(Utc::now() + chrono::Duration::hours(24)),
                }
            }
            _ => AuthStatus {
                authenticated: false,
                user_id: None,
                expires_at: None,
            },
        }
    }
}

// Add required dependencies to Cargo.toml:
// totp-lite = "2.0"
// base32 = "0.4"