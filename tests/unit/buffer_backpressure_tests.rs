//! Comprehensive Buffer Management and Backpressure Tests
//!
//! This module tests the buffer and backpressure handling components in detail:
//! - Dynamic buffer sizing and adaptive thresholds
//! - Backpressure detection and message dropping
//! - Buffer statistics and monitoring
//! - Memory usage validation
//! - Performance under various load conditions

use std::time::{Duration, Instant};
use tokio::time::sleep;
use riptide_api::streaming::buffer::{
    BackpressureHandler, BufferConfig, BufferManager, BufferStats, DynamicBuffer
};
use riptide_api::streaming::error::StreamingResult;
use std::sync::Arc;

/// Test dynamic buffer creation and basic functionality
#[tokio::test]
async fn test_dynamic_buffer_creation_and_config() {
    let config = BufferConfig {
        initial_size: 128,
        max_size: 1024,
        min_size: 32,
        growth_factor: 1.5,
        shrink_factor: 0.8,
        slow_send_threshold_ms: 150,
        max_slow_sends: 5,
        backpressure_window: 25,
    };

    let buffer = DynamicBuffer::with_config(config.clone());
    assert_eq!(buffer.capacity(), 128, "Should start with initial size");

    let stats = buffer.stats().await;
    assert_eq!(stats.current_size, 128);
    assert_eq!(stats.total_messages, 0);
    assert_eq!(stats.dropped_messages, 0);
    assert_eq!(stats.resizes, 0);
}

/// Test buffer growth under high drop rate
#[tokio::test]
async fn test_buffer_growth_on_high_drop_rate() {
    let mut config = BufferConfig::default();
    config.initial_size = 64;
    config.max_size = 512;
    config.growth_factor = 2.0;

    let buffer = DynamicBuffer::with_config(config);
    let initial_capacity = buffer.capacity();

    // Simulate high drop rate to trigger growth
    for _ in 0..50 {
        buffer.record_drop().await;
    }

    // Add some successful sends to trigger adjustment
    buffer.record_send(Duration::from_millis(50)).await.unwrap();
    
    let stats = buffer.stats().await;
    println!("Initial: {}, Final: {}, Resizes: {}", 
             initial_capacity, stats.current_size, stats.resizes);
    
    // Should have grown due to high drop rate
    assert!(stats.current_size > initial_capacity, 
            "Buffer should grow with high drop rate: {} -> {}", 
            initial_capacity, stats.current_size);
    assert!(stats.resizes > 0, "Should have recorded resize operations");
}

/// Test buffer shrinkage under backpressure
#[tokio::test]
async fn test_buffer_shrinkage_on_backpressure() {
    let mut config = BufferConfig::default();
    config.initial_size = 256;
    config.min_size = 64;
    config.shrink_factor = 0.75;
    config.slow_send_threshold_ms = 100;
    config.backpressure_window = 10;

    let buffer = DynamicBuffer::with_config(config);
    let initial_capacity = buffer.capacity();

    // Simulate consistent slow sends to trigger backpressure
    for _ in 0..15 {
        buffer.record_send(Duration::from_millis(150)).await.unwrap();
    }

    let stats = buffer.stats().await;
    println!("Initial: {}, Final: {}, Slow sends: {}", 
             initial_capacity, stats.current_size, stats.slow_sends);
    
    // Should have shrunk due to backpressure
    assert!(stats.current_size < initial_capacity,
            "Buffer should shrink under backpressure: {} -> {}",
            initial_capacity, stats.current_size);
    assert!(stats.slow_sends > 10, "Should have recorded slow sends");
}

/// Test backpressure detection accuracy
#[tokio::test]
async fn test_backpressure_detection() {
    let mut config = BufferConfig::default();
    config.slow_send_threshold_ms = 100;
    config.backpressure_window = 10;

    let buffer = DynamicBuffer::with_config(config);

    // Initially no backpressure
    assert!(!buffer.is_under_backpressure().await, "Should not detect backpressure initially");

    // Add fast sends - should not trigger backpressure
    for _ in 0..8 {
        buffer.record_send(Duration::from_millis(50)).await.unwrap();
    }
    assert!(!buffer.is_under_backpressure().await, "Fast sends should not trigger backpressure");

    // Add slow sends to trigger backpressure
    for _ in 0..8 {
        buffer.record_send(Duration::from_millis(150)).await.unwrap();
    }
    
    assert!(buffer.is_under_backpressure().await, "Slow sends should trigger backpressure");
}

/// Test backpressure handler message dropping
#[tokio::test]
async fn test_backpressure_handler_message_dropping() {
    let buffer = Arc::new(DynamicBuffer::new());
    let mut handler = BackpressureHandler::new("test-conn".to_string(), buffer.clone());

    // Low queue size should not drop
    assert!(!handler.should_drop_message(50).await, "Low queue size should not drop messages");

    // High queue size should drop
    assert!(handler.should_drop_message(1500).await, "High queue size should drop messages");

    // Verify metrics
    let metrics = handler.metrics();
    assert_eq!(metrics.dropped_messages, 1, "Should have recorded 1 dropped message");
    assert!(metrics.last_drop_time.is_some(), "Should have recorded drop time");
}

/// Test adaptive drop threshold adjustment
#[tokio::test]
async fn test_adaptive_drop_threshold() {
    let buffer = Arc::new(DynamicBuffer::new());
    let mut handler = BackpressureHandler::new("adaptive-test".to_string(), buffer.clone());

    // Record many successful sends to establish good performance
    for _ in 0..95 {
        handler.record_send_time(Duration::from_millis(50)).await.unwrap();
    }

    // Should not be marked as slow connection
    assert!(!handler.is_connection_slow(), "Should not be marked as slow with fast sends");

    // Record slow sends to degrade performance
    for _ in 0..20 {
        handler.record_send_time(Duration::from_millis(200)).await.unwrap();
    }

    // Should now be marked as slow
    assert!(handler.is_connection_slow(), "Should be marked as slow after slow sends");

    let metrics = handler.metrics();
    assert!(metrics.average_send_time_ms > 100.0, "Average send time should reflect slow sends");
    assert!(metrics.slow_sends > 10, "Should have recorded slow sends");
}

/// Test buffer manager multi-connection handling
#[tokio::test]
async fn test_buffer_manager_multi_connection() {
    let manager = BufferManager::new();

    // Get buffers for different connections
    let buffer1 = manager.get_buffer("conn-1").await;
    let buffer2 = manager.get_buffer("conn-2").await;
    let buffer1_again = manager.get_buffer("conn-1").await;

    // Should reuse same buffer for same connection
    assert!(Arc::ptr_eq(&buffer1, &buffer1_again), 
            "Should reuse buffer for same connection");
    
    // Should create different buffers for different connections
    assert!(!Arc::ptr_eq(&buffer1, &buffer2), 
            "Should create different buffers for different connections");

    // Record activity on each buffer
    buffer1.record_send(Duration::from_millis(100)).await.unwrap();
    buffer2.record_send(Duration::from_millis(200)).await.unwrap();

    // Get global stats
    let global_stats = manager.global_stats().await;
    assert_eq!(global_stats.len(), 2, "Should track 2 connections");
    
    assert!(global_stats.contains_key("conn-1"), "Should have stats for conn-1");
    assert!(global_stats.contains_key("conn-2"), "Should have stats for conn-2");

    // Remove a buffer
    manager.remove_buffer("conn-1").await;
    let global_stats_after = manager.global_stats().await;
    assert_eq!(global_stats_after.len(), 1, "Should have 1 connection after removal");
    assert!(!global_stats_after.contains_key("conn-1"), "Should not have stats for removed connection");
}

/// Test buffer channel creation and capacity
#[tokio::test]
async fn test_buffer_channel_creation() {
    let mut config = BufferConfig::default();
    config.initial_size = 100;
    let buffer = DynamicBuffer::with_config(config);

    let (tx, _rx) = buffer.create_channel::<String>().await;
    
    // Should respect buffer capacity
    assert_eq!(tx.capacity(), 100, "Channel capacity should match buffer capacity");
    
    // Change buffer capacity and create new channel
    for _ in 0..20 {
        buffer.record_drop().await;
    }
    buffer.record_send(Duration::from_millis(50)).await.unwrap(); // Trigger adjustment
    
    let (tx2, _rx2) = buffer.create_channel::<String>().await;
    let new_capacity = tx2.capacity();
    
    println!("Original capacity: 100, New capacity: {}", new_capacity);
    // Capacity may have changed due to buffer adjustment
}

/// Test statistics accuracy and tracking
#[tokio::test]
async fn test_buffer_statistics_accuracy() {
    let buffer = DynamicBuffer::new();
    
    // Record various activities
    for i in 0..10 {
        let delay = Duration::from_millis(50 + i * 20);
        buffer.record_send(delay).await.unwrap();
    }
    
    for _ in 0..3 {
        buffer.record_drop().await;
    }
    
    let stats = buffer.stats().await;
    assert_eq!(stats.total_messages, 10, "Should track total messages");
    assert_eq!(stats.dropped_messages, 3, "Should track dropped messages");
    assert!(stats.average_send_time_ms > 0.0, "Should calculate average send time");
    assert!(stats.slow_sends > 0, "Should track slow sends with varying delays");
    assert!(stats.peak_size >= stats.current_size, "Peak size should be >= current size");

    // Test stats reset
    buffer.reset_stats().await;
    let reset_stats = buffer.stats().await;
    assert_eq!(reset_stats.total_messages, 0, "Total messages should reset");
    assert_eq!(reset_stats.dropped_messages, 0, "Dropped messages should reset");
    assert_eq!(reset_stats.slow_sends, 0, "Slow sends should reset");
}

/// Test buffer behavior under sustained load
#[tokio::test]
async fn test_buffer_sustained_load() {
    let mut config = BufferConfig::default();
    config.initial_size = 32;
    config.max_size = 256;
    config.backpressure_window = 20;
    
    let buffer = Arc::new(DynamicBuffer::with_config(config));
    let mut handler = BackpressureHandler::new("load-test".to_string(), buffer.clone());
    
    let start_time = Instant::now();
    let mut dropped_count = 0;
    let total_messages = 200;
    
    // Simulate sustained mixed load
    for i in 0..total_messages {
        let delay = if i % 5 == 0 { 
            Duration::from_millis(300) // Occasional slow message
        } else { 
            Duration::from_millis(50) // Mostly fast
        };
        
        if handler.should_drop_message(i % 50).await {
            dropped_count += 1;
        } else {
            handler.record_send_time(delay).await.unwrap();
        }
    }
    
    let total_time = start_time.elapsed();
    let stats = buffer.stats().await;
    let handler_metrics = handler.metrics();
    
    println!(
        "Sustained Load Test Results:\n" +
        "  Duration: {}ms\n" +
        "  Total Messages: {}\n" +
        "  Dropped (handler): {}\n" +
        "  Dropped (buffer): {}\n" +
        "  Buffer Resizes: {}\n" +
        "  Final Buffer Size: {}\n" +
        "  Average Send Time: {:.2}ms\n" +
        "  Slow Sends: {}\n" +
        "  Drop Rate: {:.2}%",
        total_time.as_millis(),
        total_messages,
        dropped_count,
        stats.dropped_messages,
        stats.resizes,
        stats.current_size,
        handler_metrics.average_send_time_ms,
        stats.slow_sends,
        (dropped_count as f64 / total_messages as f64) * 100.0
    );
    
    // Buffer should adapt to load
    assert!(stats.resizes > 0, "Buffer should have resized under load");
    
    // Should maintain reasonable drop rate
    let drop_rate = dropped_count as f64 / total_messages as f64;
    assert!(drop_rate < 0.3, "Drop rate should be reasonable: {:.2}%", drop_rate * 100.0);
}

/// Test memory usage estimation and limits
#[tokio::test]
async fn test_buffer_memory_usage_estimation() {
    let config = BufferConfig {
        initial_size: 1000,
        max_size: 5000,
        ..BufferConfig::default()
    };
    
    let buffer_manager = BufferManager::new();
    
    // Create multiple connections with activity
    for conn_id in 0..10 {
        let buffer = buffer_manager.get_buffer(&format!("conn-{}", conn_id)).await;
        
        // Simulate activity
        for _ in 0..50 {
            buffer.record_send(Duration::from_millis(100)).await.unwrap();
        }
    }
    
    let global_stats = buffer_manager.global_stats().await;
    let total_estimated_memory: usize = global_stats.values()
        .map(|stats| stats.current_size * 1024) // Rough estimation
        .sum();
    
    println!("Connections: {}, Estimated Memory: {} bytes ({:.2} MB)",
             global_stats.len(),
             total_estimated_memory,
             total_estimated_memory as f64 / (1024.0 * 1024.0));
    
    assert_eq!(global_stats.len(), 10, "Should track all connections");
    assert!(total_estimated_memory > 0, "Should estimate memory usage");
    
    // Memory should be reasonable for the scale
    let mb_usage = total_estimated_memory as f64 / (1024.0 * 1024.0);
    assert!(mb_usage < 100.0, "Memory usage should be reasonable: {:.2} MB", mb_usage);
}

/// Test edge cases and error conditions
#[tokio::test]
async fn test_buffer_edge_cases() {
    let mut config = BufferConfig::default();
    config.min_size = 1;
    config.max_size = 10;
    config.initial_size = 5;
    
    let buffer = DynamicBuffer::with_config(config);
    
    // Test extreme shrinking
    for _ in 0..100 {
        buffer.record_send(Duration::from_millis(500)).await.unwrap();
    }
    
    let stats_after_slow = buffer.stats().await;
    assert!(stats_after_slow.current_size >= 1, "Should not shrink below minimum");
    assert!(stats_after_slow.current_size <= 10, "Should not grow above maximum");
    
    // Test extreme growth pressure
    for _ in 0..100 {
        buffer.record_drop().await;
    }
    
    buffer.record_send(Duration::from_millis(10)).await.unwrap(); // Trigger adjustment
    
    let stats_after_drops = buffer.stats().await;
    assert!(stats_after_drops.current_size >= 1, "Should respect minimum even with drops");
    assert!(stats_after_drops.current_size <= 10, "Should respect maximum even with drops");
    
    println!("Edge case results: min_size=1, max_size=10, final_size={}", 
             stats_after_drops.current_size);
}

/// Test concurrent access to buffers
#[tokio::test]
async fn test_buffer_concurrent_access() {
    let buffer = Arc::new(DynamicBuffer::new());
    let buffer_manager = BufferManager::new();
    
    // Spawn multiple tasks accessing the same buffer concurrently
    let mut handles = Vec::new();
    
    for i in 0..5 {
        let buffer_clone = buffer.clone();
        let manager_clone = Arc::new(buffer_manager);
        
        let handle = tokio::spawn(async move {
            let conn_id = format!("concurrent-{}", i);
            let managed_buffer = manager_clone.get_buffer(&conn_id).await;
            
            // Perform operations on both shared buffer and managed buffer
            for j in 0..20 {
                let delay = Duration::from_millis(10 + j * 5);
                buffer_clone.record_send(delay).await.unwrap();
                managed_buffer.record_send(delay).await.unwrap();
                
                if j % 5 == 0 {
                    buffer_clone.record_drop().await;
                    managed_buffer.record_drop().await;
                }
            }
            
            (buffer_clone.stats().await, managed_buffer.stats().await)
        });
        
        handles.push(handle);
    }
    
    // Wait for all tasks to complete
    let results = futures::future::join_all(handles).await;
    
    // Verify all operations completed successfully
    let mut total_shared_messages = 0;
    let mut total_managed_messages = 0;
    
    for result in results {
        let (shared_stats, managed_stats) = result.unwrap();
        total_shared_messages += shared_stats.total_messages;
        total_managed_messages += managed_stats.total_messages;
    }
    
    println!("Concurrent access results: shared={}, managed={}",
             total_shared_messages, total_managed_messages);
    
    assert_eq!(total_shared_messages, 100, "Shared buffer should have recorded all messages");
    assert_eq!(total_managed_messages, 100, "Managed buffers should have recorded all messages");
}

/// Benchmark buffer performance
#[tokio::test]
async fn test_buffer_performance_benchmark() {
    let buffer = DynamicBuffer::new();
    let operations = 10000;
    
    let start_time = Instant::now();
    
    // Benchmark record_send operations
    for i in 0..operations {
        let delay = Duration::from_millis(if i % 10 == 0 { 100 } else { 50 });
        buffer.record_send(delay).await.unwrap();
    }
    
    let send_time = start_time.elapsed();
    
    // Benchmark stats retrieval
    let stats_start = Instant::now();
    let stats = buffer.stats().await;
    let stats_time = stats_start.elapsed();
    
    let ops_per_second = operations as f64 / send_time.as_secs_f64();
    
    println!(
        "Buffer Performance Benchmark:\n" +
        "  Operations: {}\n" +
        "  Total Time: {}ms\n" +
        "  Ops/Second: {:.0}\n" +
        "  Stats Retrieval: {}μs\n" +
        "  Final Buffer Size: {}\n" +
        "  Total Messages: {}\n" +
        "  Average Send Time: {:.2}ms",
        operations,
        send_time.as_millis(),
        ops_per_second,
        stats_time.as_micros(),
        stats.current_size,
        stats.total_messages,
        stats.average_send_time_ms
    );
    
    // Performance assertions
    assert!(ops_per_second > 1000.0, "Should handle at least 1000 ops/second");
    assert!(stats_time.as_millis() < 10, "Stats retrieval should be fast");
    assert_eq!(stats.total_messages, operations, "Should track all operations");
}
