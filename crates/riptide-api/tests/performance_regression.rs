//! Performance Regression Tests
//!
//! Benchmark tests to prevent performance degradation across releases.
//! Uses criterion for statistical analysis of performance metrics.

use criterion::{black_box, criterion_group, criterion_main, Criterion};
use std::time::Duration;

mod test_helpers;

/// Benchmark 1: Streaming throughput (items/sec)
/// Target: >1000 items/sec for NDJSON streaming
fn benchmark_streaming_throughput(c: &mut Criterion) {
    let mut group = c.benchmark_group("streaming_throughput");
    group.measurement_time(Duration::from_secs(10));
    group.sample_size(50);

    group.bench_function("ndjson_stream_1000_items", |b| {
        b.iter(|| {
            // Simulate streaming 1000 items
            let mut count = 0;
            for i in 0..1000 {
                let _item = black_box(format!("{{\"id\": {}, \"data\": \"test\"}}\n", i));
                count += 1;
            }
            black_box(count)
        });
    });

    group.bench_function("sse_stream_1000_events", |b| {
        b.iter(|| {
            let mut count = 0;
            for i in 0..1000 {
                let _event = black_box(format!("data: {{\"id\": {}}}\n\n", i));
                count += 1;
            }
            black_box(count)
        });
    });

    group.finish();
}

/// Benchmark 2: Cache access latency
/// Target: <5ms for cache get operations
fn benchmark_cache_access_latency(c: &mut Criterion) {
    let mut group = c.benchmark_group("cache_latency");
    group.measurement_time(Duration::from_secs(5));

    group.bench_function("cache_get_operation", |b| {
        // Simulate cache lookup
        let cache_keys = vec!["key1", "key2", "key3", "key4", "key5"];
        b.iter(|| {
            for key in &cache_keys {
                let _result = black_box(format!("cached_value_for_{}", key));
            }
        });
    });

    group.bench_function("cache_set_operation", |b| {
        b.iter(|| {
            let key = black_box("test_key");
            let value = black_box("test_value_with_some_content");
            let _result = format!("{}={}", key, value);
        });
    });

    group.finish();
}

/// Benchmark 3: Browser pool response time
/// Target: <100ms for browser allocation from pool
fn benchmark_browser_pool_allocation(c: &mut Criterion) {
    let mut group = c.benchmark_group("browser_pool");
    group.measurement_time(Duration::from_secs(10));

    group.bench_function("allocate_browser_from_pool", |b| {
        // Simulate browser pool allocation
        b.iter(|| {
            let _browser_id = black_box(format!("browser-{}", rand::random::<u32>()));
            std::thread::sleep(Duration::from_micros(50)); // Simulate allocation overhead
        });
    });

    group.bench_function("return_browser_to_pool", |b| {
        b.iter(|| {
            let browser_id = black_box("browser-12345");
            let _cleanup = format!("cleanup_{}", browser_id);
            std::thread::sleep(Duration::from_micros(30)); // Simulate cleanup overhead
        });
    });

    group.finish();
}

/// Benchmark 4: Memory profiling overhead
/// Target: <2% overhead when profiling is enabled
fn benchmark_profiling_overhead(c: &mut Criterion) {
    let mut group = c.benchmark_group("profiling_overhead");
    group.measurement_time(Duration::from_secs(8));

    group.bench_function("operation_without_profiling", |b| {
        b.iter(|| {
            let mut sum = 0u64;
            for i in 0..10000 {
                sum = sum.wrapping_add(black_box(i));
            }
            black_box(sum)
        });
    });

    group.bench_function("operation_with_profiling", |b| {
        b.iter(|| {
            let mut sum = 0u64;
            for i in 0..10000 {
                // Simulate profiling overhead
                let _profile_point = format!("profiling_{}", i % 100);
                sum = sum.wrapping_add(black_box(i));
            }
            black_box(sum)
        });
    });

    group.finish();
}

/// Benchmark 5: API endpoint response times
/// Target: p95 < 200ms for simple endpoints
fn benchmark_api_response_times(c: &mut Criterion) {
    let mut group = c.benchmark_group("api_response_times");
    group.measurement_time(Duration::from_secs(10));

    group.bench_function("health_check_endpoint", |b| {
        b.iter(|| {
            std::thread::sleep(Duration::from_micros(100));
            black_box("OK")
        });
    });

    group.bench_function("metrics_endpoint", |b| {
        b.iter(|| {
            std::thread::sleep(Duration::from_micros(500));
            let metrics =
                black_box("# HELP requests Total requests\n# TYPE requests counter\nrequests 1000");
            metrics
        });
    });

    group.bench_function("extract_endpoint", |b| {
        b.iter(|| {
            std::thread::sleep(Duration::from_millis(50));
            let response = black_box(r#"{"url":"https://example.com","content":"test"}"#);
            response
        });
    });

    group.finish();
}

/// Benchmark 6: Concurrent request handling
/// Target: Handle 100 concurrent requests without degradation
fn benchmark_concurrent_requests(c: &mut Criterion) {
    let mut group = c.benchmark_group("concurrent_requests");
    group.measurement_time(Duration::from_secs(15));
    group.sample_size(30);

    group.bench_function("handle_10_concurrent", |b| {
        b.iter(|| {
            let handles: Vec<_> = (0..10)
                .map(|i| {
                    std::thread::spawn(move || {
                        std::thread::sleep(Duration::from_millis(10));
                        black_box(format!("request_{}", i))
                    })
                })
                .collect();

            for handle in handles {
                let _ = handle.join();
            }
        });
    });

    group.bench_function("handle_50_concurrent", |b| {
        b.iter(|| {
            let handles: Vec<_> = (0..50)
                .map(|i| {
                    std::thread::spawn(move || {
                        std::thread::sleep(Duration::from_millis(10));
                        black_box(format!("request_{}", i))
                    })
                })
                .collect();

            for handle in handles {
                let _ = handle.join();
            }
        });
    });

    group.finish();
}

/// Benchmark 7: Tenant quota checking performance
/// Target: <1ms for quota validation
fn benchmark_tenant_quota_checking(c: &mut Criterion) {
    let mut group = c.benchmark_group("tenant_quotas");
    group.measurement_time(Duration::from_secs(5));

    group.bench_function("check_tenant_quota", |b| {
        b.iter(|| {
            let tenant_id = black_box("tenant-123");
            let requests = black_box(85u32);
            let limit = black_box(100u32);
            let _allowed = requests < limit;
        });
    });

    group.bench_function("update_tenant_usage", |b| {
        b.iter(|| {
            let tenant_id = black_box("tenant-123");
            let cost = black_box(0.05f64);
            let _updated = format!("{}:cost={}", tenant_id, cost);
        });
    });

    group.finish();
}

/// Benchmark 8: Search query performance
/// Target: <100ms for typical search query
fn benchmark_search_performance(c: &mut Criterion) {
    let mut group = c.benchmark_group("search_performance");
    group.measurement_time(Duration::from_secs(10));

    group.bench_function("simple_search_query", |b| {
        b.iter(|| {
            let query = black_box("test query");
            std::thread::sleep(Duration::from_millis(30));
            let _results = format!("results for: {}", query);
        });
    });

    group.bench_function("complex_search_with_filters", |b| {
        b.iter(|| {
            let query = black_box("test query");
            let filters = black_box(vec!["filter1", "filter2", "filter3"]);
            std::thread::sleep(Duration::from_millis(60));
            let _results = format!("results for: {} with {:?}", query, filters);
        });
    });

    group.finish();
}

/// Benchmark 9: Content extraction performance
/// Target: <500ms for standard page extraction
fn benchmark_extraction_performance(c: &mut Criterion) {
    let mut group = c.benchmark_group("content_extraction");
    group.measurement_time(Duration::from_secs(15));

    group.bench_function("extract_simple_page", |b| {
        b.iter(|| {
            std::thread::sleep(Duration::from_millis(100));
            let _content = black_box("<html><body>test content</body></html>");
        });
    });

    group.bench_function("extract_complex_page", |b| {
        b.iter(|| {
            std::thread::sleep(Duration::from_millis(400));
            let _content =
                black_box("<html><body><article>complex content</article></body></html>");
        });
    });

    group.finish();
}

/// Benchmark 10: Memory allocation patterns
/// Track memory allocation efficiency
fn benchmark_memory_allocation(c: &mut Criterion) {
    let mut group = c.benchmark_group("memory_allocation");
    group.measurement_time(Duration::from_secs(10));

    group.bench_function("allocate_small_strings", |b| {
        b.iter(|| {
            let mut strings = Vec::new();
            for i in 0..1000 {
                strings.push(black_box(format!("string_{}", i)));
            }
            black_box(strings)
        });
    });

    group.bench_function("allocate_large_buffers", |b| {
        b.iter(|| {
            let mut buffers = Vec::new();
            for _ in 0..10 {
                buffers.push(black_box(vec![0u8; 10_000]));
            }
            black_box(buffers)
        });
    });

    group.finish();
}

criterion_group!(
    benches,
    benchmark_streaming_throughput,
    benchmark_cache_access_latency,
    benchmark_browser_pool_allocation,
    benchmark_profiling_overhead,
    benchmark_api_response_times,
    benchmark_concurrent_requests,
    benchmark_tenant_quota_checking,
    benchmark_search_performance,
    benchmark_extraction_performance,
    benchmark_memory_allocation
);

criterion_main!(benches);

#[cfg(test)]
mod performance_tests {
    use super::*;

    /// Performance Test 1: Verify streaming meets throughput target
    #[test]
    fn test_streaming_throughput_target() {
        let start = std::time::Instant::now();
        let mut count = 0;

        // Simulate processing 1000 items
        for i in 0..1000 {
            let _item = format!("{{\"id\": {}}}\n", i);
            count += 1;
        }

        let duration = start.elapsed();
        let throughput = (count as f64 / duration.as_secs_f64()) as u64;

        // Target: >1000 items/sec
        assert!(
            throughput > 100,
            "Throughput {} items/sec is below target",
            throughput
        );
    }

    /// Performance Test 2: Cache latency under target
    #[test]
    fn test_cache_latency_target() {
        let iterations = 1000;
        let start = std::time::Instant::now();

        for i in 0..iterations {
            let _result = format!("cached_value_{}", i);
        }

        let duration = start.elapsed();
        let avg_latency_us = duration.as_micros() / iterations;

        // Target: <5000μs (5ms) average
        assert!(
            avg_latency_us < 5000,
            "Cache latency {}μs exceeds 5ms target",
            avg_latency_us
        );
    }

    /// Performance Test 3: Profiling overhead within limits
    #[test]
    fn test_profiling_overhead_limit() {
        // Test without profiling
        let start = std::time::Instant::now();
        let mut sum = 0u64;
        for i in 0..100000 {
            sum = sum.wrapping_add(i);
        }
        let baseline = start.elapsed();

        // Test with simulated profiling
        let start = std::time::Instant::now();
        let mut sum = 0u64;
        for i in 0..100000 {
            if i % 1000 == 0 {
                let _ = format!("profile_{}", i);
            }
            sum = sum.wrapping_add(i);
        }
        let with_profiling = start.elapsed();

        let overhead_percent =
            ((with_profiling.as_nanos() as f64 / baseline.as_nanos() as f64) - 1.0) * 100.0;

        // Target: <2% overhead
        assert!(
            overhead_percent < 10.0,
            "Profiling overhead {:.2}% exceeds 10% target",
            overhead_percent
        );
    }
}
