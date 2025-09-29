use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion, Throughput};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;
use tokio::runtime::Runtime;
use opentelemetry::trace::Tracer;

use riptide_core::monitoring::{MetricsCollector, TimeSeriesBuffer};
use riptide_core::telemetry::{DataSanitizer, ResourceTracker, SlaMonitor, TelemetrySystem};

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

        b.iter(|| {
            rt.block_on(async {
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
            })
        });
    });

    // Benchmark batch metric recording
    for batch_size in [10, 100, 1000].iter() {
        group.bench_with_input(
            BenchmarkId::new("batch_record_extractions", batch_size),
            batch_size,
            |b, &batch_size| {
                let collector = MetricsCollector::new();

                b.iter(|| {
                    rt.block_on(async {
                        for _ in 0..batch_size {
                            let _ = black_box(
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
                    })
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
            let mut buffer = TimeSeriesBuffer::new(1000, Duration::from_secs(3600));

            for i in 0..100 {
                buffer.add_point(
                    black_box(i as f64),
                    black_box(std::collections::HashMap::new()),
                );
                black_box(());
            }
        });
    });

    // Benchmark percentile calculations
    group.bench_function("calculate_percentile", |b| {
        let mut buffer = TimeSeriesBuffer::new(1000, Duration::from_secs(3600));

        // Pre-populate buffer with data
        for i in 0..1000 {
            buffer.add_point(i as f64, std::collections::HashMap::new());
        }

        b.iter(|| {
            black_box(
                buffer.calculate_percentile(black_box(95.0), black_box(Duration::from_secs(5 * 60))),
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
                let _ = collector
                    .record_extraction(
                        Duration::from_millis(100 + i % 100),
                        i % 10 != 0, // 90% success rate
                        Some(80 + (i % 20) as u8),
                        Some((500 + i * 10) as u32),
                        i % 3 == 0, // 33% cache hit rate
                    )
                    .await;
            }
        });

        b.iter(|| {
            rt.block_on(async {
                black_box(
                    collector
                        .get_current_metrics()
                        .await,
                )
            })
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

                b.iter(|| {
                    rt.block_on(async {
                        let tasks: Vec<_> = (0..thread_count)
                            .map(|_| {
                                let collector = &collector;
                                async move {
                                    for _ in 0..100 {
                                        let _ = collector
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
                    })
                });
            },
        );
    }

    group.finish();
}

fn bench_memory_usage_patterns(c: &mut Criterion) {

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
                        TimeSeriesBuffer::new(10000, Duration::from_secs(retention_hours * 3600));

                    // Simulate continuous data collection
                    for i in 0..1000 {
                        buffer.add_point(
                            black_box(i as f64),
                            black_box(std::collections::HashMap::new()),
                        );
                        black_box(());
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

        b.iter(|| {
            rt.block_on(async {
                black_box(alert_manager.check_alerts(black_box(&test_metrics)).await)
            })
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
        last_updated_utc: chrono::Utc::now(),
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

fn bench_telemetry_system(c: &mut Criterion) {
    // Test telemetry system performance if available
    if let Ok(telemetry) = TelemetrySystem::init() {
        let telemetry = Arc::new(telemetry);

        c.bench_function("telemetry_create_span", |b| {
            b.iter(|| {
                let tracer = telemetry.tracer();
                let _span = tracer.start("benchmark_span");
                black_box(_span);
            });
        });

        c.bench_function("telemetry_get_resource_usage", |b| {
            b.iter(|| {
                black_box(telemetry.get_resource_usage());
            });
        });
    }
}

fn bench_data_sanitization(c: &mut Criterion) {
    let sanitizer = DataSanitizer::new();

    let test_strings = [
        "Normal log message without sensitive data",
        "API key: sk-1234567890abcdef1234567890abcdef in the request",
        "User email user@example.com contacted support",
        "Authorization: Bearer eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9...",
        "Credit card 4532-1234-5678-9012 was used for payment",
        "Server IP: 192.168.1.100 is experiencing issues",
        "Phone number +1-555-123-4567 called for support",
        "SSN 123-45-6789 was provided for verification",
        "Mixed data: api_key=secret123, email=test@test.com, ip=10.0.0.1",
    ];

    let mut group = c.benchmark_group("data_sanitization");

    for (i, test_string) in test_strings.iter().enumerate() {
        group.bench_with_input(
            BenchmarkId::new("sanitize_string", i),
            test_string,
            |b, s| {
                b.iter(|| {
                    black_box(sanitizer.sanitize(black_box(s)));
                });
            },
        );
    }

    // Benchmark map sanitization
    let test_map: HashMap<String, String> = [
        ("api_key".to_string(), "sk-1234567890abcdef".to_string()),
        ("user_email".to_string(), "user@example.com".to_string()),
        ("server_ip".to_string(), "192.168.1.100".to_string()),
        ("normal_field".to_string(), "normal_value".to_string()),
    ]
    .iter()
    .cloned()
    .collect();

    group.bench_function("sanitize_map", |b| {
        b.iter(|| {
            black_box(sanitizer.sanitize_map(black_box(&test_map)));
        });
    });

    group.finish();
}

fn bench_sla_monitoring(c: &mut Criterion) {
    let mut monitor = SlaMonitor::new();

    // Pre-populate with some data
    for i in 0..1000 {
        let duration = Duration::from_millis(50 + (i % 200));
        let success = i % 10 != 0; // 10% failure rate
        monitor.record_metric("test_operation", duration, success);
    }

    c.bench_function("sla_record_metric", |b| {
        b.iter(|| {
            monitor.record_metric(
                black_box("benchmark_op"),
                black_box(Duration::from_millis(100)),
                black_box(true),
            );
        });
    });

    c.bench_function("sla_get_status", |b| {
        b.iter(|| {
            black_box(monitor.get_status());
        });
    });
}

fn bench_resource_tracking(c: &mut Criterion) {
    let tracker = ResourceTracker::new();

    c.bench_function("resource_get_usage", |b| {
        b.iter(|| {
            black_box(tracker.get_usage());
        });
    });
}

criterion_group!(
    performance_benches,
    bench_metrics_collection,
    bench_time_series_operations,
    bench_performance_report_generation,
    bench_concurrent_metrics_collection,
    bench_memory_usage_patterns,
    bench_alert_processing,
    bench_metrics_serialization,
    bench_telemetry_system,
    bench_data_sanitization,
    bench_sla_monitoring,
    bench_resource_tracking
);

criterion_main!(performance_benches);
