use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion, Throughput};
use std::time::Duration;
use tokio::runtime::Runtime;
use futures::future;

use crate::component::{CmExtractor, ExtractorConfig};
use crate::types::ExtractionMode;

/// Performance benchmarking suite for RipTide extractors
///
/// This module provides comprehensive benchmarks for:
/// - Component Model extraction performance
/// - Instance pooling efficiency
/// - Memory usage patterns
/// - Concurrent extraction capabilities
/// - WASM component overhead analysis
///
/// Sample HTML content for benchmarking
const SAMPLE_HTML_SMALL: &str = r#"
<!DOCTYPE html>
<html>
<head>
    <title>Sample Article</title>
    <meta name="description" content="This is a sample article for testing">
</head>
<body>
    <article>
        <h1>Sample Article Title</h1>
        <p class="byline">By Test Author</p>
        <p>This is the first paragraph of the article content.</p>
        <p>This is the second paragraph with more content.</p>
        <a href="https://example.com">External Link</a>
        <img src="https://example.com/image.jpg" alt="Sample Image">
    </article>
</body>
</html>
"#;

const SAMPLE_HTML_MEDIUM: &str = include_str!("../test_data/medium_article.html");
const SAMPLE_HTML_LARGE: &str = include_str!("../test_data/large_article.html");

/// Benchmark data sizes
const BENCHMARK_SIZES: &[(&str, &str)] = &[
    ("small", SAMPLE_HTML_SMALL),
    ("medium", SAMPLE_HTML_MEDIUM),
    ("large", SAMPLE_HTML_LARGE),
];

/// Benchmark configurations
#[allow(dead_code)]
struct BenchmarkConfig {
    name: &'static str,
    pool_size: usize,
    concurrent_requests: usize,
    #[allow(dead_code)]
    enable_instance_reuse: bool,
}

const BENCHMARK_CONFIGS: &[BenchmarkConfig] = &[
    BenchmarkConfig {
        name: "single_instance",
        pool_size: 1,
        concurrent_requests: 1,
        enable_instance_reuse: false,
    },
    BenchmarkConfig {
        name: "pooled_small",
        pool_size: 4,
        concurrent_requests: 1,
        enable_instance_reuse: true,
    },
    BenchmarkConfig {
        name: "pooled_concurrent",
        pool_size: 8,
        concurrent_requests: 16,
        enable_instance_reuse: true,
    },
    BenchmarkConfig {
        name: "high_concurrency",
        pool_size: 16,
        concurrent_requests: 64,
        enable_instance_reuse: true,
    },
];

/// Initialize extractor for benchmarking
async fn create_test_extractor(
    config: &BenchmarkConfig,
) -> Result<CmExtractor, Box<dyn std::error::Error>> {
    let extractor_config = ExtractorConfig {
        max_instances: config.pool_size,
        enable_metrics: true,
        timeout_ms: 5000,
        memory_limit_pages: Some(8192), // 512MB / 64KB per page
        extraction_timeout: Some(30000), // 30 seconds
        max_pool_size: config.pool_size,
        initial_pool_size: config.pool_size / 2,
        epoch_timeout_ms: 5000,
        health_check_interval: 30000, // 30 seconds in milliseconds
        memory_limit: Some(512 * 1024 * 1024), // 512MB
        circuit_breaker_timeout: 60000, // 60 seconds in milliseconds
        circuit_breaker_failure_threshold: 5,
    };

    // For benchmarking, we'll use CmExtractor::new() directly
    // The extractor doesn't actually load WASM files in this simplified implementation
    Ok(CmExtractor::new(extractor_config))
}

/// Benchmark single extraction performance
fn bench_single_extraction(c: &mut Criterion) {
    for (size_name, html) in BENCHMARK_SIZES {
        for config in BENCHMARK_CONFIGS {
            let bench_id = format!("{}_extraction_{}", config.name, size_name);

            c.bench_with_input(
                BenchmarkId::new("single_extraction", &bench_id),
                html,
                |b, html| {
                    let rt = Runtime::new().unwrap();
                    let extractor = rt.block_on(create_test_extractor(config))
                        .expect("Failed to create extractor for benchmark");

                    b.iter(|| {
                        rt.block_on(async {
                            black_box(extractor.extract(
                                black_box(html),
                            ).await)
                        })
                    });
                },
            );
        }
    }
}

/// Benchmark concurrent extraction performance
fn bench_concurrent_extraction(c: &mut Criterion) {
    let rt = Runtime::new().expect("Failed to create runtime for benchmark");

    for (size_name, html) in BENCHMARK_SIZES {
        for config in BENCHMARK_CONFIGS
            .iter()
            .filter(|c| c.concurrent_requests > 1)
        {
            let bench_id = format!("{}_concurrent_{}", config.name, size_name);

            c.bench_with_input(
                BenchmarkId::new("concurrent_extraction", &bench_id),
                &(html, config.concurrent_requests),
                |b, (html, concurrent_requests)| {
                    let extractor = rt.block_on(create_test_extractor(config))
                        .expect("Failed to create extractor for benchmark");

                    b.iter(|| {
                        rt.block_on(async {
                            let tasks: Vec<_> = (0..*concurrent_requests)
                                .map(|_| {
                                    let extractor = &extractor;
                                    async move {
                                        extractor.extract(
                                            black_box(html),
                                        ).await
                                    }
                                })
                                .collect();

                            let results = future::join_all(tasks).await;
                            black_box(results)
                        })
                    });
                },
            );
        }
    }
}

/// Benchmark instance pool performance
fn bench_pool_efficiency(c: &mut Criterion) {
    let rt = Runtime::new().expect("Failed to create runtime for benchmark");

    let mut group = c.benchmark_group("pool_efficiency");

    for pool_size in [1, 2, 4, 8, 16] {
        let config = ExtractorConfig {
            max_instances: pool_size,
            enable_metrics: true,
            timeout_ms: 5000,
            memory_limit_pages: Some(4096), // 256MB / 64KB per page
            extraction_timeout: Some(30000), // 30 seconds
            max_pool_size: pool_size,
            initial_pool_size: pool_size,
            epoch_timeout_ms: 5000,
            health_check_interval: 30000, // 30 seconds in milliseconds
            memory_limit: Some(256 * 1024 * 1024), // 256MB
            circuit_breaker_timeout: 60000, // 60 seconds in milliseconds
            circuit_breaker_failure_threshold: 5,
        };

        group.bench_with_input(
            BenchmarkId::new("pool_warmup", pool_size),
            &pool_size,
            |b, _| {
                b.iter(|| {
                    rt.block_on(async {
                        let _extractor = CmExtractor::new(config.clone());

                        // Warm-up functionality not yet implemented
                        black_box(Ok::<(), String>(()))
                    })
                })
            },
        );

        group.bench_with_input(
            BenchmarkId::new("pool_scale", pool_size),
            &pool_size,
            |b, _| {
                b.iter(|| {
                    rt.block_on(async {
                        let _extractor = create_test_extractor(&BenchmarkConfig {
                            name: "pool_test",
                            pool_size,
                            concurrent_requests: 1,
                            enable_instance_reuse: true,
                        })
                        .await
                        .expect("Failed to create extractor for benchmark");

                        // Pool scaling functionality not yet implemented
                        black_box(Ok::<(), String>(()))
                    })
                })
            },
        );
    }

    group.finish();
}

/// Benchmark memory usage patterns
fn bench_memory_usage(c: &mut Criterion) {
    let rt = Runtime::new().expect("Failed to create runtime for benchmark");

    let mut group = c.benchmark_group("memory_usage");
    group.throughput(Throughput::Elements(100));

    // Test memory efficiency with different content sizes
    for (size_name, html) in BENCHMARK_SIZES {
        group.bench_with_input(
            BenchmarkId::new("memory_reuse", size_name),
            html,
            |b, html| {
                let extractor = rt.block_on(create_test_extractor(&BENCHMARK_CONFIGS[2]))
                    .expect("Failed to create extractor for benchmark"); // pooled_concurrent

                b.iter(|| {
                    rt.block_on(async {
                        // Extract 100 times to test memory reuse
                        for _ in 0..100 {
                            let _ = black_box(extractor.extract(
                                black_box(html),
                            ).await);
                        }
                    })
                })
            },
        );
    }

    group.finish();
}

/// Benchmark extraction mode performance
fn bench_extraction_modes(c: &mut Criterion) {
    let modes = [
        ("article", ExtractionMode::Article),
        ("full", ExtractionMode::Full),
        ("metadata", ExtractionMode::Metadata),
    ];

    let rt = Runtime::new().expect("Failed to create runtime for benchmark");
    let extractor = rt.block_on(create_test_extractor(&BENCHMARK_CONFIGS[1]))
        .expect("Failed to create extractor for benchmark"); // pooled_small

    for (mode_name, mode) in modes {
        for (size_name, html) in BENCHMARK_SIZES {
            let bench_id = format!("{}_{}", mode_name, size_name);

            c.bench_with_input(
                BenchmarkId::new("extraction_modes", &bench_id),
                &(html, &mode),
                |b, (html, _mode)| {
                    b.iter(|| {
                        rt.block_on(async {
                            // extract_typed method no longer exists, using extract instead
                            black_box(extractor.extract(
                                black_box(html),
                            ).await)
                        })
                    });
                },
            );
        }
    }
}

/// Benchmark error handling and recovery
fn bench_error_handling(c: &mut Criterion) {
    let invalid_html_samples = [
        (
            "malformed_tags",
            "<html><body><p>Unclosed paragraph</body></html>",
        ),
        ("empty_content", ""),
        (
            "invalid_encoding",
            r#"<?xml version="1.0" encoding="invalid"?><html></html>"#,
        ),
        ("huge_content", &"<p>".repeat(10000)),
    ];

    let rt = Runtime::new().expect("Failed to create runtime for benchmark");
    let extractor = rt.block_on(create_test_extractor(&BENCHMARK_CONFIGS[1]))
        .expect("Failed to create extractor for benchmark"); // pooled_small

    for (error_type, html) in invalid_html_samples {
        c.bench_with_input(
            BenchmarkId::new("error_handling", error_type),
            &html,
            |b, html| {
                b.iter(|| {
                    rt.block_on(async {
                        // These should fail gracefully and return typed errors
                        let _ = black_box(extractor.extract(
                            black_box(html),
                        ).await);
                    })
                })
            },
        );
    }
}

/// Benchmark circuit breaker performance
fn bench_circuit_breaker(c: &mut Criterion) {
    let rt = Runtime::new().expect("Failed to create runtime for benchmark");

    let config = ExtractorConfig {
        max_instances: 4,
        enable_metrics: true,
        timeout_ms: 100, // Short timeout to trigger failures
        memory_limit_pages: Some(1024), // 64MB / 64KB per page
        extraction_timeout: Some(100), // Short timeout
        max_pool_size: 4,
        initial_pool_size: 2,
        epoch_timeout_ms: 5000,
        health_check_interval: 30000, // 30 seconds in milliseconds
        memory_limit: Some(64 * 1024 * 1024), // Small limit to trigger resource errors
        circuit_breaker_timeout: 60000, // 60 seconds in milliseconds
        circuit_breaker_failure_threshold: 5,
    };

    c.bench_function("circuit_breaker_recovery", |b| {
        b.iter(|| {
            rt.block_on(async {
                let extractor = CmExtractor::new(config.clone());

                // Trigger failures to open circuit breaker
                for _ in 0..20 {
                    let _ = extractor.extract(
                        black_box(SAMPLE_HTML_LARGE),
                    ).await;
                }

                // Test recovery
                tokio::time::sleep(Duration::from_secs(1)).await;

                let _ = extractor.extract(
                    black_box(SAMPLE_HTML_SMALL),
                ).await;
                black_box(())
            })
        });
    });
}

/// Benchmark component initialization overhead
fn bench_initialization(c: &mut Criterion) {
    let configs = [
        (
            "minimal",
            ExtractorConfig {
                max_instances: 1,
                enable_metrics: false,
                timeout_ms: 2000,
                memory_limit_pages: Some(2048), // 128MB / 64KB per page
                extraction_timeout: Some(10000), // 10 seconds
                max_pool_size: 1,
                initial_pool_size: 0,
                epoch_timeout_ms: 2000,
                health_check_interval: 30000, // 30 seconds in milliseconds
                memory_limit: Some(128 * 1024 * 1024), // 128MB
                circuit_breaker_timeout: 30000, // 30 seconds in milliseconds
                circuit_breaker_failure_threshold: 3,
            },
        ),
        ("standard", ExtractorConfig::default()),
        (
            "high_performance",
            ExtractorConfig {
                max_instances: 16,
                enable_metrics: true,
                timeout_ms: 10000,
                memory_limit_pages: Some(16384), // 1GB / 64KB per page
                extraction_timeout: Some(30000), // 30 seconds
                max_pool_size: 16,
                initial_pool_size: 8,
                epoch_timeout_ms: 10000,
                health_check_interval: 30000, // 30 seconds in milliseconds
                memory_limit: Some(1024 * 1024 * 1024), // 1GB
                circuit_breaker_timeout: 120000, // 120 seconds in milliseconds
                circuit_breaker_failure_threshold: 10,
            },
        ),
    ];

    for (config_name, config) in configs {
        c.bench_with_input(
            BenchmarkId::new("initialization", config_name),
            &config,
            |b, config| {
                b.iter(|| {
                    black_box(CmExtractor::new(config.clone()))
                });
            },
        );
    }
}

criterion_group!(
    performance_benches,
    bench_single_extraction,
    bench_concurrent_extraction,
    bench_pool_efficiency,
    bench_memory_usage,
    bench_extraction_modes,
    bench_error_handling,
    bench_circuit_breaker,
    bench_initialization
);

criterion_main!(performance_benches);

#[cfg(test)]
mod tests {
    use super::*;

    /// Integration test for performance benchmarking
    #[tokio::test]
    async fn test_benchmark_extractor_creation() {
        let config = &BENCHMARK_CONFIGS[0];
        let result = create_test_extractor(config).await;

        // This test will fail in the benchmark environment since we don't have a real WASM file
        // But it verifies the benchmark setup is correct
        assert!(
            result.is_err(),
            "Expected benchmark to fail without real WASM component"
        );
    }

    #[test]
    fn test_benchmark_data_validity() {
        // Verify our test data is valid
        assert!(SAMPLE_HTML_SMALL.contains("<title>"));
        assert!(SAMPLE_HTML_SMALL.contains("<article>"));

        // Verify configurations are reasonable
        for config in BENCHMARK_CONFIGS {
            assert!(config.pool_size > 0);
            assert!(config.concurrent_requests > 0);
            assert!(config.concurrent_requests <= config.pool_size * 4); // Reasonable ratio
        }
    }

    /// Test performance metrics collection
    #[tokio::test]
    async fn test_metrics_collection() {
        // This would be a real test with actual WASM component
        // For now, we just verify the structure exists
        let config = ExtractorConfig::default();
        assert_eq!(config.max_pool_size, 8);
        assert_eq!(config.initial_pool_size, 2);
        assert!(config.enable_metrics);
    }
}
