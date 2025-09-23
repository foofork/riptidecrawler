//! Comprehensive tests for the streaming module.
//!
//! This module contains integration tests, unit tests, and benchmarks
//! for all streaming components to ensure reliability and performance.

#![cfg(test)]

use super::*;
use crate::models::*;
use crate::state::AppState;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::mpsc;
use tokio::time::timeout;

/// Test fixtures and utilities
mod fixtures {
    use super::*;

    pub async fn create_test_app() -> AppState {
        AppState::new().await.expect("Failed to create test AppState")
    }

    pub fn create_test_crawl_body(url_count: usize) -> CrawlBody {
        CrawlBody {
            urls: (0..url_count)
                .map(|i| format!("https://example{}.com", i))
                .collect(),
            options: Some(CrawlOptions::default()),
        }
    }

    pub fn create_test_deepsearch_body() -> DeepSearchBody {
        DeepSearchBody {
            query: "test query".to_string(),
            limit: Some(5),
            include_content: Some(true),
            crawl_options: Some(CrawlOptions::default()),
        }
    }
}

/// Buffer management tests
mod buffer_tests {
    use super::*;
    use crate::streaming::buffer::*;

    #[tokio::test]
    async fn test_dynamic_buffer_creation() {
        let buffer = DynamicBuffer::new();
        assert_eq!(buffer.capacity(), 256); // Default initial size
    }

    #[tokio::test]
    async fn test_buffer_stats_tracking() {
        let buffer = DynamicBuffer::new();

        // Initial stats should be empty
        let stats = buffer.stats().await;
        assert_eq!(stats.total_messages, 0);
        assert_eq!(stats.dropped_messages, 0);

        // Record some operations
        buffer.record_send(Duration::from_millis(50)).await.unwrap();
        buffer.record_send(Duration::from_millis(150)).await.unwrap();
        buffer.record_drop().await;

        let stats = buffer.stats().await;
        assert_eq!(stats.total_messages, 2);
        assert_eq!(stats.dropped_messages, 1);
        assert_eq!(stats.slow_sends, 1); // 150ms is above 100ms threshold
    }

    #[tokio::test]
    async fn test_backpressure_detection() {
        let buffer = DynamicBuffer::new();

        // Simulate many slow sends
        for _ in 0..30 {
            buffer.record_send(Duration::from_millis(150)).await.unwrap();
        }

        assert!(buffer.is_under_backpressure().await);
    }

    #[tokio::test]
    async fn test_buffer_growth_and_shrink() {
        let mut config = BufferConfig::default();
        config.initial_size = 64;
        config.max_size = 256;
        config.growth_factor = 2.0;

        let buffer = DynamicBuffer::with_config(config);
        let initial_capacity = buffer.capacity();

        // Simulate high drop rate to trigger growth
        for _ in 0..100 {
            buffer.record_drop().await;
        }

        // Trigger adjustment
        buffer.record_send(Duration::from_millis(50)).await.unwrap();

        let stats = buffer.stats().await;
        assert!(stats.current_size >= initial_capacity);
    }

    #[tokio::test]
    async fn test_backpressure_handler() {
        let buffer = Arc::new(DynamicBuffer::new());
        let mut handler = BackpressureHandler::new("test-conn".to_string(), buffer);

        // Should not drop with low queue size
        assert!(!handler.should_drop_message(10).await);

        // Should drop with high queue size
        assert!(handler.should_drop_message(1500).await);

        // Test metrics tracking
        handler.record_send_time(Duration::from_millis(50)).await.unwrap();
        handler.record_send_time(Duration::from_millis(200)).await.unwrap();

        let metrics = handler.metrics();
        assert_eq!(metrics.total_messages, 2);
        assert_eq!(metrics.slow_sends, 1);
    }

    #[tokio::test]
    async fn test_buffer_manager() {
        let manager = BufferManager::new();

        // Test buffer creation and retrieval
        let buffer1 = manager.get_buffer("conn1").await;
        let buffer2 = manager.get_buffer("conn1").await;

        // Should return the same buffer for same connection
        assert!(Arc::ptr_eq(&buffer1, &buffer2));

        // Test buffer removal
        manager.remove_buffer("conn1").await;
        let buffer3 = manager.get_buffer("conn1").await;

        // Should create a new buffer after removal
        assert!(!Arc::ptr_eq(&buffer1, &buffer3));

        // Test global stats
        let stats = manager.global_stats().await;
        assert!(stats.contains_key("conn1"));
    }
}

/// Configuration tests
mod config_tests {
    use super::*;
    use crate::streaming::config::*;

    #[test]
    fn test_default_config_validation() {
        let config = StreamConfig::default();
        assert!(config.validate().is_ok());
    }

    #[test]
    fn test_invalid_buffer_config() {
        let mut config = StreamConfig::default();
        config.buffer.min_size = 1000;
        config.buffer.default_size = 500; // Less than min_size
        assert!(config.validate().is_err());
    }

    #[test]
    fn test_config_optimal_buffer_size() {
        let config = StreamConfig::default();

        // Slow connection should get minimum buffer
        assert_eq!(config.optimal_buffer_size(true, 10.0), config.buffer.min_size);

        // High message rate should get larger buffer
        assert!(config.optimal_buffer_size(false, 150.0) > config.buffer.default_size);

        // Normal rate should get default buffer
        assert_eq!(config.optimal_buffer_size(false, 50.0), config.buffer.default_size);
    }

    #[test]
    fn test_streaming_health_check() {
        let config = StreamConfig::default();

        // Healthy state
        assert!(config.is_streaming_healthy(100, 0.01));

        // Unhealthy due to high connection count
        assert!(!config.is_streaming_healthy(950, 0.01));

        // Unhealthy due to high error rate
        assert!(!config.is_streaming_healthy(100, 0.10));
    }

    #[test]
    fn test_rate_limit_config() {
        let rate_limit = RateLimitConfig::default();
        assert!(rate_limit.enabled);
        assert_eq!(rate_limit.requests_per_second, 10);
        assert!(matches!(rate_limit.action, RateLimitAction::Error));
    }
}

/// Error handling tests
mod error_tests {
    use super::*;
    use crate::streaming::error::*;

    #[test]
    fn test_error_creation() {
        let err = StreamingError::buffer_overflow("Buffer full");
        assert!(matches!(err, StreamingError::BufferOverflow { .. }));
        assert!(err.is_retryable());
        assert!(!err.is_client_error());
    }

    #[test]
    fn test_error_retryability() {
        assert!(StreamingError::connection("test").is_retryable());
        assert!(!StreamingError::invalid_request("test").is_retryable());
        assert!(!StreamingError::client_disconnected("test").is_retryable());
    }

    #[test]
    fn test_client_error_classification() {
        assert!(StreamingError::backpressure_exceeded("test").is_client_error());
        assert!(StreamingError::invalid_request("test").is_client_error());
        assert!(!StreamingError::buffer_overflow("test").is_client_error());
    }

    #[test]
    fn test_recovery_strategies() {
        let err = StreamingError::buffer_overflow("test");
        assert!(matches!(err.recovery_strategy(), RecoveryStrategy::Drop));

        let err = StreamingError::backpressure_exceeded("test");
        assert!(matches!(err.recovery_strategy(), RecoveryStrategy::Disconnect));

        let err = StreamingError::connection("test");
        assert!(matches!(err.recovery_strategy(), RecoveryStrategy::Retry { .. }));
    }
}

/// Processor tests
mod processor_tests {
    use super::*;
    use crate::streaming::processor::*;

    #[tokio::test]
    async fn test_processing_stats() {
        let mut stats = ProcessingStats::default();
        stats.total_urls = 10;

        // Test successful processing
        stats.update(100, false, true);
        assert_eq!(stats.completed_count, 1);
        assert_eq!(stats.success_rate(), 1.0);
        assert_eq!(stats.fastest_processing_ms, 100);

        // Test cached result
        stats.update(0, true, true);
        assert_eq!(stats.cache_hits, 1);
        assert_eq!(stats.cache_hit_rate(), 0.1);

        // Test failed processing
        stats.update(200, false, false);
        assert_eq!(stats.error_count, 1);
        assert!(stats.success_rate() < 1.0);
    }

    #[test]
    fn test_throughput_calculation() {
        let stats = ProcessingStats {
            completed_count: 60,
            error_count: 10,
            ..Default::default()
        };

        let throughput = stats.throughput(Duration::from_secs(60));
        assert_eq!(throughput, 70.0 / 60.0);
    }

    #[test]
    fn test_progress_percentage() {
        let mut stats = ProcessingStats::default();
        stats.total_urls = 100;
        stats.completed_count = 80;
        stats.error_count = 10;

        assert_eq!(stats.progress_percentage(), 90.0);
    }

    #[tokio::test]
    async fn test_stream_processor_creation() {
        let app = fixtures::create_test_app().await;
        let pipeline = PipelineOrchestrator::new(app, CrawlOptions::default());
        let processor = StreamProcessor::new(pipeline, "test-123".to_string(), 5);

        assert_eq!(processor.request_id, "test-123");
        assert_eq!(processor.stats.total_urls, 5);
    }

    #[test]
    fn test_performance_monitor() {
        let mut monitor = PerformanceMonitor::new();

        monitor.checkpoint("start", 0);
        std::thread::sleep(Duration::from_millis(10));
        monitor.checkpoint("middle", 50);
        std::thread::sleep(Duration::from_millis(10));
        monitor.checkpoint("end", 100);

        let analysis = monitor.analyze();
        assert_eq!(analysis.total_items, 100);
        assert!(analysis.total_duration_ms >= 20);
        assert_eq!(analysis.phase_durations.len(), 2);
    }
}

/// Pipeline tests
mod pipeline_tests {
    use super::*;
    use crate::streaming::pipeline::*;

    #[tokio::test]
    async fn test_streaming_pipeline_creation() {
        let app = fixtures::create_test_app().await;
        let pipeline = StreamingPipeline::new(app, Some("test-123".to_string()));
        assert_eq!(pipeline.request_id(), "test-123");
    }

    #[test]
    fn test_stream_event_types() {
        let metadata = StreamMetadata {
            total_urls: 5,
            request_id: "test-123".to_string(),
            timestamp: "2024-01-01T00:00:00Z".to_string(),
            stream_type: "crawl".to_string(),
        };

        let event = StreamEvent::Metadata(metadata);
        assert!(matches!(event, StreamEvent::Metadata(_)));
    }

    #[test]
    fn test_stream_execution_summary() {
        let summary = StreamExecutionSummary {
            request_id: "test-123".to_string(),
            total_urls: 10,
            successful: 8,
            failed: 2,
            from_cache: 3,
            total_duration_ms: 5000,
            throughput: 2.0,
            backpressure_events: 1,
        };

        assert_eq!(summary.total_urls, 10);
        assert_eq!(summary.successful, 8);
        assert_eq!(summary.failed, 2);
        assert_eq!(summary.throughput, 2.0);
    }
}

/// Integration tests
mod integration_tests {
    use super::*;

    #[tokio::test]
    async fn test_ndjson_streaming_flow() {
        let app = fixtures::create_test_app().await;
        let body = fixtures::create_test_crawl_body(3);

        // Create a channel to capture streaming output
        let (tx, mut rx) = mpsc::channel(100);

        // This would normally be done via HTTP, but we're testing the core logic
        let pipeline = StreamingPipeline::new(app, Some("test-ndjson".to_string()));

        // Mock sender function that captures events
        let sender_fn = |event: &StreamEvent| -> Result<(), StreamingError> {
            let tx = tx.clone();
            let event_type = match event {
                StreamEvent::Metadata(_) => "metadata",
                StreamEvent::Result(_) => "result",
                StreamEvent::Summary(_) => "summary",
                _ => "other",
            };

            tokio::spawn(async move {
                let _ = tx.send(event_type.to_string()).await;
            });

            Ok(())
        };

        // Note: This test is incomplete as it requires mocking the pipeline execution
        // In a real scenario, you'd mock the PipelineOrchestrator or use a test environment
    }

    #[tokio::test]
    async fn test_buffer_backpressure_simulation() {
        let buffer = Arc::new(DynamicBuffer::new());
        let mut handler = BackpressureHandler::new("test-conn".to_string(), buffer);

        // Simulate high-frequency sends
        for i in 0..100 {
            let delay = if i % 10 == 0 { 200 } else { 50 }; // Every 10th is slow
            handler.record_send_time(Duration::from_millis(delay)).await.unwrap();
        }

        let metrics = handler.metrics();
        assert_eq!(metrics.total_messages, 100);
        assert_eq!(metrics.slow_sends, 10); // 10 slow sends
        assert!(handler.is_connection_slow());
    }

    #[tokio::test]
    async fn test_streaming_module_lifecycle() {
        let module = StreamingModule::new(None);
        assert!(module.validate().is_ok());
        assert!(module.is_healthy().await);

        // Start maintenance tasks
        assert!(module.start_maintenance_tasks().await.is_ok());

        // Update metrics
        module.update_metrics(|metrics| {
            metrics.active_connections = 10;
            metrics.total_connections = 20;
            metrics.total_messages_sent = 1000;
            metrics.total_messages_dropped = 50;
            metrics.error_rate = 0.02;
        }).await;

        let metrics = module.metrics().await;
        assert_eq!(metrics.active_connections, 10);
        assert_eq!(metrics.total_connections, 20);
        assert!(metrics.efficiency() > 0.9);
        assert_eq!(metrics.health_status, StreamingHealth::Healthy);
    }
}

/// Benchmark tests (optional, only run with --features benchmark)
#[cfg(feature = "benchmark")]
mod benchmark_tests {
    use super::*;
    use std::time::Instant;

    #[tokio::test]
    async fn benchmark_buffer_operations() {
        let buffer = DynamicBuffer::new();
        let start = Instant::now();

        // Benchmark buffer operations
        for _ in 0..10000 {
            buffer.record_send(Duration::from_millis(50)).await.unwrap();
        }

        let duration = start.elapsed();
        println!("Buffer operations benchmark: {:?} for 10k operations", duration);

        // Should complete in reasonable time (< 1 second for 10k operations)
        assert!(duration < Duration::from_secs(1));
    }

    #[tokio::test]
    async fn benchmark_backpressure_handler() {
        let buffer = Arc::new(DynamicBuffer::new());
        let mut handler = BackpressureHandler::new("bench-conn".to_string(), buffer);
        let start = Instant::now();

        // Benchmark backpressure detection
        for i in 0..1000 {
            handler.should_drop_message(i % 100).await;
            handler.record_send_time(Duration::from_millis(50)).await.unwrap();
        }

        let duration = start.elapsed();
        println!("Backpressure handler benchmark: {:?} for 1k operations", duration);

        // Should complete in reasonable time
        assert!(duration < Duration::from_millis(500));
    }
}

/// Stress tests
mod stress_tests {
    use super::*;

    #[tokio::test]
    async fn stress_test_concurrent_connections() {
        let manager = Arc::new(BufferManager::new());
        let mut handles = Vec::new();

        // Simulate 100 concurrent connections
        for i in 0..100 {
            let manager_clone = manager.clone();
            let handle = tokio::spawn(async move {
                let conn_id = format!("conn-{}", i);
                let buffer = manager_clone.get_buffer(&conn_id).await;

                // Simulate some activity
                for _ in 0..10 {
                    buffer.record_send(Duration::from_millis(50)).await.unwrap();
                }

                // Clean up
                manager_clone.remove_buffer(&conn_id).await;
            });
            handles.push(handle);
        }

        // Wait for all connections to complete
        for handle in handles {
            handle.await.unwrap();
        }

        // All buffers should be cleaned up
        let stats = manager.global_stats().await;
        assert!(stats.is_empty());
    }

    #[tokio::test]
    async fn stress_test_high_message_volume() {
        let buffer = Arc::new(DynamicBuffer::new());
        let mut handler = BackpressureHandler::new("stress-conn".to_string(), buffer);

        // Send 10,000 messages rapidly
        for _ in 0..10000 {
            // Vary the send times to simulate real conditions
            let send_time = if rand::random::<f32>() < 0.1 {
                Duration::from_millis(200) // 10% slow
            } else {
                Duration::from_millis(20)  // 90% fast
            };

            handler.record_send_time(send_time).await.unwrap();
        }

        let metrics = handler.metrics();
        assert_eq!(metrics.total_messages, 10000);

        // Should detect some slow sends but not be overwhelmed
        assert!(metrics.slow_sends > 0);
        assert!(metrics.slow_sends < 2000); // Less than 20%
    }
}

// Test utilities and helpers
impl StreamingError {
    /// Helper for testing error types
    pub fn error_type(&self) -> &'static str {
        match self {
            StreamingError::BufferOverflow { .. } => "buffer_overflow",
            StreamingError::Connection { .. } => "connection",
            StreamingError::Serialization { .. } => "serialization",
            StreamingError::Channel { .. } => "channel",
            StreamingError::BackpressureExceeded { .. } => "backpressure_exceeded",
            StreamingError::ClientDisconnected { .. } => "client_disconnected",
            StreamingError::Pipeline { .. } => "pipeline",
            StreamingError::InvalidRequest { .. } => "invalid_request",
            StreamingError::Timeout { .. } => "timeout",
        }
    }
}

// Random number generation for stress tests
mod rand {
    pub fn random<T>() -> T
    where
        T: From<u8>,
    {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};
        use std::time::{SystemTime, UNIX_EPOCH};

        let mut hasher = DefaultHasher::new();
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos()
            .hash(&mut hasher);

        T::from((hasher.finish() % 256) as u8)
    }
}

impl From<u8> for f32 {
    fn from(value: u8) -> Self {
        value as f32 / 255.0
    }
}