/// Performance Benchmark Template
///
/// This template shows how to write performance benchmarks using criterion.
/// Benchmarks should be in `benches/` directory with [[bench]] in Cargo.toml.

use criterion::{black_box, criterion_group, criterion_main, Criterion, BenchmarkId};
use std::time::Duration;

/// Basic benchmark: Single function performance
fn benchmark_basic_operation(c: &mut Criterion) {
    c.bench_function("parse_url", |b| {
        b.iter(|| {
            parse_url(black_box("https://example.com/path?query=value"))
        });
    });
}

/// Parametrized benchmark: Test with different input sizes
fn benchmark_crawl_scaling(c: &mut Criterion) {
    let mut group = c.benchmark_group("crawl_scaling");

    for num_urls in [10, 50, 100, 500, 1000].iter() {
        group.bench_with_input(
            BenchmarkId::from_parameter(num_urls),
            num_urls,
            |b, &num_urls| {
                let urls = generate_test_urls(num_urls);
                b.iter(|| {
                    crawl_urls(black_box(&urls))
                });
            },
        );
    }

    group.finish();
}

/// Async benchmark: Measure async operation performance
fn benchmark_async_fetch(c: &mut Criterion) {
    let runtime = tokio::runtime::Runtime::new().unwrap();

    c.bench_function("async_fetch_page", |b| {
        b.to_async(&runtime).iter(|| async {
            let client = FetchClient::new();
            client.get(black_box("https://example.com")).await
        });
    });
}

/// Throughput benchmark: Measure operations per second
fn benchmark_throughput(c: &mut Criterion) {
    let mut group = c.benchmark_group("throughput");
    group.throughput(criterion::Throughput::Elements(1000));

    group.bench_function("process_1000_urls", |b| {
        let urls = generate_test_urls(1000);
        b.iter(|| {
            process_batch(black_box(&urls))
        });
    });

    group.finish();
}

/// Memory benchmark: Measure memory allocation
fn benchmark_memory_usage(c: &mut Criterion) {
    let mut group = c.benchmark_group("memory");
    group.measurement_time(Duration::from_secs(10));

    group.bench_function("large_crawl_memory", |b| {
        b.iter(|| {
            let crawler = Crawler::new();
            let urls = generate_test_urls(10000);

            // Force allocation
            let results = crawler.crawl_all(black_box(&urls));

            // Ensure compiler doesn't optimize away
            black_box(results);
        });
    });

    group.finish();
}

/// Comparison benchmark: Compare different implementations
fn benchmark_spider_vs_chrome(c: &mut Criterion) {
    let mut group = c.benchmark_group("crawl_methods");

    let test_url = "https://example.com";

    group.bench_function("spider_crawl", |b| {
        let spider = SpiderCrawler::new();
        b.iter(|| {
            spider.crawl(black_box(test_url))
        });
    });

    group.bench_function("chrome_crawl", |b| {
        let chrome = ChromeCrawler::new();
        b.iter(|| {
            chrome.crawl(black_box(test_url))
        });
    });

    group.bench_function("hybrid_crawl", |b| {
        let hybrid = HybridCrawler::new();
        b.iter(|| {
            hybrid.crawl(black_box(test_url))
        });
    });

    group.finish();
}

/// Regression benchmark: Detect performance regressions
fn benchmark_regression_baseline(c: &mut Criterion) {
    let mut group = c.benchmark_group("regression_baseline");

    // Set strict thresholds for regression detection
    group.significance_level(0.05);
    group.noise_threshold(0.02);
    group.sample_size(100);

    group.bench_function("critical_path_performance", |b| {
        let input = setup_benchmark_input();
        b.iter(|| {
            critical_operation(black_box(&input))
        });
    });

    group.finish();
}

/// Concurrency benchmark: Test parallel execution
fn benchmark_concurrent_crawls(c: &mut Criterion) {
    let runtime = tokio::runtime::Runtime::new().unwrap();
    let mut group = c.benchmark_group("concurrency");

    for num_concurrent in [1, 5, 10, 20, 50].iter() {
        group.bench_with_input(
            BenchmarkId::from_parameter(num_concurrent),
            num_concurrent,
            |b, &num_concurrent| {
                b.to_async(&runtime).iter(|| async move {
                    let crawler = Crawler::new();
                    let urls = generate_test_urls(num_concurrent);

                    // Spawn concurrent tasks
                    let futures: Vec<_> = urls
                        .iter()
                        .map(|url| crawler.crawl(url))
                        .collect();

                    // Wait for all to complete
                    futures::future::join_all(futures).await
                });
            },
        );
    }

    group.finish();
}

/// Custom measurement: Measure specific metrics
fn benchmark_custom_metrics(c: &mut Criterion) {
    use criterion::measurement::WallTime;

    let mut group = c.benchmark_group("custom_metrics");

    group.bench_function("memory_allocations", |b| {
        b.iter_custom(|iters| {
            let start = std::time::Instant::now();
            let start_allocs = get_allocation_count();

            for _ in 0..iters {
                let _ = black_box(allocate_and_process());
            }

            let end_allocs = get_allocation_count();
            let elapsed = start.elapsed();

            // Log custom metrics
            println!("Allocations: {}", end_allocs - start_allocs);

            elapsed
        });
    });

    group.finish();
}

/// Helper functions for benchmarks

fn generate_test_urls(count: usize) -> Vec<String> {
    (0..count)
        .map(|i| format!("https://example.com/page{}", i))
        .collect()
}

fn setup_benchmark_input() -> BenchmarkInput {
    BenchmarkInput {
        urls: generate_test_urls(100),
        depth: 2,
        config: CrawlConfig::default(),
    }
}

fn get_allocation_count() -> usize {
    // Platform-specific memory tracking
    #[cfg(target_os = "linux")]
    {
        use std::fs;
        let status = fs::read_to_string("/proc/self/status").unwrap();
        status
            .lines()
            .find(|line| line.starts_with("VmRSS:"))
            .and_then(|line| line.split_whitespace().nth(1))
            .and_then(|s| s.parse().ok())
            .unwrap_or(0)
    }

    #[cfg(not(target_os = "linux"))]
    {
        0 // Fallback for other platforms
    }
}

// Benchmark group configuration
criterion_group! {
    name = benches;
    config = Criterion::default()
        .measurement_time(Duration::from_secs(10))
        .sample_size(100)
        .warm_up_time(Duration::from_secs(3));
    targets =
        benchmark_basic_operation,
        benchmark_crawl_scaling,
        benchmark_async_fetch,
        benchmark_throughput,
        benchmark_memory_usage,
        benchmark_spider_vs_chrome,
        benchmark_regression_baseline,
        benchmark_concurrent_crawls,
        benchmark_custom_metrics
}

criterion_main!(benches);

/// Add to Cargo.toml:
/// ```toml
/// [[bench]]
/// name = "performance_benchmarks"
/// harness = false
/// ```
///
/// Run benchmarks:
/// ```bash
/// cargo bench
/// cargo bench -- --save-baseline main
/// cargo bench -- --baseline main  # Compare against baseline
/// ```
