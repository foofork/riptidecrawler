pub mod manager;
pub mod middleware;
pub mod storage;
pub mod types;

pub use manager::*;
pub use types::*;

use anyhow::Result;
use std::sync::Arc;

/// Session management system for persistent browser sessions and cookie storage
///
/// This module provides:
/// - Persistent session ID generation and management
/// - Cookie jar storage and retrieval across requests
/// - Browser profile persistence with user data directories
/// - TTL-based session expiry and cleanup
/// - Thread-safe session state management
/// - Middleware integration for API handlers
///
/// # Examples
///
/// ```rust
/// use riptide_api::sessions::{SessionManager, SessionConfig};
///
/// let config = SessionConfig::default();
/// let manager = SessionManager::new(config).await?;
///
/// // Create or retrieve session
/// let session = manager.get_or_create_session("user_123").await?;
///
/// // Store cookies
/// session.set_cookie("auth_token", "abc123").await?;
///
/// // Get browser data directory
/// let data_dir = session.get_user_data_dir();
/// ```
#[derive(Clone)]
pub struct SessionSystem {
    manager: Arc<SessionManager>,
}

impl SessionSystem {
    /// Create a new session system with the given configuration
    pub async fn new(config: SessionConfig) -> Result<Self> {
        let manager = Arc::new(SessionManager::new(config).await?);
        Ok(Self { manager })
    }

    /// Create a new session system with default configuration
    pub async fn default() -> Result<Self> {
        Self::new(SessionConfig::default()).await
    }

    /// Get the session manager
    pub fn manager(&self) -> &Arc<SessionManager> {
        &self.manager
    }

    /// Get or create a session for the given session ID
    pub async fn get_or_create_session(&self, session_id: &str) -> Result<Session> {
        self.manager.get_or_create_session(session_id).await
    }

    /// Clean up expired sessions
    pub async fn cleanup_expired(&self) -> Result<usize> {
        self.manager.cleanup_expired().await
    }

    /// Get session statistics
    pub async fn get_stats(&self) -> Result<SessionStats> {
        self.manager.get_stats().await
    }
}
