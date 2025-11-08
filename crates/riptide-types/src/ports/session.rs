//! Session storage port for session management
//!
//! This module provides backend-agnostic session management interfaces that enable:
//! - Storing and retrieving user sessions
//! - Session lifecycle management (creation, validation, expiration)
//! - Multi-tenancy support with tenant isolation
//! - Testing with in-memory session stores
//! - Swapping storage backends (PostgreSQL, Redis, etc.)
//!
//! # Design Goals
//!
//! - **Security**: Secure session storage with expiration
//! - **Multi-tenancy**: Tenant-isolated session management
//! - **Testability**: In-memory store for unit tests
//! - **Flexibility**: Support various storage backends
//! - **Performance**: Fast session lookup and validation
//!
//! # Example
//!
//! ```rust,ignore
//! use riptide_types::ports::SessionStorage;
//! use std::time::{Duration, SystemTime};
//!
//! async fn example(storage: &dyn SessionStorage) -> Result<()> {
//!     // Create session
//!     let session = Session {
//!         id: "session-123".to_string(),
//!         user_id: "user-456".to_string(),
//!         tenant_id: "tenant-789".to_string(),
//!         created_at: SystemTime::now(),
//!         expires_at: SystemTime::now() + Duration::from_secs(3600),
//!         metadata: HashMap::new(),
//!     };
//!
//!     storage.save_session(&session).await?;
//!
//!     // Retrieve session
//!     if let Some(s) = storage.get_session("session-123").await? {
//!         println!("Session found for user: {}", s.user_id);
//!     }
//!
//!     // Cleanup expired
//!     let cleaned = storage.cleanup_expired().await?;
//!     println!("Cleaned {} expired sessions", cleaned);
//!
//!     Ok(())
//! }
//! ```

use crate::error::Result as RiptideResult;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::SystemTime;

/// Session data structure
///
/// Represents an authenticated user session with multi-tenant support.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Session {
    /// Unique session identifier
    pub id: String,

    /// User identifier who owns this session
    pub user_id: String,

    /// Tenant identifier for multi-tenancy isolation
    pub tenant_id: String,

    /// Session creation timestamp
    pub created_at: SystemTime,

    /// Session expiration timestamp
    pub expires_at: SystemTime,

    /// Additional session metadata (permissions, preferences, etc.)
    pub metadata: HashMap<String, String>,
}

impl Session {
    /// Check if session is expired
    pub fn is_expired(&self) -> bool {
        self.expires_at < SystemTime::now()
    }

    /// Check if session is active (not expired)
    pub fn is_active(&self) -> bool {
        !self.is_expired()
    }

    /// Get remaining TTL in seconds, returns 0 if expired
    pub fn remaining_ttl_secs(&self) -> u64 {
        match self.expires_at.duration_since(SystemTime::now()) {
            Ok(duration) => duration.as_secs(),
            Err(_) => 0, // Expired
        }
    }
}

/// Session filter criteria for querying sessions
#[derive(Debug, Clone, Default)]
pub struct SessionFilter {
    /// Filter by user ID
    pub user_id: Option<String>,

    /// Filter by tenant ID
    pub tenant_id: Option<String>,

    /// Filter only active (non-expired) sessions
    pub active_only: bool,
}

/// Backend-agnostic session storage port
///
/// Implementations provide the anti-corruption layer between domain logic
/// and infrastructure storage (PostgreSQL, Redis, in-memory, etc.).
///
/// # Thread Safety
///
/// All implementations must be `Send + Sync` for use in async contexts.
///
/// # Error Handling
///
/// All methods return `RiptideResult<T>` for consistent error handling.
#[async_trait]
pub trait SessionStorage: Send + Sync {
    /// Retrieve a session by ID
    ///
    /// Returns `None` if session doesn't exist, `Some(Session)` if found.
    /// Does NOT automatically filter expired sessions - caller must check.
    async fn get_session(&self, id: &str) -> RiptideResult<Option<Session>>;

    /// Save a session (insert or update)
    ///
    /// If session with same ID exists, it will be updated.
    async fn save_session(&self, session: &Session) -> RiptideResult<()>;

    /// Delete a session by ID
    ///
    /// Returns Ok(()) regardless of whether session existed.
    async fn delete_session(&self, id: &str) -> RiptideResult<()>;

    /// List sessions matching filter criteria
    ///
    /// Returns all matching sessions. May be expensive for large datasets.
    async fn list_sessions(&self, filter: SessionFilter) -> RiptideResult<Vec<Session>>;

    /// Remove all expired sessions from storage
    ///
    /// Returns count of sessions deleted.
    async fn cleanup_expired(&self) -> RiptideResult<usize>;
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;

    #[test]
    fn test_session_is_expired() {
        let past = SystemTime::now() - Duration::from_secs(3600);
        let future = SystemTime::now() + Duration::from_secs(3600);

        let expired = Session {
            id: "test".to_string(),
            user_id: "user".to_string(),
            tenant_id: "tenant".to_string(),
            created_at: SystemTime::now(),
            expires_at: past,
            metadata: HashMap::new(),
        };

        let active = Session {
            id: "test2".to_string(),
            user_id: "user".to_string(),
            tenant_id: "tenant".to_string(),
            created_at: SystemTime::now(),
            expires_at: future,
            metadata: HashMap::new(),
        };

        assert!(expired.is_expired());
        assert!(!active.is_expired());
        assert!(active.is_active());
        assert!(!expired.is_active());
    }

    #[test]
    fn test_session_remaining_ttl() {
        let future = SystemTime::now() + Duration::from_secs(3600);
        let past = SystemTime::now() - Duration::from_secs(3600);

        let active = Session {
            id: "test".to_string(),
            user_id: "user".to_string(),
            tenant_id: "tenant".to_string(),
            created_at: SystemTime::now(),
            expires_at: future,
            metadata: HashMap::new(),
        };

        let expired = Session {
            id: "test2".to_string(),
            user_id: "user".to_string(),
            tenant_id: "tenant".to_string(),
            created_at: SystemTime::now(),
            expires_at: past,
            metadata: HashMap::new(),
        };

        // Active session should have ~3600 seconds remaining
        assert!(active.remaining_ttl_secs() > 3590);
        assert!(active.remaining_ttl_secs() <= 3600);

        // Expired session should have 0 remaining
        assert_eq!(expired.remaining_ttl_secs(), 0);
    }
}
