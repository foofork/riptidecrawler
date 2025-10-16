//! Performance benchmark tests for metrics collection systems
//!
//! These tests measure the performance overhead of metrics collection
//! to ensure it doesn't significantly impact application performance.

use criterion::{black_box, criterion_group, criterion_main, Criterion, BenchmarkId};
use riptide_pdf::metrics::PdfMetricsCollector;
use riptide_intelligence::metrics::{MetricsCollector, TimeWindow};
use riptide_intelligence::{CompletionRequest, CompletionResponse, Message, Usage};
use std::sync::Arc;
use std::time::Duration;
use tokio::runtime::Runtime;

fn benchmark_pdf_metrics_recording(c: &mut Criterion) {
    let mut group = c.benchmark_group("pdf_metrics_recording");

    group.bench_function("single_success", |b| {
        let collector = PdfMetricsCollector::new();
        b.iter(|| {
            collector.record_processing_success(
                black_box(Duration::from_millis(1000)),
                black_box(10),
                black_box(50 * 1024 * 1024),
            );
        });
    });

    group.bench_function("single_failure", |b| {
        let collector = PdfMetricsCollector::new();
        b.iter(|| {
            collector.record_processing_failure(black_box(false));
        });
    });

    group.bench_function("memory_spike", |b| {
        let collector = PdfMetricsCollector::new();
        b.iter(|| {
            collector.record_memory_spike_detected();
        });
    });

    group.bench_function("pages_per_second", |b| {
        let collector = PdfMetricsCollector::new();
        b.iter(|| {
            collector.record_pages_per_second(black_box(12.5));
        });
    });

    group.finish();
}

fn benchmark_pdf_metrics_snapshot(c: &mut Criterion) {
    let mut group = c.benchmark_group("pdf_metrics_snapshot");

    // Benchmark with different amounts of recorded data
    for num_ops in [10, 100, 1000, 10000].iter() {
        let collector = PdfMetricsCollector::new();

        // Pre-populate with data
        for _ in 0..*num_ops {
            collector.record_processing_success(
                Duration::from_millis(100),
                10,
                50 * 1024 * 1024,
            );
        }

        group.bench_with_input(BenchmarkId::from_parameter(num_ops), num_ops, |b, _| {
            b.iter(|| {
                black_box(collector.get_snapshot());
            });
        });
    }

    group.finish();
}

fn benchmark_pdf_metrics_export(c: &mut Criterion) {
    let collector = PdfMetricsCollector::new();

    // Pre-populate with data
    for _ in 0..1000 {
        collector.record_processing_success(
            Duration::from_millis(100),
            10,
            50 * 1024 * 1024,
        );
    }

    c.bench_function("pdf_prometheus_export", |b| {
        b.iter(|| {
            black_box(collector.export_for_prometheus());
        });
    });
}

fn benchmark_pdf_metrics_concurrent(c: &mut Criterion) {
    let mut group = c.benchmark_group("pdf_metrics_concurrent");

    for num_threads in [2, 4, 8, 16].iter() {
        group.bench_with_input(
            BenchmarkId::from_parameter(num_threads),
            num_threads,
            |b, &num_threads| {
                b.iter(|| {
                    let collector = Arc::new(PdfMetricsCollector::new());
                    let mut handles = vec![];

                    for _ in 0..num_threads {
                        let c = Arc::clone(&collector);
                        let handle = std::thread::spawn(move || {
                            for _ in 0..100 {
                                c.record_processing_success(
                                    Duration::from_millis(100),
                                    10,
                                    50 * 1024 * 1024,
                                );
                            }
                        });
                        handles.push(handle);
                    }

                    for handle in handles {
                        handle.join().unwrap();
                    }
                });
            },
        );
    }

    group.finish();
}

fn benchmark_intelligence_metrics_recording(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    let mut group = c.benchmark_group("intelligence_metrics_recording");

    group.bench_function("single_request_lifecycle", |b| {
        b.to_async(&rt).iter(|| async {
            let collector = MetricsCollector::new(30);
            let request = CompletionRequest::new(
                "gpt-4".to_string(),
                vec![Message::user("test")],
            );

            let request_id = collector
                .start_request(&request, "openai", Some("tenant1".to_string()))
                .await;

            let response = CompletionResponse::new(
                request.id,
                "response",
                "gpt-4",
                Usage {
                    prompt_tokens: 10,
                    completion_tokens: 20,
                    total_tokens: 30,
                },
            );

            collector
                .complete_request_success(black_box(request_id), &response, None)
                .await;
        });
    });

    group.finish();
}

fn benchmark_intelligence_metrics_aggregation(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    let mut group = c.benchmark_group("intelligence_metrics_aggregation");

    for num_requests in [10, 100, 1000].iter() {
        group.bench_with_input(
            BenchmarkId::from_parameter(num_requests),
            num_requests,
            |b, &num_requests| {
                b.to_async(&rt).iter(|| async move {
                    let collector = MetricsCollector::new(30);

                    // Pre-populate with data
                    for _ in 0..num_requests {
                        let request = CompletionRequest::new(
                            "gpt-4".to_string(),
                            vec![Message::user("test")],
                        );

                        let request_id = collector
                            .start_request(&request, "openai", Some("tenant1".to_string()))
                            .await;

                        let response = CompletionResponse::new(
                            request.id,
                            "response",
                            "gpt-4",
                            Usage {
                                prompt_tokens: 10,
                                completion_tokens: 20,
                                total_tokens: 30,
                            },
                        );

                        collector
                            .complete_request_success(request_id, &response, None)
                            .await;
                    }

                    // Benchmark aggregation
                    black_box(
                        collector
                            .get_aggregated_metrics(TimeWindow::LastHour)
                            .await,
                    );
                });
            },
        );
    }

    group.finish();
}

fn benchmark_intelligence_dashboard_generation(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();

    c.bench_function("intelligence_full_dashboard", |b| {
        b.to_async(&rt).iter(|| async {
            let collector = MetricsCollector::new(30);

            // Pre-populate with varied data
            for i in 0..100 {
                let provider = if i % 2 == 0 { "openai" } else { "anthropic" };
                let model = if i % 2 == 0 { "gpt-4" } else { "claude-3" };

                let request = CompletionRequest::new(
                    model.to_string(),
                    vec![Message::user("test")],
                );

                let request_id = collector
                    .start_request(&request, provider, Some(format!("tenant{}", i % 5)))
                    .await;

                if i % 10 != 0 {
                    // 90% success rate
                    let response = CompletionResponse::new(
                        request.id,
                        "response",
                        model,
                        Usage {
                            prompt_tokens: 10,
                            completion_tokens: 20,
                            total_tokens: 30,
                        },
                    );

                    collector
                        .complete_request_success(request_id, &response, None)
                        .await;
                } else {
                    // 10% error rate
                    collector
                        .complete_request_error(request_id, "APIError", "Error")
                        .await;
                }
            }

            black_box(
                collector
                    .generate_dashboard(TimeWindow::LastHour)
                    .await,
            );
        });
    });
}

fn benchmark_intelligence_concurrent(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    let mut group = c.benchmark_group("intelligence_metrics_concurrent");

    for num_tasks in [2, 4, 8, 16].iter() {
        group.bench_with_input(
            BenchmarkId::from_parameter(num_tasks),
            num_tasks,
            |b, &num_tasks| {
                b.to_async(&rt).iter(|| async move {
                    let collector = Arc::new(MetricsCollector::new(30));
                    let mut handles = vec![];

                    for _ in 0..num_tasks {
                        let c = Arc::clone(&collector);
                        let handle = tokio::spawn(async move {
                            for _ in 0..50 {
                                let request = CompletionRequest::new(
                                    "gpt-4".to_string(),
                                    vec![Message::user("test")],
                                );

                                let request_id = c
                                    .start_request(&request, "openai", Some("tenant1".to_string()))
                                    .await;

                                let response = CompletionResponse::new(
                                    request.id,
                                    "response",
                                    "gpt-4",
                                    Usage {
                                        prompt_tokens: 10,
                                        completion_tokens: 20,
                                        total_tokens: 30,
                                    },
                                );

                                c.complete_request_success(request_id, &response, None)
                                    .await;
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
    pdf_benches,
    benchmark_pdf_metrics_recording,
    benchmark_pdf_metrics_snapshot,
    benchmark_pdf_metrics_export,
    benchmark_pdf_metrics_concurrent
);

criterion_group!(
    intelligence_benches,
    benchmark_intelligence_metrics_recording,
    benchmark_intelligence_metrics_aggregation,
    benchmark_intelligence_dashboard_generation,
    benchmark_intelligence_concurrent
);

criterion_main!(pdf_benches, intelligence_benches);
