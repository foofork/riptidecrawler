/*!
# Persistence Layer Integration Tests

Comprehensive integration tests for riptide-persistence layer integration
into riptide-api.

## Test Coverage

- Multi-tenant cache isolation
- TTL expiration handling
- Cache warming and performance
- Hot reload configuration
- Tenant provisioning and management
- Usage tracking and billing
- State checkpoint/restore
- Cache statistics and metrics
- Error handling and quotas
- Concurrent access patterns
- Performance benchmarks

## Test Requirements

- Running Redis instance at localhost:6379 (or REDIS_URL env)
- Set SKIP_PERSISTENCE_TESTS=1 to skip in CI without Redis
*/

#![cfg(all(test, feature = "persistence"))]

use anyhow::Result;
use riptide_api::persistence_adapter::PersistenceAdapter;
use riptide_persistence::{BillingPlan, PersistenceConfig, TenantConfig, TenantOwner};
use std::collections::HashMap;
use std::time::Duration;
use tokio::time::sleep;

/// Helper to create test persistence adapter
async fn create_test_adapter() -> Result<PersistenceAdapter> {
    let redis_url =
        std::env::var("REDIS_URL").unwrap_or_else(|_| "redis://localhost:6379".to_string());
    let config = PersistenceConfig::default();

    PersistenceAdapter::new(&redis_url, config).await
}

/// Check if Redis tests should be skipped
fn should_skip_redis_tests() -> bool {
    std::env::var("SKIP_PERSISTENCE_TESTS").is_ok()
}

/// Test 1: Create tenant with quota limits
#[tokio::test]
async fn test_create_tenant_with_quotas() -> Result<()> {
    if should_skip_redis_tests() {
        println!("Skipping Redis-dependent test");
        return Ok(());
    }

    let adapter = create_test_adapter().await?;

    let mut quotas = HashMap::new();
    quotas.insert("memory_mb".to_string(), 100);
    quotas.insert("operations_per_minute".to_string(), 1000);

    let tenant_config = TenantConfig {
        tenant_id: String::new(),
        name: "Test Tenant".to_string(),
        quotas: quotas.clone(),
        isolation_level: riptide_persistence::config::TenantIsolationLevel::Strong,
        encryption_enabled: true,
        settings: HashMap::new(),
        created_at: chrono::Utc::now(),
        updated_at: chrono::Utc::now(),
    };

    let owner = TenantOwner {
        id: uuid::Uuid::new_v4().to_string(),
        name: "Test Owner".to_string(),
        email: "test@example.com".to_string(),
        organization: None,
    };

    let tenant_id = adapter
        .create_tenant(tenant_config, owner, BillingPlan::Free)
        .await?;

    assert!(!tenant_id.is_empty(), "Tenant ID should be generated");

    // Verify tenant exists
    let tenant = adapter.get_tenant(&tenant_id).await?;
    assert!(tenant.is_some(), "Created tenant should exist");

    // Cleanup
    adapter.delete_tenant(&tenant_id).await?;

    Ok(())
}

/// Test 2: Multi-tenant cache isolation
#[tokio::test]
async fn test_multi_tenant_cache_isolation() -> Result<()> {
    if should_skip_redis_tests() {
        return Ok(());
    }

    let adapter = create_test_adapter().await?;

    // Create two tenants
    let tenant1_id = "tenant-isolation-1";
    let tenant2_id = "tenant-isolation-2";

    // Set same key for different tenants with different values
    adapter
        .set_cached(
            "shared_key",
            &"tenant1_value",
            Duration::from_secs(60),
            Some(tenant1_id),
        )
        .await?;
    adapter
        .set_cached(
            "shared_key",
            &"tenant2_value",
            Duration::from_secs(60),
            Some(tenant2_id),
        )
        .await?;

    // Verify isolation
    let value1: Option<String> = adapter.get_cached("shared_key", Some(tenant1_id)).await?;
    let value2: Option<String> = adapter.get_cached("shared_key", Some(tenant2_id)).await?;

    assert_eq!(
        value1,
        Some("tenant1_value".to_string()),
        "Tenant 1 should have its own value"
    );
    assert_eq!(
        value2,
        Some("tenant2_value".to_string()),
        "Tenant 2 should have its own value"
    );

    // Cleanup
    adapter.invalidate("shared_key", Some(tenant1_id)).await?;
    adapter.invalidate("shared_key", Some(tenant2_id)).await?;

    Ok(())
}

/// Test 3: TTL expiration handling
#[tokio::test]
async fn test_ttl_expiration() -> Result<()> {
    if should_skip_redis_tests() {
        return Ok(());
    }

    let adapter = create_test_adapter().await?;

    // Set value with 2-second TTL
    adapter
        .set_cached("ttl_test", &"expires_soon", Duration::from_secs(2), None)
        .await?;

    // Verify it exists immediately
    let value: Option<String> = adapter.get_cached("ttl_test", None).await?;
    assert_eq!(
        value,
        Some("expires_soon".to_string()),
        "Value should exist immediately"
    );

    // Wait for expiration
    sleep(Duration::from_secs(3)).await;

    // Verify it's gone
    let expired: Option<String> = adapter.get_cached("ttl_test", None).await?;
    assert_eq!(expired, None, "Value should expire after TTL");

    Ok(())
}

/// Test 4: Cache warming performance
#[tokio::test]
async fn test_cache_warming_performance() -> Result<()> {
    if should_skip_redis_tests() {
        return Ok(());
    }

    let adapter = create_test_adapter().await?;

    // Pre-populate cache with test data
    for i in 0..10 {
        let key = format!("warm_key_{}", i);
        adapter
            .set_cached(
                &key,
                &format!("value_{}", i),
                Duration::from_secs(300),
                None,
            )
            .await?;
    }

    // Warm cache with keys
    let keys: Vec<String> = (0..10).map(|i| format!("warm_key_{}", i)).collect();

    let start = std::time::Instant::now();
    let warmed = adapter.warm_cache(keys).await?;
    let duration = start.elapsed();

    assert_eq!(warmed, 10, "All 10 keys should be warmed");
    assert!(
        duration.as_millis() < 1000,
        "Warming should complete in <1 second"
    );

    // Cleanup
    for i in 0..10 {
        adapter.invalidate(&format!("warm_key_{}", i), None).await?;
    }

    Ok(())
}

/// Test 5: Cache statistics accuracy
#[tokio::test]
async fn test_cache_statistics() -> Result<()> {
    if should_skip_redis_tests() {
        return Ok(());
    }

    let adapter = create_test_adapter().await?;

    // Clear state and set some test data
    let test_key = "stats_test";
    adapter
        .set_cached(test_key, &"test_value", Duration::from_secs(60), None)
        .await?;

    // Get cache stats
    let stats = adapter.get_cache_stats().await?;

    assert!(stats.total_entries > 0, "Should have at least one entry");
    assert!(
        stats.hit_rate >= 0.0 && stats.hit_rate <= 1.0,
        "Hit rate should be 0-1"
    );

    // Cleanup
    adapter.invalidate(test_key, None).await?;

    Ok(())
}

/// Test 6: Tenant usage tracking
#[tokio::test]
async fn test_tenant_usage_tracking() -> Result<()> {
    if should_skip_redis_tests() {
        return Ok(());
    }

    let adapter = create_test_adapter().await?;

    // Create tenant
    let tenant_config = TenantConfig {
        tenant_id: String::new(),
        name: "Usage Test Tenant".to_string(),
        quotas: HashMap::new(),
        isolation_level: riptide_persistence::config::TenantIsolationLevel::Strong,
        encryption_enabled: true,
        settings: HashMap::new(),
        created_at: chrono::Utc::now(),
        updated_at: chrono::Utc::now(),
    };

    let owner = TenantOwner {
        id: uuid::Uuid::new_v4().to_string(),
        name: "Usage Tester".to_string(),
        email: "usage@example.com".to_string(),
        organization: None,
    };

    let tenant_id = adapter
        .create_tenant(tenant_config, owner, BillingPlan::Basic)
        .await?;

    // Perform some operations
    for i in 0..5 {
        adapter
            .set_cached(
                &format!("usage_key_{}", i),
                &i,
                Duration::from_secs(60),
                Some(&tenant_id),
            )
            .await?;
    }

    // Get usage stats
    let usage = adapter.get_tenant_usage(&tenant_id).await?;

    assert!(
        usage.total_operations >= 5,
        "Should track at least 5 operations"
    );

    // Cleanup
    adapter.delete_tenant(&tenant_id).await?;

    Ok(())
}

/// Test 7: Billing calculation
#[tokio::test]
async fn test_billing_calculation() -> Result<()> {
    if should_skip_redis_tests() {
        return Ok(());
    }

    let adapter = create_test_adapter().await?;

    // Create tenant with Professional plan
    let tenant_config = TenantConfig {
        tenant_id: String::new(),
        name: "Billing Test Tenant".to_string(),
        quotas: HashMap::new(),
        isolation_level: riptide_persistence::config::TenantIsolationLevel::Strong,
        encryption_enabled: true,
        settings: HashMap::new(),
        created_at: chrono::Utc::now(),
        updated_at: chrono::Utc::now(),
    };

    let owner = TenantOwner {
        id: uuid::Uuid::new_v4().to_string(),
        name: "Billing Tester".to_string(),
        email: "billing@example.com".to_string(),
        organization: None,
    };

    let tenant_id = adapter
        .create_tenant(tenant_config, owner, BillingPlan::Professional)
        .await?;

    // Get billing info
    let billing = adapter.get_tenant_billing(&tenant_id).await?;

    assert_eq!(
        format!("{:?}", billing.plan),
        "Professional",
        "Billing plan should match"
    );
    assert!(
        billing.current_period_cost >= 0.0,
        "Cost should be non-negative"
    );

    // Cleanup
    adapter.delete_tenant(&tenant_id).await?;

    Ok(())
}

/// Test 8: State checkpoint creation
#[tokio::test]
async fn test_checkpoint_creation() -> Result<()> {
    if should_skip_redis_tests() {
        return Ok(());
    }

    let adapter = create_test_adapter().await?;

    // Create checkpoint
    let checkpoint = adapter.create_checkpoint().await?;

    assert!(!checkpoint.id.is_empty(), "Checkpoint should have ID");
    assert!(checkpoint.data.len() > 0, "Checkpoint should have data");

    // List checkpoints
    let checkpoints = adapter.list_checkpoints().await?;
    assert!(checkpoints.len() > 0, "Should have at least one checkpoint");

    Ok(())
}

/// Test 9: State checkpoint restore
#[tokio::test]
async fn test_checkpoint_restore() -> Result<()> {
    if should_skip_redis_tests() {
        return Ok(());
    }

    let adapter = create_test_adapter().await?;

    // Create checkpoint
    let checkpoint = adapter.create_checkpoint().await?;

    // Restore from checkpoint
    let result = adapter.restore_checkpoint(&checkpoint.id).await;

    assert!(result.is_ok(), "Checkpoint restore should succeed");

    Ok(())
}

/// Test 10: Error handling for quota exceeded
#[tokio::test]
async fn test_quota_exceeded_error() -> Result<()> {
    if should_skip_redis_tests() {
        return Ok(());
    }

    let adapter = create_test_adapter().await?;

    // Create tenant with very low quota
    let mut quotas = HashMap::new();
    quotas.insert("operations_per_minute".to_string(), 1);

    let tenant_config = TenantConfig {
        tenant_id: String::new(),
        name: "Quota Test Tenant".to_string(),
        quotas,
        isolation_level: riptide_persistence::config::TenantIsolationLevel::Strong,
        encryption_enabled: true,
        settings: HashMap::new(),
        created_at: chrono::Utc::now(),
        updated_at: chrono::Utc::now(),
    };

    let owner = TenantOwner {
        id: uuid::Uuid::new_v4().to_string(),
        name: "Quota Tester".to_string(),
        email: "quota@example.com".to_string(),
        organization: None,
    };

    let tenant_id = adapter
        .create_tenant(tenant_config, owner, BillingPlan::Free)
        .await?;

    // First operation should succeed
    let result1 = adapter
        .check_quota(&tenant_id, "operations_per_minute", 1)
        .await;
    assert!(
        result1.is_ok(),
        "First operation within quota should succeed"
    );

    // Second operation should fail (quota exceeded)
    let result2 = adapter
        .check_quota(&tenant_id, "operations_per_minute", 1)
        .await;
    assert!(result2.is_err(), "Second operation should exceed quota");

    // Cleanup
    adapter.delete_tenant(&tenant_id).await?;

    Ok(())
}

/// Test 11: Concurrent cache access
#[tokio::test]
async fn test_concurrent_cache_access() -> Result<()> {
    if should_skip_redis_tests() {
        return Ok(());
    }

    let adapter = create_test_adapter().await?;

    // Spawn 10 concurrent write operations
    let mut handles = vec![];
    for i in 0..10 {
        let adapter_clone = adapter.clone();
        let handle = tokio::spawn(async move {
            let key = format!("concurrent_key_{}", i);
            adapter_clone
                .set_cached(&key, &i, Duration::from_secs(60), None)
                .await
        });
        handles.push(handle);
    }

    // Wait for all to complete
    for handle in handles {
        handle.await??;
    }

    // Verify all keys exist
    for i in 0..10 {
        let key = format!("concurrent_key_{}", i);
        let value: Option<i32> = adapter.get_cached(&key, None).await?;
        assert_eq!(value, Some(i), "Concurrent write {} should succeed", i);
        adapter.invalidate(&key, None).await?;
    }

    Ok(())
}

/// Test 12: Cache invalidation propagation
#[tokio::test]
async fn test_cache_invalidation_propagation() -> Result<()> {
    if should_skip_redis_tests() {
        return Ok(());
    }

    let adapter = create_test_adapter().await?;

    // Set value
    adapter
        .set_cached(
            "invalidate_test",
            &"will_be_deleted",
            Duration::from_secs(300),
            None,
        )
        .await?;

    // Verify it exists
    let value: Option<String> = adapter.get_cached("invalidate_test", None).await?;
    assert!(value.is_some(), "Value should exist before invalidation");

    // Invalidate
    adapter.invalidate("invalidate_test", None).await?;

    // Verify it's gone
    let gone: Option<String> = adapter.get_cached("invalidate_test", None).await?;
    assert!(gone.is_none(), "Value should be gone after invalidation");

    Ok(())
}

/// Test 13: Tenant deletion cleanup
#[tokio::test]
async fn test_tenant_deletion_cleanup() -> Result<()> {
    if should_skip_redis_tests() {
        return Ok(());
    }

    let adapter = create_test_adapter().await?;

    // Create tenant
    let tenant_config = TenantConfig {
        tenant_id: String::new(),
        name: "Delete Test Tenant".to_string(),
        quotas: HashMap::new(),
        isolation_level: riptide_persistence::config::TenantIsolationLevel::Strong,
        encryption_enabled: true,
        settings: HashMap::new(),
        created_at: chrono::Utc::now(),
        updated_at: chrono::Utc::now(),
    };

    let owner = TenantOwner {
        id: uuid::Uuid::new_v4().to_string(),
        name: "Delete Tester".to_string(),
        email: "delete@example.com".to_string(),
        organization: None,
    };

    let tenant_id = adapter
        .create_tenant(tenant_config, owner, BillingPlan::Free)
        .await?;

    // Add some tenant data
    adapter
        .set_cached(
            "tenant_data",
            &"test",
            Duration::from_secs(60),
            Some(&tenant_id),
        )
        .await?;

    // Delete tenant
    adapter.delete_tenant(&tenant_id).await?;

    // Verify tenant is gone
    let tenant = adapter.get_tenant(&tenant_id).await?;
    assert!(tenant.is_none(), "Deleted tenant should not exist");

    Ok(())
}

/// Test 14: Performance benchmark - sub-5ms cache reads
#[tokio::test]
async fn test_cache_read_performance() -> Result<()> {
    if should_skip_redis_tests() {
        return Ok(());
    }

    let adapter = create_test_adapter().await?;

    // Pre-populate cache
    adapter
        .set_cached(
            "perf_test",
            &"benchmark_value",
            Duration::from_secs(300),
            None,
        )
        .await?;

    // Warm up
    for _ in 0..10 {
        let _: Option<String> = adapter.get_cached("perf_test", None).await?;
    }

    // Benchmark 100 reads
    let start = std::time::Instant::now();
    for _ in 0..100 {
        let _: Option<String> = adapter.get_cached("perf_test", None).await?;
    }
    let duration = start.elapsed();

    let avg_ms = duration.as_micros() as f64 / 100.0 / 1000.0;

    println!("Average cache read time: {:.2}ms", avg_ms);
    assert!(
        avg_ms < 5.0,
        "Average read time should be <5ms, got {:.2}ms",
        avg_ms
    );

    // Cleanup
    adapter.invalidate("perf_test", None).await?;

    Ok(())
}

/// Test 15: Persistence layer health check
#[tokio::test]
async fn test_persistence_health_check() -> Result<()> {
    if should_skip_redis_tests() {
        return Ok(());
    }

    let adapter = create_test_adapter().await?;

    // Health check should pass
    let healthy = adapter.health_check().await;
    assert!(healthy, "Persistence layer should be healthy");

    Ok(())
}
