/*!
# Performance Integration Tests

Tests to validate performance targets and benchmarks for the persistence layer.
*/

use super::*;
use riptide_persistence::{PersistentCacheManager, StateManager, TenantManager};
use std::time::{Duration, Instant};
use std::sync::Arc;
use tokio::sync::RwLock;

#[tokio::test]
async fn test_cache_access_performance_target() -> Result<(), Box<dyn std::error::Error>> {
    let config = create_test_cache_config();
    let redis_url = get_test_redis_url();

    cleanup_test_data(&redis_url, &config.key_prefix).await?;

    let cache_manager = PersistentCacheManager::new(&redis_url, config.clone()).await?;

    // Pre-populate cache with test data
    let test_data = serde_json::json!({
        "performance_test": true,
        "data": "x".repeat(1024), // 1KB data
        "timestamp": chrono::Utc::now().to_rfc3339()
    });

    cache_manager.set("perf_test_key", &test_data, None, None, None).await?;

    // Measure cache access performance (target: <5ms)
    let iterations = 100;
    let mut total_time = Duration::new(0, 0);

    for i in 0..iterations {
        let start = Instant::now();
        let _retrieved: Option<serde_json::Value> = cache_manager.get("perf_test_key", None).await?;
        let elapsed = start.elapsed();
        total_time += elapsed;

        // Individual access should be under target
        if elapsed.as_millis() > 50 {
            println!("Warning: Cache access {} took {}ms (target: <50ms for tests)", i, elapsed.as_millis());
        }
    }

    let avg_time = total_time / iterations;
    println!("Average cache access time: {:?} (target: <5ms production, <50ms tests)", avg_time);

    // In tests, we use a more relaxed target due to potential CI/test environment constraints
    assert!(avg_time.as_millis() < 50, "Average cache access time should be under 50ms in tests");

    cleanup_test_data(&redis_url, &config.key_prefix).await?;
    Ok(())
}

#[tokio::test]
async fn test_cache_throughput() -> Result<(), Box<dyn std::error::Error>> {
    let config = create_test_cache_config();
    let redis_url = get_test_redis_url();

    cleanup_test_data(&redis_url, &config.key_prefix).await?;

    let cache_manager = PersistentCacheManager::new(&redis_url, config.clone()).await?;

    // Test write throughput
    let write_start = Instant::now();
    let write_operations = 100;

    for i in 0..write_operations {
        let data = serde_json::json!({
            "id": i,
            "data": format!("throughput_test_data_{}", i),
            "timestamp": chrono::Utc::now().to_rfc3339()
        });

        cache_manager.set(&format!("throughput_key_{}", i), &data, None, None, None).await?;
    }

    let write_duration = write_start.elapsed();
    let write_ops_per_sec = write_operations as f64 / write_duration.as_secs_f64();

    println!("Write throughput: {:.2} ops/sec", write_ops_per_sec);

    // Test read throughput
    let read_start = Instant::now();
    let read_operations = 100;

    for i in 0..read_operations {
        let _retrieved: Option<serde_json::Value> = cache_manager.get(&format!("throughput_key_{}", i), None).await?;
    }

    let read_duration = read_start.elapsed();
    let read_ops_per_sec = read_operations as f64 / read_duration.as_secs_f64();

    println!("Read throughput: {:.2} ops/sec", read_ops_per_sec);

    // Verify reasonable throughput (adjust based on environment)
    assert!(write_ops_per_sec > 10.0, "Write throughput should be > 10 ops/sec");
    assert!(read_ops_per_sec > 20.0, "Read throughput should be > 20 ops/sec");

    cleanup_test_data(&redis_url, &config.key_prefix).await?;
    Ok(())
}

#[tokio::test]
async fn test_batch_operation_performance() -> Result<(), Box<dyn std::error::Error>> {
    let config = create_test_cache_config();
    let redis_url = get_test_redis_url();

    cleanup_test_data(&redis_url, &config.key_prefix).await?;

    let cache_manager = PersistentCacheManager::new(&redis_url, config.clone()).await?;

    // Test batch set performance
    let batch_size = 50;
    let mut batch_data = std::collections::HashMap::new();

    for i in 0..batch_size {
        batch_data.insert(
            format!("batch_key_{}", i),
            serde_json::json!({
                "id": i,
                "data": format!("batch_data_{}", i),
                "timestamp": chrono::Utc::now().to_rfc3339()
            })
        );
    }

    let batch_set_start = Instant::now();
    cache_manager.set_batch(batch_data.clone(), None, None).await?;
    let batch_set_duration = batch_set_start.elapsed();

    println!("Batch set ({} items): {:?}", batch_size, batch_set_duration);

    // Test batch get performance
    let keys: Vec<String> = batch_data.keys().cloned().collect();

    let batch_get_start = Instant::now();
    let retrieved: std::collections::HashMap<String, serde_json::Value> = cache_manager.get_batch(&keys, None).await?;
    let batch_get_duration = batch_get_start.elapsed();

    println!("Batch get ({} items): {:?}", batch_size, batch_get_duration);

    assert_eq!(retrieved.len(), batch_data.len());

    // Batch operations should be more efficient than individual operations
    assert!(batch_set_duration.as_millis() < batch_size as u128 * 10, "Batch set should be efficient");
    assert!(batch_get_duration.as_millis() < batch_size as u128 * 5, "Batch get should be efficient");

    cleanup_test_data(&redis_url, &config.key_prefix).await?;
    Ok(())
}

#[tokio::test]
async fn test_concurrent_access_performance() -> Result<(), Box<dyn std::error::Error>> {
    let config = create_test_cache_config();
    let redis_url = get_test_redis_url();

    cleanup_test_data(&redis_url, &config.key_prefix).await?;

    let cache_manager = Arc::new(PersistentCacheManager::new(&redis_url, config.clone()).await?);

    // Pre-populate data
    for i in 0..20 {
        cache_manager.set(
            &format!("concurrent_key_{}", i),
            &format!("concurrent_value_{}", i),
            None,
            None,
            None,
        ).await?;
    }

    // Test concurrent reads
    let concurrent_start = Instant::now();
    let concurrent_tasks = 10;
    let reads_per_task = 10;

    let mut handles = Vec::new();

    for task_id in 0..concurrent_tasks {
        let cache_manager_clone = Arc::clone(&cache_manager);
        let handle = tokio::spawn(async move {
            for i in 0..reads_per_task {
                let key = format!("concurrent_key_{}", i % 20);
                let _retrieved: Option<String> = cache_manager_clone.get(&key, None).await.unwrap();
            }
        });
        handles.push(handle);
    }

    // Wait for all tasks to complete
    for handle in handles {
        handle.await?;
    }

    let concurrent_duration = concurrent_start.elapsed();
    let total_operations = concurrent_tasks * reads_per_task;
    let concurrent_ops_per_sec = total_operations as f64 / concurrent_duration.as_secs_f64();

    println!("Concurrent access ({} tasks, {} ops each): {:.2} ops/sec",
             concurrent_tasks, reads_per_task, concurrent_ops_per_sec);

    // Concurrent access should maintain reasonable performance
    assert!(concurrent_ops_per_sec > 20.0, "Concurrent access should maintain > 20 ops/sec");

    cleanup_test_data(&redis_url, &config.key_prefix).await?;
    Ok(())
}

#[tokio::test]
async fn test_compression_performance() -> Result<(), Box<dyn std::error::Error>> {
    let config = create_test_cache_config();
    let redis_url = get_test_redis_url();

    cleanup_test_data(&redis_url, &config.key_prefix).await?;

    let cache_manager = PersistentCacheManager::new(&redis_url, config.clone()).await?;

    // Test with compressible data
    let compressible_data = "x".repeat(5120); // 5KB of repeating data

    let compress_start = Instant::now();
    cache_manager.set("compress_test", &compressible_data, None, None, None).await?;
    let compress_duration = compress_start.elapsed();

    let decompress_start = Instant::now();
    let retrieved: Option<String> = cache_manager.get("compress_test", None).await?;
    let decompress_duration = decompress_start.elapsed();

    assert!(retrieved.is_some());
    assert_eq!(retrieved.unwrap(), compressible_data);

    println!("Compression set: {:?}, get: {:?}", compress_duration, decompress_duration);

    // Compression should not significantly impact performance
    assert!(compress_duration.as_millis() < 100, "Compression should be fast");
    assert!(decompress_duration.as_millis() < 50, "Decompression should be fast");

    cleanup_test_data(&redis_url, &config.key_prefix).await?;
    Ok(())
}

#[tokio::test]
async fn test_session_management_performance() -> Result<(), Box<dyn std::error::Error>> {
    let config = create_test_state_config();
    let redis_url = get_test_redis_url();

    let state_manager = StateManager::new(&redis_url, config).await?;

    // Test session creation performance
    let session_count = 50;
    let create_start = Instant::now();

    let mut session_ids = Vec::new();
    for i in 0..session_count {
        let metadata = riptide_persistence::state::SessionMetadata {
            client_ip: Some(format!("192.168.1.{}", i + 1)),
            user_agent: Some(format!("PerfTestAgent/{}", i)),
            source: Some("performance_test".to_string()),
            attributes: std::collections::HashMap::new(),
        };

        let session_id = state_manager.create_session(
            Some(format!("perf_user_{}", i)),
            metadata,
            Some(3600),
        ).await?;

        session_ids.push(session_id);
    }

    let create_duration = create_start.elapsed();
    let create_rate = session_count as f64 / create_duration.as_secs_f64();

    println!("Session creation rate: {:.2} sessions/sec", create_rate);

    // Test session retrieval performance
    let retrieve_start = Instant::now();

    for session_id in &session_ids {
        let session = state_manager.get_session(session_id).await?;
        assert!(session.is_some(), "Session should exist during performance test");
    }

    let retrieve_duration = retrieve_start.elapsed();
    let retrieve_rate = session_count as f64 / retrieve_duration.as_secs_f64();

    println!("Session retrieval rate: {:.2} sessions/sec", retrieve_rate);

    // Performance assertions
    assert!(create_rate > 5.0, "Session creation should be > 5 sessions/sec");
    assert!(retrieve_rate > 10.0, "Session retrieval should be > 10 sessions/sec");

    // Clean up
    for session_id in session_ids {
        state_manager.terminate_session(&session_id).await?;
    }

    Ok(())
}

#[tokio::test]
async fn test_tenant_operations_performance() -> Result<(), Box<dyn std::error::Error>> {
    let config = create_test_tenant_config();
    let redis_url = get_test_redis_url();

    let tenant_metrics = Arc::new(RwLock::new(riptide_persistence::TenantMetrics::new(
        Arc::new(prometheus::Registry::new())
    )));
    let tenant_manager = TenantManager::new(&redis_url, config.clone(), tenant_metrics).await?;

    // Test tenant creation performance
    let tenant_count = 10;
    let create_start = Instant::now();

    let mut tenant_ids = Vec::new();
    for i in 0..tenant_count {
        let owner = riptide_persistence::TenantOwner {
            id: format!("perf_owner_{}", i),
            name: format!("Performance Owner {}", i),
            email: format!("perf{}@example.com", i),
            organization: Some(format!("Perf Corp {}", i)),
        };

        let tenant_config = riptide_persistence::tenant::TenantConfig {
            tenant_id: "".to_string(),
            name: format!("Performance Tenant {}", i),
            quotas: config.default_quotas.clone(),
            isolation_level: riptide_persistence::TenantIsolationLevel::Logical,
            encryption_enabled: false, // Disable for performance test
            settings: std::collections::HashMap::new(),
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
        };

        let tenant_id = tenant_manager.create_tenant(
            tenant_config,
            owner,
            riptide_persistence::BillingPlan::Basic,
        ).await?;

        tenant_ids.push(tenant_id);
    }

    let create_duration = create_start.elapsed();
    let create_rate = tenant_count as f64 / create_duration.as_secs_f64();

    println!("Tenant creation rate: {:.2} tenants/sec", create_rate);

    // Test tenant access validation performance
    let validation_start = Instant::now();
    let validation_count = tenant_count * 5; // 5 validations per tenant

    for tenant_id in &tenant_ids {
        for j in 0..5 {
            let _access = tenant_manager.validate_access(
                tenant_id,
                &format!("resource_{}", j),
                "read",
            ).await?;
        }
    }

    let validation_duration = validation_start.elapsed();
    let validation_rate = validation_count as f64 / validation_duration.as_secs_f64();

    println!("Access validation rate: {:.2} validations/sec", validation_rate);

    // Performance assertions
    assert!(create_rate > 1.0, "Tenant creation should be > 1 tenant/sec");
    assert!(validation_rate > 10.0, "Access validation should be > 10 validations/sec");

    // Clean up
    for tenant_id in tenant_ids {
        tenant_manager.delete_tenant(&tenant_id).await?;
    }

    Ok(())
}

#[tokio::test]
async fn test_memory_usage_patterns() -> Result<(), Box<dyn std::error::Error>> {
    let config = create_test_cache_config();
    let redis_url = get_test_redis_url();

    cleanup_test_data(&redis_url, &config.key_prefix).await?;

    let cache_manager = PersistentCacheManager::new(&redis_url, config.clone()).await?;

    // Initial memory baseline
    let initial_stats = cache_manager.get_stats().await?;
    let initial_memory = initial_stats.memory_usage_bytes;

    println!("Initial memory usage: {} bytes", initial_memory);

    // Add data and measure memory growth
    let data_points = 20;
    let data_size = 1024; // 1KB per entry

    for i in 0..data_points {
        let data = "x".repeat(data_size);
        cache_manager.set(&format!("memory_test_{}", i), &data, None, None, None).await?;
    }

    let after_write_stats = cache_manager.get_stats().await?;
    let after_write_memory = after_write_stats.memory_usage_bytes;

    println!("Memory usage after {} entries: {} bytes", data_points, after_write_memory);

    // Clear cache and measure memory cleanup
    let cleared_count = cache_manager.clear().await?;
    assert!(cleared_count > 0, "Should have cleared some entries");
    // Give some time for cleanup
    tokio::time::sleep(Duration::from_millis(100)).await;

    let final_stats = cache_manager.get_stats().await?;
    let final_memory = final_stats.memory_usage_bytes;

    println!("Memory usage after clear: {} bytes", final_memory);

    // Verify memory usage patterns
    assert!(after_write_memory > initial_memory, "Memory should increase with data");
    assert!(final_stats.total_keys == 0, "All keys should be cleared");

    cleanup_test_data(&redis_url, &config.key_prefix).await?;
    Ok(())
}

#[tokio::test]
async fn test_checkpoint_performance() -> Result<(), Box<dyn std::error::Error>> {
    let config = create_test_state_config();
    let redis_url = get_test_redis_url();

    let state_manager = StateManager::new(&redis_url, config).await?;

    // Create substantial state to checkpoint
    let session_count = 20;
    let mut session_ids = Vec::new();

    for i in 0..session_count {
        let metadata = riptide_persistence::state::SessionMetadata {
            client_ip: Some(format!("10.0.0.{}", i + 1)),
            user_agent: Some(format!("CheckpointAgent/{}", i)),
            source: Some("checkpoint_performance_test".to_string()),
            attributes: std::collections::HashMap::new(),
        };

        let session_id = state_manager.create_session(
            Some(format!("checkpoint_user_{}", i)),
            metadata,
            Some(7200), // 2 hours
        ).await?;

        // Add data to each session
        state_manager.update_session_data(
            &session_id,
            "performance_data",
            serde_json::json!({
                "large_array": (0..100).collect::<Vec<i32>>(),
                "text_data": "y".repeat(512),
                "nested": {
                    "level1": {"level2": {"data": format!("session_{}", i)}}
                }
            })
        ).await?;

        session_ids.push(session_id);
    }

    // Test checkpoint creation performance
    let checkpoint_start = Instant::now();
    let checkpoint_id = state_manager.create_checkpoint(
        riptide_persistence::state::CheckpointType::Manual,
        Some("Performance test checkpoint".to_string()),
    ).await?;
    let checkpoint_duration = checkpoint_start.elapsed();

    println!("Checkpoint creation ({} sessions): {:?}", session_count, checkpoint_duration);

    // Test checkpoint restoration performance
    let restore_start = Instant::now();
    state_manager.restore_from_checkpoint(&checkpoint_id).await?;
    let restore_duration = restore_start.elapsed();

    println!("Checkpoint restoration: {:?}", restore_duration);

    // Performance assertions
    assert!(checkpoint_duration.as_secs() < 10, "Checkpoint creation should be < 10 seconds");
    assert!(restore_duration.as_secs() < 10, "Checkpoint restoration should be < 10 seconds");

    // Clean up
    for session_id in session_ids {
        state_manager.terminate_session(&session_id).await?;
    }

    Ok(())
}