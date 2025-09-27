/*!
# State Management Integration Tests

Tests for session persistence, configuration hot-reload, and checkpoint/restore functionality.
*/

use super::*;
use riptide_persistence::{
    StateManager, SessionMetadata, CheckpointType, Checkpoint
};
use std::collections::HashMap;
use tokio::time::Duration;

#[tokio::test]
async fn test_session_lifecycle() -> Result<(), Box<dyn std::error::Error>> {
    let config = create_test_state_config();
    let redis_url = get_test_redis_url();

    let state_manager = StateManager::new(&redis_url, config).await?;

    // Create session
    let metadata = SessionMetadata {
        client_ip: Some("192.168.1.100".to_string()),
        user_agent: Some("TestAgent/1.0".to_string()),
        source: Some("integration_test".to_string()),
        attributes: {
            let mut attrs = HashMap::new();
            attrs.insert("test_attr".to_string(), "test_value".to_string());
            attrs
        },
    };

    let session_id = state_manager.create_session(
        Some("test_user_123".to_string()),
        metadata,
        Some(3600), // 1 hour
    ).await?;

    assert!(!session_id.is_empty());

    // Retrieve session
    let session = state_manager.get_session(&session_id).await?;
    assert!(session.is_some());

    let session = session.unwrap();
    assert_eq!(session.user_id.unwrap(), "test_user_123");
    assert_eq!(session.metadata.client_ip.unwrap(), "192.168.1.100");

    // Update session data
    state_manager.update_session_data(
        &session_id,
        "key1",
        serde_json::json!("value1")
    ).await?;

    state_manager.update_session_data(
        &session_id,
        "key2",
        serde_json::json!({"nested": "object"})
    ).await?;

    // Verify updates
    let updated_session = state_manager.get_session(&session_id).await?;
    assert!(updated_session.is_some());

    let updated_session = updated_session.unwrap();
    assert_eq!(updated_session.data.get("key1").unwrap(), &serde_json::json!("value1"));
    assert_eq!(
        updated_session.data.get("key2").unwrap(),
        &serde_json::json!({"nested": "object"})
    );

    // Get all active sessions
    let active_sessions = state_manager.get_active_sessions().await?;
    assert!(active_sessions.len() >= 1);

    // Terminate session
    let terminated = state_manager.terminate_session(&session_id).await?;
    assert!(terminated);

    // Verify session is gone
    let terminated_session = state_manager.get_session(&session_id).await?;
    assert!(terminated_session.is_none());

    Ok(())
}

#[tokio::test]
async fn test_multiple_sessions() -> Result<(), Box<dyn std::error::Error>> {
    let config = create_test_state_config();
    let redis_url = get_test_redis_url();

    let state_manager = StateManager::new(&redis_url, config).await?;

    // Create multiple sessions
    let mut session_ids = Vec::new();
    for i in 0..5 {
        let metadata = SessionMetadata {
            client_ip: Some(format!("192.168.1.{}", 100 + i)),
            user_agent: Some(format!("TestAgent/{}", i)),
            source: Some("batch_test".to_string()),
            attributes: HashMap::new(),
        };

        let session_id = state_manager.create_session(
            Some(format!("user_{}", i)),
            metadata,
            Some(1800), // 30 minutes
        ).await?;

        session_ids.push(session_id);
    }

    // Verify all sessions exist
    let active_sessions = state_manager.get_active_sessions().await?;
    assert!(active_sessions.len() >= 5);

    // Add data to each session
    for (i, session_id) in session_ids.iter().enumerate() {
        state_manager.update_session_data(
            session_id,
            "user_data",
            serde_json::json!({"user_id": i, "name": format!("User {}", i)})
        ).await?;
    }

    // Verify data for random session
    if let Some(session_id) = session_ids.get(2) {
        let session = state_manager.get_session(session_id).await?;
        assert!(session.is_some());

        let session = session.unwrap();
        let user_data = session.data.get("user_data").unwrap();
        assert_eq!(user_data["user_id"], 2);
        assert_eq!(user_data["name"], "User 2");
    }

    // Clean up
    for session_id in session_ids {
        state_manager.terminate_session(&session_id).await?;
    }

    Ok(())
}

#[tokio::test]
async fn test_session_expiration() -> Result<(), Box<dyn std::error::Error>> {
    let mut config = create_test_state_config();
    config.session_timeout_seconds = 2; // 2 seconds for test

    let redis_url = get_test_redis_url();
    let state_manager = StateManager::new(&redis_url, config).await?;

    // Create session with short TTL
    let metadata = SessionMetadata {
        client_ip: Some("127.0.0.1".to_string()),
        user_agent: Some("TestAgent".to_string()),
        source: Some("expiration_test".to_string()),
        attributes: HashMap::new(),
    };

    let session_id = state_manager.create_session(
        Some("test_user".to_string()),
        metadata,
        Some(1), // 1 second TTL
    ).await?;

    // Should exist immediately
    let session = state_manager.get_session(&session_id).await?;
    assert!(session.is_some());

    // Wait for expiration
    tokio::time::sleep(Duration::from_secs(2)).await;

    // Should be expired now
    let expired_session = state_manager.get_session(&session_id).await?;
    assert!(expired_session.is_none());

    Ok(())
}

#[tokio::test]
async fn test_checkpoint_create_and_restore() -> Result<(), Box<dyn std::error::Error>> {
    let config = create_test_state_config();
    let redis_url = get_test_redis_url();

    let state_manager = StateManager::new(&redis_url, config).await?;

    // Create some state to checkpoint
    let session_ids = vec![
        state_manager.create_session(
            Some("user1".to_string()),
            SessionMetadata {
                client_ip: Some("192.168.1.1".to_string()),
                user_agent: Some("Agent1".to_string()),
                source: Some("checkpoint_test".to_string()),
                attributes: HashMap::new(),
            },
            Some(3600),
        ).await?,
        state_manager.create_session(
            Some("user2".to_string()),
            SessionMetadata {
                client_ip: Some("192.168.1.2".to_string()),
                user_agent: Some("Agent2".to_string()),
                source: Some("checkpoint_test".to_string()),
                attributes: HashMap::new(),
            },
            Some(3600),
        ).await?,
    ];

    // Add data to sessions
    for (i, session_id) in session_ids.iter().enumerate() {
        state_manager.update_session_data(
            session_id,
            "checkpoint_data",
            serde_json::json!({"checkpoint_id": i, "data": format!("checkpoint_data_{}", i)})
        ).await?;
    }

    // Create checkpoint
    let checkpoint_id = state_manager.create_checkpoint(
        CheckpointType::Manual,
        Some("Test checkpoint for restoration".to_string()),
    ).await?;

    assert!(!checkpoint_id.is_empty());

    // Clear sessions to simulate data loss
    for session_id in &session_ids {
        state_manager.terminate_session(session_id).await?;
    }

    // Verify sessions are gone
    let active_sessions_before_restore = state_manager.get_active_sessions().await?;
    assert!(active_sessions_before_restore.len() < session_ids.len());

    // Restore from checkpoint
    state_manager.restore_from_checkpoint(&checkpoint_id).await?;

    // Verify sessions are restored
    let active_sessions_after_restore = state_manager.get_active_sessions().await?;
    assert!(active_sessions_after_restore.len() >= session_ids.len());

    // Verify session data is restored
    for (i, session_id) in session_ids.iter().enumerate() {
        if let Some(restored_session) = state_manager.get_session(session_id).await? {
            let checkpoint_data = restored_session.data.get("checkpoint_data").unwrap();
            assert_eq!(checkpoint_data["checkpoint_id"], i);
        }
    }

    Ok(())
}

#[tokio::test]
async fn test_scheduled_checkpoints() -> Result<(), Box<dyn std::error::Error>> {
    let mut config = create_test_state_config();
    config.checkpoint_interval_seconds = 1; // 1 second for test

    let redis_url = get_test_redis_url();
    let state_manager = StateManager::new(&redis_url, config).await?;

    // Create some state
    let session_id = state_manager.create_session(
        Some("scheduled_test_user".to_string()),
        SessionMetadata {
            client_ip: Some("127.0.0.1".to_string()),
            user_agent: Some("ScheduledTestAgent".to_string()),
            source: Some("scheduled_checkpoint_test".to_string()),
            attributes: HashMap::new(),
        },
        Some(3600),
    ).await?;

    // Wait for scheduled checkpoint to trigger
    tokio::time::sleep(Duration::from_secs(2)).await;

    // Clean up
    state_manager.terminate_session(&session_id).await?;

    Ok(())
}

#[tokio::test]
async fn test_graceful_shutdown() -> Result<(), Box<dyn std::error::Error>> {
    let config = create_test_state_config();
    let redis_url = get_test_redis_url();

    let state_manager = StateManager::new(&redis_url, config).await?;

    // Create some sessions
    let session_ids = vec![
        state_manager.create_session(
            Some("shutdown_user1".to_string()),
            SessionMetadata {
                client_ip: Some("192.168.1.10".to_string()),
                user_agent: Some("ShutdownAgent1".to_string()),
                source: Some("shutdown_test".to_string()),
                attributes: HashMap::new(),
            },
            Some(3600),
        ).await?,
        state_manager.create_session(
            Some("shutdown_user2".to_string()),
            SessionMetadata {
                client_ip: Some("192.168.1.11".to_string()),
                user_agent: Some("ShutdownAgent2".to_string()),
                source: Some("shutdown_test".to_string()),
                attributes: HashMap::new(),
            },
            Some(3600),
        ).await?,
    ];

    // Verify sessions exist
    let active_before = state_manager.get_active_sessions().await?;
    assert!(active_before.len() >= 2);

    // Trigger graceful shutdown
    state_manager.shutdown_gracefully().await?;

    // Verify sessions are terminated
    let active_after = state_manager.get_active_sessions().await?;
    assert!(active_after.len() < active_before.len());

    Ok(())
}

#[tokio::test]
async fn test_checkpoint_compression() -> Result<(), Box<dyn std::error::Error>> {
    let mut config = create_test_state_config();
    config.checkpoint_compression = true;

    let redis_url = get_test_redis_url();
    let state_manager = StateManager::new(&redis_url, config).await?;

    // Create sessions with substantial data
    let session_id = state_manager.create_session(
        Some("compression_user".to_string()),
        SessionMetadata {
            client_ip: Some("192.168.1.100".to_string()),
            user_agent: Some("CompressionTestAgent".to_string()),
            source: Some("compression_test".to_string()),
            attributes: HashMap::new(),
        },
        Some(3600),
    ).await?;

    // Add large data to trigger compression
    let large_data = serde_json::json!({
        "large_text": "x".repeat(2048),
        "array": (0..100).collect::<Vec<i32>>(),
        "nested": {
            "level1": {
                "level2": {
                    "data": "y".repeat(1024)
                }
            }
        }
    });

    state_manager.update_session_data(&session_id, "large_data", large_data).await?;

    // Create compressed checkpoint
    let checkpoint_id = state_manager.create_checkpoint(
        CheckpointType::Manual,
        Some("Compression test checkpoint".to_string()),
    ).await?;

    // Restore to verify compression worked correctly
    state_manager.restore_from_checkpoint(&checkpoint_id).await?;

    // Verify data integrity after compression/decompression
    let restored_session = state_manager.get_session(&session_id).await?;
    assert!(restored_session.is_some());

    let restored_session = restored_session.unwrap();
    let restored_data = restored_session.data.get("large_data").unwrap();
    assert_eq!(restored_data["large_text"].as_str().unwrap().len(), 2048);
    assert_eq!(restored_data["array"].as_array().unwrap().len(), 100);

    Ok(())
}