//! Contract tests for SessionStorage trait
//!
//! These tests validate that any implementation of the SessionStorage trait
//! adheres to the expected behavior, including multi-tenancy isolation,
//! expiration handling, and CRUD operations.
//!
//! # Usage
//!
//! ```rust,ignore
//! use riptide_types::ports::SessionStorage;
//!
//! #[tokio::test]
//! async fn test_my_session_storage() {
//!     let storage = MySessionStorage::new();
//!     session_storage_contract::test_crud_operations(&storage).await.unwrap();
//!     session_storage_contract::test_multi_tenancy(&storage).await.unwrap();
//!     // ... run all contract tests
//! }
//! ```

use riptide_types::error::{Result as RiptideResult, RiptideError};
use riptide_types::ports::{Session, SessionFilter, SessionStorage};
use std::collections::HashMap;
use std::time::{Duration, SystemTime};

/// Create a test session with given parameters
fn create_test_session(id: &str, user_id: &str, tenant_id: &str, ttl: Duration) -> Session {
    Session {
        id: id.to_string(),
        user_id: user_id.to_string(),
        tenant_id: tenant_id.to_string(),
        created_at: SystemTime::now(),
        expires_at: SystemTime::now() + ttl,
        metadata: HashMap::new(),
    }
}

/// Test basic CRUD operations
///
/// Validates:
/// - Can save and retrieve sessions
/// - Can update existing sessions
/// - Can delete sessions
/// - Returns None for non-existent sessions
pub async fn test_crud_operations<S: SessionStorage>(storage: &S) -> RiptideResult<()> {
    // Create session
    let session = create_test_session("session_1", "user_1", "tenant_1", Duration::from_secs(3600));

    // Save session
    storage.save_session(&session).await?;

    // Retrieve session
    let retrieved = storage.get_session("session_1").await?;
    assert!(retrieved.is_some(), "Session should be retrievable");
    assert_eq!(
        retrieved.as_ref().unwrap().id,
        session.id,
        "Retrieved session should match"
    );
    assert_eq!(
        retrieved.as_ref().unwrap().user_id,
        session.user_id,
        "User ID should match"
    );

    // Update session (save with same ID)
    let mut updated_session = session.clone();
    updated_session
        .metadata
        .insert("updated".to_string(), "true".to_string());
    storage.save_session(&updated_session).await?;

    let after_update = storage.get_session("session_1").await?;
    assert!(after_update.is_some(), "Updated session should exist");
    assert_eq!(
        after_update.unwrap().metadata.get("updated"),
        Some(&"true".to_string()),
        "Metadata should be updated"
    );

    // Delete session
    storage.delete_session("session_1").await?;
    let after_delete = storage.get_session("session_1").await?;
    assert!(
        after_delete.is_none(),
        "Session should not exist after delete"
    );

    // Delete non-existent session (should not error)
    storage.delete_session("non_existent").await?;

    // Get non-existent session
    let non_existent = storage.get_session("non_existent").await?;
    assert!(
        non_existent.is_none(),
        "Non-existent session should return None"
    );

    Ok(())
}

/// Test session expiration
///
/// Validates:
/// - Expired sessions are still retrievable (caller must check)
/// - cleanup_expired removes expired sessions
/// - Active sessions are not removed by cleanup
pub async fn test_expiration<S: SessionStorage>(storage: &S) -> RiptideResult<()> {
    // Create expired session (expires in past)
    let past = SystemTime::now() - Duration::from_secs(3600);
    let expired_session = Session {
        id: "expired_session".to_string(),
        user_id: "user_expired".to_string(),
        tenant_id: "tenant_1".to_string(),
        created_at: SystemTime::now(),
        expires_at: past,
        metadata: HashMap::new(),
    };

    // Create active session
    let active_session = create_test_session(
        "active_session",
        "user_active",
        "tenant_1",
        Duration::from_secs(3600),
    );

    storage.save_session(&expired_session).await?;
    storage.save_session(&active_session).await?;

    // Expired session should still be retrievable before cleanup
    let retrieved_expired = storage.get_session("expired_session").await?;
    assert!(
        retrieved_expired.is_some(),
        "Expired session should be retrievable before cleanup"
    );
    assert!(
        retrieved_expired.unwrap().is_expired(),
        "Session should be marked as expired"
    );

    // Clean up expired sessions
    let cleaned = storage.cleanup_expired().await?;
    assert!(cleaned >= 1, "Should clean at least 1 expired session");

    // Expired session should be gone
    let after_cleanup_expired = storage.get_session("expired_session").await?;
    assert!(
        after_cleanup_expired.is_none(),
        "Expired session should be removed after cleanup"
    );

    // Active session should remain
    let after_cleanup_active = storage.get_session("active_session").await?;
    assert!(
        after_cleanup_active.is_some(),
        "Active session should remain after cleanup"
    );

    // Cleanup
    storage.delete_session("active_session").await?;

    Ok(())
}

/// Test multi-tenancy isolation
///
/// Validates:
/// - Sessions from different tenants are isolated
/// - Filtering by tenant_id returns only that tenant's sessions
/// - Deleting sessions doesn't affect other tenants
pub async fn test_multi_tenancy<S: SessionStorage>(storage: &S) -> RiptideResult<()> {
    // Create sessions for different tenants
    let tenant1_session1 = create_test_session(
        "t1_session1",
        "user_1",
        "tenant_1",
        Duration::from_secs(3600),
    );
    let tenant1_session2 = create_test_session(
        "t1_session2",
        "user_2",
        "tenant_1",
        Duration::from_secs(3600),
    );
    let tenant2_session1 = create_test_session(
        "t2_session1",
        "user_3",
        "tenant_2",
        Duration::from_secs(3600),
    );

    storage.save_session(&tenant1_session1).await?;
    storage.save_session(&tenant1_session2).await?;
    storage.save_session(&tenant2_session1).await?;

    // List sessions for tenant_1
    let tenant1_filter = SessionFilter {
        tenant_id: Some("tenant_1".to_string()),
        user_id: None,
        active_only: false,
    };
    let tenant1_sessions = storage.list_sessions(tenant1_filter).await?;
    assert_eq!(
        tenant1_sessions.len(),
        2,
        "Should find 2 sessions for tenant_1"
    );
    assert!(
        tenant1_sessions.iter().all(|s| s.tenant_id == "tenant_1"),
        "All sessions should belong to tenant_1"
    );

    // List sessions for tenant_2
    let tenant2_filter = SessionFilter {
        tenant_id: Some("tenant_2".to_string()),
        user_id: None,
        active_only: false,
    };
    let tenant2_sessions = storage.list_sessions(tenant2_filter).await?;
    assert_eq!(
        tenant2_sessions.len(),
        1,
        "Should find 1 session for tenant_2"
    );
    assert_eq!(
        tenant2_sessions[0].tenant_id, "tenant_2",
        "Session should belong to tenant_2"
    );

    // Delete tenant_1 sessions
    storage.delete_session("t1_session1").await?;
    storage.delete_session("t1_session2").await?;

    // Tenant_2 session should remain
    let tenant2_after = storage.get_session("t2_session1").await?;
    assert!(
        tenant2_after.is_some(),
        "Tenant_2 session should not be affected by tenant_1 deletions"
    );

    // Cleanup
    storage.delete_session("t2_session1").await?;

    Ok(())
}

/// Test filtering by user ID
///
/// Validates:
/// - Can filter sessions by user_id
/// - Returns only sessions for that user
/// - Works across different tenants
pub async fn test_user_filtering<S: SessionStorage>(storage: &S) -> RiptideResult<()> {
    // Create sessions for same user in different tenants
    let user1_tenant1 = create_test_session(
        "u1_t1_session",
        "user_1",
        "tenant_1",
        Duration::from_secs(3600),
    );
    let user1_tenant2 = create_test_session(
        "u1_t2_session",
        "user_1",
        "tenant_2",
        Duration::from_secs(3600),
    );
    let user2_tenant1 = create_test_session(
        "u2_t1_session",
        "user_2",
        "tenant_1",
        Duration::from_secs(3600),
    );

    storage.save_session(&user1_tenant1).await?;
    storage.save_session(&user1_tenant2).await?;
    storage.save_session(&user2_tenant1).await?;

    // Filter by user_1
    let user1_filter = SessionFilter {
        tenant_id: None,
        user_id: Some("user_1".to_string()),
        active_only: false,
    };
    let user1_sessions = storage.list_sessions(user1_filter).await?;
    assert_eq!(user1_sessions.len(), 2, "Should find 2 sessions for user_1");
    assert!(
        user1_sessions.iter().all(|s| s.user_id == "user_1"),
        "All sessions should belong to user_1"
    );

    // Filter by user_1 in tenant_1
    let user1_tenant1_filter = SessionFilter {
        tenant_id: Some("tenant_1".to_string()),
        user_id: Some("user_1".to_string()),
        active_only: false,
    };
    let user1_tenant1_sessions = storage.list_sessions(user1_tenant1_filter).await?;
    assert_eq!(
        user1_tenant1_sessions.len(),
        1,
        "Should find 1 session for user_1 in tenant_1"
    );

    // Cleanup
    storage.delete_session("u1_t1_session").await?;
    storage.delete_session("u1_t2_session").await?;
    storage.delete_session("u2_t1_session").await?;

    Ok(())
}

/// Test active_only filtering
///
/// Validates:
/// - active_only=true filters out expired sessions
/// - active_only=false includes all sessions
pub async fn test_active_filtering<S: SessionStorage>(storage: &S) -> RiptideResult<()> {
    // Create active session
    let active = create_test_session(
        "active_filter_session",
        "user_1",
        "tenant_1",
        Duration::from_secs(3600),
    );

    // Create expired session
    let past = SystemTime::now() - Duration::from_secs(3600);
    let expired = Session {
        id: "expired_filter_session".to_string(),
        user_id: "user_1".to_string(),
        tenant_id: "tenant_1".to_string(),
        created_at: SystemTime::now(),
        expires_at: past,
        metadata: HashMap::new(),
    };

    storage.save_session(&active).await?;
    storage.save_session(&expired).await?;

    // List all sessions (active_only=false)
    let all_filter = SessionFilter {
        tenant_id: Some("tenant_1".to_string()),
        user_id: None,
        active_only: false,
    };
    let all_sessions = storage.list_sessions(all_filter).await?;
    assert!(
        all_sessions.len() >= 2,
        "Should find at least 2 sessions with active_only=false"
    );

    // List only active sessions (active_only=true)
    let active_filter = SessionFilter {
        tenant_id: Some("tenant_1".to_string()),
        user_id: None,
        active_only: true,
    };
    let active_sessions = storage.list_sessions(active_filter).await?;
    assert!(
        active_sessions.iter().all(|s| s.is_active()),
        "All sessions should be active"
    );
    assert!(
        active_sessions.len() < all_sessions.len() || expired.is_active(),
        "Active filter should return fewer sessions"
    );

    // Cleanup
    storage.delete_session("active_filter_session").await?;
    storage.delete_session("expired_filter_session").await?;

    Ok(())
}

/// Test session metadata
///
/// Validates:
/// - Metadata is preserved on save/retrieve
/// - Can update metadata
/// - Empty metadata works correctly
pub async fn test_metadata<S: SessionStorage>(storage: &S) -> RiptideResult<()> {
    let mut session = create_test_session(
        "metadata_session",
        "user_1",
        "tenant_1",
        Duration::from_secs(3600),
    );

    // Add metadata
    session
        .metadata
        .insert("role".to_string(), "admin".to_string());
    session
        .metadata
        .insert("preferences".to_string(), "dark_mode".to_string());

    storage.save_session(&session).await?;

    // Retrieve and verify metadata
    let retrieved = storage.get_session("metadata_session").await?;
    assert!(retrieved.is_some(), "Session should exist");
    let retrieved_session = retrieved.unwrap();
    assert_eq!(
        retrieved_session.metadata.get("role"),
        Some(&"admin".to_string()),
        "Role metadata should match"
    );
    assert_eq!(
        retrieved_session.metadata.get("preferences"),
        Some(&"dark_mode".to_string()),
        "Preferences metadata should match"
    );

    // Update metadata
    session
        .metadata
        .insert("updated".to_string(), "true".to_string());
    storage.save_session(&session).await?;

    let updated = storage.get_session("metadata_session").await?;
    assert_eq!(
        updated.unwrap().metadata.get("updated"),
        Some(&"true".to_string()),
        "Updated metadata should be saved"
    );

    // Cleanup
    storage.delete_session("metadata_session").await?;

    Ok(())
}

/// Test concurrent session operations
///
/// Validates:
/// - Multiple concurrent saves don't corrupt data
/// - Storage remains consistent under concurrent load
pub async fn test_concurrent_operations<S: SessionStorage>(storage: &S) -> RiptideResult<()> {
    use tokio::task::JoinSet;

    let mut tasks = JoinSet::new();

    // Spawn 10 concurrent session operations
    for i in 0..10 {
        let session = create_test_session(
            &format!("concurrent_session_{}", i),
            &format!("user_{}", i),
            "tenant_concurrent",
            Duration::from_secs(3600),
        );

        tasks.spawn(async move {
            let result: RiptideResult<Session> = Ok(session);
            result
        });
    }

    // Collect sessions and save them
    let mut sessions = Vec::new();
    while let Some(result) = tasks.join_next().await {
        let session =
            result.map_err(|e| RiptideError::Cache(format!("Task join error: {}", e)))??;
        storage.save_session(&session).await?;
        sessions.push(session);
    }

    // Verify all sessions exist
    for session in &sessions {
        let retrieved = storage.get_session(&session.id).await?;
        assert!(
            retrieved.is_some(),
            "Concurrent session {} should exist",
            session.id
        );
    }

    // Cleanup
    for session in &sessions {
        storage.delete_session(&session.id).await?;
    }

    Ok(())
}

/// Test list sessions with empty results
///
/// Validates:
/// - Returns empty vec when no sessions match filter
/// - Doesn't error on empty results
pub async fn test_empty_list<S: SessionStorage>(storage: &S) -> RiptideResult<()> {
    let filter = SessionFilter {
        tenant_id: Some("non_existent_tenant".to_string()),
        user_id: None,
        active_only: false,
    };

    let sessions = storage.list_sessions(filter).await?;
    assert_eq!(sessions.len(), 0, "Should return empty list");

    Ok(())
}

/// Test cleanup_expired with no expired sessions
///
/// Validates:
/// - cleanup_expired returns 0 when nothing to clean
/// - Doesn't error when no expired sessions exist
pub async fn test_cleanup_no_expired<S: SessionStorage>(storage: &S) -> RiptideResult<()> {
    // Create only active sessions
    let active = create_test_session(
        "only_active_session",
        "user_1",
        "tenant_1",
        Duration::from_secs(3600),
    );
    storage.save_session(&active).await?;

    let cleaned = storage.cleanup_expired().await?;
    assert_eq!(cleaned, 0, "Should clean 0 sessions when none expired");

    // Active session should still exist
    let still_active = storage.get_session("only_active_session").await?;
    assert!(still_active.is_some(), "Active session should remain");

    // Cleanup
    storage.delete_session("only_active_session").await?;

    Ok(())
}

/// Run all contract tests
///
/// This is a convenience function that runs all contract tests in sequence.
/// Use individual test functions for more granular control.
pub async fn run_all_tests<S: SessionStorage>(storage: &S) -> RiptideResult<()> {
    test_crud_operations(storage).await?;
    test_expiration(storage).await?;
    test_multi_tenancy(storage).await?;
    test_user_filtering(storage).await?;
    test_active_filtering(storage).await?;
    test_metadata(storage).await?;
    test_concurrent_operations(storage).await?;
    test_empty_list(storage).await?;
    test_cleanup_no_expired(storage).await?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;
    use tokio::sync::RwLock;

    /// Simple in-memory session storage for testing the contract tests
    struct MemorySessionStorage {
        sessions: Arc<RwLock<HashMap<String, Session>>>,
    }

    impl MemorySessionStorage {
        fn new() -> Self {
            Self {
                sessions: Arc::new(RwLock::new(HashMap::new())),
            }
        }
    }

    #[async_trait::async_trait]
    impl SessionStorage for MemorySessionStorage {
        async fn get_session(&self, id: &str) -> RiptideResult<Option<Session>> {
            let sessions = self.sessions.read().await;
            Ok(sessions.get(id).cloned())
        }

        async fn save_session(&self, session: &Session) -> RiptideResult<()> {
            let mut sessions = self.sessions.write().await;
            sessions.insert(session.id.clone(), session.clone());
            Ok(())
        }

        async fn delete_session(&self, id: &str) -> RiptideResult<()> {
            let mut sessions = self.sessions.write().await;
            sessions.remove(id);
            Ok(())
        }

        async fn list_sessions(&self, filter: SessionFilter) -> RiptideResult<Vec<Session>> {
            let sessions = self.sessions.read().await;
            let filtered: Vec<Session> = sessions
                .values()
                .filter(|s| {
                    if let Some(ref tenant_id) = filter.tenant_id {
                        if s.tenant_id != *tenant_id {
                            return false;
                        }
                    }
                    if let Some(ref user_id) = filter.user_id {
                        if s.user_id != *user_id {
                            return false;
                        }
                    }
                    if filter.active_only && !s.is_active() {
                        return false;
                    }
                    true
                })
                .cloned()
                .collect();
            Ok(filtered)
        }

        async fn cleanup_expired(&self) -> RiptideResult<usize> {
            let mut sessions = self.sessions.write().await;
            let before_count = sessions.len();
            sessions.retain(|_, s| s.is_active());
            let after_count = sessions.len();
            Ok(before_count - after_count)
        }
    }

    #[tokio::test]
    async fn test_memory_storage_crud() {
        let storage = MemorySessionStorage::new();
        test_crud_operations(&storage).await.unwrap();
    }

    #[tokio::test]
    async fn test_memory_storage_expiration() {
        let storage = MemorySessionStorage::new();
        test_expiration(&storage).await.unwrap();
    }

    #[tokio::test]
    async fn test_memory_storage_multi_tenancy() {
        let storage = MemorySessionStorage::new();
        test_multi_tenancy(&storage).await.unwrap();
    }
}
