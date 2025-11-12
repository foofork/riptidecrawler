//! In-memory session storage implementation for testing and development
//!
//! This module provides a thread-safe in-memory session storage that implements
//! the `SessionStorage` trait. It's ideal for:
//! - Unit testing without PostgreSQL
//! - Development environments
//! - Embedded scenarios
//!
//! # Features
//!
//! - Thread-safe with `DashMap` for concurrent access without lock contention
//! - TTL-based expiration with automatic cleanup
//! - Multi-tenancy isolation support
//! - Background cleanup task for expired sessions
//! - Zero external database dependencies
//!
//! # Example
//!
//! ```rust,ignore
//! use riptide_types::ports::{SessionStorage, InMemorySessionStorage};
//! use std::time::Duration;
//!
//! #[tokio::main]
//! async fn main() -> anyhow::Result<()> {
//!     let storage = InMemorySessionStorage::new();
//!
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
//!     if let Some(s) = storage.get_session("session-123").await? {
//!         println!("Session found for user: {}", s.user_id);
//!     }
//!
//!     Ok(())
//! }
//! ```

use crate::error::Result as RiptideResult;
use crate::ports::session::{Session, SessionFilter, SessionStorage};
use async_trait::async_trait;
use dashmap::DashMap;
use std::sync::Arc;
use std::time::Duration;
use tokio::task::JoinHandle;
use tracing::{debug, info, instrument};

/// Thread-safe in-memory session storage implementation
///
/// Uses `DashMap` for lock-free concurrent access with excellent read performance.
/// Automatically cleans up expired sessions in a background task.
///
/// # Thread Safety
///
/// This implementation is fully thread-safe and can be shared across threads
/// via `Arc` or used directly as it implements `Clone`.
#[derive(Clone)]
pub struct InMemorySessionStorage {
    /// Session storage using DashMap for concurrent access
    sessions: Arc<DashMap<String, Session>>,
    /// Background cleanup task handle
    cleanup_handle: Arc<tokio::sync::Mutex<Option<JoinHandle<()>>>>,
}

impl InMemorySessionStorage {
    /// Create a new in-memory session storage
    ///
    /// By default, starts a background cleanup task that runs every 60 seconds.
    pub fn new() -> Self {
        Self::with_cleanup_interval(Duration::from_secs(60))
    }

    /// Create a session storage with custom cleanup interval
    ///
    /// # Arguments
    ///
    /// * `interval` - How often to run the background cleanup task
    pub fn with_cleanup_interval(interval: Duration) -> Self {
        let sessions = Arc::new(DashMap::new());
        let storage = Self {
            sessions: sessions.clone(),
            cleanup_handle: Arc::new(tokio::sync::Mutex::new(None)),
        };

        // Start background cleanup task
        let cleanup_sessions = sessions.clone();
        let handle = tokio::spawn(async move {
            let mut ticker = tokio::time::interval(interval);
            loop {
                ticker.tick().await;
                Self::cleanup_expired_internal(&cleanup_sessions).await;
            }
        });

        // Store the handle
        let storage_clone = storage.clone();
        tokio::spawn(async move {
            let mut cleanup_handle = storage_clone.cleanup_handle.lock().await;
            *cleanup_handle = Some(handle);
        });

        storage
    }

    /// Create a session storage without background cleanup
    ///
    /// Useful for testing when you want manual control over cleanup.
    pub fn without_cleanup() -> Self {
        Self {
            sessions: Arc::new(DashMap::new()),
            cleanup_handle: Arc::new(tokio::sync::Mutex::new(None)),
        }
    }

    /// Internal cleanup implementation that can be called without &self
    async fn cleanup_expired_internal(sessions: &DashMap<String, Session>) -> usize {
        let expired_keys: Vec<String> = sessions
            .iter()
            .filter(|entry| entry.value().is_expired())
            .map(|entry| entry.key().clone())
            .collect();

        let count = expired_keys.len();
        for key in expired_keys {
            sessions.remove(&key);
        }

        if count > 0 {
            debug!("Background cleanup removed {} expired sessions", count);
        }

        count
    }

    /// Get the current number of sessions (including expired ones)
    pub fn len(&self) -> usize {
        self.sessions.len()
    }

    /// Check if the storage is empty
    pub fn is_empty(&self) -> bool {
        self.sessions.is_empty()
    }

    /// Clear all sessions from storage
    pub fn clear(&self) {
        self.sessions.clear();
    }

    /// Stop the background cleanup task
    pub async fn stop_cleanup(&self) {
        let mut handle = self.cleanup_handle.lock().await;
        if let Some(h) = handle.take() {
            h.abort();
            debug!("Background cleanup task stopped");
        }
    }
}

impl Default for InMemorySessionStorage {
    fn default() -> Self {
        Self::new()
    }
}

impl Drop for InMemorySessionStorage {
    fn drop(&mut self) {
        // Note: We can't use async in Drop, but the cleanup task will be
        // automatically cancelled when the last Arc reference is dropped
        debug!("InMemorySessionStorage dropped");
    }
}

#[async_trait]
impl SessionStorage for InMemorySessionStorage {
    #[instrument(skip(self), fields(session_id = %id))]
    async fn get_session(&self, id: &str) -> RiptideResult<Option<Session>> {
        debug!("Fetching session from in-memory storage");

        if let Some(entry) = self.sessions.get(id) {
            let session = entry.value().clone();
            debug!(
                "Session found: user_id={}, expired={}",
                session.user_id,
                session.is_expired()
            );
            Ok(Some(session))
        } else {
            debug!("Session not found");
            Ok(None)
        }
    }

    #[instrument(skip(self, session), fields(session_id = %session.id))]
    async fn save_session(&self, session: &Session) -> RiptideResult<()> {
        debug!("Saving session to in-memory storage");
        self.sessions.insert(session.id.clone(), session.clone());
        info!("Session saved successfully");
        Ok(())
    }

    #[instrument(skip(self), fields(session_id = %id))]
    async fn delete_session(&self, id: &str) -> RiptideResult<()> {
        debug!("Deleting session from in-memory storage");

        if self.sessions.remove(id).is_some() {
            info!("Session deleted successfully");
        } else {
            debug!("Session not found for deletion");
        }

        Ok(())
    }

    #[instrument(skip(self))]
    async fn list_sessions(&self, filter: SessionFilter) -> RiptideResult<Vec<Session>> {
        debug!("Listing sessions with filter");

        let sessions: Vec<Session> = self
            .sessions
            .iter()
            .filter(|entry| {
                let session = entry.value();

                // Filter by user_id if specified
                if let Some(ref user_id) = filter.user_id {
                    if &session.user_id != user_id {
                        return false;
                    }
                }

                // Filter by tenant_id if specified
                if let Some(ref tenant_id) = filter.tenant_id {
                    if &session.tenant_id != tenant_id {
                        return false;
                    }
                }

                // Filter by active status if requested
                if filter.active_only && session.is_expired() {
                    return false;
                }

                true
            })
            .map(|entry| entry.value().clone())
            .collect();

        info!("Listed {} sessions", sessions.len());
        Ok(sessions)
    }

    #[instrument(skip(self))]
    async fn cleanup_expired(&self) -> RiptideResult<usize> {
        debug!("Cleaning up expired sessions");

        let expired_keys: Vec<String> = self
            .sessions
            .iter()
            .filter(|entry| entry.value().is_expired())
            .map(|entry| entry.key().clone())
            .collect();

        let count = expired_keys.len();
        for key in expired_keys {
            self.sessions.remove(&key);
        }

        info!("Cleaned up {} expired sessions", count);
        Ok(count)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;
    use std::time::SystemTime;

    fn create_test_session(
        id: &str,
        user_id: &str,
        tenant_id: &str,
        ttl: Duration,
    ) -> Session {
        Session {
            id: id.to_string(),
            user_id: user_id.to_string(),
            tenant_id: tenant_id.to_string(),
            created_at: SystemTime::now(),
            expires_at: SystemTime::now() + ttl,
            metadata: HashMap::new(),
        }
    }

    #[tokio::test]
    async fn test_save_and_get_session() {
        let storage = InMemorySessionStorage::without_cleanup();

        let mut metadata = HashMap::new();
        metadata.insert("role".to_string(), "admin".to_string());

        let session = Session {
            id: "test-session-1".to_string(),
            user_id: "user-123".to_string(),
            tenant_id: "tenant-456".to_string(),
            created_at: SystemTime::now(),
            expires_at: SystemTime::now() + Duration::from_secs(3600),
            metadata,
        };

        // Save session
        storage.save_session(&session).await.unwrap();

        // Retrieve session
        let retrieved = storage.get_session("test-session-1").await.unwrap();
        assert!(retrieved.is_some());
        let retrieved = retrieved.unwrap();
        assert_eq!(retrieved.id, session.id);
        assert_eq!(retrieved.user_id, session.user_id);
        assert_eq!(retrieved.metadata.get("role"), Some(&"admin".to_string()));
    }

    #[tokio::test]
    async fn test_delete_session() {
        let storage = InMemorySessionStorage::without_cleanup();

        let session = create_test_session(
            "test-session-2",
            "user-123",
            "tenant-456",
            Duration::from_secs(3600),
        );

        storage.save_session(&session).await.unwrap();
        storage.delete_session("test-session-2").await.unwrap();

        let retrieved = storage.get_session("test-session-2").await.unwrap();
        assert!(retrieved.is_none());
    }

    #[tokio::test]
    async fn test_cleanup_expired() {
        let storage = InMemorySessionStorage::without_cleanup();

        // Create expired session
        let expired = Session {
            id: "expired-session".to_string(),
            user_id: "user-123".to_string(),
            tenant_id: "tenant-456".to_string(),
            created_at: SystemTime::now() - Duration::from_secs(7200),
            expires_at: SystemTime::now() - Duration::from_secs(3600),
            metadata: HashMap::new(),
        };

        // Create active session
        let active = create_test_session(
            "active-session",
            "user-123",
            "tenant-456",
            Duration::from_secs(3600),
        );

        storage.save_session(&expired).await.unwrap();
        storage.save_session(&active).await.unwrap();

        let cleaned = storage.cleanup_expired().await.unwrap();
        assert_eq!(cleaned, 1);

        assert!(storage
            .get_session("expired-session")
            .await
            .unwrap()
            .is_none());
        assert!(storage
            .get_session("active-session")
            .await
            .unwrap()
            .is_some());
    }

    #[tokio::test]
    async fn test_list_sessions_by_tenant() {
        let storage = InMemorySessionStorage::without_cleanup();

        let session1 =
            create_test_session("s1", "user1", "tenant1", Duration::from_secs(3600));
        let session2 =
            create_test_session("s2", "user2", "tenant1", Duration::from_secs(3600));
        let session3 =
            create_test_session("s3", "user3", "tenant2", Duration::from_secs(3600));

        storage.save_session(&session1).await.unwrap();
        storage.save_session(&session2).await.unwrap();
        storage.save_session(&session3).await.unwrap();

        let filter = SessionFilter {
            tenant_id: Some("tenant1".to_string()),
            user_id: None,
            active_only: false,
        };

        let sessions = storage.list_sessions(filter).await.unwrap();
        assert_eq!(sessions.len(), 2);
        assert!(sessions.iter().all(|s| s.tenant_id == "tenant1"));
    }

    #[tokio::test]
    async fn test_list_sessions_by_user() {
        let storage = InMemorySessionStorage::without_cleanup();

        let session1 =
            create_test_session("s1", "user1", "tenant1", Duration::from_secs(3600));
        let session2 =
            create_test_session("s2", "user1", "tenant2", Duration::from_secs(3600));
        let session3 =
            create_test_session("s3", "user2", "tenant1", Duration::from_secs(3600));

        storage.save_session(&session1).await.unwrap();
        storage.save_session(&session2).await.unwrap();
        storage.save_session(&session3).await.unwrap();

        let filter = SessionFilter {
            tenant_id: None,
            user_id: Some("user1".to_string()),
            active_only: false,
        };

        let sessions = storage.list_sessions(filter).await.unwrap();
        assert_eq!(sessions.len(), 2);
        assert!(sessions.iter().all(|s| s.user_id == "user1"));
    }

    #[tokio::test]
    async fn test_list_sessions_active_only() {
        let storage = InMemorySessionStorage::without_cleanup();

        // Create active session
        let active = create_test_session("s1", "user1", "tenant1", Duration::from_secs(3600));

        // Create expired session
        let expired = Session {
            id: "s2".to_string(),
            user_id: "user1".to_string(),
            tenant_id: "tenant1".to_string(),
            created_at: SystemTime::now() - Duration::from_secs(7200),
            expires_at: SystemTime::now() - Duration::from_secs(3600),
            metadata: HashMap::new(),
        };

        storage.save_session(&active).await.unwrap();
        storage.save_session(&expired).await.unwrap();

        let filter = SessionFilter {
            tenant_id: Some("tenant1".to_string()),
            user_id: None,
            active_only: true,
        };

        let sessions = storage.list_sessions(filter).await.unwrap();
        assert_eq!(sessions.len(), 1);
        assert!(sessions.iter().all(|s| s.is_active()));
    }

    #[tokio::test]
    async fn test_update_session() {
        let storage = InMemorySessionStorage::without_cleanup();

        let mut session =
            create_test_session("s1", "user1", "tenant1", Duration::from_secs(3600));
        storage.save_session(&session).await.unwrap();

        // Update session metadata
        session
            .metadata
            .insert("updated".to_string(), "true".to_string());
        storage.save_session(&session).await.unwrap();

        let retrieved = storage.get_session("s1").await.unwrap().unwrap();
        assert_eq!(
            retrieved.metadata.get("updated"),
            Some(&"true".to_string())
        );
    }

    #[tokio::test]
    async fn test_concurrent_operations() {
        let storage = InMemorySessionStorage::without_cleanup();
        let storage = Arc::new(storage);

        let mut handles = vec![];

        // Spawn 10 concurrent tasks
        for i in 0..10 {
            let storage_clone = storage.clone();
            let handle = tokio::spawn(async move {
                let session = create_test_session(
                    &format!("session-{}", i),
                    &format!("user-{}", i),
                    "tenant-test",
                    Duration::from_secs(3600),
                );
                storage_clone.save_session(&session).await.unwrap();
            });
            handles.push(handle);
        }

        // Wait for all tasks
        for handle in handles {
            handle.await.unwrap();
        }

        // Verify all sessions exist
        let filter = SessionFilter {
            tenant_id: Some("tenant-test".to_string()),
            user_id: None,
            active_only: false,
        };
        let sessions = storage.list_sessions(filter).await.unwrap();
        assert_eq!(sessions.len(), 10);
    }

    #[tokio::test]
    async fn test_background_cleanup() {
        // Create storage with short cleanup interval
        let storage = InMemorySessionStorage::with_cleanup_interval(Duration::from_millis(100));

        // Create expired session
        let expired = Session {
            id: "expired".to_string(),
            user_id: "user1".to_string(),
            tenant_id: "tenant1".to_string(),
            created_at: SystemTime::now() - Duration::from_secs(7200),
            expires_at: SystemTime::now() - Duration::from_secs(3600),
            metadata: HashMap::new(),
        };

        storage.save_session(&expired).await.unwrap();

        // Wait for background cleanup to run
        tokio::time::sleep(Duration::from_millis(200)).await;

        // Expired session should be removed
        assert!(storage.get_session("expired").await.unwrap().is_none());

        // Stop cleanup task
        storage.stop_cleanup().await;
    }

    #[tokio::test]
    async fn test_empty_list() {
        let storage = InMemorySessionStorage::without_cleanup();

        let filter = SessionFilter {
            tenant_id: Some("nonexistent".to_string()),
            user_id: None,
            active_only: false,
        };

        let sessions = storage.list_sessions(filter).await.unwrap();
        assert_eq!(sessions.len(), 0);
    }

    #[tokio::test]
    async fn test_delete_nonexistent() {
        let storage = InMemorySessionStorage::without_cleanup();

        // Should not error
        storage.delete_session("nonexistent").await.unwrap();
    }

    #[tokio::test]
    async fn test_combined_filters() {
        let storage = InMemorySessionStorage::without_cleanup();

        let session1 =
            create_test_session("s1", "user1", "tenant1", Duration::from_secs(3600));
        let session2 =
            create_test_session("s2", "user1", "tenant2", Duration::from_secs(3600));
        let session3 =
            create_test_session("s3", "user2", "tenant1", Duration::from_secs(3600));

        storage.save_session(&session1).await.unwrap();
        storage.save_session(&session2).await.unwrap();
        storage.save_session(&session3).await.unwrap();

        // Filter by both user and tenant
        let filter = SessionFilter {
            tenant_id: Some("tenant1".to_string()),
            user_id: Some("user1".to_string()),
            active_only: false,
        };

        let sessions = storage.list_sessions(filter).await.unwrap();
        assert_eq!(sessions.len(), 1);
        assert_eq!(sessions[0].id, "s1");
    }
}
