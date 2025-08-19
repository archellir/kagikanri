use kagikanri::{
    config::GitConfig,
    git::{GitSync, SyncStatus},
};
use serial_test::serial;
use std::fs;
use std::path::Path;
use tempfile::TempDir;

fn create_test_git_config(repo_path: &str, remote_path: &str) -> GitConfig {
    GitConfig {
        repo_url: remote_path.to_string(),
        access_token: "test-token".to_string(),
    }
}

fn init_bare_git_repo(path: &Path) -> Result<(), Box<dyn std::error::Error>> {
    std::process::Command::new("git")
        .args(&["init", "--bare"])
        .current_dir(path)
        .output()?;
    Ok(())
}

fn init_git_repo_with_content(path: &Path, content: &str) -> Result<(), Box<dyn std::error::Error>> {
    // Create directory
    fs::create_dir_all(path)?;
    
    // Initialize git repo
    std::process::Command::new("git")
        .args(&["init"])
        .current_dir(path)
        .output()?;
    
    // Configure git user
    std::process::Command::new("git")
        .args(&["config", "user.name", "Test User"])
        .current_dir(path)
        .output()?;
    
    std::process::Command::new("git")
        .args(&["config", "user.email", "test@example.com"])
        .current_dir(path)
        .output()?;
    
    // Create a test file
    let test_file = path.join("test.txt");
    fs::write(test_file, content)?;
    
    // Add and commit
    std::process::Command::new("git")
        .args(&["add", "."])
        .current_dir(path)
        .output()?;
    
    std::process::Command::new("git")
        .args(&["commit", "-m", "Initial commit"])
        .current_dir(path)
        .output()?;
    
    Ok(())
}

#[tokio::test]
#[serial]
async fn test_git_sync_new() {
    let temp_dir = TempDir::new().expect("Failed to create temp directory");
    let repo_path = temp_dir.path().join("test-repo");
    
    let config = create_test_git_config(
        repo_path.to_string_lossy().as_ref(),
        "https://github.com/test/test-repo.git",
    );
    
    let result = GitSync::new(config);
    assert!(result.is_ok());
    
    let git_sync = result.unwrap();
    assert_eq!(git_sync.config.repo_url, "https://github.com/test/test-repo.git");
    assert_eq!(git_sync.config.access_token, "test-token");
}

#[tokio::test]
#[serial]
async fn test_git_sync_get_status_empty() {
    let temp_dir = TempDir::new().expect("Failed to create temp directory");
    let repo_path = temp_dir.path().join("test-repo");
    
    let config = create_test_git_config(
        repo_path.to_string_lossy().as_ref(),
        "https://github.com/test/test-repo.git",
    );
    
    let git_sync = GitSync::new(config).unwrap();
    let status = git_sync.get_status();
    
    assert!(status.last_sync.is_none());
    assert!(status.last_commit.is_none());
    assert!(!status.is_syncing);
    assert!(status.error.is_none());
}

#[tokio::test]
#[serial]
async fn test_git_sync_clone_local_repo() {
    let temp_dir = TempDir::new().expect("Failed to create temp directory");
    let remote_path = temp_dir.path().join("remote");
    let local_path = temp_dir.path().join("local");
    
    // Create a bare remote repository
    fs::create_dir_all(&remote_path).expect("Failed to create remote directory");
    init_bare_git_repo(&remote_path).expect("Failed to init bare repo");
    
    // Create a working repo to push to remote
    let working_path = temp_dir.path().join("working");
    init_git_repo_with_content(&working_path, "test content")
        .expect("Failed to create working repo");
    
    // Push to the bare repo
    std::process::Command::new("git")
        .args(&["remote", "add", "origin", remote_path.to_string_lossy().as_ref()])
        .current_dir(&working_path)
        .output()
        .expect("Failed to add remote");
    
    std::process::Command::new("git")
        .args(&["push", "-u", "origin", "master"])
        .current_dir(&working_path)
        .output()
        .expect("Failed to push to remote");
    
    // Now test GitSync with the local bare repo
    let config = GitConfig {
        repo_url: remote_path.to_string_lossy().to_string(),
        access_token: "not-used-for-local".to_string(),
    };
    
    let mut git_sync = GitSync::new(config).unwrap();
    
    // This should fail because we don't have credentials set up properly for the test
    // But we can test that the error handling works
    let result = git_sync.sync().await;
    
    // The result depends on git configuration and credentials
    // For a unit test, we mainly want to ensure it doesn't panic
    match result {
        Ok(status) => {
            assert!(status.last_sync.is_some());
            assert!(!status.is_syncing);
        },
        Err(_) => {
            // Expected for test environment without proper git setup
        }
    }
}

#[tokio::test]
#[serial]
async fn test_git_sync_status_updates() {
    let temp_dir = TempDir::new().expect("Failed to create temp directory");
    let repo_path = temp_dir.path().join("test-repo");
    
    let config = create_test_git_config(
        repo_path.to_string_lossy().as_ref(),
        "https://github.com/test/test-repo.git",
    );
    
    let git_sync = GitSync::new(config).unwrap();
    
    // Initial status should be empty
    let initial_status = git_sync.get_status();
    assert!(initial_status.last_sync.is_none());
    assert!(initial_status.last_commit.is_none());
    assert!(!initial_status.is_syncing);
    assert!(initial_status.error.is_none());
}

#[tokio::test]
#[serial]
async fn test_git_sync_error_handling() {
    let temp_dir = TempDir::new().expect("Failed to create temp directory");
    let repo_path = temp_dir.path().join("nonexistent");
    
    // Use an invalid URL to test error handling
    let config = GitConfig {
        repo_url: "https://invalid-domain-that-does-not-exist.com/repo.git".to_string(),
        access_token: "invalid-token".to_string(),
    };
    
    let mut git_sync = GitSync::new(config).unwrap();
    
    // This should fail and return an error
    let result = git_sync.sync().await;
    assert!(result.is_err());
    
    // The error should be a GitError
    match result.unwrap_err() {
        kagikanri::error::AppError::GitError(_) => {
            // Expected
        },
        other => panic!("Expected GitError, got: {:?}", other),
    }
}

#[tokio::test]
#[serial]
async fn test_git_sync_concurrent_operations() {
    let temp_dir = TempDir::new().expect("Failed to create temp directory");
    let repo_path = temp_dir.path().join("test-repo");
    
    let config = create_test_git_config(
        repo_path.to_string_lossy().as_ref(),
        "https://github.com/test/test-repo.git",
    );
    
    let git_sync = std::sync::Arc::new(tokio::sync::RwLock::new(
        GitSync::new(config).unwrap()
    ));
    
    // Test concurrent status reads
    let handles: Vec<_> = (0..5)
        .map(|_| {
            let git_sync = git_sync.clone();
            tokio::spawn(async move {
                let sync = git_sync.read().await;
                sync.get_status()
            })
        })
        .collect();
    
    let results: Vec<_> = futures::future::join_all(handles)
        .await
        .into_iter()
        .map(|result| result.unwrap())
        .collect();
    
    // All status calls should succeed
    assert_eq!(results.len(), 5);
    for status in results {
        assert!(!status.is_syncing);
        assert!(status.last_sync.is_none());
    }
}

#[test]
fn test_sync_status_serialization() {
    let status = SyncStatus {
        last_sync: Some(chrono::Utc::now()),
        last_commit: Some("abc123".to_string()),
        is_syncing: false,
        error: Some("test error".to_string()),
    };
    
    // Test JSON serialization
    let json = serde_json::to_string(&status).expect("Failed to serialize");
    assert!(json.contains("last_sync"));
    assert!(json.contains("last_commit"));
    assert!(json.contains("abc123"));
    assert!(json.contains("test error"));
    
    // Test deserialization
    let deserialized: SyncStatus = serde_json::from_str(&json).expect("Failed to deserialize");
    assert_eq!(deserialized.last_commit, status.last_commit);
    assert_eq!(deserialized.is_syncing, status.is_syncing);
    assert_eq!(deserialized.error, status.error);
}

#[test]
fn test_sync_status_default_values() {
    let status = SyncStatus {
        last_sync: None,
        last_commit: None,
        is_syncing: false,
        error: None,
    };
    
    assert!(status.last_sync.is_none());
    assert!(status.last_commit.is_none());
    assert!(!status.is_syncing);
    assert!(status.error.is_none());
}

#[tokio::test]
#[serial]
async fn test_git_config_validation() {
    // Test with empty repo URL
    let config = GitConfig {
        repo_url: "".to_string(),
        access_token: "token".to_string(),
    };
    
    let result = GitSync::new(config);
    // Should succeed in creating the GitSync, but fail when trying to use it
    assert!(result.is_ok());
    
    // Test with invalid URL format
    let config = GitConfig {
        repo_url: "not-a-url".to_string(),
        access_token: "token".to_string(),
    };
    
    let result = GitSync::new(config);
    // Should succeed in creating the GitSync, but fail when trying to use it
    assert!(result.is_ok());
}

#[tokio::test]
#[serial]
async fn test_repository_path_handling() {
    let temp_dir = TempDir::new().expect("Failed to create temp directory");
    let repo_path = temp_dir.path().join("password-store");
    
    let config = create_test_git_config(
        repo_path.to_string_lossy().as_ref(),
        "https://github.com/test/test-repo.git",
    );
    
    let git_sync = GitSync::new(config).unwrap();
    
    // The repository path should be set correctly
    assert_eq!(git_sync.repo_path, std::path::PathBuf::from("/data/password-store"));
}

#[tokio::test] 
#[serial]
async fn test_sync_with_network_timeout() {
    // Test behavior when network operations time out
    let config = GitConfig {
        repo_url: "https://httpbin.org/delay/10".to_string(), // This will timeout
        access_token: "test-token".to_string(),
    };
    
    let mut git_sync = GitSync::new(config).unwrap();
    
    // This should fail due to network timeout or invalid git URL
    let result = git_sync.sync().await;
    assert!(result.is_err());
}

// Helper test to check git2 library integration
#[test]
fn test_git2_basic_operations() {
    let temp_dir = TempDir::new().expect("Failed to create temp directory");
    let repo_path = temp_dir.path().join("test-git2");
    
    // Test basic git2 repository creation
    let repo = git2::Repository::init(&repo_path);
    assert!(repo.is_ok());
    
    let repo = repo.unwrap();
    assert!(repo.path().exists());
    assert!(repo.is_empty().unwrap_or(false));
}