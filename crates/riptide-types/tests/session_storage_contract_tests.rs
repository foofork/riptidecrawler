//! Integration tests for session_storage_contract module
//!
//! These tests validate that the contract test suite itself works correctly
//! by running it against a simple in-memory implementation.

mod contracts;

use contracts::session_storage_contract;
use riptide_types::error::Result as RiptideResult;
use riptide_types::ports::{Session, SessionFilter, SessionStorage};
use std::collections::HashMap;
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
async fn test_memory_session_storage_crud() {
    let storage = MemorySessionStorage::new();
    session_storage_contract::test_crud_operations(&storage)
        .await
        .unwrap();
}

#[tokio::test]
async fn test_memory_session_storage_expiration() {
    let storage = MemorySessionStorage::new();
    session_storage_contract::test_expiration(&storage)
        .await
        .unwrap();
}

#[tokio::test]
async fn test_memory_session_storage_multi_tenancy() {
    let storage = MemorySessionStorage::new();
    session_storage_contract::test_multi_tenancy(&storage)
        .await
        .unwrap();
}

#[tokio::test]
async fn test_memory_session_storage_user_filtering() {
    let storage = MemorySessionStorage::new();
    session_storage_contract::test_user_filtering(&storage)
        .await
        .unwrap();
}

#[tokio::test]
async fn test_memory_session_storage_active_filtering() {
    let storage = MemorySessionStorage::new();
    session_storage_contract::test_active_filtering(&storage)
        .await
        .unwrap();
}

#[tokio::test]
async fn test_memory_session_storage_metadata() {
    let storage = MemorySessionStorage::new();
    session_storage_contract::test_metadata(&storage)
        .await
        .unwrap();
}

#[tokio::test]
async fn test_memory_session_storage_concurrent() {
    let storage = MemorySessionStorage::new();
    session_storage_contract::test_concurrent_operations(&storage)
        .await
        .unwrap();
}

#[tokio::test]
async fn test_memory_session_storage_empty_list() {
    let storage = MemorySessionStorage::new();
    session_storage_contract::test_empty_list(&storage)
        .await
        .unwrap();
}

#[tokio::test]
async fn test_memory_session_storage_cleanup_no_expired() {
    let storage = MemorySessionStorage::new();
    session_storage_contract::test_cleanup_no_expired(&storage)
        .await
        .unwrap();
}

#[tokio::test]
async fn test_memory_session_storage_all_contracts() {
    let storage = MemorySessionStorage::new();
    session_storage_contract::run_all_tests(&storage)
        .await
        .unwrap();
}
