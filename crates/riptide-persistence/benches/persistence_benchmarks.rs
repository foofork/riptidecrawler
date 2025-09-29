/*!
# Persistence Layer Benchmarks

Comprehensive benchmarks for validating performance targets and identifying bottlenecks.
*/

use criterion::{black_box, criterion_group, criterion_main, Criterion, BenchmarkId};
use riptide_persistence::{
    PersistentCacheManager, StateManager, TenantManager,
    TenantOwner, BillingPlan, ResourceUsageRecord,
    config::{CacheConfig, StateConfig, TenantConfig, CompressionAlgorithm, EvictionPolicy, TenantIsolationLevel},
    metrics::TenantMetrics
};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;
use tokio::runtime::Runtime;
use tokio::sync::RwLock;
use uuid::Uuid;

/// Runtime for async benchmarks
fn create_runtime() -> Runtime {
    tokio::runtime::Runtime::new().unwrap()
}

/// Create test cache configuration
fn create_bench_cache_config() -> CacheConfig {
    CacheConfig {
        default_ttl_seconds: 3600,
        max_entry_size_bytes: 10 * 1024 * 1024, // 10MB
        key_prefix: format!("bench_{}", Uuid::new_v4()),
        version: "bench_v1".to_string(),
        enable_compression: true,
        compression_threshold_bytes: 1024,
        compression_algorithm: CompressionAlgorithm::Lz4,
        enable_warming: false, // Disable for consistent benchmarks
        warming_batch_size: 100,
        max_memory_bytes: None,
        eviction_policy: EvictionPolicy::LRU,
    }
}

/// Get test Redis URL
fn get_bench_redis_url() -> String {
    std::env::var("REDIS_URL").unwrap_or_else(|_| "redis://localhost:6379/14".to_string())
}

/// Cache single operation benchmarks
fn bench_cache_single_ops(c: &mut Criterion) {
    let rt = create_runtime();
    let config = create_bench_cache_config();
    let redis_url = get_bench_redis_url();

    let cache_manager = rt.block_on(async {
        PersistentCacheManager::new(&redis_url, config).await.unwrap()
    });

    let mut group = c.benchmark_group("cache_single_ops");
    group.measurement_time(Duration::from_secs(10));

    // Benchmark cache set operations
    group.bench_function("set_small_1kb", |b| {
        let data = "x".repeat(1024); // 1KB
        b.to_async(&rt).iter(|| async {
            cache_manager.set(
                &format!("bench_key_{}", black_box(Uuid::new_v4())),
                black_box(&data),
                None,
                None,
                None,
            ).await.unwrap();
        });
    });

    group.bench_function("set_medium_10kb", |b| {
        let data = "x".repeat(10 * 1024); // 10KB
        b.to_async(&rt).iter(|| async {
            cache_manager.set(
                &format!("bench_key_{}", black_box(Uuid::new_v4())),
                black_box(&data),
                None,
                None,
                None,
            ).await.unwrap();
        });
    });

    group.bench_function("set_large_100kb", |b| {
        let data = "x".repeat(100 * 1024); // 100KB
        b.to_async(&rt).iter(|| async {
            cache_manager.set(
                &format!("bench_key_{}", black_box(Uuid::new_v4())),
                black_box(&data),
                None,
                None,
                None,
            ).await.unwrap();
        });
    });

    // Pre-populate for get benchmarks
    rt.block_on(async {
        for i in 0..1000 {
            let data = format!("benchmark_data_{}", i);
            cache_manager.set(&format!("get_bench_key_{}", i), &data, None, None, None).await.unwrap();
        }
    });

    // Benchmark cache get operations
    group.bench_function("get_existing", |b| {
        b.to_async(&rt).iter(|| async {
            let key = format!("get_bench_key_{}", black_box(fastrand::usize(0..1000)));
            let _result: Option<String> = cache_manager.get(&key, None).await.unwrap();
        });
    });

    group.bench_function("get_missing", |b| {
        b.to_async(&rt).iter(|| async {
            let key = format!("missing_key_{}", black_box(Uuid::new_v4()));
            let _result: Option<String> = cache_manager.get(&key, None).await.unwrap();
        });
    });

    group.finish();
}

/// Cache batch operation benchmarks
fn bench_cache_batch_ops(c: &mut Criterion) {
    let rt = create_runtime();
    let config = create_bench_cache_config();
    let redis_url = get_bench_redis_url();

    let cache_manager = rt.block_on(async {
        PersistentCacheManager::new(&redis_url, config).await.unwrap()
    });

    let mut group = c.benchmark_group("cache_batch_ops");
    group.measurement_time(Duration::from_secs(10));

    // Benchmark batch sizes
    for batch_size in [10, 50, 100, 200].iter() {
        group.bench_with_input(
            BenchmarkId::new("batch_set", batch_size),
            batch_size,
            |b, &size| {
                b.to_async(&rt).iter(|| async {
                    let mut batch_data = HashMap::new();
                    for i in 0..size {
                        batch_data.insert(
                            format!("batch_key_{}_{}", black_box(Uuid::new_v4()), i),
                            format!("batch_value_{}", i),
                        );
                    }
                    cache_manager.set_batch(batch_data, None, None).await.unwrap();
                });
            },
        );

        // Pre-populate for batch get benchmarks
        rt.block_on(async {
            let mut batch_data = HashMap::new();
            for i in 0..*batch_size {
                batch_data.insert(
                    format!("batch_get_key_{}", i),
                    format!("batch_get_value_{}", i),
                );
            }
            cache_manager.set_batch(batch_data, None, None).await.unwrap();
        });

        group.bench_with_input(
            BenchmarkId::new("batch_get", batch_size),
            batch_size,
            |b, &size| {
                b.to_async(&rt).iter(|| async {
                    let keys: Vec<String> = (0..size)
                        .map(|i| format!("batch_get_key_{}", i))
                        .collect();
                    let _result: HashMap<String, String> = cache_manager.get_batch(&keys, None).await.unwrap();
                });
            },
        );
    }

    group.finish();
}

/// Compression benchmark
fn bench_compression(c: &mut Criterion) {
    let rt = create_runtime();
    let config = create_bench_cache_config();
    let redis_url = get_bench_redis_url();

    let cache_manager = rt.block_on(async {
        PersistentCacheManager::new(&redis_url, config).await.unwrap()
    });

    let mut group = c.benchmark_group("compression");
    group.measurement_time(Duration::from_secs(10));

    // Test different data sizes with compression
    for data_size in [1024, 5120, 10240, 51200].iter() {
        let compressible_data = "x".repeat(*data_size);
        let incompressible_data = (0..*data_size)
            .map(|_| fastrand::u8(0..255) as char)
            .collect::<String>();

        group.bench_with_input(
            BenchmarkId::new("compressible_set", data_size),
            &compressible_data,
            |b, data| {
                b.to_async(&rt).iter(|| async {
                    cache_manager.set(
                        &format!("compress_key_{}", black_box(Uuid::new_v4())),
                        black_box(data),
                        None,
                        None,
                        None,
                    ).await.unwrap();
                });
            },
        );

        group.bench_with_input(
            BenchmarkId::new("incompressible_set", data_size),
            &incompressible_data,
            |b, data| {
                b.to_async(&rt).iter(|| async {
                    cache_manager.set(
                        &format!("incompress_key_{}", black_box(Uuid::new_v4())),
                        black_box(data),
                        None,
                        None,
                        None,
                    ).await.unwrap();
                });
            },
        );
    }

    group.finish();
}

/// Session management benchmarks
fn bench_session_management(c: &mut Criterion) {
    let rt = create_runtime();
    let redis_url = get_bench_redis_url();

    let state_config = StateConfig {
        session_timeout_seconds: 3600,
        enable_hot_reload: false,
        config_watch_paths: vec![],
        checkpoint_interval_seconds: 0, // Disable for benchmarks
        max_checkpoints: 5,
        checkpoint_compression: true,
        enable_graceful_shutdown: false,
        shutdown_timeout_seconds: 30,
    };

    let state_manager = rt.block_on(async {
        StateManager::new(&redis_url, state_config).await.unwrap()
    });

    let mut group = c.benchmark_group("session_management");
    group.measurement_time(Duration::from_secs(10));

    // Benchmark session creation
    group.bench_function("create_session", |b| {
        b.to_async(&rt).iter(|| async {
            let metadata = riptide_persistence::state::SessionMetadata {
                client_ip: Some("192.168.1.100".to_string()),
                user_agent: Some("BenchmarkAgent".to_string()),
                source: Some("benchmark".to_string()),
                attributes: HashMap::new(),
            };

            let _session_id = state_manager.create_session(
                Some(format!("bench_user_{}", black_box(Uuid::new_v4()))),
                metadata,
                Some(3600),
            ).await.unwrap();
        });
    });

    // Pre-create sessions for get benchmarks
    let session_ids = rt.block_on(async {
        let mut ids = Vec::new();
        for i in 0..100 {
            let metadata = riptide_persistence::state::SessionMetadata {
                client_ip: Some(format!("192.168.1.{}", i + 1)),
                user_agent: Some("BenchmarkAgent".to_string()),
                source: Some("benchmark".to_string()),
                attributes: HashMap::new(),
            };

            let session_id = state_manager.create_session(
                Some(format!("bench_user_{}", i)),
                metadata,
                Some(3600),
            ).await.unwrap();
            ids.push(session_id);
        }
        ids
    });

    // Benchmark session retrieval
    group.bench_function("get_session", |b| {
        b.to_async(&rt).iter(|| async {
            let session_id = &session_ids[black_box(fastrand::usize(0..session_ids.len()))];
            let _session = state_manager.get_session(session_id).await.unwrap();
        });
    });

    // Benchmark session data updates
    group.bench_function("update_session_data", |b| {
        b.to_async(&rt).iter(|| async {
            let session_id = &session_ids[black_box(fastrand::usize(0..session_ids.len()))];
            state_manager.update_session_data(
                session_id,
                "bench_key",
                serde_json::json!({"value": Uuid::new_v4().to_string()}),
            ).await.unwrap();
        });
    });

    group.finish();
}

/// Tenant management benchmarks
fn bench_tenant_management(c: &mut Criterion) {
    let rt = create_runtime();
    let redis_url = get_bench_redis_url();

    let tenant_config = TenantConfig {
        enabled: true,
        default_quotas: {
            let mut quotas = HashMap::new();
            quotas.insert("memory_bytes".to_string(), 100 * 1024 * 1024); // 100MB
            quotas.insert("operations_per_minute".to_string(), 10000);
            quotas
        },
        isolation_level: TenantIsolationLevel::Logical,
        enable_billing: true,
        billing_interval_seconds: 60,
        max_tenants: 1000,
        enable_encryption: false, // Disable for performance
    };

    let tenant_metrics = Arc::new(RwLock::new(TenantMetrics::new(Arc::new(prometheus::Registry::new()))));
    let tenant_manager = rt.block_on(async {
        TenantManager::new(&redis_url, tenant_config.clone(), tenant_metrics).await.unwrap()
    });

    let mut group = c.benchmark_group("tenant_management");
    group.measurement_time(Duration::from_secs(10));

    // Benchmark tenant creation
    group.bench_function("create_tenant", |b| {
        b.to_async(&rt).iter(|| async {
            let owner = TenantOwner {
                id: format!("bench_owner_{}", black_box(Uuid::new_v4())),
                name: "Benchmark Owner".to_string(),
                email: "bench@example.com".to_string(),
                organization: Some("Benchmark Corp".to_string()),
            };

            let tenant_config = riptide_persistence::tenant::TenantConfig {
                tenant_id: "".to_string(),
                name: format!("Benchmark Tenant {}", Uuid::new_v4()),
                quotas: tenant_config.default_quotas.clone(),
                isolation_level: TenantIsolationLevel::Logical,
                encryption_enabled: false,
                settings: HashMap::new(),
                created_at: chrono::Utc::now(),
                updated_at: chrono::Utc::now(),
            };

            let _tenant_id = tenant_manager.create_tenant(
                tenant_config,
                owner,
                BillingPlan::Basic,
            ).await.unwrap();
        });
    });

    // Pre-create tenants for other benchmarks
    let tenant_ids = rt.block_on(async {
        let mut ids = Vec::new();
        for i in 0..50 {
            let owner = TenantOwner {
                id: format!("bench_owner_{}", i),
                name: format!("Benchmark Owner {}", i),
                email: format!("bench{}@example.com", i),
                organization: Some("Benchmark Corp".to_string()),
            };

            let tenant_config = riptide_persistence::tenant::TenantConfig {
                tenant_id: "".to_string(),
                name: format!("Benchmark Tenant {}", i),
                quotas: tenant_config.default_quotas.clone(),
                isolation_level: TenantIsolationLevel::Logical,
                encryption_enabled: false,
                settings: HashMap::new(),
                created_at: chrono::Utc::now(),
                updated_at: chrono::Utc::now(),
            };

            let tenant_id = tenant_manager.create_tenant(
                tenant_config,
                owner,
                BillingPlan::Basic,
            ).await.unwrap();
            ids.push(tenant_id);
        }
        ids
    });

    // Benchmark tenant retrieval
    group.bench_function("get_tenant", |b| {
        b.to_async(&rt).iter(|| async {
            let tenant_id = &tenant_ids[black_box(fastrand::usize(0..tenant_ids.len()))];
            let _tenant = tenant_manager.get_tenant(tenant_id).await.unwrap();
        });
    });

    // Benchmark access validation
    group.bench_function("validate_access", |b| {
        b.to_async(&rt).iter(|| async {
            let tenant_id = &tenant_ids[black_box(fastrand::usize(0..tenant_ids.len()))];
            let _access = tenant_manager.validate_access(
                tenant_id,
                "resource",
                "action",
            ).await.unwrap();
        });
    });

    // Benchmark quota checking
    group.bench_function("check_quota", |b| {
        b.to_async(&rt).iter(|| async {
            let tenant_id = &tenant_ids[black_box(fastrand::usize(0..tenant_ids.len()))];
            let _quota = tenant_manager.check_quota(
                tenant_id,
                "memory_bytes",
                black_box(1024),
            ).await.unwrap();
        });
    });

    // Benchmark usage recording
    group.bench_function("record_usage", |b| {
        b.to_async(&rt).iter(|| async {
            let tenant_id = &tenant_ids[black_box(fastrand::usize(0..tenant_ids.len()))];
            let usage_record = ResourceUsageRecord {
                operation_count: 1,
                data_bytes: 1024,
                compute_time_ms: 50,
                storage_bytes: 2048,
                timestamp: chrono::Utc::now(),
            };

            tenant_manager.record_usage(
                tenant_id,
                "benchmark_operation",
                usage_record,
            ).await.unwrap();
        });
    });

    group.finish();
}

/// Concurrent access benchmarks
fn bench_concurrent_access(c: &mut Criterion) {
    let rt = create_runtime();
    let config = create_bench_cache_config();
    let redis_url = get_bench_redis_url();

    let cache_manager = Arc::new(rt.block_on(async {
        PersistentCacheManager::new(&redis_url, config).await.unwrap()
    }));

    // Pre-populate data
    rt.block_on(async {
        for i in 0..1000 {
            cache_manager.set(
                &format!("concurrent_key_{}", i),
                &format!("concurrent_value_{}", i),
                None,
                None,
                None,
            ).await.unwrap();
        }
    });

    let mut group = c.benchmark_group("concurrent_access");
    group.measurement_time(Duration::from_secs(15));

    // Benchmark concurrent reads
    for concurrency in [1, 2, 4, 8, 16].iter() {
        group.bench_with_input(
            BenchmarkId::new("concurrent_reads", concurrency),
            concurrency,
            |b, &concurrent_tasks| {
                b.to_async(&rt).iter(|| async {
                    let mut handles = Vec::new();

                    for _ in 0..concurrent_tasks {
                        let cache_manager_clone = Arc::clone(&cache_manager);
                        let handle = tokio::spawn(async move {
                            for _ in 0..10 {
                                let key = format!("concurrent_key_{}", fastrand::usize(0..1000));
                                let _result: Option<String> = cache_manager_clone.get(&key, None).await.unwrap();
                            }
                        });
                        handles.push(handle);
                    }

                    for handle in handles {
                        handle.await.unwrap();
                    }
                });
            },
        );
    }

    group.finish();
}

criterion_group!(
    benches,
    bench_cache_single_ops,
    bench_cache_batch_ops,
    bench_compression,
    bench_session_management,
    bench_tenant_management,
    bench_concurrent_access
);

criterion_main!(benches);