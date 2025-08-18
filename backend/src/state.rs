use crate::{
    config::Config,
    error::{AppError, AppResult},
    git::GitSync,
    pass::PassInterface,
    passkey::PasskeyStore,
};
use std::sync::Arc;
use tokio::sync::RwLock;

#[derive(Clone)]
pub struct AppState {
    pub config: Config,
    pub pass: Arc<PassInterface>,
    pub passkey_store: Arc<PasskeyStore>,
    pub git_sync: Arc<RwLock<GitSync>>,
    pub session_store: Arc<RwLock<SessionStore>>,
}

impl AppState {
    pub async fn new(config: Config) -> AppResult<Self> {
        // Initialize pass interface
        let pass = Arc::new(PassInterface::new(config.pass.clone())?);
        
        // Initialize passkey store with encrypted database
        let passkey_store = Arc::new(PasskeyStore::new(&config.database).await?);
        
        // Initialize git sync
        let git_sync = Arc::new(RwLock::new(GitSync::new(config.git.clone())?));
        
        // Initialize session store
        let session_store = Arc::new(RwLock::new(SessionStore::new()));

        let state = AppState {
            config,
            pass,
            passkey_store,
            git_sync,
            session_store,
        };

        // Perform initial git sync
        state.sync_git().await?;

        Ok(state)
    }

    pub async fn sync_git(&self) -> AppResult<()> {
        let mut git_sync = self.git_sync.write().await;
        git_sync.sync().await?;
        Ok(())
    }

    pub async fn is_authenticated(&self, session_id: &str) -> bool {
        let session_store = self.session_store.read().await;
        session_store.is_valid(session_id)
    }

    pub async fn create_session(&self, user_id: &str) -> String {
        let mut session_store = self.session_store.write().await;
        session_store.create_session(user_id)
    }

    pub async fn remove_session(&self, session_id: &str) {
        let mut session_store = self.session_store.write().await;
        session_store.remove_session(session_id);
    }
}

use chrono::{DateTime, Utc};
use std::collections::HashMap;
use uuid::Uuid;

pub struct SessionStore {
    sessions: HashMap<String, Session>,
}

#[derive(Debug, Clone)]
pub struct Session {
    pub user_id: String,
    pub created_at: DateTime<Utc>,
    pub expires_at: DateTime<Utc>,
}

impl SessionStore {
    pub fn new() -> Self {
        Self {
            sessions: HashMap::new(),
        }
    }

    pub fn create_session(&mut self, user_id: &str) -> String {
        let session_id = Uuid::new_v4().to_string();
        let now = Utc::now();
        let expires_at = now + chrono::Duration::hours(24); // TODO: Use config

        let session = Session {
            user_id: user_id.to_string(),
            created_at: now,
            expires_at,
        };

        self.sessions.insert(session_id.clone(), session);
        
        // Clean up expired sessions
        self.cleanup_expired();
        
        session_id
    }

    pub fn is_valid(&self, session_id: &str) -> bool {
        if let Some(session) = self.sessions.get(session_id) {
            session.expires_at > Utc::now()
        } else {
            false
        }
    }

    pub fn remove_session(&mut self, session_id: &str) {
        self.sessions.remove(session_id);
    }

    pub fn get_session(&self, session_id: &str) -> Option<&Session> {
        self.sessions.get(session_id)
    }

    fn cleanup_expired(&mut self) {
        let now = Utc::now();
        self.sessions.retain(|_, session| session.expires_at > now);
    }
}