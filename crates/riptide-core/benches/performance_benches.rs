use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion, Throughput};
use std::time::Duration;
use tokio::runtime::Runtime;

use riptide_core::monitoring::{MetricsCollector, TimeSeriesBuffer};

/// Performance benchmarks for the monitoring and metrics system
/// This file contains Criterion benchmarks that can be run with:
/// cargo bench --features benchmarks

fn bench_metrics_collection(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();

    let mut group = c.benchmark_group("metrics_collection");
    group.throughput(Throughput::Elements(1));

    // Benchmark single metric recording
    group.bench_function("record_extraction", |b| {
        let collector = MetricsCollector::new();

        b.to_async(&rt).iter(|| async {
            black_box(
                collector
                    .record_extraction(
                        black_box(Duration::from_millis(100)),
                        black_box(true),
                        black_box(Some(85)),
                        black_box(Some(500)),
                        black_box(false),
                    )
                    .await,
            )
        });
    });

    // Benchmark batch metric recording
    for batch_size in [10, 100, 1000].iter() {
        group.bench_with_input(
            BenchmarkId::new("batch_record_extractions", batch_size),
            batch_size,
            |b, &batch_size| {
                let collector = MetricsCollector::new();

                b.to_async(&rt).iter(|| async {
                    for _ in 0..batch_size {
                        black_box(
                            collector
                                .record_extraction(
                                    black_box(Duration::from_millis(100)),
                                    black_box(true),
                                    black_box(Some(85)),
                                    black_box(Some(500)),
                                    black_box(false),
                                )
                                .await,
                        );
                    }
                });
            },
        );
    }

    group.finish();
}

fn bench_time_series_operations(c: &mut Criterion) {
    let mut group = c.benchmark_group("time_series");

    // Benchmark time series buffer operations
    group.bench_function("add_data_point", |b| {
        b.iter(|| {
            let mut buffer = TimeSeriesBuffer::new(1000, Duration::from_hours(1));

            for i in 0..100 {
                black_box(buffer.add_point(
                    black_box(i as f64),
                    black_box(std::collections::HashMap::new()),
                ));
            }
        });
    });

    // Benchmark percentile calculations
    group.bench_function("calculate_percentile", |b| {
        let mut buffer = TimeSeriesBuffer::new(1000, Duration::from_hours(1));

        // Pre-populate buffer with data
        for i in 0..1000 {
            buffer.add_point(i as f64, std::collections::HashMap::new());
        }

        b.iter(|| {
            black_box(
                buffer.calculate_percentile(black_box(95.0), black_box(Duration::from_minutes(5))),
            )
        });
    });

    group.finish();
}

fn bench_performance_report_generation(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();

    c.bench_function("generate_performance_report", |b| {
        let collector = MetricsCollector::new();

        // Pre-populate with data
        rt.block_on(async {
            for i in 0..1000 {
                collector
                    .record_extraction(
                        Duration::from_millis(100 + i % 100),
                        i % 10 != 0, // 90% success rate
                        Some(80 + (i % 20) as u8),
                        Some(500 + i * 10),
                        i % 3 == 0, // 33% cache hit rate
                    )
                    .await;
            }
        });

        b.to_async(&rt).iter(|| async {
            black_box(
                collector
                    .get_performance_report(black_box(Duration::from_minutes(5)))
                    .await,
            )
        });
    });
}

fn bench_concurrent_metrics_collection(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();

    let mut group = c.benchmark_group("concurrent_metrics");

    for thread_count in [1, 4, 8, 16].iter() {
        group.bench_with_input(
            BenchmarkId::new("concurrent_record_extractions", thread_count),
            thread_count,
            |b, &thread_count| {
                let collector = MetricsCollector::new();

                b.to_async(&rt).iter(|| async {
                    let tasks: Vec<_> = (0..thread_count)
                        .map(|_| {
                            let collector = &collector;
                            async move {
                                for _ in 0..100 {
                                    collector
                                        .record_extraction(
                                            Duration::from_millis(100),
                                            true,
                                            Some(85),
                                            Some(500),
                                            false,
                                        )
                                        .await;
                                }
                            }
                        })
                        .collect();

                    futures::future::join_all(tasks).await;
                });
            },
        );
    }

    group.finish();
}

fn bench_memory_usage_patterns(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();

    let mut group = c.benchmark_group("memory_patterns");
    group.throughput(Throughput::Elements(1000));

    // Benchmark memory efficiency with different retention periods
    for retention_hours in [1, 6, 24].iter() {
        group.bench_with_input(
            BenchmarkId::new("memory_retention", retention_hours),
            retention_hours,
            |b, &retention_hours| {
                b.iter(|| {
                    let mut buffer =
                        TimeSeriesBuffer::new(10000, Duration::from_hours(retention_hours));

                    // Simulate continuous data collection
                    for i in 0..1000 {
                        black_box(buffer.add_point(
                            black_box(i as f64),
                            black_box(std::collections::HashMap::new()),
                        ));
                    }
                });
            },
        );
    }

    group.finish();
}

fn bench_alert_processing(c: &mut Criterion) {
    use riptide_core::monitoring::{AlertManager, PerformanceMetrics};

    let rt = Runtime::new().unwrap();

    c.bench_function("alert_processing", |b| {
        let mut alert_manager = AlertManager::new();

        let test_metrics = PerformanceMetrics {
            error_rate: 15.0, // This will trigger alerts
            cpu_usage_percent: 95.0,
            health_score: 40.0,
            p99_extraction_time_ms: 12000.0,
            ..Default::default()
        };

        b.to_async(&rt).iter(|| async {
            black_box(alert_manager.check_alerts(black_box(&test_metrics)).await)
        });
    });
}

fn bench_metrics_serialization(c: &mut Criterion) {
    use riptide_core::monitoring::PerformanceMetrics;

    let test_metrics = PerformanceMetrics {
        avg_extraction_time_ms: 150.5,
        p95_extraction_time_ms: 300.0,
        p99_extraction_time_ms: 500.0,
        requests_per_second: 1000.0,
        successful_extractions: 950000,
        failed_extractions: 50000,
        total_extractions: 1000000,
        memory_usage_bytes: 512 * 1024 * 1024,
        cpu_usage_percent: 75.5,
        pool_size: 8,
        active_instances: 6,
        idle_instances: 2,
        avg_content_quality_score: 85.5,
        avg_extracted_word_count: 750.0,
        cache_hit_ratio: 0.75,
        error_rate: 5.0,
        timeout_rate: 1.5,
        circuit_breaker_trips: 3,
        health_score: 85.0,
        uptime_seconds: 86400,
        last_updated: std::time::Instant::now(),
    };

    let mut group = c.benchmark_group("serialization");

    group.bench_function("serialize_json", |b| {
        b.iter(|| black_box(serde_json::to_string(black_box(&test_metrics))));
    });

    group.bench_function("deserialize_json", |b| {
        let json_data = serde_json::to_string(&test_metrics).unwrap();

        b.iter(|| {
            black_box(serde_json::from_str::<PerformanceMetrics>(black_box(
                &json_data,
            )))
        });
    });

    group.finish();
}

criterion_group!(
    performance_benches,
    bench_metrics_collection,
    bench_time_series_operations,
    bench_performance_report_generation,
    bench_concurrent_metrics_collection,
    bench_memory_usage_patterns,
    bench_alert_processing,
    bench_metrics_serialization
);

criterion_main!(performance_benches);
