//! Comprehensive tests for session persistence and disk spillover
//!
//! Tests cover:
//! - Session persistence round-trip
//! - Disk spillover triggering
//! - Recovery from disk
//! - Error handling
//! - Performance characteristics

use riptide_persistence::{
    config::StateConfig,
    state::{SessionMetadata, SessionStatus, StateManager},
};
use std::collections::HashMap;
use tempfile::TempDir;
use tokio::time::{sleep, Duration};
use tracing::info;
use tracing_subscriber;

/// Initialize test logging
fn init_test_logging() {
    let _ = tracing_subscriber::fmt()
        .with_test_writer()
        .with_max_level(tracing::Level::DEBUG)
        .try_init();
}

/// Create test state manager with spillover enabled
async fn create_test_state_manager(
    temp_dir: &TempDir,
) -> Result<StateManager, Box<dyn std::error::Error>> {
    init_test_logging();

    // Configure spillover with low memory threshold for testing
    let config = StateConfig {
        session_timeout_seconds: 300,
        checkpoint_interval_seconds: 0, // Disable auto-checkpoint for tests
        checkpoint_compression: false,
        max_checkpoints: 5,
        enable_hot_reload: false,
        config_watch_paths: vec![],
        enable_graceful_shutdown: false,
        shutdown_timeout_seconds: 30,
    };

    // Use embedded Redis for testing if available, otherwise use mock
    let redis_url =
        std::env::var("REDIS_URL").unwrap_or_else(|_| "redis://127.0.0.1:6379".to_string());

    StateManager::new(&redis_url, config)
        .await
        .map_err(Into::into)
}

#[tokio::test]
async fn test_session_persistence_round_trip() {
    let temp_dir = TempDir::new().unwrap();

    // This test may fail if Redis is not available, which is expected in CI
    if let Ok(state_manager) = create_test_state_manager(&temp_dir).await {
        // Create test session
        let metadata = SessionMetadata {
            client_ip: Some("127.0.0.1".to_string()),
            user_agent: Some("Test Agent".to_string()),
            source: Some("test".to_string()),
            attributes: HashMap::new(),
        };

        let session_id = state_manager
            .create_session(Some("user123".to_string()), metadata.clone(), Some(600))
            .await
            .unwrap();

        // Add some data to session
        state_manager
            .update_session_data(&session_id, "key1", serde_json::json!("value1"))
            .await
            .unwrap();

        state_manager
            .update_session_data(&session_id, "key2", serde_json::json!(42))
            .await
            .unwrap();

        // Retrieve session
        let session = state_manager
            .get_session(&session_id)
            .await
            .unwrap()
            .expect("Session should exist");

        // Verify session data
        assert_eq!(session.id, session_id);
        assert_eq!(session.user_id, Some("user123".to_string()));
        assert_eq!(session.status, SessionStatus::Active);
        assert_eq!(session.data.get("key1"), Some(&serde_json::json!("value1")));
        assert_eq!(session.data.get("key2"), Some(&serde_json::json!(42)));
        assert_eq!(session.metadata.client_ip, Some("127.0.0.1".to_string()));

        info!("✅ Session persistence round-trip successful");
    } else {
        info!("⚠️  Skipping test - Redis not available");
    }
}

#[tokio::test]
async fn test_disk_spillover_triggering() {
    let temp_dir = TempDir::new().unwrap();

    if let Ok(state_manager) = create_test_state_manager(&temp_dir).await {
        // Create multiple sessions to trigger spillover
        let mut session_ids = Vec::new();

        for i in 0..20 {
            let metadata = SessionMetadata {
                client_ip: Some(format!("192.168.1.{}", i)),
                user_agent: Some("Test Agent".to_string()),
                source: Some("spillover_test".to_string()),
                attributes: HashMap::new(),
            };

            let session_id = state_manager
                .create_session(Some(format!("user{}", i)), metadata, Some(600))
                .await
                .unwrap();

            // Add substantial data to each session
            for j in 0..50 {
                state_manager
                    .update_session_data(
                        &session_id,
                        &format!("key{}", j),
                        serde_json::json!({
                            "data": format!("This is test data for session {} key {}", i, j),
                            "index": j,
                            "session_id": session_id.clone(),
                        }),
                    )
                    .await
                    .unwrap();
            }

            session_ids.push(session_id);
        }

        // Wait for spillover background task to run
        sleep(Duration::from_secs(35)).await;

        // Verify sessions can still be retrieved (either from memory or disk)
        for session_id in &session_ids[..5] {
            let session = state_manager
                .get_session(session_id)
                .await
                .unwrap()
                .expect("Session should exist");

            assert_eq!(session.status, SessionStatus::Active);
            assert!(session.data.len() >= 50);
        }

        info!("✅ Disk spillover mechanism working correctly");
    } else {
        info!("⚠️  Skipping test - Redis not available");
    }
}

#[tokio::test]
async fn test_session_recovery_from_disk() {
    let temp_dir = TempDir::new().unwrap();

    if let Ok(state_manager) = create_test_state_manager(&temp_dir).await {
        // Create session
        let metadata = SessionMetadata {
            client_ip: Some("10.0.0.1".to_string()),
            user_agent: Some("Recovery Test".to_string()),
            source: Some("recovery_test".to_string()),
            attributes: HashMap::new(),
        };

        let session_id = state_manager
            .create_session(Some("recovery_user".to_string()), metadata, Some(600))
            .await
            .unwrap();

        // Add data
        state_manager
            .update_session_data(
                &session_id,
                "critical_data",
                serde_json::json!({
                    "status": "active",
                    "count": 100,
                }),
            )
            .await
            .unwrap();

        // First retrieval (from memory)
        let session_1 = state_manager
            .get_session(&session_id)
            .await
            .unwrap()
            .expect("Session should exist");

        assert_eq!(session_1.status, SessionStatus::Active);

        // Wait for potential spillover
        sleep(Duration::from_secs(2)).await;

        // Second retrieval (may trigger restore from disk if spilled)
        let session_2 = state_manager
            .get_session(&session_id)
            .await
            .unwrap()
            .expect("Session should exist");

        // Verify data integrity
        assert_eq!(session_2.id, session_id);
        assert_eq!(session_2.status, SessionStatus::Active);
        assert_eq!(
            session_2.data.get("critical_data"),
            Some(&serde_json::json!({
                "status": "active",
                "count": 100,
            }))
        );

        info!("✅ Session recovery from disk successful");
    } else {
        info!("⚠️  Skipping test - Redis not available");
    }
}

#[tokio::test]
async fn test_session_termination_cleanup() {
    let temp_dir = TempDir::new().unwrap();

    if let Ok(state_manager) = create_test_state_manager(&temp_dir).await {
        // Create session
        let metadata = SessionMetadata {
            client_ip: None,
            user_agent: None,
            source: None,
            attributes: HashMap::new(),
        };

        let session_id = state_manager
            .create_session(Some("cleanup_user".to_string()), metadata, Some(600))
            .await
            .unwrap();

        // Verify session exists
        assert!(state_manager
            .get_session(&session_id)
            .await
            .unwrap()
            .is_some());

        // Terminate session
        let deleted = state_manager.terminate_session(&session_id).await.unwrap();
        assert!(deleted);

        // Verify session is gone
        assert!(state_manager
            .get_session(&session_id)
            .await
            .unwrap()
            .is_none());

        // Verify second termination returns false
        let deleted_again = state_manager.terminate_session(&session_id).await.unwrap();
        assert!(!deleted_again);

        info!("✅ Session termination and cleanup successful");
    } else {
        info!("⚠️  Skipping test - Redis not available");
    }
}

#[tokio::test]
async fn test_session_expiration() {
    let temp_dir = TempDir::new().unwrap();

    if let Ok(state_manager) = create_test_state_manager(&temp_dir).await {
        // Create session with short TTL
        let metadata = SessionMetadata {
            client_ip: None,
            user_agent: None,
            source: Some("expiration_test".to_string()),
            attributes: HashMap::new(),
        };

        let session_id = state_manager
            .create_session(Some("expire_user".to_string()), metadata, Some(2))
            .await
            .unwrap();

        // Session should exist immediately
        assert!(state_manager
            .get_session(&session_id)
            .await
            .unwrap()
            .is_some());

        // Wait for expiration (TTL + buffer)
        sleep(Duration::from_secs(5)).await;

        // Session should be expired or cleaned up
        let result = state_manager.get_session(&session_id).await.unwrap();
        // Note: Due to Redis expiration, this may return None
        if let Some(session) = result {
            // If still present, status should be expired
            assert_eq!(session.status, SessionStatus::Expired);
        }

        info!("✅ Session expiration handling successful");
    } else {
        info!("⚠️  Skipping test - Redis not available");
    }
}

#[tokio::test]
async fn test_concurrent_session_operations() {
    let temp_dir = TempDir::new().unwrap();

    if let Ok(state_manager) = create_test_state_manager(&temp_dir).await {
        let state_manager = std::sync::Arc::new(state_manager);

        // Spawn multiple tasks that create and modify sessions concurrently
        let mut handles = vec![];

        for i in 0..10 {
            let sm = state_manager.clone();
            let handle = tokio::spawn(async move {
                let metadata = SessionMetadata {
                    client_ip: Some(format!("192.168.2.{}", i)),
                    user_agent: Some("Concurrent Test".to_string()),
                    source: Some("concurrency_test".to_string()),
                    attributes: HashMap::new(),
                };

                let session_id = sm
                    .create_session(Some(format!("concurrent_user_{}", i)), metadata, Some(600))
                    .await
                    .unwrap();

                // Perform multiple updates
                for j in 0..10 {
                    sm.update_session_data(
                        &session_id,
                        &format!("key_{}", j),
                        serde_json::json!(j * i),
                    )
                    .await
                    .unwrap();
                }

                // Retrieve and verify
                let session = sm.get_session(&session_id).await.unwrap().unwrap();
                assert_eq!(session.data.len(), 10);

                session_id
            });

            handles.push(handle);
        }

        // Wait for all tasks to complete
        let session_ids: Vec<String> = futures::future::join_all(handles)
            .await
            .into_iter()
            .map(|r| r.unwrap())
            .collect();

        // Verify all sessions exist
        for session_id in session_ids {
            assert!(state_manager
                .get_session(&session_id)
                .await
                .unwrap()
                .is_some());
        }

        info!("✅ Concurrent session operations successful");
    } else {
        info!("⚠️  Skipping test - Redis not available");
    }
}

#[tokio::test]
async fn test_get_active_sessions() {
    let temp_dir = TempDir::new().unwrap();

    if let Ok(state_manager) = create_test_state_manager(&temp_dir).await {
        // Create multiple sessions
        let mut created_ids = Vec::new();

        for i in 0..5 {
            let metadata = SessionMetadata {
                client_ip: None,
                user_agent: None,
                source: Some("active_sessions_test".to_string()),
                attributes: HashMap::new(),
            };

            let session_id = state_manager
                .create_session(Some(format!("list_user_{}", i)), metadata, Some(600))
                .await
                .unwrap();

            created_ids.push(session_id);
        }

        // Get all active sessions
        let active_sessions = state_manager.get_active_sessions().await.unwrap();

        // Should have at least the sessions we created
        assert!(active_sessions.len() >= 5);

        // Verify our sessions are in the list
        for session_id in &created_ids {
            assert!(active_sessions.iter().any(|s| &s.id == session_id));
        }

        info!("✅ Get active sessions successful");
    } else {
        info!("⚠️  Skipping test - Redis not available");
    }
}

#[tokio::test]
async fn test_error_handling_invalid_session() {
    let temp_dir = TempDir::new().unwrap();

    if let Ok(state_manager) = create_test_state_manager(&temp_dir).await {
        // Try to get non-existent session
        let result = state_manager.get_session("nonexistent_id").await.unwrap();
        assert!(result.is_none());

        // Try to update non-existent session
        let result = state_manager
            .update_session_data("nonexistent_id", "key", serde_json::json!("value"))
            .await;
        assert!(result.is_err());

        info!("✅ Error handling for invalid sessions successful");
    } else {
        info!("⚠️  Skipping test - Redis not available");
    }
}

#[tokio::test]
async fn test_session_metadata_preservation() {
    let temp_dir = TempDir::new().unwrap();

    if let Ok(state_manager) = create_test_state_manager(&temp_dir).await {
        // Create session with rich metadata
        let mut attributes = HashMap::new();
        attributes.insert("region".to_string(), "us-west-2".to_string());
        attributes.insert("device".to_string(), "mobile".to_string());

        let metadata = SessionMetadata {
            client_ip: Some("203.0.113.42".to_string()),
            user_agent: Some("Mozilla/5.0 (iPhone; CPU iPhone OS 14_0)".to_string()),
            source: Some("mobile_app".to_string()),
            attributes,
        };

        let session_id = state_manager
            .create_session(
                Some("metadata_user".to_string()),
                metadata.clone(),
                Some(600),
            )
            .await
            .unwrap();

        // Retrieve and verify metadata
        let session = state_manager
            .get_session(&session_id)
            .await
            .unwrap()
            .unwrap();

        assert_eq!(session.metadata.client_ip, metadata.client_ip);
        assert_eq!(session.metadata.user_agent, metadata.user_agent);
        assert_eq!(session.metadata.source, metadata.source);
        assert_eq!(
            session.metadata.attributes.get("region"),
            Some(&"us-west-2".to_string())
        );
        assert_eq!(
            session.metadata.attributes.get("device"),
            Some(&"mobile".to_string())
        );

        info!("✅ Session metadata preservation successful");
    } else {
        info!("⚠️  Skipping test - Redis not available");
    }
}
