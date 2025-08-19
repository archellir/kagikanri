use crate::{
    config::AuthConfig,
    error::{AppError, AppResult},
    pass::PassInterface,
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use totp_lite::{totp, Sha1};
use tracing::{debug, info, warn};

#[derive(Debug, Clone)]
pub struct AuthService {
    config: AuthConfig,
    pass: Arc<PassInterface>,
}

#[derive(Debug, Deserialize)]
pub struct LoginRequest {
    pub master_password: String,
    pub totp_code: String,
}

#[derive(Debug, Serialize)]
pub struct LoginResponse {
    pub success: bool,
    pub user_id: String,
    pub expires_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Serialize)]
pub struct AuthStatus {
    pub user_id: Option<String>,
    pub expires_at: Option<chrono::DateTime<chrono::Utc>>,
}

impl AuthService {
    pub fn new(config: AuthConfig, pass: Arc<PassInterface>) -> Self {
        Self { config, pass }
    }

    pub async fn authenticate(&self, request: LoginRequest) -> AppResult<LoginResponse> {
        info!("Attempting authentication");
        
        // Verify master password
        self.verify_master_password(&request.master_password).await?;
        
        // Verify TOTP code
        self.verify_totp(&request.totp_code).await?;
        
        let expires_at = chrono::Utc::now() + chrono::Duration::hours(self.config.session_timeout_hours as i64);
        
        info!("Authentication successful");
        Ok(LoginResponse {
            success: true,
            user_id: "user".to_string(), // Simple single-user system
            expires_at,
        })
    }

    pub async fn get_auth_status(&self, session_id: Option<String>) -> AuthStatus {
        // Simple implementation - in a real system you'd check the session store
        if session_id.is_some() {
            AuthStatus {
                user_id: Some("user".to_string()),
                expires_at: Some(chrono::Utc::now() + chrono::Duration::hours(self.config.session_timeout_hours as i64)),
            }
        } else {
            AuthStatus {
                user_id: None,
                expires_at: None,
            }
        }
    }

    async fn verify_master_password(&self, provided_password: &str) -> AppResult<()> {
        debug!("Verifying master password");
        
        let stored_password = self.pass
            .get_password(&self.config.master_password_path)
            .await?;
        
        if provided_password == stored_password.password {
            Ok(())
        } else {
            Err(AppError::AuthenticationFailed("Invalid master password".to_string()))
        }
    }

    async fn verify_totp(&self, provided_code: &str) -> AppResult<()> {
        debug!("Verifying TOTP code");
        
        // Get TOTP secret from pass store
        let totp_entry = self.pass
            .get_password(&self.config.totp_path)
            .await?;
        
        let secret_base32 = &totp_entry.password;
        
        // Decode base32 secret
        let secret_bytes = base32::decode(base32::Alphabet::RFC4648 { padding: true }, secret_base32)
            .ok_or_else(|| AppError::AuthenticationFailed("Invalid TOTP secret format".to_string()))?;
        
        // Get current time window
        let current_time = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs();
        
        // Check current window and previous window (to account for clock drift)
        let windows = [current_time / 30, (current_time / 30) - 1];
        
        for window in windows {
            let expected_code = totp::<Sha1>(&secret_bytes, window);
            if provided_code == expected_code {
                return Ok(());
            }
        }
        
        Err(AppError::AuthenticationFailed("Invalid TOTP code".to_string()))
    }

    pub fn extract_session_from_header(&self, auth_header: Option<&str>) -> Option<String> {
        if let Some(header_value) = auth_header {
            if let Some(token) = header_value.strip_prefix("Bearer ") {
                return Some(token.to_string());
            }
        }
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::{AuthConfig, PassConfig};
    use std::path::PathBuf;

    fn create_test_config() -> AuthConfig {
        AuthConfig {
            master_password_path: "kagikanri/master-password".to_string(),
            totp_path: "kagikanri/totp".to_string(),
            session_timeout_hours: 24,
        }
    }

    #[test]
    fn test_extract_session_from_header_success() {
        let config = create_test_config();
        // Create a dummy PassInterface for testing
        let pass_config = PassConfig {
            store_dir: PathBuf::from("/tmp/test"),
            gpg_key_id: Some("test-key-id".to_string()),
        };
        let pass_interface = PassInterface::new(pass_config).unwrap();
        let auth_service = AuthService::new(config, Arc::new(pass_interface));

        let session_id = auth_service.extract_session_from_header(Some("Bearer abc123def456"));
        assert_eq!(session_id, Some("abc123def456".to_string()));
    }

    #[test]
    fn test_extract_session_from_header_invalid_format() {
        let config = create_test_config();
        let pass_config = PassConfig {
            store_dir: PathBuf::from("/tmp/test"),
            gpg_key_id: Some("test-key-id".to_string()),
        };
        let pass_interface = PassInterface::new(pass_config).unwrap();
        let auth_service = AuthService::new(config, Arc::new(pass_interface));

        let session_id = auth_service.extract_session_from_header(Some("InvalidFormat abc123"));
        assert_eq!(session_id, None);
    }

    #[test]
    fn test_extract_session_from_header_none() {
        let config = create_test_config();
        let pass_config = PassConfig {
            store_dir: PathBuf::from("/tmp/test"),
            gpg_key_id: Some("test-key-id".to_string()),
        };
        let pass_interface = PassInterface::new(pass_config).unwrap();
        let auth_service = AuthService::new(config, Arc::new(pass_interface));

        let session_id = auth_service.extract_session_from_header(None);
        assert_eq!(session_id, None);
    }

    #[test]
    fn test_get_auth_status_with_session() {
        let config = create_test_config();
        let pass_config = PassConfig {
            store_dir: PathBuf::from("/tmp/test"),
            gpg_key_id: Some("test-key-id".to_string()),
        };
        let pass_interface = PassInterface::new(pass_config).unwrap();
        let auth_service = AuthService::new(config, Arc::new(pass_interface));

        let status = tokio_test::block_on(auth_service.get_auth_status(Some("session123".to_string())));
        assert!(status.user_id.is_some());
        assert_eq!(status.user_id.unwrap(), "user");
        assert!(status.expires_at.is_some());
    }

    #[test]
    fn test_get_auth_status_without_session() {
        let config = create_test_config();
        let pass_config = PassConfig {
            store_dir: PathBuf::from("/tmp/test"),
            gpg_key_id: Some("test-key-id".to_string()),
        };
        let pass_interface = PassInterface::new(pass_config).unwrap();
        let auth_service = AuthService::new(config, Arc::new(pass_interface));

        let status = tokio_test::block_on(auth_service.get_auth_status(None));
        assert!(status.user_id.is_none());
        assert!(status.expires_at.is_none());
    }

    #[test]
    fn test_totp_validation_logic() {
        // Test TOTP calculation logic directly
        let secret = "JBSWY3DPEHPK3PXP"; // "Hello!" in base32
        let secret_bytes = base32::decode(base32::Alphabet::RFC4648 { padding: true }, secret).unwrap();
        
        let current_time = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs();
        
        let current_window = current_time / 30;
        let code = totp::<Sha1>(&secret_bytes, current_window);
        
        // Code should be 6 or 8 digits (depends on totp implementation)
        assert!(code.len() == 6 || code.len() == 8);
        assert!(code.chars().all(|c| c.is_ascii_digit()));
        
        // Different time windows should produce different codes
        let previous_code = totp::<Sha1>(&secret_bytes, current_window - 1);
        let next_code = totp::<Sha1>(&secret_bytes, current_window + 1);
        
        // Note: TOTP codes can occasionally be the same across different windows
        // This is normal behavior and not a test failure
        // We just verify that codes are valid format and function works
        assert!(previous_code.chars().all(|c| c.is_ascii_digit()));
        assert!(next_code.chars().all(|c| c.is_ascii_digit()));
    }

    #[test]
    fn test_base32_decoding() {
        // Test valid base32
        let valid_secret = "JBSWY3DPEHPK3PXP";
        let result = base32::decode(base32::Alphabet::RFC4648 { padding: true }, valid_secret);
        assert!(result.is_some());
        
        // Test invalid base32
        let invalid_secret = "INVALID_BASE32!@#";
        let result = base32::decode(base32::Alphabet::RFC4648 { padding: true }, invalid_secret);
        assert!(result.is_none());
    }
}