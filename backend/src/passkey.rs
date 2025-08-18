use crate::{
    config::DatabaseConfig,
    error::{AppError, AppResult},
};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::{sqlite::SqlitePoolOptions, Row, SqlitePool};
use std::collections::HashMap;
use uuid::Uuid;
use webauthn_rs::{
    prelude::*,
    WebauthnBuilder,
    Webauthn,
};

#[derive(Debug, Clone)]
pub struct PasskeyStore {
    pool: SqlitePool,
    webauthn: Webauthn,
    encryption_key: [u8; 32],
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StoredPasskey {
    pub id: String,
    pub domain: String,
    pub user_handle: Option<Vec<u8>>,
    pub credential_id: Vec<u8>,
    pub public_key: Vec<u8>,
    pub counter: u32,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PasskeyRegistrationStart {
    pub challenge: String,
    pub user_id: String,
    pub domain: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PasskeyRegistrationFinish {
    pub challenge: String,
    pub response: String, // JSON from WebAuthn API
}

impl PasskeyStore {
    pub async fn new(config: &DatabaseConfig) -> AppResult<Self> {
        // Parse encryption key from hex
        let encryption_key = hex::decode(&config.encryption_key)
            .map_err(|e| AppError::DatabaseError(format!("Invalid encryption key: {}", e)))?;
        
        if encryption_key.len() != 32 {
            return Err(AppError::DatabaseError(
                "Encryption key must be exactly 32 bytes".to_string(),
            ));
        }

        let mut key_array = [0u8; 32];
        key_array.copy_from_slice(&encryption_key);

        // Create SQLite pool with SQLCipher
        let pool = SqlitePoolOptions::new()
            .max_connections(5)
            .connect(&format!("{}?mode=rwc", config.url))
            .await?;

        // Set encryption key for SQLCipher
        sqlx::query(&format!("PRAGMA key = 'x\"{}\"'", config.encryption_key))
            .execute(&pool)
            .await?;

        // Initialize WebAuthn
        let rp_id = "kagikanri.local"; // TODO: Make this configurable
        let rp_origin = url::Url::parse("https://kagikanri.local")
            .map_err(|e| AppError::ConfigError(format!("Invalid WebAuthn origin URL: {}", e)))?;
        let webauthn = WebauthnBuilder::new(rp_id, &rp_origin)
            .map_err(|e| AppError::WebAuthnError(format!("Failed to build WebAuthn: {}", e)))?
            .build()
            .map_err(|e| AppError::WebAuthnError(format!("Failed to initialize WebAuthn: {}", e)))?;

        let store = Self {
            pool,
            webauthn,
            encryption_key: key_array,
        };

        // Initialize database schema
        store.init_schema().await?;

        Ok(store)
    }

    async fn init_schema(&self) -> AppResult<()> {
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS passkeys (
                id TEXT PRIMARY KEY,
                domain TEXT NOT NULL,
                user_handle BLOB,
                credential_id BLOB NOT NULL,
                public_key BLOB NOT NULL,
                private_key_encrypted BLOB NOT NULL,
                counter INTEGER DEFAULT 0,
                created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
                salt BLOB NOT NULL
            );

            CREATE INDEX IF NOT EXISTS idx_passkeys_domain ON passkeys(domain);

            CREATE TABLE IF NOT EXISTS db_metadata (
                key TEXT PRIMARY KEY,
                value TEXT NOT NULL
            );

            INSERT OR IGNORE INTO db_metadata (key, value) VALUES ('version', '1.0');
            INSERT OR IGNORE INTO db_metadata (key, value) VALUES ('created_at', datetime('now'));
            "#,
        )
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    pub async fn start_registration(&self, domain: &str, user_id: &str) -> AppResult<PasskeyRegistrationStart> {
        // This is a simplified implementation for demonstration
        // In a real implementation, you'd use proper WebAuthn credential creation

        // Store registration state (in a real implementation, you'd store this in a session or temporary storage)
        // For now, we'll include the challenge in the response and expect it back

        Ok(PasskeyRegistrationStart {
            challenge: format!("challenge_for_{}_{}", domain, user_id), // Placeholder
            user_id: user_id.to_string(),
            domain: domain.to_string(),
        })
    }

    pub async fn finish_registration(
        &self,
        request: PasskeyRegistrationFinish,
    ) -> AppResult<StoredPasskey> {
        // This is a simplified implementation
        // In a real implementation, you'd need to properly handle the WebAuthn flow
        
        let id = Uuid::new_v4().to_string();
        let now = Utc::now();
        
        // For now, create a placeholder entry
        let passkey = StoredPasskey {
            id: id.clone(),
            domain: "example.com".to_string(), // TODO: Extract from request
            user_handle: Some(vec![1, 2, 3, 4]), // Placeholder
            credential_id: vec![5, 6, 7, 8], // Placeholder
            public_key: vec![9, 10, 11, 12], // Placeholder
            counter: 0,
            created_at: now,
        };

        // Store in database
        let salt = self.generate_salt();
        let encrypted_private_key = self.encrypt_data(&[13, 14, 15, 16], &salt)?; // Placeholder

        sqlx::query(
            r#"
            INSERT INTO passkeys (id, domain, user_handle, credential_id, public_key, private_key_encrypted, counter, created_at, salt)
            VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)
            "#,
        )
        .bind(&id)
        .bind(&passkey.domain)
        .bind(&passkey.user_handle)
        .bind(&passkey.credential_id)
        .bind(&passkey.public_key)
        .bind(&encrypted_private_key)
        .bind(passkey.counter as i64)
        .bind(passkey.created_at)
        .bind(&salt)
        .execute(&self.pool)
        .await?;

        Ok(passkey)
    }

    pub async fn list_passkeys(&self) -> AppResult<Vec<StoredPasskey>> {
        let rows = sqlx::query(
            "SELECT id, domain, user_handle, credential_id, public_key, counter, created_at FROM passkeys ORDER BY created_at DESC"
        )
        .fetch_all(&self.pool)
        .await?;

        let mut passkeys = Vec::new();
        for row in rows {
            let passkey = StoredPasskey {
                id: row.get("id"),
                domain: row.get("domain"),
                user_handle: row.get("user_handle"),
                credential_id: row.get("credential_id"),
                public_key: row.get("public_key"),
                counter: row.get::<i64, _>("counter") as u32,
                created_at: row.get("created_at"),
            };
            passkeys.push(passkey);
        }

        Ok(passkeys)
    }

    pub async fn delete_passkey(&self, id: &str) -> AppResult<()> {
        let result = sqlx::query("DELETE FROM passkeys WHERE id = ?1")
            .bind(id)
            .execute(&self.pool)
            .await?;

        if result.rows_affected() == 0 {
            return Err(AppError::NotFound(format!("Passkey not found: {}", id)));
        }

        Ok(())
    }

    fn generate_salt(&self) -> Vec<u8> {
        use rand::RngCore;
        let mut salt = vec![0u8; 32];
        rand::thread_rng().fill_bytes(&mut salt);
        salt
    }

    fn encrypt_data(&self, data: &[u8], salt: &[u8]) -> AppResult<Vec<u8>> {
        // Simple XOR encryption for demonstration
        // In a real implementation, use proper encryption like AES-GCM
        let mut encrypted = Vec::new();
        for (i, &byte) in data.iter().enumerate() {
            let key_byte = self.encryption_key[i % 32];
            let salt_byte = salt[i % salt.len()];
            encrypted.push(byte ^ key_byte ^ salt_byte);
        }
        Ok(encrypted)
    }

    fn decrypt_data(&self, encrypted_data: &[u8], salt: &[u8]) -> AppResult<Vec<u8>> {
        // Simple XOR decryption (XOR is its own inverse)
        self.encrypt_data(encrypted_data, salt)
    }
}

// Convert webauthn-rs error to our error type
impl From<WebauthnError> for AppError {
    fn from(err: WebauthnError) -> Self {
        AppError::WebAuthnError(err.to_string())
    }
}


impl From<hex::FromHexError> for AppError {
    fn from(err: hex::FromHexError) -> Self {
        AppError::DatabaseError(format!("Hex decode error: {}", err))
    }
}