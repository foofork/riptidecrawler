//! Tests for Core Orchestration Logic (Post-Cleanup)
//!
//! These tests define the expected behavior for the cleaned riptide-core
//! that focuses only on orchestration, pipeline management, and core infrastructure.

use anyhow::Result;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::RwLock;
use tokio::time::timeout;

use riptide_core::{
    cache::{Cache, CacheConfig, CacheStats},
    cache_warming::{CacheWarmingConfig, CacheWarmingStrategy, WarmingSchedule},
    circuit::CircuitBreaker,
    events::{EventBus, EventBusConfig, EventSeverity, BaseEvent, EventEmitter, EventHandler},
    instance_pool::{InstancePool, InstancePoolConfig},
    memory_manager::{MemoryManager, MemoryManagerConfig},
    spider::{Spider, SpiderConfig, FrontierManager},
    telemetry::{TelemetryCollector, TelemetryConfig},
    types::{ComponentInfo, HealthStatus},
};

/// Test that core orchestration components can be initialized
#[tokio::test]
async fn test_core_orchestration_initialization() -> Result<()> {
    // Test Event Bus initialization
    let event_bus_config = EventBusConfig::default();
    let event_bus = EventBus::new(event_bus_config)?;
    assert!(event_bus.is_healthy().await);

    // Test Cache initialization
    let cache_config = CacheConfig::default();
    let cache = Cache::new(cache_config)?;
    assert!(cache.is_healthy());

    // Test Circuit Breaker initialization
    let circuit_breaker = CircuitBreaker::new("test", 5, Duration::from_secs(60));
    assert!(circuit_breaker.is_closed());

    // Test Memory Manager initialization
    let memory_config = MemoryManagerConfig::default();
    let memory_manager = MemoryManager::new(memory_config)?;
    assert!(memory_manager.is_healthy());

    Ok(())
}

/// Test pipeline orchestration flow
#[tokio::test]
async fn test_pipeline_orchestration() -> Result<()> {
    // Initialize core components
    let event_bus = Arc::new(EventBus::new(EventBusConfig::default())?);
    let cache = Arc::new(Cache::new(CacheConfig::default())?);
    let memory_manager = Arc::new(MemoryManager::new(MemoryManagerConfig::default())?);

    // Test component integration
    let pipeline_id = "test_pipeline";

    // Start orchestration
    let start_event = BaseEvent::new(
        "pipeline.started",
        "core_orchestrator",
        EventSeverity::Info,
    );
    event_bus.emit_event(start_event).await?;

    // Simulate cache warming
    cache.warm("test_key", "test_value", Duration::from_secs(60))?;
    assert!(cache.contains("test_key"));

    // Test memory allocation tracking
    memory_manager.allocate(1024 * 1024)?; // 1MB
    let usage = memory_manager.get_usage();
    assert!(usage.allocated_bytes > 0);

    // Complete orchestration
    let complete_event = BaseEvent::new(
        "pipeline.completed",
        "core_orchestrator",
        EventSeverity::Info,
    );
    event_bus.emit_event(complete_event).await?;

    Ok(())
}

/// Test circuit breaker integration in pipeline
#[tokio::test]
async fn test_circuit_breaker_orchestration() -> Result<()> {
    let circuit_breaker = Arc::new(CircuitBreaker::new("test_service", 2, Duration::from_secs(5)));

    // Simulate successful operations
    for _ in 0..3 {
        let result = circuit_breaker.call(|| async {
            Ok::<String, anyhow::Error>("success".to_string())
        }).await;
        assert!(result.is_ok());
    }

    // Simulate failures to trip circuit breaker
    for _ in 0..3 {
        let result = circuit_breaker.call(|| async {
            Err::<String, anyhow::Error>(anyhow::anyhow!("service failure"))
        }).await;
        assert!(result.is_err());
    }

    // Circuit should now be open
    assert!(circuit_breaker.is_open());

    // Wait for half-open state
    tokio::time::sleep(Duration::from_secs(6)).await;

    // Should allow one test call
    let result = circuit_breaker.call(|| async {
        Ok::<String, anyhow::Error>("recovery".to_string())
    }).await;
    assert!(result.is_ok());

    Ok(())
}

/// Test instance pool orchestration
#[tokio::test]
async fn test_instance_pool_orchestration() -> Result<()> {
    let pool_config = InstancePoolConfig {
        min_instances: 2,
        max_instances: 5,
        max_idle_time: Duration::from_secs(30),
        health_check_interval: Duration::from_secs(10),
    };

    let pool = InstancePool::new(pool_config)?;

    // Pool should start with minimum instances
    assert_eq!(pool.instance_count(), 2);

    // Acquire instances
    let instance1 = pool.acquire().await?;
    let instance2 = pool.acquire().await?;

    assert!(instance1.is_healthy());
    assert!(instance2.is_healthy());

    // Return instances
    pool.release(instance1).await?;
    pool.release(instance2).await?;

    // Pool should still maintain minimum instances
    assert_eq!(pool.instance_count(), 2);

    Ok(())
}

/// Test spider orchestration (basic frontier management only)
#[tokio::test]
async fn test_spider_orchestration_basic() -> Result<()> {
    let spider_config = SpiderConfig {
        max_concurrent_requests: 2,
        request_delay: Duration::from_millis(100),
        max_depth: 3,
        max_pages: 10,
        ..Default::default()
    };

    let spider = Spider::new(spider_config)?;

    // Test frontier management
    let start_url = "https://example.com";
    spider.add_seed_url(start_url)?;

    assert!(spider.has_pending_urls());
    assert_eq!(spider.pending_url_count(), 1);

    // Test basic crawl state
    let state = spider.get_crawl_state();
    assert_eq!(state.total_processed, 0);
    assert!(state.is_active);

    Ok(())
}

/// Test telemetry integration
#[tokio::test]
async fn test_telemetry_orchestration() -> Result<()> {
    let telemetry_config = TelemetryConfig::default();
    let telemetry = TelemetryCollector::new(telemetry_config)?;

    // Record metrics
    telemetry.record_counter("requests_total", 1)?;
    telemetry.record_histogram("request_duration", 100.0)?;
    telemetry.record_gauge("active_connections", 5.0)?;

    // Get metrics
    let metrics = telemetry.get_metrics();
    assert!(!metrics.is_empty());

    // Test health status
    let health = telemetry.health_status();
    assert_eq!(health.status, "healthy");

    Ok(())
}

/// Test cache warming orchestration
#[tokio::test]
async fn test_cache_warming_orchestration() -> Result<()> {
    let cache = Arc::new(Cache::new(CacheConfig::default())?);
    let warming_config = CacheWarmingConfig {
        enabled: true,
        strategy: CacheWarmingStrategy::Predictive,
        schedule: WarmingSchedule::Interval(Duration::from_secs(10)),
        batch_size: 100,
    };

    // Test warming strategy initialization
    let warmer = cache.create_warmer(warming_config)?;
    assert!(warmer.is_enabled());

    // Test predictive warming
    cache.warm("frequently_accessed", "data", Duration::from_secs(300))?;
    assert!(cache.contains("frequently_accessed"));

    // Simulate access pattern
    for _ in 0..10 {
        let _ = cache.get("frequently_accessed");
    }

    // Cache should track access patterns
    let stats = cache.get_stats();
    assert!(stats.hit_count > 0);

    Ok(())
}

/// Test error handling and resilience in orchestration
#[tokio::test]
async fn test_orchestration_resilience() -> Result<()> {
    let event_bus = Arc::new(EventBus::new(EventBusConfig::default())?);
    let cache = Arc::new(Cache::new(CacheConfig::default())?);

    // Test error event handling
    let error_event = BaseEvent::new(
        "component.error",
        "test_component",
        EventSeverity::Error,
    );

    // Should not crash orchestration
    let result = event_bus.emit_event(error_event).await;
    assert!(result.is_ok());

    // Cache should handle invalid operations gracefully
    let invalid_result = cache.get("nonexistent_key");
    assert!(invalid_result.is_none());

    // System should remain operational
    assert!(event_bus.is_healthy().await);
    assert!(cache.is_healthy());

    Ok(())
}

/// Test component health monitoring
#[tokio::test]
async fn test_component_health_monitoring() -> Result<()> {
    let event_bus = EventBus::new(EventBusConfig::default())?;
    let cache = Cache::new(CacheConfig::default())?;
    let memory_manager = MemoryManager::new(MemoryManagerConfig::default())?;

    // All components should report healthy initially
    assert!(event_bus.is_healthy().await);
    assert!(cache.is_healthy());
    assert!(memory_manager.is_healthy());

    // Test health status collection
    let components = vec![
        ("event_bus", event_bus.health_status().await),
        ("cache", cache.health_status()),
        ("memory_manager", memory_manager.health_status()),
    ];

    for (name, health) in components {
        assert_eq!(health.status, "healthy", "Component {} should be healthy", name);
        assert!(!health.version.is_empty(), "Component {} should have version", name);
    }

    Ok(())
}

/// Test concurrent orchestration operations
#[tokio::test]
async fn test_concurrent_orchestration() -> Result<()> {
    let event_bus = Arc::new(EventBus::new(EventBusConfig::default())?);
    let cache = Arc::new(Cache::new(CacheConfig::default())?);

    // Spawn concurrent operations
    let mut handles = vec![];

    // Concurrent event emission
    for i in 0..10 {
        let bus_clone = Arc::clone(&event_bus);
        let handle = tokio::spawn(async move {
            let event = BaseEvent::new(
                "test.concurrent",
                &format!("worker_{}", i),
                EventSeverity::Info,
            );
            bus_clone.emit_event(event).await
        });
        handles.push(handle);
    }

    // Concurrent cache operations
    for i in 0..10 {
        let cache_clone = Arc::clone(&cache);
        let handle = tokio::spawn(async move {
            let key = format!("concurrent_key_{}", i);
            let value = format!("value_{}", i);
            cache_clone.warm(&key, &value, Duration::from_secs(60))
        });
        handles.push(handle);
    }

    // Wait for all operations to complete
    for handle in handles {
        let result = timeout(Duration::from_secs(5), handle).await??;
        assert!(result.is_ok());
    }

    // Verify cache contains all concurrent writes
    for i in 0..10 {
        let key = format!("concurrent_key_{}", i);
        assert!(cache.contains(&key));
    }

    Ok(())
}

/// Mock implementations for testing
struct MockInstance {
    id: String,
    healthy: bool,
}

impl MockInstance {
    fn new(id: String) -> Self {
        Self { id, healthy: true }
    }

    fn is_healthy(&self) -> bool {
        self.healthy
    }
}