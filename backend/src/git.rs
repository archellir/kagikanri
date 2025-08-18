use crate::{
    config::GitConfig,
    error::{AppError, AppResult},
};
use chrono::{DateTime, Utc};
use git2::{
    Cred, PushOptions, RemoteCallbacks, Repository, RepositoryInitOptions, Signature,
};
use serde::{Deserialize, Serialize};
use std::path::Path;
use tracing::{error, info, warn};

#[derive(Debug, Clone)]
pub struct GitSync {
    config: GitConfig,
    repo_path: std::path::PathBuf,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyncStatus {
    pub last_sync: Option<DateTime<Utc>>,
    pub last_commit: Option<String>,
    pub is_syncing: bool,
    pub error: Option<String>,
}

impl GitSync {
    pub fn new(config: GitConfig) -> AppResult<Self> {
        let repo_path = std::path::PathBuf::from("/data/password-store");
        
        Ok(Self {
            config,
            repo_path,
        })
    }

    pub async fn sync(&mut self) -> AppResult<SyncStatus> {
        info!("Starting Git sync");
        
        // Ensure repository exists first
        self.ensure_repository().await?;
        
        // Open repository for each operation to avoid holding across await
        let repo = Repository::open(&self.repo_path)
            .map_err(|e| AppError::GitError(format!("Failed to open repository: {}", e)))?;
        
        // Pull latest changes
        self.pull(&repo)?;
        
        // Push any local changes
        self.push(&repo)?;
        
        let last_commit = self.get_last_commit_hash(&repo)?;
        
        Ok(SyncStatus {
            last_sync: Some(Utc::now()),
            last_commit,
            is_syncing: false,
            error: None,
        })
    }

    async fn ensure_repository(&self) -> AppResult<()> {
        if self.repo_path.exists() && self.repo_path.join(".git").exists() {
            // Repository exists, just verify it can be opened
            Repository::open(&self.repo_path)
                .map_err(|e| AppError::GitError(format!("Failed to open repository: {}", e)))?;
            Ok(())
        } else {
            // Clone the repository
            self.clone_repository().await?;
            Ok(())
        }
    }

    async fn clone_repository(&self) -> AppResult<()> {
        info!("Cloning repository from {}", self.config.repo_url);
        
        // Ensure parent directory exists
        if let Some(parent) = self.repo_path.parent() {
            std::fs::create_dir_all(parent)?;
        }

        let mut callbacks = RemoteCallbacks::new();
        callbacks.credentials(|_url, username_from_url, _allowed_types| {
            Cred::userpass_plaintext(
                username_from_url.unwrap_or("git"),
                &self.config.access_token,
            )
        });

        let mut fetch_options = git2::FetchOptions::new();
        fetch_options.remote_callbacks(callbacks);

        let mut builder = git2::build::RepoBuilder::new();
        builder.fetch_options(fetch_options);

        builder
            .clone(&self.config.repo_url, &self.repo_path)
            .map_err(|e| AppError::GitError(format!("Failed to clone repository: {}", e)))?;

        info!("Repository cloned successfully");
        Ok(())
    }

    fn pull(&self, repo: &Repository) -> AppResult<()> {
        info!("Pulling latest changes");
        
        let mut remote = repo
            .find_remote("origin")
            .map_err(|e| AppError::GitError(format!("Failed to find remote: {}", e)))?;

        let mut callbacks = RemoteCallbacks::new();
        callbacks.credentials(|_url, username_from_url, _allowed_types| {
            Cred::userpass_plaintext(
                username_from_url.unwrap_or("git"),
                &self.config.access_token,
            )
        });

        let mut fetch_options = git2::FetchOptions::new();
        fetch_options.remote_callbacks(callbacks);

        remote
            .fetch(&["refs/heads/*:refs/remotes/origin/*"], Some(&mut fetch_options), None)
            .map_err(|e| AppError::GitError(format!("Failed to fetch: {}", e)))?;

        // Get the current branch
        let head = repo.head()?;
        let branch_name = head
            .shorthand()
            .ok_or_else(|| AppError::GitError("Failed to get branch name".to_string()))?;

        // Get remote branch reference
        let remote_branch_name = format!("refs/remotes/origin/{}", branch_name);
        let remote_ref = repo
            .find_reference(&remote_branch_name)
            .map_err(|e| AppError::GitError(format!("Failed to find remote branch: {}", e)))?;

        let remote_commit = remote_ref.peel_to_commit()?;
        
        // Fast-forward merge if possible
        let local_commit = head.peel_to_commit()?;
        
        if local_commit.id() != remote_commit.id() {
            info!("Updating local branch to match remote");
            
            // Reset to remote commit (this is a force update)
            repo.reset(
                remote_commit.as_object(),
                git2::ResetType::Hard,
                None,
            )?;
            
            info!("Successfully updated to latest remote changes");
        } else {
            info!("Local branch is up to date with remote");
        }

        Ok(())
    }

    fn push(&self, repo: &Repository) -> AppResult<()> {
        // Check if there are any local changes to push
        let statuses = repo.statuses(None)?;
        
        if !statuses.is_empty() {
            info!("Found local changes, committing and pushing");
            
            // Stage all changes
            let mut index = repo.index()?;
            index.add_all(["*"].iter(), git2::IndexAddOption::DEFAULT, None)?;
            index.write()?;
            
            // Create commit
            let signature = Signature::now("Kagikanri", "kagikanri@localhost")?;
            let tree_id = index.write_tree()?;
            let tree = repo.find_tree(tree_id)?;
            
            let head = repo.head()?;
            let parent_commit = head.peel_to_commit()?;
            
            let commit_id = repo.commit(
                Some("HEAD"),
                &signature,
                &signature,
                "Auto-commit from Kagikanri",
                &tree,
                &[&parent_commit],
            )?;
            
            info!("Created commit: {}", commit_id);
        }

        // Push to remote
        let mut remote = repo.find_remote("origin")?;
        
        let mut callbacks = RemoteCallbacks::new();
        callbacks.credentials(|_url, username_from_url, _allowed_types| {
            Cred::userpass_plaintext(
                username_from_url.unwrap_or("git"),
                &self.config.access_token,
            )
        });

        let mut push_options = PushOptions::new();
        push_options.remote_callbacks(callbacks);

        let head = repo.head()?;
        let branch_name = head
            .shorthand()
            .ok_or_else(|| AppError::GitError("Failed to get branch name".to_string()))?;

        let refspec = format!("refs/heads/{}:refs/heads/{}", branch_name, branch_name);
        
        remote
            .push(&[&refspec], Some(&mut push_options))
            .map_err(|e| AppError::GitError(format!("Failed to push: {}", e)))?;

        info!("Successfully pushed changes to remote");
        Ok(())
    }

    fn get_last_commit_hash(&self, repo: &Repository) -> AppResult<Option<String>> {
        match repo.head() {
            Ok(head) => {
                let commit = head.peel_to_commit()?;
                Ok(Some(commit.id().to_string()))
            }
            Err(_) => Ok(None),
        }
    }

    pub fn get_status(&self) -> SyncStatus {
        SyncStatus {
            last_sync: None, // TODO: Store this in state
            last_commit: None,
            is_syncing: false,
            error: None,
        }
    }
}