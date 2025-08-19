use crate::{config::PassConfig, error::{AppError, AppResult}};
use serde::{Deserialize, Serialize};
use std::{
    collections::HashMap,
    path::{Path, PathBuf},
    process::Command,
};
use tracing::{debug, info};

#[derive(Debug, Clone)]
pub struct PassInterface {
    config: PassConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PasswordEntry {
    pub password: String,
    pub metadata: HashMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PasswordList {
    pub entries: Vec<PasswordItem>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PasswordItem {
    pub path: String,
    pub name: String,
    pub is_folder: bool,
}

impl PassInterface {
    pub fn new(config: PassConfig) -> AppResult<Self> {
        // Verify pass is available
        Command::new("pass")
            .arg("--version")
            .output()
            .map_err(|e| AppError::PassError(format!("Pass CLI not available: {}", e)))?;

        Ok(Self { config })
    }

    pub async fn list_passwords(&self) -> AppResult<PasswordList> {
        info!("Listing all passwords");
        
        let output = self.run_pass_command(&["ls"]).await?;
        let entries = self.parse_password_list(&output);
        
        Ok(PasswordList { entries })
    }

    pub async fn get_password(&self, path: &str) -> AppResult<PasswordEntry> {
        info!("Getting password for path: {}", path);
        
        let output = self.run_pass_command(&["show", path]).await?;
        let entry = self.parse_password_entry(&output)?;
        
        Ok(entry)
    }

    pub async fn create_or_update_password(&self, path: &str, entry: &PasswordEntry) -> AppResult<()> {
        info!("Creating/updating password at path: {}", path);
        
        let content = self.format_password_content(entry);
        
        // Use echo to pipe password to pass insert
        let mut cmd = Command::new("sh");
        cmd.arg("-c")
           .arg(format!("echo '{}' | pass insert --multiline --force '{}'", content, path));
        
        let output = cmd.output()
            .map_err(|e| AppError::PassError(format!("Failed to run pass insert: {}", e)))?;
        
        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(AppError::PassError(format!("Pass insert failed: {}", stderr)));
        }
        
        Ok(())
    }

    pub async fn delete_password(&self, path: &str) -> AppResult<()> {
        info!("Deleting password at path: {}", path);
        
        // Use --force to avoid interactive confirmation
        let output = self.run_pass_command(&["rm", "--force", path]).await?;
        
        if output.contains("removed successfully") || output.is_empty() {
            Ok(())
        } else {
            Err(AppError::PassError(format!("Failed to delete password: {}", output)))
        }
    }

    pub async fn get_otp(&self, path: &str) -> AppResult<String> {
        info!("Getting OTP for path: {}", path);
        
        let output = self.run_pass_command(&["otp", path]).await?;
        let code = output.trim().to_string();
        
        if code.len() == 6 && code.chars().all(|c| c.is_ascii_digit()) {
            Ok(code)
        } else {
            Err(AppError::PassError(format!("Invalid OTP code format: {}", code)))
        }
    }

    pub async fn create_otp(&self, path: &str, secret: &str) -> AppResult<()> {
        info!("Creating OTP at path: {}", path);
        
        // Insert the TOTP secret using pass otp
        let mut cmd = Command::new("sh");
        cmd.arg("-c")
           .arg(format!("echo '{}' | pass otp insert '{}'", secret, path));
        
        let output = cmd.output()
            .map_err(|e| AppError::PassError(format!("Failed to run pass otp insert: {}", e)))?;
        
        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(AppError::PassError(format!("Pass OTP insert failed: {}", stderr)));
        }
        
        Ok(())
    }

    async fn run_pass_command(&self, args: &[&str]) -> AppResult<String> {
        debug!("Running pass command: {:?}", args);
        
        let mut cmd = Command::new("pass");
        cmd.args(args);
        
        // Set PASSWORD_STORE_DIR
        cmd.env("PASSWORD_STORE_DIR", &self.config.store_dir);
        
        let output = cmd.output()
            .map_err(|e| AppError::PassError(format!("Failed to run pass command: {}", e)))?;
        
        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(AppError::PassError(format!("Pass command failed: {}", stderr)));
        }
        
        let stdout = String::from_utf8(output.stdout)
            .map_err(|e| AppError::PassError(format!("Invalid UTF-8 output: {}", e)))?;
        
        Ok(stdout)
    }

    fn parse_password_list(&self, output: &str) -> Vec<PasswordItem> {
        let mut entries = Vec::new();
        
        for line in output.lines() {
            if line.trim().is_empty() {
                continue;
            }
            
            // Remove tree characters and extract path
            let clean_line = line.replace("├── ", "").replace("└── ", "").replace("│   ", "").trim().to_string();
            
            if clean_line.is_empty() {
                continue;
            }
            
            let is_folder = !clean_line.ends_with(".gpg");
            let path = if is_folder {
                clean_line.clone()
            } else {
                clean_line.strip_suffix(".gpg").unwrap_or(&clean_line).to_string()
            };
            
            let name = path.split('/').last().unwrap_or(&path).to_string();
            
            entries.push(PasswordItem {
                path,
                name,
                is_folder,
            });
        }
        
        entries
    }

    fn parse_password_entry(&self, content: &str) -> AppResult<PasswordEntry> {
        let lines: Vec<&str> = content.lines().collect();
        
        if lines.is_empty() {
            return Err(AppError::PassError("Empty password entry".to_string()));
        }
        
        let password = lines[0].to_string();
        let mut metadata = HashMap::new();
        
        // Parse metadata from subsequent lines
        for line in lines.iter().skip(1) {
            if line.trim().is_empty() {
                continue;
            }
            
            if let Some(colon_pos) = line.find(':') {
                let key = line[..colon_pos].trim().to_string();
                let value = line[colon_pos + 1..].trim().to_string();
                metadata.insert(key, value);
            }
        }
        
        Ok(PasswordEntry { password, metadata })
    }

    fn format_password_content(&self, entry: &PasswordEntry) -> String {
        let mut content = entry.password.clone();
        
        for (key, value) in &entry.metadata {
            content.push_str(&format!("\n{}: {}", key, value));
        }
        
        content
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;
    use std::{collections::HashMap, path::PathBuf};

    fn create_test_pass_interface() -> PassInterface {
        let config = PassConfig {
            store_dir: PathBuf::from("/tmp/test-password-store"),
            gpg_key_id: Some("test-key-id".to_string()),
        };
        PassInterface { config }
    }

    #[test]
    fn test_parse_password_list_empty() {
        let pass = create_test_pass_interface();
        let entries = pass.parse_password_list("");
        assert!(entries.is_empty());
    }

    #[test]
    fn test_parse_password_list_simple() {
        let pass = create_test_pass_interface();
        let output = "github.com.gpg\ntwitter.com.gpg";
        let entries = pass.parse_password_list(output);
        
        assert_eq!(entries.len(), 2);
        assert_eq!(entries[0].path, "github.com");
        assert_eq!(entries[0].name, "github.com");
        assert_eq!(entries[0].is_folder, false);
        assert_eq!(entries[1].path, "twitter.com");
        assert_eq!(entries[1].name, "twitter.com");
        assert_eq!(entries[1].is_folder, false);
    }

    #[test]
    fn test_parse_password_list_with_tree_format() {
        let pass = create_test_pass_interface();
        let output = r#"Password Store
├── Email
│   ├── gmail.com.gpg
│   └── work
│       └── company.com.gpg
├── Social
│   ├── facebook.com.gpg
│   └── twitter.com.gpg
└── Banking
    └── bank.com.gpg"#;
        
        let entries = pass.parse_password_list(output);
        
        // Should parse all entries, both folders and files
        assert!(entries.len() >= 5);
        
        // Find specific entries
        let gmail_entry = entries.iter().find(|e| e.path == "gmail.com").unwrap();
        assert_eq!(gmail_entry.name, "gmail.com");
        assert_eq!(gmail_entry.is_folder, false);
        
        let email_folder = entries.iter().find(|e| e.path == "Email");
        if let Some(folder) = email_folder {
            assert_eq!(folder.is_folder, true);
        }
    }

    #[test]
    fn test_parse_password_entry_password_only() {
        let pass = create_test_pass_interface();
        let content = "super_secret_password";
        let entry = pass.parse_password_entry(content).unwrap();
        
        assert_eq!(entry.password, "super_secret_password");
        assert!(entry.metadata.is_empty());
    }

    #[test]
    fn test_parse_password_entry_with_metadata() {
        let pass = create_test_pass_interface();
        let content = r#"super_secret_password
username: john.doe@example.com
url: https://example.com
notes: My important account"#;
        
        let entry = pass.parse_password_entry(content).unwrap();
        
        assert_eq!(entry.password, "super_secret_password");
        assert_eq!(entry.metadata.len(), 3);
        assert_eq!(entry.metadata.get("username"), Some(&"john.doe@example.com".to_string()));
        assert_eq!(entry.metadata.get("url"), Some(&"https://example.com".to_string()));
        assert_eq!(entry.metadata.get("notes"), Some(&"My important account".to_string()));
    }

    #[test]
    fn test_parse_password_entry_empty() {
        let pass = create_test_pass_interface();
        let result = pass.parse_password_entry("");
        assert!(result.is_err());
        assert!(matches!(result, Err(AppError::PassError(_))));
    }

    #[test]
    fn test_format_password_content_password_only() {
        let pass = create_test_pass_interface();
        let entry = PasswordEntry {
            password: "test_password".to_string(),
            metadata: HashMap::new(),
        };
        
        let content = pass.format_password_content(&entry);
        assert_eq!(content, "test_password");
    }

    #[test]
    fn test_format_password_content_with_metadata() {
        let pass = create_test_pass_interface();
        let mut metadata = HashMap::new();
        metadata.insert("username".to_string(), "testuser".to_string());
        metadata.insert("url".to_string(), "https://test.com".to_string());
        
        let entry = PasswordEntry {
            password: "test_password".to_string(),
            metadata,
        };
        
        let content = pass.format_password_content(&entry);
        assert!(content.starts_with("test_password\n"));
        assert!(content.contains("username: testuser"));
        assert!(content.contains("url: https://test.com"));
    }

    #[test]
    fn test_parse_password_list_with_whitespace() {
        let pass = create_test_pass_interface();
        let output = r#"
        
├── test1.gpg
   
└── test2.gpg
        "#;
        
        let entries = pass.parse_password_list(output);
        assert_eq!(entries.len(), 2);
        assert_eq!(entries[0].path, "test1");
        assert_eq!(entries[1].path, "test2");
    }

    #[test]
    fn test_parse_nested_folders() {
        let pass = create_test_pass_interface();
        let output = r#"├── folder1
│   ├── subfolder
│   │   └── deep.gpg
│   └── file.gpg
└── folder2
    └── another.gpg"#;
        
        let entries = pass.parse_password_list(output);
        
        // Should find the deep nested file
        let deep_entry = entries.iter().find(|e| e.path.contains("deep"));
        assert!(deep_entry.is_some());
    }
}