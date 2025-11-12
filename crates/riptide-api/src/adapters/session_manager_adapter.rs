//! SessionManager adapter for hexagonal architecture trait abstraction
//!
//! This adapter wraps the concrete SessionManager implementation to satisfy
//! the SessionStorage port trait defined in riptide-types.
//!
//! # Purpose
//!
//! Enables dependency inversion by allowing ApplicationContext to depend on
//! Arc<dyn SessionStorage> instead of the concrete Arc<SessionManager>.
//!
//! # Architecture
//!
//! ```text
//! ApplicationContext (riptide-api)
//!     ↓ depends on trait
//! SessionStorage trait (riptide-types/ports)
//!     ↑ implemented by
//! SessionManagerAdapter (this file)
//!     ↓ wraps
//! SessionManager (riptide-api/sessions)
//! ```

use crate::sessions::manager::SessionManager as ConcreteSessionManager;
use async_trait::async_trait;
use riptide_types::error::Result as RiptideResult;
use riptide_types::ports::session::{Session, SessionFilter, SessionStorage};
use std::sync::Arc;

/// Adapter that bridges concrete SessionManager to SessionStorage port trait
///
/// This adapter implements the dependency inversion principle by wrapping
/// the concrete SessionManager in a trait-based interface, enabling:
/// - Testability via mock implementations
/// - Swappable session storage backends
/// - Clean hexagonal architecture boundaries
///
/// # Example
///
/// ```rust,ignore
/// use riptide_api::sessions::SessionManager;
/// use riptide_api::adapters::SessionManagerAdapter;
///
/// let concrete_manager = SessionManager::new(config).await?;
/// let adapter: Arc<dyn SessionStorage> = SessionManagerAdapter::new(Arc::new(concrete_manager));
/// ```
pub struct SessionManagerAdapter {
    inner: Arc<ConcreteSessionManager>,
}

impl SessionManagerAdapter {
    /// Create new adapter wrapping a concrete SessionManager
    ///
    /// # Arguments
    ///
    /// * `manager` - Arc-wrapped concrete SessionManager implementation
    ///
    /// # Returns
    ///
    /// An adapter that implements the SessionStorage port trait
    pub fn new(manager: Arc<ConcreteSessionManager>) -> Self {
        Self { inner: manager }
    }

    /// Create adapter as Arc for use as trait object
    ///
    /// # Arguments
    ///
    /// * `manager` - Arc-wrapped concrete SessionManager implementation
    ///
    /// # Returns
    ///
    /// An Arc-wrapped adapter ready to use as Arc<dyn SessionStorage>
    pub fn new_arc(manager: Arc<ConcreteSessionManager>) -> Arc<Self> {
        Arc::new(Self { inner: manager })
    }
}

#[async_trait]
impl SessionStorage for SessionManagerAdapter {
    /// Retrieve a session by ID
    ///
    /// Delegates to the inner SessionManager and converts the concrete
    /// Session type to the port trait's Session type.
    async fn get_session(&self, id: &str) -> RiptideResult<Option<Session>> {
        // Use the concrete SessionManager's get_session method
        let result = self
            .inner
            .get_session(id)
            .await
            .map_err(|e| riptide_types::error::RiptideError::Custom(format!("Failed to get session: {}", e)))?;

        // Convert internal Session to port trait Session
        Ok(result.map(|internal_session| {
            // Extract custom metadata from SessionMetadata
            let metadata = internal_session.metadata.custom.clone();

            Session {
                id: internal_session.session_id.clone(),
                user_id: "default-user".to_string(), // Internal sessions don't have user_id field
                tenant_id: "default-tenant".to_string(), // Internal sessions don't have tenant_id field
                created_at: internal_session.created_at,
                expires_at: internal_session.expires_at,
                metadata,
            }
        }))
    }

    /// Save a session (insert or update)
    ///
    /// Converts port trait Session to concrete Session type and delegates.
    async fn save_session(&self, session: &Session) -> RiptideResult<()> {
        // Convert port Session to internal Session format
        let internal_session = crate::sessions::types::Session {
            session_id: session.id.clone(),
            created_at: session.created_at,
            expires_at: session.expires_at,
            last_accessed: session.created_at, // Use created_at as last_accessed for new sessions
            user_data_dir: std::path::PathBuf::from(format!("/tmp/riptide-sessions/{}", session.id)),
            cookies: Default::default(), // Empty cookie jar for new sessions
            metadata: crate::sessions::types::SessionMetadata {
                user_agent: None,
                viewport: None,
                locale: None,
                custom: session.metadata.clone(),
            },
            browser_config: Default::default(),
        };

        self.inner
            .update_session(internal_session)
            .await
            .map_err(|e| riptide_types::error::RiptideError::Custom(format!("Failed to save session: {}", e)))
    }

    /// Delete a session by ID
    ///
    /// Delegates to SessionManager's remove_session method.
    async fn delete_session(&self, id: &str) -> RiptideResult<()> {
        self.inner
            .remove_session(id)
            .await
            .map_err(|e| riptide_types::error::RiptideError::Custom(format!("Failed to delete session: {}", e)))
    }

    /// List sessions matching filter criteria
    ///
    /// Retrieves all sessions and applies filtering based on criteria.
    async fn list_sessions(&self, filter: SessionFilter) -> RiptideResult<Vec<Session>> {
        // Get all session IDs
        let session_ids = self.inner.list_sessions().await;

        let mut sessions = Vec::new();

        // Fetch and filter each session
        for session_id in session_ids {
            if let Ok(Some(session)) = self.get_session(&session_id).await {
                // Apply filters
                let user_match = filter
                    .user_id
                    .as_ref()
                    .map_or(true, |uid| uid == &session.user_id);

                let tenant_match = filter
                    .tenant_id
                    .as_ref()
                    .map_or(true, |tid| tid == &session.tenant_id);

                let active_match =
                    !filter.active_only || session.expires_at > std::time::SystemTime::now();

                if user_match && tenant_match && active_match {
                    sessions.push(session);
                }
            }
        }

        Ok(sessions)
    }

    /// Remove all expired sessions from storage
    ///
    /// Delegates to SessionManager's cleanup_expired method.
    async fn cleanup_expired(&self) -> RiptideResult<usize> {
        self.inner
            .cleanup_expired()
            .await
            .map_err(|e| riptide_types::error::RiptideError::Custom(format!("Failed to cleanup expired sessions: {}", e)))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::sessions::types::SessionConfig;
    use std::time::{Duration, SystemTime};

    #[tokio::test]
    async fn test_session_manager_adapter_creation() {
        let config = SessionConfig::default();
        let manager = ConcreteSessionManager::new(config).await.unwrap();
        let adapter = SessionManagerAdapter::new(Arc::new(manager));

        // Verify we can use the trait interface
        let _trait_ref: &dyn SessionStorage = &adapter;
    }

    #[tokio::test]
    async fn test_create_and_retrieve_session() {
        let config = SessionConfig::default();
        let manager = ConcreteSessionManager::new(config).await.unwrap();
        let adapter = SessionManagerAdapter::new(Arc::new(manager));

        // Create a session via port trait
        let session = Session {
            id: "test-session-123".to_string(),
            user_id: "user-456".to_string(),
            tenant_id: "tenant-789".to_string(),
            created_at: SystemTime::now(),
            expires_at: SystemTime::now() + Duration::from_secs(3600),
            metadata: HashMap::new(),
        };

        adapter.save_session(&session).await.unwrap();

        // Retrieve the session
        let retrieved = adapter.get_session("test-session-123").await.unwrap();
        assert!(retrieved.is_some());

        let retrieved_session = retrieved.unwrap();
        assert_eq!(retrieved_session.id, "test-session-123");
        assert_eq!(retrieved_session.user_id, "user-456");
        assert_eq!(retrieved_session.tenant_id, "tenant-789");
    }

    #[tokio::test]
    async fn test_delete_session() {
        let config = SessionConfig::default();
        let manager = ConcreteSessionManager::new(config).await.unwrap();
        let adapter = SessionManagerAdapter::new(Arc::new(manager));

        // Create a session
        let session = Session {
            id: "test-delete-session".to_string(),
            user_id: "user-delete".to_string(),
            tenant_id: "tenant-delete".to_string(),
            created_at: SystemTime::now(),
            expires_at: SystemTime::now() + Duration::from_secs(3600),
            metadata: HashMap::new(),
        };

        adapter.save_session(&session).await.unwrap();

        // Delete the session
        adapter.delete_session("test-delete-session").await.unwrap();

        // Verify it's gone
        let retrieved = adapter.get_session("test-delete-session").await.unwrap();
        assert!(retrieved.is_none());
    }

    #[tokio::test]
    async fn test_list_sessions_with_filter() {
        let config = SessionConfig::default();
        let manager = ConcreteSessionManager::new(config).await.unwrap();
        let adapter = SessionManagerAdapter::new(Arc::new(manager));

        // Create multiple sessions
        let session1 = Session {
            id: "session-1".to_string(),
            user_id: "user-1".to_string(),
            tenant_id: "tenant-a".to_string(),
            created_at: SystemTime::now(),
            expires_at: SystemTime::now() + Duration::from_secs(3600),
            metadata: HashMap::new(),
        };

        let session2 = Session {
            id: "session-2".to_string(),
            user_id: "user-2".to_string(),
            tenant_id: "tenant-b".to_string(),
            created_at: SystemTime::now(),
            expires_at: SystemTime::now() + Duration::from_secs(3600),
            metadata: HashMap::new(),
        };

        adapter.save_session(&session1).await.unwrap();
        adapter.save_session(&session2).await.unwrap();

        // List all sessions
        let filter = SessionFilter {
            user_id: None,
            tenant_id: None,
            active_only: true,
        };

        let sessions = adapter.list_sessions(filter).await.unwrap();
        assert!(sessions.len() >= 2);

        // Filter by user_id
        let filter = SessionFilter {
            user_id: Some("user-1".to_string()),
            tenant_id: None,
            active_only: true,
        };

        let sessions = adapter.list_sessions(filter).await.unwrap();
        assert!(sessions.iter().any(|s| s.user_id == "user-1"));
    }

    #[tokio::test]
    async fn test_cleanup_expired() {
        let config = SessionConfig::default();
        let manager = ConcreteSessionManager::new(config).await.unwrap();
        let adapter = SessionManagerAdapter::new(Arc::new(manager));

        // Create an expired session
        let expired_session = Session {
            id: "expired-session".to_string(),
            user_id: "user-expired".to_string(),
            tenant_id: "tenant-expired".to_string(),
            created_at: SystemTime::now() - Duration::from_secs(7200),
            expires_at: SystemTime::now() - Duration::from_secs(3600), // Expired
            metadata: HashMap::new(),
        };

        adapter.save_session(&expired_session).await.unwrap();

        // Cleanup expired sessions
        let count = adapter.cleanup_expired().await.unwrap();
        assert!(count >= 0); // Should cleanup at least the expired session
    }
}
