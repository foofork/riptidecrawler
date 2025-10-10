//! Benchmarks for core extraction strategies and performance
//!
//! Note: CSS/Regex extraction and chunking features have been moved to riptide-html crate.
//! This benchmark focuses on the core strategy manager and metadata extraction.

use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion, Throughput};
use riptide_core::strategies::*;
use tokio::runtime::Runtime;

fn create_test_content(size: usize) -> String {
    let base_html = r#"
    <html>
    <head>
        <title>Benchmark Test Article</title>
        <meta name="description" content="Performance benchmarking test content">
        <meta name="author" content="Benchmark Author">
        <meta property="og:title" content="OG Benchmark Article">
        <meta property="og:description" content="OpenGraph description for benchmarking">
        <meta property="article:published_time" content="2023-12-01T10:00:00Z">
    </head>
    <body>
        <article>
            <h1>Benchmark Test Article</h1>
            <div class="author">Benchmark Author</div>
            <time datetime="2023-12-01">December 1, 2023</time>
            <div class="content">
    "#;

    let end_html = r#"
            </div>
        </article>
        <script type="application/ld+json">
        {
            "@context": "https://schema.org",
            "@type": "Article",
            "headline": "JSON-LD Benchmark Article",
            "author": {"@type": "Person", "name": "JSON-LD Author"},
            "datePublished": "2023-12-01T10:00:00Z"
        }
        </script>
    </body>
    </html>
    "#;

    let mut content = String::from(base_html);
    let paragraph = "<p>This is a benchmark paragraph containing substantial content for performance testing. It includes enough text to provide meaningful performance metrics while representing realistic content extraction scenarios. The paragraph contains various HTML elements and text patterns that extraction strategies need to process efficiently.</p>\n";

    while content.len() + end_html.len() < size {
        content.push_str(paragraph);
    }

    content.push_str(end_html);
    content
}

fn bench_core_extraction(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    let sizes = vec![1024, 10240, 102400]; // 1KB, 10KB, 100KB

    let mut group = c.benchmark_group("core_extraction");

    for size in sizes {
        let content = create_test_content(size);
        group.throughput(Throughput::Bytes(content.len() as u64));

        // Strategy manager extraction benchmark (core implementation)
        group.bench_with_input(
            BenchmarkId::new("strategy_manager_extract", size),
            &content,
            |b, content| {
                b.iter(|| {
                    rt.block_on(async {
                        let config = StrategyConfig::default();
                        let mut manager = StrategyManager::new(config);
                        black_box(
                            manager
                                .extract_content(black_box(content), "http://example.com")
                                .await
                                .unwrap(),
                        )
                    })
                })
            },
        );
    }

    group.finish();
}

fn bench_metadata_extraction(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    let sizes = vec![1024, 10240, 102400];

    let mut group = c.benchmark_group("metadata_extraction");

    for size in sizes {
        let content = create_test_content(size);
        group.throughput(Throughput::Bytes(content.len() as u64));

        group.bench_with_input(
            BenchmarkId::new("metadata", size),
            &content,
            |b, content| {
                b.iter(|| {
                    rt.block_on(async {
                        black_box(
                            metadata::extract_metadata(black_box(content), "http://example.com")
                                .await
                                .unwrap(),
                        )
                    })
                })
            },
        );
    }

    group.finish();
}

fn bench_strategy_manager(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    let sizes = vec![("small", 1024), ("medium", 10240), ("large", 102400)];

    let mut group = c.benchmark_group("strategy_manager");

    for (name, size) in sizes {
        let content = create_test_content(size);
        group.throughput(Throughput::Bytes(content.len() as u64));

        group.bench_with_input(BenchmarkId::new("process", name), &content, |b, content| {
            b.iter(|| {
                rt.block_on(async {
                    let config = StrategyConfig::default();
                    let mut manager = StrategyManager::new(config);
                    black_box(
                        manager
                            .extract_content(black_box(content), "http://example.com")
                            .await
                            .unwrap(),
                    )
                })
            })
        });
    }

    group.finish();
}

fn bench_performance_metrics(c: &mut Criterion) {
    let mut group = c.benchmark_group("performance_metrics");

    // Create metrics with sample data
    let mut metrics = PerformanceMetrics::new();
    for i in 0..1000 {
        metrics.record_extraction(
            &ExtractionStrategy::Trek,
            std::time::Duration::from_millis(10 + (i % 20)),
            (1000 + (i * 10)) as usize,
            (5 + (i % 10)) as usize,
        );
    }

    group.bench_function("get_summary", |b| {
        b.iter(|| black_box(metrics.get_summary()))
    });

    group.bench_function("record_extraction", |b| {
        b.iter(|| {
            metrics.record_extraction(
                &ExtractionStrategy::Trek,
                std::time::Duration::from_millis(15),
                2000,
                8,
            )
        })
    });

    group.finish();
}

criterion_group!(
    benches,
    bench_core_extraction,
    bench_metadata_extraction,
    bench_strategy_manager,
    bench_performance_metrics
);
criterion_main!(benches);
