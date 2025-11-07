//! Comprehensive tests for session cleanup functionality

use super::storage::SessionStorage;
use super::types::SessionConfig;
use std::path::PathBuf;
use std::time::Duration;

/// Create a unique test configuration with isolated storage
fn create_test_config(test_name: &str) -> SessionConfig {
    SessionConfig {
        // Use a unique directory for each test to avoid interference
        base_data_dir: PathBuf::from(format!("/tmp/riptide-test-sessions-{}", test_name)),
        ..Default::default()
    }
}

#[tokio::test]
async fn test_cleanup_removes_expired_sessions() {
    // Create storage with short TTL for testing
    let mut config = create_test_config("removes_expired");
    config.default_ttl = Duration::from_millis(100); // 100ms TTL
    config.cleanup_interval = Duration::from_secs(3600); // Don't auto-cleanup during test

    let storage = SessionStorage::new(config.clone())
        .await
        .expect("Failed to create storage");

    // Create sessions
    let session1 = storage
        .create_session("test_expired_1".to_string())
        .await
        .expect("Failed to create session");
    let session2 = storage
        .create_session("test_expired_2".to_string())
        .await
        .expect("Failed to create session");

    // Verify sessions exist
    assert_eq!(storage.list_sessions().await.len(), 2);

    // Wait for sessions to expire
    tokio::time::sleep(Duration::from_millis(150)).await;

    // Run cleanup
    let stats = storage
        .cleanup_expired_with_stats()
        .await
        .expect("Cleanup failed");

    // Verify cleanup results
    assert_eq!(
        stats.sessions_removed, 2,
        "Should remove 2 expired sessions"
    );
    assert_eq!(
        stats.sessions_remaining, 0,
        "Should have 0 remaining sessions"
    );
    assert!(stats.memory_freed_bytes > 0, "Should report memory freed");
    assert!(
        stats.cleanup_duration_ms < 1000,
        "Cleanup should complete in under 1 second"
    );

    // Verify sessions are gone
    assert!(storage
        .get_session(&session1.session_id)
        .await
        .unwrap()
        .is_none());
    assert!(storage
        .get_session(&session2.session_id)
        .await
        .unwrap()
        .is_none());

    // Cleanup
    storage.shutdown();
}

#[tokio::test]
async fn test_cleanup_preserves_active_sessions() {
    let mut config = create_test_config("preserves_active");
    config.default_ttl = Duration::from_secs(3600); // 1 hour TTL
    config.cleanup_interval = Duration::from_secs(3600);

    let storage = SessionStorage::new(config)
        .await
        .expect("Failed to create storage");

    // Create active sessions
    let session1 = storage
        .create_session("test_active_1".to_string())
        .await
        .expect("Failed to create session");
    let session2 = storage
        .create_session("test_active_2".to_string())
        .await
        .expect("Failed to create session");

    // Run cleanup immediately
    let stats = storage
        .cleanup_expired_with_stats()
        .await
        .expect("Cleanup failed");

    // Verify no sessions were removed
    assert_eq!(
        stats.sessions_removed, 0,
        "Should not remove active sessions"
    );
    assert_eq!(
        stats.sessions_remaining, 2,
        "Should preserve both active sessions"
    );
    assert_eq!(
        stats.memory_freed_bytes, 0,
        "Should not free any memory for active sessions"
    );

    // Verify sessions still exist
    assert!(storage
        .get_session(&session1.session_id)
        .await
        .unwrap()
        .is_some());
    assert!(storage
        .get_session(&session2.session_id)
        .await
        .unwrap()
        .is_some());

    // Cleanup
    storage.shutdown();
}

#[tokio::test]
async fn test_cleanup_mixed_expired_and_active() {
    let mut config = create_test_config("mixed_expired_active");
    config.default_ttl = Duration::from_millis(100); // Short TTL for testing
    config.cleanup_interval = Duration::from_secs(3600);

    let storage = SessionStorage::new(config.clone())
        .await
        .expect("Failed to create storage");

    // Create an expired session first
    let expired_session = storage
        .create_session("test_expired".to_string())
        .await
        .expect("Failed to create session");

    // Wait for it to expire
    tokio::time::sleep(Duration::from_millis(150)).await;

    // Now create a fresh active session (this will have a new expiry time)
    let active_session = storage
        .create_session("test_active".to_string())
        .await
        .expect("Failed to create active session");

    // Verify initial state - both sessions should be in memory
    let initial_sessions = storage.list_sessions().await;
    assert_eq!(
        initial_sessions.len(),
        2,
        "Should have 2 sessions before cleanup"
    );

    // Run cleanup
    let stats = storage
        .cleanup_expired_with_stats()
        .await
        .expect("Cleanup failed");

    // Verify cleanup removed only expired session
    assert_eq!(stats.sessions_removed, 1, "Should remove 1 expired session");
    assert_eq!(
        stats.sessions_remaining, 1,
        "Should preserve 1 active session"
    );

    // Verify correct sessions were removed/preserved
    assert!(
        storage
            .get_session(&expired_session.session_id)
            .await
            .unwrap()
            .is_none(),
        "Expired session should be removed"
    );
    assert!(
        storage
            .get_session(&active_session.session_id)
            .await
            .unwrap()
            .is_some(),
        "Active session should be preserved"
    );

    // Cleanup
    storage.shutdown();
}

#[tokio::test]
async fn test_cleanup_thread_safety() {
    let mut config = create_test_config("thread_safety");
    config.default_ttl = Duration::from_millis(50);
    config.cleanup_interval = Duration::from_secs(3600);

    let storage = SessionStorage::new(config)
        .await
        .expect("Failed to create storage");

    // Spawn multiple concurrent tasks creating sessions
    let mut handles = vec![];
    for i in 0..10 {
        let storage_clone = storage.clone();
        let handle = tokio::spawn(async move {
            storage_clone
                .create_session(format!("concurrent_{}", i))
                .await
                .expect("Failed to create session");
        });
        handles.push(handle);
    }

    // Wait for all sessions to be created
    for handle in handles {
        handle.await.expect("Task failed");
    }

    // Verify all sessions created
    assert_eq!(storage.list_sessions().await.len(), 10);

    // Wait for expiration
    tokio::time::sleep(Duration::from_millis(100)).await;

    // Run multiple concurrent cleanups
    let storage1 = storage.clone();
    let storage2 = storage.clone();
    let storage3 = storage.clone();

    let (result1, result2, result3) = tokio::join!(
        storage1.cleanup_expired_with_stats(),
        storage2.cleanup_expired_with_stats(),
        storage3.cleanup_expired_with_stats(),
    );

    // At least one cleanup should succeed
    assert!(
        result1.is_ok() || result2.is_ok() || result3.is_ok(),
        "At least one cleanup should succeed"
    );

    // Eventually all sessions should be removed
    let final_count = storage.list_sessions().await.len();
    assert_eq!(final_count, 0, "All expired sessions should be removed");

    // Cleanup
    storage.shutdown();
}

#[tokio::test]
async fn test_cleanup_statistics_tracking() {
    let mut config = create_test_config("stats_tracking");
    config.default_ttl = Duration::from_millis(100);
    config.cleanup_interval = Duration::from_secs(3600);

    let storage = SessionStorage::new(config)
        .await
        .expect("Failed to create storage");

    // Create sessions
    for i in 0..5 {
        storage
            .create_session(format!("test_stats_{}", i))
            .await
            .expect("Failed to create session");
    }

    // Wait for expiration
    tokio::time::sleep(Duration::from_millis(150)).await;

    // Run cleanup
    let stats = storage
        .cleanup_expired_with_stats()
        .await
        .expect("Cleanup failed");

    // Verify statistics
    assert_eq!(stats.sessions_removed, 5);
    assert_eq!(stats.sessions_remaining, 0);
    assert!(stats.memory_freed_bytes > 0);
    assert!(stats.cleanup_duration_ms < 5000);

    // Verify stats are stored
    let last_stats = storage
        .get_last_cleanup_stats()
        .await
        .expect("Should have last cleanup stats");
    assert_eq!(last_stats.sessions_removed, 5);

    // Verify session stats updated
    let session_stats = storage.get_stats().await.expect("Failed to get stats");
    assert_eq!(session_stats.expired_sessions_cleaned, 5);

    // Cleanup
    storage.shutdown();
}

#[tokio::test]
async fn test_graceful_shutdown() {
    let mut config = create_test_config("graceful_shutdown");
    config.default_ttl = Duration::from_secs(1);
    config.cleanup_interval = Duration::from_millis(100); // Fast cleanup for testing

    let storage = SessionStorage::new(config)
        .await
        .expect("Failed to create storage");

    // Create some sessions
    for i in 0..3 {
        storage
            .create_session(format!("shutdown_test_{}", i))
            .await
            .expect("Failed to create session");
    }

    // Let cleanup run at least once
    tokio::time::sleep(Duration::from_millis(150)).await;

    // Signal shutdown
    storage.shutdown();

    // Wait briefly for shutdown to complete
    tokio::time::sleep(Duration::from_millis(100)).await;

    // Verify storage is still functional after shutdown
    // (shutdown only affects background task)
    let remaining = storage.list_sessions().await;
    assert!(remaining.len() <= 3);
}

#[tokio::test]
async fn test_cleanup_empty_storage() {
    let config = create_test_config("empty_storage");
    let storage = SessionStorage::new(config)
        .await
        .expect("Failed to create storage");

    // Run cleanup on empty storage
    let stats = storage
        .cleanup_expired_with_stats()
        .await
        .expect("Cleanup should succeed on empty storage");

    assert_eq!(stats.sessions_removed, 0);
    assert_eq!(stats.sessions_remaining, 0);
    assert_eq!(stats.memory_freed_bytes, 0);

    // Cleanup
    storage.shutdown();
}

#[tokio::test]
async fn test_session_config_from_env() {
    // Set environment variables
    std::env::set_var("SESSION_TTL_SECS", "7200");
    std::env::set_var("SESSION_CLEANUP_INTERVAL_SECS", "600");
    std::env::set_var("SESSION_MAX_SESSIONS", "500");
    std::env::set_var("SESSION_PERSIST_COOKIES", "false");

    let config = SessionConfig::from_env();

    assert_eq!(config.default_ttl, Duration::from_secs(7200));
    assert_eq!(config.cleanup_interval, Duration::from_secs(600));
    assert_eq!(config.max_sessions, 500);
    assert!(!config.persist_cookies);

    // Clean up environment
    std::env::remove_var("SESSION_TTL_SECS");
    std::env::remove_var("SESSION_CLEANUP_INTERVAL_SECS");
    std::env::remove_var("SESSION_MAX_SESSIONS");
    std::env::remove_var("SESSION_PERSIST_COOKIES");
}

#[tokio::test]
async fn test_memory_freed_estimation() {
    let mut config = create_test_config("memory_estimation");
    config.default_ttl = Duration::from_millis(50);
    config.cleanup_interval = Duration::from_secs(3600);

    let storage = SessionStorage::new(config)
        .await
        .expect("Failed to create storage");

    // Create exactly 10 sessions
    for i in 0..10 {
        storage
            .create_session(format!("memory_test_{}", i))
            .await
            .expect("Failed to create session");
    }

    // Wait for expiration
    tokio::time::sleep(Duration::from_millis(100)).await;

    // Run cleanup
    let stats = storage
        .cleanup_expired_with_stats()
        .await
        .expect("Cleanup failed");

    // Each session should free ~8KB (8192 bytes)
    let expected_memory = 10 * 8192;
    assert_eq!(
        stats.memory_freed_bytes, expected_memory,
        "Memory freed should be 10 sessions * 8KB"
    );

    // Cleanup
    storage.shutdown();
}
