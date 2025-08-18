use crate::{
    config::PassConfig,
    error::{AppError, AppResult},
};
use serde::{Deserialize, Serialize};
use std::{
    path::{Path, PathBuf},
    process::Command,
};
use tokio::process::Command as AsyncCommand;

#[derive(Debug, Clone)]
pub struct PassInterface {
    config: PassConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PasswordEntry {
    pub path: String,
    pub password: String,
    pub metadata: Option<PasswordMetadata>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PasswordMetadata {
    pub username: Option<String>,
    pub url: Option<String>,
    pub notes: Option<String>,
    pub custom_fields: std::collections::HashMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PasswordList {
    pub entries: Vec<String>,
}

impl PassInterface {
    pub fn new(config: PassConfig) -> AppResult<Self> {
        // Ensure password store directory exists
        if !config.store_dir.exists() {
            return Err(AppError::PassError(format!(
                "Password store directory does not exist: {}",
                config.store_dir.display()
            )));
        }

        Ok(Self { config })
    }

    pub async fn list_passwords(&self) -> AppResult<PasswordList> {
        let output = self.run_pass_command(&["ls"]).await?;
        
        let entries = output
            .lines()
            .filter(|line| !line.is_empty() && !line.starts_with("Password Store"))
            .map(|line| {
                // Remove tree structure characters and .gpg extension
                line.replace("├── ", "")
                    .replace("└── ", "")
                    .replace("│   ", "")
                    .replace("    ", "")
                    .replace(".gpg", "")
                    .trim()
                    .to_string()
            })
            .filter(|line| !line.is_empty())
            .collect();

        Ok(PasswordList { entries })
    }

    pub async fn get_password(&self, path: &str) -> AppResult<PasswordEntry> {
        let output = self.run_pass_command(&["show", path]).await?;
        
        let lines: Vec<&str> = output.lines().collect();
        if lines.is_empty() {
            return Err(AppError::NotFound(format!("Password not found: {}", path)));
        }

        let password = lines[0].to_string();
        let metadata = self.parse_metadata(&lines[1..]);

        Ok(PasswordEntry {
            path: path.to_string(),
            password,
            metadata,
        })
    }

    pub async fn create_or_update_password(&self, path: &str, entry: &PasswordEntry) -> AppResult<()> {
        let content = self.format_password_content(entry);
        
        // Use echo to pipe content to pass insert
        let mut cmd = AsyncCommand::new("bash");
        cmd.arg("-c")
           .arg(format!("echo '{}' | {} insert --multiline --force '{}'", 
                       content, self.pass_binary(), path))
           .env("PASSWORD_STORE_DIR", &self.config.store_dir);

        let output = cmd.output().await?;
        
        if !output.status.success() {
            let error = String::from_utf8_lossy(&output.stderr);
            return Err(AppError::PassError(format!("Failed to insert password: {}", error)));
        }

        Ok(())
    }

    pub async fn delete_password(&self, path: &str) -> AppResult<()> {
        let output = self.run_pass_command(&["rm", "--force", path]).await?;
        
        // pass rm always returns success, check if file actually existed
        if output.contains("is not in the password store") {
            return Err(AppError::NotFound(format!("Password not found: {}", path)));
        }

        Ok(())
    }

    pub async fn get_otp(&self, path: &str) -> AppResult<String> {
        let output = self.run_pass_command(&["otp", path]).await?;
        
        if output.trim().is_empty() {
            return Err(AppError::NotFound(format!("OTP not found: {}", path)));
        }

        Ok(output.trim().to_string())
    }

    pub async fn create_otp(&self, path: &str, secret: &str) -> AppResult<()> {
        let output = self.run_pass_command(&["otp", "insert", path, secret]).await?;
        
        if output.contains("Error") || output.contains("error") {
            return Err(AppError::PassError(format!("Failed to insert OTP: {}", output)));
        }

        Ok(())
    }

    async fn run_pass_command(&self, args: &[&str]) -> AppResult<String> {
        let mut cmd = AsyncCommand::new(self.pass_binary());
        cmd.args(args)
           .env("PASSWORD_STORE_DIR", &self.config.store_dir);

        // Set GPG key if specified
        if let Some(ref key_id) = self.config.gpg_key_id {
            cmd.env("PASSWORD_STORE_KEY", key_id);
        }

        let output = cmd.output().await?;
        
        if !output.status.success() {
            let error = String::from_utf8_lossy(&output.stderr);
            return Err(AppError::PassError(format!("Pass command failed: {}", error)));
        }

        Ok(String::from_utf8_lossy(&output.stdout).to_string())
    }

    fn pass_binary(&self) -> &str {
        "pass"
    }

    fn parse_metadata(&self, lines: &[&str]) -> Option<PasswordMetadata> {
        if lines.is_empty() {
            return None;
        }

        let mut username = None;
        let mut url = None;
        let mut notes = Vec::new();
        let mut custom_fields = std::collections::HashMap::new();

        for line in lines {
            let line = line.trim();
            if line.is_empty() {
                continue;
            }

            if line.starts_with("username: ") || line.starts_with("user: ") {
                username = Some(line.split_once(": ").unwrap().1.to_string());
            } else if line.starts_with("url: ") || line.starts_with("website: ") {
                url = Some(line.split_once(": ").unwrap().1.to_string());
            } else if line.contains(": ") {
                if let Some((key, value)) = line.split_once(": ") {
                    custom_fields.insert(key.to_string(), value.to_string());
                }
            } else {
                notes.push(line);
            }
        }

        let notes_text = if notes.is_empty() {
            None
        } else {
            Some(notes.join("\n"))
        };

        Some(PasswordMetadata {
            username,
            url,
            notes: notes_text,
            custom_fields,
        })
    }

    fn format_password_content(&self, entry: &PasswordEntry) -> String {
        let mut content = vec![entry.password.clone()];

        if let Some(ref metadata) = entry.metadata {
            if let Some(ref username) = metadata.username {
                content.push(format!("username: {}", username));
            }
            if let Some(ref url) = metadata.url {
                content.push(format!("url: {}", url));
            }
            
            for (key, value) in &metadata.custom_fields {
                content.push(format!("{}: {}", key, value));
            }
            
            if let Some(ref notes) = metadata.notes {
                content.push(String::new()); // Empty line before notes
                content.push(notes.clone());
            }
        }

        content.join("\n")
    }
}