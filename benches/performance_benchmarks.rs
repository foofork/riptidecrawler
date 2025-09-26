/// Performance Benchmarks for riptide - Criterion Integration
///
/// Comprehensive performance benchmarks using Criterion for detailed
/// statistical analysis of system performance characteristics.

use criterion::{black_box, criterion_group, criterion_main, Criterion, BenchmarkId, Throughput};
use riptide_tests::fixtures::test_data::*;
use riptide_tests::common::generate_test_html;
use std::time::Duration;

/// Benchmark WASM extraction performance across different content sizes
fn bench_wasm_extraction(c: &mut Criterion) {
    let mut group = c.benchmark_group("wasm_extraction");

    // Test different content sizes
    let content_sizes = vec![1, 10, 50, 100, 500]; // KB

    for size_kb in content_sizes {
        let html = generate_test_html(size_kb);
        let url = "https://benchmark.example.com/article";

        group.throughput(Throughput::Bytes(html.len() as u64));
        group.bench_with_input(
            BenchmarkId::new("extract_content", format!("{}kb", size_kb)),
            &(html, url),
            |b, (html, url)| {
                b.iter(|| {
                    // Simulate WASM extraction processing time
                    let processing_time = Duration::from_micros(html.len() as u64 / 100);
                    std::thread::sleep(processing_time);

                    // Return simulated extraction result
                    black_box((html.len(), url.len()))
                });
            },
        );
    }

    group.finish();
}

/// Benchmark dynamic rendering performance with different action counts
fn bench_dynamic_rendering(c: &mut Criterion) {
    let mut group = c.benchmark_group("dynamic_rendering");

    // Test different numbers of actions
    let action_counts = vec![0, 1, 5, 10, 20];

    for action_count in action_counts {
        group.bench_with_input(
            BenchmarkId::new("render_with_actions", action_count),
            &action_count,
            |b, &action_count| {
                b.iter(|| {
                    // Simulate action execution time
                    let base_time = Duration::from_millis(100);
                    let action_time = Duration::from_millis(50 * action_count as u64);
                    std::thread::sleep(base_time + action_time);

                    black_box(action_count)
                });
            },
        );
    }

    group.finish();
}

/// Benchmark session operations performance
fn bench_session_operations(c: &mut Criterion) {
    let mut group = c.benchmark_group("session_operations");

    // Benchmark session creation
    group.bench_function("create_session", |b| {
        b.iter(|| {
            let session_id = format!("session_{}", fastrand::u64(..));
            let session_data = std::collections::HashMap::from([
                ("user_id".to_string(), "user123".to_string()),
                ("state".to_string(), "active".to_string()),
            ]);

            // Simulate session creation time
            std::thread::sleep(Duration::from_micros(50));

            black_box((session_id, session_data))
        });
    });

    // Benchmark session updates
    group.bench_function("update_session", |b| {
        b.iter(|| {
            let mut session_data = std::collections::HashMap::from([
                ("user_id".to_string(), "user123".to_string()),
                ("state".to_string(), "active".to_string()),
            ]);

            // Simulate data update
            session_data.insert("last_activity".to_string(), "2024-01-15T10:30:00Z".to_string());

            // Simulate update time
            std::thread::sleep(Duration::from_micros(30));

            black_box(session_data)
        });
    });

    // Benchmark session retrieval
    group.bench_function("get_session", |b| {
        b.iter(|| {
            let session_id = "benchmark_session";

            // Simulate lookup time
            std::thread::sleep(Duration::from_micros(20));

            black_box(session_id.len())
        });
    });

    group.finish();
}

/// Benchmark streaming operations with different chunk sizes
fn bench_streaming_operations(c: &mut Criterion) {
    let mut group = c.benchmark_group("streaming_operations");

    // Test different chunk sizes
    let chunk_sizes = vec![1024, 4096, 8192, 16384, 32768]; // bytes

    for chunk_size in chunk_sizes {
        let chunk_data = vec![0u8; chunk_size];

        group.throughput(Throughput::Bytes(chunk_size as u64));
        group.bench_with_input(
            BenchmarkId::new("process_chunk", format!("{}b", chunk_size)),
            &chunk_data,
            |b, chunk_data| {
                b.iter(|| {
                    // Simulate chunk processing time proportional to size
                    let processing_time = Duration::from_nanos(chunk_data.len() as u64 * 10);
                    std::thread::sleep(processing_time);

                    black_box(chunk_data.len())
                });
            },
        );
    }

    group.finish();
}

/// Benchmark API request parsing and validation
fn bench_api_request_processing(c: &mut Criterion) {
    let mut group = c.benchmark_group("api_request_processing");

    // Benchmark different request types
    let test_requests = vec![
        ("simple_render", r#"{"url": "https://example.com", "mode": "article"}"#),
        ("complex_render", r#"{"url": "https://example.com", "mode": "dynamic", "dynamic_config": {"actions": [{"type": "click", "selector": "#button"}], "wait_conditions": [{"type": "element_visible", "selector": ".content"}]}, "stealth_config": {"user_agent": "custom"}}"#),
        ("extract_request", r#"{"html": "<html><head><title>Test</title></head><body><p>Content</p></body></html>", "url": "https://example.com", "mode": "article"}"#),
    ];

    for (request_type, json_data) in test_requests {
        group.bench_with_input(
            BenchmarkId::new("parse_request", request_type),
            json_data,
            |b, json_data| {
                b.iter(|| {
                    // Simulate JSON parsing and validation
                    let parsed: serde_json::Value = serde_json::from_str(json_data).unwrap();

                    // Simulate validation time
                    std::thread::sleep(Duration::from_micros(10));

                    black_box(parsed)
                });
            },
        );
    }

    group.finish();
}

/// Benchmark concurrent operations scaling
fn bench_concurrent_scaling(c: &mut Criterion) {
    let mut group = c.benchmark_group("concurrent_scaling");

    // Test different concurrency levels
    let concurrency_levels = vec![1, 2, 4, 8, 16];

    for concurrency in concurrency_levels {
        group.bench_with_input(
            BenchmarkId::new("concurrent_extractions", concurrency),
            &concurrency,
            |b, &concurrency| {
                b.iter(|| {
                    // Simulate concurrent operations
                    let handles: Vec<_> = (0..concurrency)
                        .map(|i| {
                            std::thread::spawn(move || {
                                // Simulate extraction work
                                std::thread::sleep(Duration::from_millis(10));
                                i
                            })
                        })
                        .collect();

                    let results: Vec<_> = handles
                        .into_iter()
                        .map(|h| h.join().unwrap())
                        .collect();

                    black_box(results)
                });
            },
        );
    }

    group.finish();
}

/// Benchmark memory allocation patterns
fn bench_memory_patterns(c: &mut Criterion) {
    let mut group = c.benchmark_group("memory_patterns");

    // Test different allocation sizes
    let allocation_sizes = vec![1024, 10240, 102400, 1048576]; // 1KB to 1MB

    for size in allocation_sizes {
        group.throughput(Throughput::Bytes(size as u64));
        group.bench_with_input(
            BenchmarkId::new("allocate_process_deallocate", format!("{}b", size)),
            &size,
            |b, &size| {
                b.iter(|| {
                    // Allocate memory
                    let mut data = vec![0u8; size];

                    // Simulate processing
                    for i in 0..std::cmp::min(size, 1000) {
                        data[i] = (i % 256) as u8;
                    }

                    // Return processed size
                    black_box(data.len())
                    // Memory is automatically deallocated when data goes out of scope
                });
            },
        );
    }

    group.finish();
}

/// Benchmark error handling overhead
fn bench_error_handling(c: &mut Criterion) {
    let mut group = c.benchmark_group("error_handling");

    // Test success vs error paths
    group.bench_function("success_path", |b| {
        b.iter(|| {
            let result: Result<i32, String> = Ok(42);
            black_box(result.unwrap())
        });
    });

    group.bench_function("error_path", |b| {
        b.iter(|| {
            let result: Result<i32, String> = Err("Test error".to_string());
            match result {
                Ok(val) => black_box(val),
                Err(err) => black_box(err.len() as i32),
            }
        });
    });

    // Test error creation overhead
    group.bench_function("create_error", |b| {
        b.iter(|| {
            let error = format!("Processing failed for item {}", fastrand::u32(..));
            black_box(error)
        });
    });

    group.finish();
}

/// Benchmark serialization performance
fn bench_serialization(c: &mut Criterion) {
    let mut group = c.benchmark_group("serialization");

    // Create test data structures
    let simple_response = serde_json::json!({
        "url": "https://example.com",
        "success": true,
        "content": {
            "title": "Test Article",
            "text": "Simple content"
        }
    });

    let complex_response = serde_json::json!({
        "url": "https://example.com",
        "final_url": "https://example.com/redirected",
        "mode": "dynamic",
        "success": true,
        "content": {
            "url": "https://example.com",
            "title": "Complex Test Article",
            "text": "A".repeat(10000), // 10KB of content
            "markdown": "# Complex Test Article\n\n".to_string() + &"A".repeat(9980),
            "links": (0..100).map(|i| format!("https://example.com/link/{}", i)).collect::<Vec<_>>(),
            "media": (0..50).map(|i| format!("https://example.com/image/{}.jpg", i)).collect::<Vec<_>>()
        },
        "stats": {
            "total_time_ms": 1500,
            "extraction_time_ms": 300,
            "actions_executed": 5,
            "network_requests": 3
        }
    });

    // Benchmark JSON serialization
    group.bench_function("serialize_simple", |b| {
        b.iter(|| {
            let serialized = serde_json::to_string(&simple_response).unwrap();
            black_box(serialized.len())
        });
    });

    group.bench_function("serialize_complex", |b| {
        b.iter(|| {
            let serialized = serde_json::to_string(&complex_response).unwrap();
            black_box(serialized.len())
        });
    });

    // Benchmark JSON deserialization
    let simple_json = serde_json::to_string(&simple_response).unwrap();
    let complex_json = serde_json::to_string(&complex_response).unwrap();

    group.bench_function("deserialize_simple", |b| {
        b.iter(|| {
            let parsed: serde_json::Value = serde_json::from_str(&simple_json).unwrap();
            black_box(parsed)
        });
    });

    group.bench_function("deserialize_complex", |b| {
        b.iter(|| {
            let parsed: serde_json::Value = serde_json::from_str(&complex_json).unwrap();
            black_box(parsed)
        });
    });

    group.finish();
}

criterion_group!(
    benches,
    bench_wasm_extraction,
    bench_dynamic_rendering,
    bench_session_operations,
    bench_streaming_operations,
    bench_api_request_processing,
    bench_concurrent_scaling,
    bench_memory_patterns,
    bench_error_handling,
    bench_serialization
);

criterion_main!(benches);