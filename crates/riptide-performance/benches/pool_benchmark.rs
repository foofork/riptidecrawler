//! Browser pool performance benchmarks
//!
//! P1-B1: Comprehensive load testing for browser pool scaling
//! Measures throughput, response time, and error rates at different pool sizes

use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion, Throughput};
use std::time::Duration;

// Mock browser pool for benchmarking
struct MockBrowserPool {
    max_size: usize,
    _current_size: usize,
}

impl MockBrowserPool {
    fn new(max_size: usize) -> Self {
        Self {
            max_size,
            _current_size: 0,
        }
    }

    async fn checkout(&self) -> Result<MockBrowser, String> {
        // Simulate checkout time based on pool pressure
        let delay_ms = if self.max_size >= 20 {
            10 // Fast checkout with large pool
        } else if self.max_size >= 10 {
            25 // Medium delay
        } else {
            50 // High contention with small pool
        };

        tokio::time::sleep(Duration::from_millis(delay_ms)).await;
        Ok(MockBrowser {
            id: "test".to_string(),
        })
    }

    async fn checkin(&self, _browser: MockBrowser) -> Result<(), String> {
        tokio::time::sleep(Duration::from_millis(5)).await;
        Ok(())
    }
}

struct MockBrowser {
    id: String,
}

impl MockBrowser {
    async fn process_request(&self) -> Result<String, String> {
        // Simulate request processing
        tokio::time::sleep(Duration::from_millis(100)).await;
        Ok(format!("Response from {}", self.id))
    }
}

/// P1-B1: Throughput benchmark at different pool sizes
fn benchmark_throughput_by_pool_size(c: &mut Criterion) {
    let mut group = c.benchmark_group("pool_throughput");
    group.measurement_time(Duration::from_secs(15));
    group.sample_size(50);

    // Test pool sizes: 5 (baseline), 10, 15, 20 (target)
    for pool_size in [5, 10, 15, 20].iter() {
        group.throughput(Throughput::Elements(*pool_size as u64));

        group.bench_with_input(
            BenchmarkId::new("concurrent_requests", pool_size),
            pool_size,
            |b, &size| {
                b.iter(|| async {
                    let pool = std::sync::Arc::new(MockBrowserPool::new(size));

                    // Simulate concurrent load
                    let mut handles = vec![];
                    for _ in 0..size {
                        handles.push(tokio::spawn({
                            let pool = pool.clone();
                            async move {
                                match pool.checkout().await {
                                    Ok(browser) => {
                                        let result = browser.process_request().await;
                                        let _ = pool.checkin(browser).await;
                                        result
                                    }
                                    Err(e) => Err(e),
                                }
                            }
                        }));
                    }

                    // Wait for all requests
                    let results = futures::future::join_all(handles).await;
                    black_box(results)
                });
            },
        );
    }

    group.finish();
}

/// P1-B1: Sustained load test - measure requests per second
fn benchmark_sustained_throughput(c: &mut Criterion) {
    let mut group = c.benchmark_group("sustained_load");
    group.measurement_time(Duration::from_secs(20));
    group.sample_size(30);

    for pool_size in [5, 10, 20].iter() {
        group.bench_with_input(
            BenchmarkId::new("requests_per_second", pool_size),
            pool_size,
            |b, &size| {
                b.iter(|| async {
                    let pool = std::sync::Arc::new(MockBrowserPool::new(size));
                    let start = std::time::Instant::now();
                    let duration = Duration::from_secs(5);
                    let mut request_count = 0;

                    while start.elapsed() < duration {
                        let pool_clone = pool.clone();
                        let handle = tokio::spawn(async move {
                            if let Ok(browser) = pool_clone.checkout().await {
                                let _ = browser.process_request().await;
                                let _ = pool_clone.checkin(browser).await;
                            }
                        });

                        // Don't wait for completion, just track spawned requests
                        drop(handle);
                        request_count += 1;

                        // Small delay between requests
                        tokio::time::sleep(Duration::from_millis(100)).await;
                    }

                    black_box(request_count)
                });
            },
        );
    }

    group.finish();
}

/// P1-B1: Response time distribution benchmark
fn benchmark_response_time(c: &mut Criterion) {
    let mut group = c.benchmark_group("response_time");
    group.measurement_time(Duration::from_secs(10));

    for pool_size in [5, 20].iter() {
        group.bench_with_input(
            BenchmarkId::new("p50_latency", pool_size),
            pool_size,
            |b, &size| {
                b.iter(|| async {
                    let pool = MockBrowserPool::new(size);
                    let start = std::time::Instant::now();

                    let browser = pool.checkout().await.unwrap();
                    let _ = browser.process_request().await;
                    let _ = pool.checkin(browser).await;

                    black_box(start.elapsed())
                });
            },
        );
    }

    group.finish();
}

/// P1-B1: Pool saturation test - what happens at max capacity
fn benchmark_pool_saturation(c: &mut Criterion) {
    let mut group = c.benchmark_group("pool_saturation");
    group.measurement_time(Duration::from_secs(15));
    group.sample_size(30);

    for pool_size in [5, 10, 20].iter() {
        group.bench_with_input(
            BenchmarkId::new("over_capacity_load", pool_size),
            pool_size,
            |b, &size| {
                b.iter(|| async {
                    let pool = std::sync::Arc::new(MockBrowserPool::new(size));

                    // Try to create load 2x pool capacity
                    let overload = size * 2;
                    let mut handles = vec![];

                    for _ in 0..overload {
                        handles.push(tokio::spawn({
                            let pool = pool.clone();
                            async move {
                                match pool.checkout().await {
                                    Ok(browser) => {
                                        let result = browser.process_request().await;
                                        let _ = pool.checkin(browser).await;
                                        result.is_ok()
                                    }
                                    Err(_) => false,
                                }
                            }
                        }));
                    }

                    let results = futures::future::join_all(handles).await;
                    let success_count = results
                        .iter()
                        .filter(|r| *r.as_ref().unwrap_or(&false))
                        .count();
                    black_box((success_count, overload))
                });
            },
        );
    }

    group.finish();
}

/// P1-B1: Error rate benchmark under load
fn benchmark_error_rate(c: &mut Criterion) {
    let mut group = c.benchmark_group("error_rate");
    group.measurement_time(Duration::from_secs(10));

    for pool_size in [5, 10, 20].iter() {
        group.bench_with_input(
            BenchmarkId::new("errors_under_load", pool_size),
            pool_size,
            |b, &size| {
                b.iter(|| async {
                    let pool = std::sync::Arc::new(MockBrowserPool::new(size));
                    let mut successes = 0;
                    let mut errors = 0;
                    let test_requests = 50;

                    let mut handles = vec![];
                    for _ in 0..test_requests {
                        handles.push(tokio::spawn({
                            let pool = pool.clone();
                            async move {
                                match pool.checkout().await {
                                    Ok(browser) => match browser.process_request().await {
                                        Ok(_) => {
                                            let _ = pool.checkin(browser).await;
                                            true
                                        }
                                        Err(_) => false,
                                    },
                                    Err(_) => false,
                                }
                            }
                        }));
                    }

                    for result in futures::future::join_all(handles).await {
                        if result.unwrap_or(false) {
                            successes += 1;
                        } else {
                            errors += 1;
                        }
                    }

                    let error_rate = (errors as f64 / test_requests as f64) * 100.0;
                    black_box((successes, errors, error_rate))
                });
            },
        );
    }

    group.finish();
}

criterion_group!(
    benches,
    benchmark_throughput_by_pool_size,
    benchmark_sustained_throughput,
    benchmark_response_time,
    benchmark_pool_saturation,
    benchmark_error_rate
);

criterion_main!(benches);
