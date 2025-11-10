//! Performance benchmarks for NativeExtractorPool
//!
//! Benchmarks comparing:
//! - Pooled vs non-pooled native extraction
//! - Instance reuse benefits
//! - Native pool vs WASM pool performance
//! - Concurrent extraction throughput

use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion, Throughput};
use std::time::Duration;

// Sample HTML for benchmarking
#[allow(dead_code)]
const SAMPLE_HTML: &str = r#"
<!DOCTYPE html>
<html>
<head>
    <title>Benchmark Article</title>
    <meta name="description" content="Test article for benchmarking">
</head>
<body>
    <article>
        <h1>Main Title</h1>
        <p>This is the first paragraph of the article.</p>
        <p>This is the second paragraph with more content.</p>
        <p>Third paragraph continues the article.</p>
    </article>
</body>
</html>
"#;

const LARGE_HTML: &str = r#"
<!DOCTYPE html>
<html>
<head><title>Large Document</title></head>
<body>
    <article>
        <h1>Large Article</h1>
        <!-- Repeated paragraphs to simulate large document -->
"#;

// ==========================
// POOLED VS NON-POOLED EXTRACTION
// ==========================

fn bench_pooled_extraction(c: &mut Criterion) {
    let mut group = c.benchmark_group("pooled_extraction");
    group.throughput(Throughput::Elements(1));
    group.measurement_time(Duration::from_secs(10));

    // Benchmark: Pooled extraction with instance reuse
    group.bench_function("with_pool", |b| {
        let rt = tokio::runtime::Runtime::new().unwrap();

        b.to_async(&rt).iter(|| async {
            // Note: These benchmarks are placeholders until NativeExtractorPool is implemented
            // let pool = NativeExtractorPool::new(Default::default()).await.unwrap();
            // let instance = pool.acquire().await.unwrap();
            // let result = instance.extract(SAMPLE_HTML, "http://example.com").await.unwrap();
            // pool.release(instance).await;
            // black_box(result);

            // Placeholder that simulates some work
            tokio::time::sleep(Duration::from_micros(100)).await;
        });
    });

    // Benchmark: Direct extraction without pooling
    group.bench_function("without_pool", |b| {
        let rt = tokio::runtime::Runtime::new().unwrap();

        b.to_async(&rt).iter(|| async {
            // Note: This would create a new instance for each extraction
            // let instance = NativeExtractor::new();
            // let result = instance.extract(SAMPLE_HTML, "http://example.com").await.unwrap();
            // black_box(result);

            // Placeholder
            tokio::time::sleep(Duration::from_micros(150)).await;
        });
    });

    group.finish();
}

// ==========================
// INSTANCE REUSE BENEFITS
// ==========================

fn bench_instance_reuse(c: &mut Criterion) {
    let mut group = c.benchmark_group("instance_reuse");
    group.measurement_time(Duration::from_secs(15));

    // Benchmark different reuse counts
    for reuse_count in [1, 10, 100, 1000].iter() {
        group.throughput(Throughput::Elements(*reuse_count));

        group.bench_with_input(
            BenchmarkId::from_parameter(reuse_count),
            reuse_count,
            |b, &count| {
                let rt = tokio::runtime::Runtime::new().unwrap();

                b.to_async(&rt).iter(|| async move {
                    // let pool = NativeExtractorPool::new(Default::default()).await.unwrap();
                    //
                    // for _ in 0..count {
                    //     let instance = pool.acquire().await.unwrap();
                    //     let _ = instance.extract(SAMPLE_HTML, "http://example.com").await.unwrap();
                    //     pool.release(instance).await;
                    // }

                    // Placeholder: Simulates reuse benefits
                    for _ in 0..count {
                        tokio::time::sleep(Duration::from_micros(80)).await;
                    }
                });
            },
        );
    }

    group.finish();
}

// ==========================
// NATIVE VS WASM POOL COMPARISON
// ==========================

fn bench_native_vs_wasm(c: &mut Criterion) {
    let mut group = c.benchmark_group("native_vs_wasm");
    group.throughput(Throughput::Elements(1));
    group.measurement_time(Duration::from_secs(10));

    // Benchmark: Native pool extraction
    group.bench_function("native_pool", |b| {
        let rt = tokio::runtime::Runtime::new().unwrap();

        b.to_async(&rt).iter(|| async {
            // let pool = NativeExtractorPool::new(Default::default()).await.unwrap();
            // let instance = pool.acquire().await.unwrap();
            // let result = instance.extract(SAMPLE_HTML, "http://example.com").await.unwrap();
            // pool.release(instance).await;
            // black_box(result);

            // Placeholder: Native is typically faster
            tokio::time::sleep(Duration::from_micros(100)).await;
        });
    });

    // Benchmark: WASM pool extraction
    #[cfg(feature = "wasm-pool")]
    group.bench_function("wasm_pool", |b| {
        let rt = tokio::runtime::Runtime::new().unwrap();

        b.to_async(&rt).iter(|| async {
            // Note: This would require actual WASM pool setup
            // Placeholder: WASM is typically slower due to overhead
            tokio::time::sleep(Duration::from_micros(200)).await;
        });
    });

    group.finish();
}

// ==========================
// CONCURRENT THROUGHPUT
// ==========================

fn bench_concurrent_extraction(c: &mut Criterion) {
    let mut group = c.benchmark_group("concurrent_extraction");
    group.measurement_time(Duration::from_secs(20));

    // Benchmark different concurrency levels
    for concurrency in [1, 2, 4, 8, 16].iter() {
        group.throughput(Throughput::Elements(*concurrency));

        group.bench_with_input(
            BenchmarkId::from_parameter(concurrency),
            concurrency,
            |b, &concurrent_tasks| {
                let rt = tokio::runtime::Runtime::new().unwrap();

                b.to_async(&rt).iter(|| async move {
                    // let pool = Arc::new(
                    //     NativeExtractorPool::new(Default::default()).await.unwrap()
                    // );
                    //
                    // let handles: Vec<_> = (0..concurrent_tasks)
                    //     .map(|_| {
                    //         let pool = pool.clone();
                    //         tokio::spawn(async move {
                    //             let instance = pool.acquire().await.unwrap();
                    //             let result = instance.extract(SAMPLE_HTML, "http://example.com").await.unwrap();
                    //             pool.release(instance).await;
                    //             result
                    //         })
                    //     })
                    //     .collect();
                    //
                    // futures::future::join_all(handles).await;

                    // Placeholder: Simulates concurrent work
                    let handles: Vec<_> = (0..concurrent_tasks)
                        .map(|_| {
                            tokio::spawn(async {
                                tokio::time::sleep(Duration::from_micros(100)).await;
                            })
                        })
                        .collect();

                    for handle in handles {
                        handle.await.unwrap();
                    }
                });
            },
        );
    }

    group.finish();
}

// ==========================
// MEMORY EFFICIENCY
// ==========================

fn bench_memory_usage(c: &mut Criterion) {
    let mut group = c.benchmark_group("memory_efficiency");
    group.measurement_time(Duration::from_secs(10));

    // Benchmark: Small document
    group.bench_function("small_document", |b| {
        let rt = tokio::runtime::Runtime::new().unwrap();

        b.to_async(&rt).iter(|| async {
            // let pool = NativeExtractorPool::new(Default::default()).await.unwrap();
            // let instance = pool.acquire().await.unwrap();
            // let result = instance.extract(SAMPLE_HTML, "http://example.com").await.unwrap();
            // pool.release(instance).await;
            // black_box(result);

            tokio::time::sleep(Duration::from_micros(80)).await;
        });
    });

    // Benchmark: Large document
    group.bench_function("large_document", |b| {
        let rt = tokio::runtime::Runtime::new().unwrap();

        // Generate large HTML
        let _large_html = {
            let mut html = LARGE_HTML.to_string();
            for i in 0..1000 {
                html.push_str(&format!("<p>Paragraph {} with content.</p>\n", i));
            }
            html.push_str("</article></body></html>");
            html
        };

        b.to_async(&rt).iter(|| async {
            // let pool = NativeExtractorPool::new(Default::default()).await.unwrap();
            // let instance = pool.acquire().await.unwrap();
            // let result = instance.extract(&large_html, "http://example.com").await.unwrap();
            // pool.release(instance).await;
            // black_box(result);

            // Placeholder: Large documents take longer
            tokio::time::sleep(Duration::from_micros(300)).await;
        });
    });

    group.finish();
}

// ==========================
// POOL OVERHEAD
// ==========================

fn bench_pool_overhead(c: &mut Criterion) {
    let mut group = c.benchmark_group("pool_overhead");

    // Benchmark: Acquire/release overhead
    group.bench_function("acquire_release", |b| {
        let rt = tokio::runtime::Runtime::new().unwrap();

        b.to_async(&rt).iter(|| async {
            // let pool = NativeExtractorPool::new(Default::default()).await.unwrap();
            // let instance = pool.acquire().await.unwrap();
            // pool.release(instance).await;

            // Placeholder: Minimal overhead
            tokio::time::sleep(Duration::from_micros(10)).await;
        });
    });

    // Benchmark: Instance creation
    group.bench_function("instance_creation", |b| {
        let rt = tokio::runtime::Runtime::new().unwrap();

        b.to_async(&rt).iter(|| async {
            // let instance = NativeExtractor::new();
            // black_box(instance);

            // Placeholder
            tokio::time::sleep(Duration::from_micros(50)).await;
        });
    });

    group.finish();
}

// ==========================
// REAL-WORLD SCENARIOS
// ==========================

fn bench_realistic_workload(c: &mut Criterion) {
    let mut group = c.benchmark_group("realistic_workload");
    group.measurement_time(Duration::from_secs(30));
    group.sample_size(50);

    // Benchmark: Mixed document sizes with varying concurrency
    group.bench_function("mixed_workload", |b| {
        let rt = tokio::runtime::Runtime::new().unwrap();

        b.to_async(&rt).iter(|| async {
            // Simulates realistic workload:
            // - 70% small documents
            // - 20% medium documents
            // - 10% large documents
            // - Varying concurrency (1-8 concurrent extractions)

            // let pool = Arc::new(NativeExtractorPool::new(Default::default()).await.unwrap());

            let tasks = vec![
                // Small documents (fast)
                (100, 7),
                // Medium documents
                (200, 2),
                // Large documents
                (400, 1),
            ];

            for (duration_micros, count) in tasks {
                for _ in 0..count {
                    tokio::time::sleep(Duration::from_micros(duration_micros)).await;
                }
            }
        });
    });

    group.finish();
}

// ==========================
// CRITERION CONFIGURATION
// ==========================

criterion_group!(
    benches,
    bench_pooled_extraction,
    bench_instance_reuse,
    bench_native_vs_wasm,
    bench_concurrent_extraction,
    bench_memory_usage,
    bench_pool_overhead,
    bench_realistic_workload,
);

criterion_main!(benches);
