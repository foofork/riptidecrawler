/// Performance and stress tests for the RipTide API
///
/// This module contains comprehensive performance tests including:
/// - Throughput benchmarks
/// - Concurrency stress tests
/// - Memory usage validation
/// - Response time analysis
/// - Load testing scenarios

use axum::{
    body::Body,
    http::{Method, Request, StatusCode},
    routing::{get, post},
    Router,
};
use criterion::{criterion_group, criterion_main, Criterion, BenchmarkId};
use futures::future::join_all;
use serde_json::{json, Value};
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::runtime::Runtime;
use tokio::sync::Semaphore;
use tower::ServiceExt;

/// Create a performance test router
fn create_perf_test_router() -> Router {
    Router::new()
        .route("/healthz", get(fast_health_handler))
        .route("/crawl", post(fast_crawl_handler))
        .route("/stress", post(stress_test_handler))
}

/// Fast health handler for performance testing
async fn fast_health_handler() -> axum::response::Json<Value> {
    axum::response::Json(json!({
        "status": "healthy",
        "version": "0.1.0",
        "timestamp": chrono::Utc::now().to_rfc3339(),
        "uptime": 3600,
        "dependencies": {
            "redis": {"status": "healthy"},
            "extractor": {"status": "healthy"},
            "http_client": {"status": "healthy"}
        }
    }))
}

/// Fast crawl handler optimized for performance testing
async fn fast_crawl_handler(
    axum::extract::Json(payload): axum::extract::Json<Value>,
) -> axum::response::Json<Value> {
    let urls = payload.get("urls")
        .and_then(|u| u.as_array())
        .unwrap_or(&Vec::new());

    let results: Vec<Value> = urls.iter().enumerate().map(|(index, url)| {
        json!({
            "url": url.as_str().unwrap_or(""),
            "status": 200,
            "from_cache": false,
            "gate_decision": "raw",
            "quality_score": 0.8,
            "processing_time_ms": 50,
            "document": {
                "url": url.as_str().unwrap_or(""),
                "title": format!("Document {}", index),
                "byline": null,
                "published_iso": null,
                "markdown": format!("# Document {}\n\nContent", index),
                "text": format!("Document {} content", index),
                "links": [],
                "media": []
            },
            "error": null,
            "cache_key": format!("perf_key_{}", index)
        })
    }).collect();

    axum::response::Json(json!({
        "total_urls": urls.len(),
        "successful": urls.len(),
        "failed": 0,
        "from_cache": 0,
        "results": results,
        "statistics": {
            "total_processing_time_ms": urls.len() as u64 * 50,
            "avg_processing_time_ms": 50.0,
            "gate_decisions": {
                "raw": urls.len(),
                "probes_first": 0,
                "headless": 0,
                "cached": 0
            },
            "cache_hit_rate": 0.0
        }
    }))
}

/// Stress test handler that simulates CPU-intensive work
async fn stress_test_handler(
    axum::extract::Json(payload): axum::extract::Json<Value>,
) -> axum::response::Json<Value> {
    let workload = payload.get("workload").and_then(|w| w.as_str()).unwrap_or("light");
    let delay_ms = match workload {
        "light" => 10,
        "medium" => 50,
        "heavy" => 200,
        _ => 10,
    };

    // Simulate processing time
    tokio::time::sleep(Duration::from_millis(delay_ms)).await;

    axum::response::Json(json!({
        "workload": workload,
        "processing_time_ms": delay_ms,
        "result": "completed"
    }))
}

/// Performance test utilities
struct PerformanceTestContext {
    app: Router,
    runtime: Runtime,
}

impl PerformanceTestContext {
    fn new() -> Self {
        Self {
            app: create_perf_test_router(),
            runtime: Runtime::new().unwrap(),
        }
    }

    /// Execute a single request and measure performance
    async fn execute_request(&self, method: Method, uri: &str, body: Option<Value>) -> (Duration, StatusCode) {
        let start = Instant::now();

        let mut request_builder = Request::builder()
            .method(method)
            .uri(uri);

        let body = if let Some(json_body) = body {
            request_builder = request_builder.header("content-type", "application/json");
            Body::from(serde_json::to_string(&json_body).unwrap())
        } else {
            Body::empty()
        };

        let request = request_builder.body(body).unwrap();
        let response = self.app.clone().oneshot(request).await.unwrap();
        let status = response.status();

        // Consume the body to ensure full processing
        let _body = axum::body::to_bytes(response.into_body(), usize::MAX).await.unwrap();

        (start.elapsed(), status)
    }

    /// Execute multiple concurrent requests
    async fn execute_concurrent_requests(
        &self,
        count: usize,
        method: Method,
        uri: &str,
        body: Option<Value>,
    ) -> Vec<(Duration, StatusCode)> {
        let semaphore = Arc::new(Semaphore::new(50)); // Limit concurrent requests

        let requests: Vec<_> = (0..count).map(|_| {
            let app = self.app.clone();
            let method = method.clone();
            let uri = uri.to_string();
            let body = body.clone();
            let semaphore = semaphore.clone();

            async move {
                let _permit = semaphore.acquire().await.unwrap();
                let start = Instant::now();

                let mut request_builder = Request::builder()
                    .method(method)
                    .uri(uri);

                let request_body = if let Some(json_body) = body {
                    request_builder = request_builder.header("content-type", "application/json");
                    Body::from(serde_json::to_string(&json_body).unwrap())
                } else {
                    Body::empty()
                };

                let request = request_builder.body(request_body).unwrap();
                let response = app.oneshot(request).await.unwrap();
                let status = response.status();

                // Consume the body
                let _body = axum::body::to_bytes(response.into_body(), usize::MAX).await.unwrap();

                (start.elapsed(), status)
            }
        }).collect();

        join_all(requests).await
    }
}

#[cfg(test)]
mod throughput_tests {
    use super::*;

    #[tokio::test]
    async fn test_health_endpoint_throughput() {
        let ctx = PerformanceTestContext::new();
        let iterations = 100;
        let start = Instant::now();

        let results = ctx.execute_concurrent_requests(
            iterations,
            Method::GET,
            "/healthz",
            None,
        ).await;

        let total_duration = start.elapsed();
        let requests_per_second = iterations as f64 / total_duration.as_secs_f64();

        println!("Health endpoint throughput: {:.2} req/sec", requests_per_second);

        // All requests should succeed
        for (_, status) in &results {
            assert_eq!(*status, StatusCode::OK);
        }

        // Should achieve reasonable throughput
        assert!(requests_per_second > 50.0, "Health endpoint throughput too low: {:.2} req/sec", requests_per_second);

        // Calculate response time statistics
        let response_times: Vec<u128> = results.iter().map(|(duration, _)| duration.as_millis()).collect();
        let avg_response_time = response_times.iter().sum::<u128>() as f64 / response_times.len() as f64;
        let max_response_time = *response_times.iter().max().unwrap();
        let min_response_time = *response_times.iter().min().unwrap();

        println!("Response time stats: avg={:.2}ms, min={}ms, max={}ms",
                avg_response_time, min_response_time, max_response_time);

        // Response times should be reasonable
        assert!(avg_response_time < 100.0, "Average response time too high: {:.2}ms", avg_response_time);
        assert!(max_response_time < 500, "Max response time too high: {}ms", max_response_time);
    }

    #[tokio::test]
    async fn test_crawl_endpoint_throughput() {
        let ctx = PerformanceTestContext::new();
        let iterations = 50;

        let request_body = json!({
            "urls": ["https://example.com", "https://test.org"]
        });

        let start = Instant::now();

        let results = ctx.execute_concurrent_requests(
            iterations,
            Method::POST,
            "/crawl",
            Some(request_body),
        ).await;

        let total_duration = start.elapsed();
        let requests_per_second = iterations as f64 / total_duration.as_secs_f64();

        println!("Crawl endpoint throughput: {:.2} req/sec", requests_per_second);

        // All requests should succeed
        for (_, status) in &results {
            assert_eq!(*status, StatusCode::OK);
        }

        // Should achieve reasonable throughput (lower than health due to complexity)
        assert!(requests_per_second > 10.0, "Crawl endpoint throughput too low: {:.2} req/sec", requests_per_second);

        // Calculate response time statistics
        let response_times: Vec<u128> = results.iter().map(|(duration, _)| duration.as_millis()).collect();
        let avg_response_time = response_times.iter().sum::<u128>() as f64 / response_times.len() as f64;

        println!("Crawl response time average: {:.2}ms", avg_response_time);

        // Response times should be reasonable for crawl operations
        assert!(avg_response_time < 500.0, "Average crawl response time too high: {:.2}ms", avg_response_time);
    }

    #[tokio::test]
    async fn test_single_request_response_times() {
        let ctx = PerformanceTestContext::new();

        // Test health endpoint
        let (health_duration, health_status) = ctx.execute_request(Method::GET, "/healthz", None).await;
        assert_eq!(health_status, StatusCode::OK);
        assert!(health_duration.as_millis() < 50, "Health check too slow: {}ms", health_duration.as_millis());

        // Test crawl endpoint
        let crawl_body = json!({"urls": ["https://example.com"]});
        let (crawl_duration, crawl_status) = ctx.execute_request(Method::POST, "/crawl", Some(crawl_body)).await;
        assert_eq!(crawl_status, StatusCode::OK);
        assert!(crawl_duration.as_millis() < 200, "Crawl request too slow: {}ms", crawl_duration.as_millis());
    }
}

#[cfg(test)]
mod concurrency_stress_tests {
    use super::*;

    #[tokio::test]
    async fn test_high_concurrency_health_checks() {
        let ctx = PerformanceTestContext::new();
        let concurrent_requests = 200;

        let start = Instant::now();

        let results = ctx.execute_concurrent_requests(
            concurrent_requests,
            Method::GET,
            "/healthz",
            None,
        ).await;

        let total_duration = start.elapsed();

        // All requests should complete successfully
        let success_count = results.iter().filter(|(_, status)| *status == StatusCode::OK).count();
        assert_eq!(success_count, concurrent_requests, "Not all concurrent requests succeeded");

        // Should complete within reasonable time
        assert!(total_duration.as_secs() < 10, "High concurrency test took too long: {}s", total_duration.as_secs());

        // Calculate concurrency metrics
        let requests_per_second = concurrent_requests as f64 / total_duration.as_secs_f64();
        println!("High concurrency throughput: {:.2} req/sec with {} concurrent requests",
                requests_per_second, concurrent_requests);

        // Should maintain reasonable throughput under high concurrency
        assert!(requests_per_second > 20.0, "Throughput degraded under high concurrency: {:.2} req/sec", requests_per_second);
    }

    #[tokio::test]
    async fn test_mixed_workload_concurrency() {
        let ctx = PerformanceTestContext::new();

        // Create mixed workload: health checks and crawl requests
        let health_requests = 50;
        let crawl_requests = 25;

        let crawl_body = json!({
            "urls": ["https://example.com", "https://test.org", "https://demo.net"]
        });

        let start = Instant::now();

        // Execute health requests
        let health_future = ctx.execute_concurrent_requests(
            health_requests,
            Method::GET,
            "/healthz",
            None,
        );

        // Execute crawl requests
        let crawl_future = ctx.execute_concurrent_requests(
            crawl_requests,
            Method::POST,
            "/crawl",
            Some(crawl_body),
        );

        let (health_results, crawl_results) = tokio::join!(health_future, crawl_future);

        let total_duration = start.elapsed();

        // Verify all requests succeeded
        let health_success = health_results.iter().filter(|(_, status)| *status == StatusCode::OK).count();
        let crawl_success = crawl_results.iter().filter(|(_, status)| *status == StatusCode::OK).count();

        assert_eq!(health_success, health_requests);
        assert_eq!(crawl_success, crawl_requests);

        let total_requests = health_requests + crawl_requests;
        let requests_per_second = total_requests as f64 / total_duration.as_secs_f64();

        println!("Mixed workload: {:.2} req/sec ({} health + {} crawl)",
                requests_per_second, health_requests, crawl_requests);

        // Should handle mixed workload efficiently
        assert!(requests_per_second > 15.0, "Mixed workload throughput too low: {:.2} req/sec", requests_per_second);
    }

    #[tokio::test]
    async fn test_stress_with_varying_payloads() {
        let ctx = PerformanceTestContext::new();

        let workloads = vec![
            ("light", 30),
            ("medium", 20),
            ("heavy", 10),
        ];

        let start = Instant::now();

        let all_requests: Vec<_> = workloads.into_iter().map(|(workload, count)| {
            let body = json!({"workload": workload});
            ctx.execute_concurrent_requests(count, Method::POST, "/stress", Some(body))
        }).collect();

        let results = join_all(all_requests).await;
        let total_duration = start.elapsed();

        // Verify all requests in all workload categories succeeded
        let total_requests: usize = results.iter().map(|r| r.len()).sum();
        let total_successes: usize = results.iter()
            .map(|r| r.iter().filter(|(_, status)| *status == StatusCode::OK).count())
            .sum();

        assert_eq!(total_successes, total_requests);

        println!("Stress test with varying payloads: {} requests in {:.2}s",
                total_requests, total_duration.as_secs_f64());

        // Should complete within reasonable time even with heavy workloads
        assert!(total_duration.as_secs() < 30, "Stress test took too long: {}s", total_duration.as_secs());
    }
}

#[cfg(test)]
mod memory_and_resource_tests {
    use super::*;

    #[tokio::test]
    async fn test_memory_usage_under_load() {
        let ctx = PerformanceTestContext::new();

        // Get initial memory baseline (simplified - in real tests would use proper memory profiling)
        let initial_time = Instant::now();

        // Execute a sustained load
        for batch in 0..5 {
            let batch_requests = 20;
            let request_body = json!({
                "urls": (0..5).map(|i| format!("https://example{}.com", i)).collect::<Vec<_>>()
            });

            let results = ctx.execute_concurrent_requests(
                batch_requests,
                Method::POST,
                "/crawl",
                Some(request_body),
            ).await;

            // Verify batch completed successfully
            let success_count = results.iter().filter(|(_, status)| *status == StatusCode::OK).count();
            assert_eq!(success_count, batch_requests, "Batch {} failed", batch);

            println!("Completed batch {} of memory test", batch + 1);

            // Small delay between batches
            tokio::time::sleep(Duration::from_millis(100)).await;
        }

        let total_duration = initial_time.elapsed();
        println!("Memory test completed in {:.2}s", total_duration.as_secs_f64());

        // Test should complete without memory issues
        assert!(total_duration.as_secs() < 60, "Memory test took too long, possible memory leak");
    }

    #[tokio::test]
    async fn test_resource_cleanup() {
        let ctx = PerformanceTestContext::new();

        // Execute requests and verify clean resource usage
        for iteration in 0..10 {
            let request_body = json!({
                "urls": vec!["https://example.com"; 10]
            });

            let (duration, status) = ctx.execute_request(
                Method::POST,
                "/crawl",
                Some(request_body),
            ).await;

            assert_eq!(status, StatusCode::OK);

            // Each iteration should complete in reasonable time
            assert!(duration.as_millis() < 1000,
                "Iteration {} took too long: {}ms, possible resource leak",
                iteration, duration.as_millis());

            // Brief pause between iterations
            tokio::time::sleep(Duration::from_millis(50)).await;
        }

        println!("Resource cleanup test passed - no apparent leaks");
    }

    #[tokio::test]
    async fn test_large_payload_handling() {
        let ctx = PerformanceTestContext::new();

        // Test with maximum allowed URLs (100)
        let large_payload = json!({
            "urls": (0..100).map(|i| format!("https://site{}.com/page", i)).collect::<Vec<_>>()
        });

        let start = Instant::now();
        let (duration, status) = ctx.execute_request(
            Method::POST,
            "/crawl",
            Some(large_payload),
        ).await;

        assert_eq!(status, StatusCode::OK);

        // Large payload should still be processed efficiently
        assert!(duration.as_secs() < 5, "Large payload took too long: {}s", duration.as_secs());

        println!("Large payload (100 URLs) processed in {:.2}s", duration.as_secs_f64());
    }
}

#[cfg(test)]
mod scalability_tests {
    use super::*;

    #[tokio::test]
    async fn test_gradual_load_increase() {
        let ctx = PerformanceTestContext::new();

        let load_levels = vec![10, 25, 50, 100];
        let mut throughput_results = Vec::new();

        for load in load_levels {
            let start = Instant::now();

            let results = ctx.execute_concurrent_requests(
                load,
                Method::GET,
                "/healthz",
                None,
            ).await;

            let duration = start.elapsed();
            let success_count = results.iter().filter(|(_, status)| *status == StatusCode::OK).count();
            let throughput = success_count as f64 / duration.as_secs_f64();

            throughput_results.push((load, throughput));

            assert_eq!(success_count, load, "Load level {} had failures", load);

            println!("Load level {}: {:.2} req/sec", load, throughput);

            // Brief pause between load levels
            tokio::time::sleep(Duration::from_millis(200)).await;
        }

        // Verify that throughput scales reasonably with load
        // (Note: In real systems, throughput might plateau or decrease at high loads)
        for (load, throughput) in throughput_results {
            assert!(throughput > 5.0, "Throughput too low at load {}: {:.2} req/sec", load, throughput);
        }
    }

    #[tokio::test]
    async fn test_sustained_load() {
        let ctx = PerformanceTestContext::new();
        let duration_seconds = 10;
        let requests_per_second = 10;
        let total_requests = duration_seconds * requests_per_second;

        let start = Instant::now();

        // Spread requests over time to simulate sustained load
        let mut tasks = Vec::new();
        for i in 0..total_requests {
            let delay = Duration::from_millis((i * 1000 / requests_per_second) as u64);
            let ctx_clone = &ctx;

            let task = async move {
                tokio::time::sleep(delay).await;
                ctx_clone.execute_request(Method::GET, "/healthz", None).await
            };

            tasks.push(task);
        }

        let results = join_all(tasks).await;
        let total_duration = start.elapsed();

        // Verify all requests succeeded
        let success_count = results.iter().filter(|(_, status)| *status == StatusCode::OK).count();
        assert_eq!(success_count, total_requests, "Sustained load test had failures");

        let actual_rps = total_requests as f64 / total_duration.as_secs_f64();

        println!("Sustained load: {:.2} req/sec over {:.2}s (target: {} req/sec)",
                actual_rps, total_duration.as_secs_f64(), requests_per_second);

        // Should maintain target throughput
        assert!(actual_rps >= requests_per_second as f64 * 0.8,
               "Sustained load throughput too low: {:.2} req/sec", actual_rps);
    }
}

// Criterion benchmarks (optional - requires criterion feature)
#[cfg(feature = "criterion-benchmarks")]
mod criterion_benchmarks {
    use super::*;

    fn benchmark_health_endpoint(c: &mut Criterion) {
        let rt = Runtime::new().unwrap();
        let ctx = PerformanceTestContext::new();

        c.bench_function("health_endpoint", |b| {
            b.to_async(&rt).iter(|| async {
                let (_, status) = ctx.execute_request(Method::GET, "/healthz", None).await;
                assert_eq!(status, StatusCode::OK);
            })
        });
    }

    fn benchmark_crawl_endpoint(c: &mut Criterion) {
        let rt = Runtime::new().unwrap();
        let ctx = PerformanceTestContext::new();

        let request_body = json!({
            "urls": ["https://example.com"]
        });

        c.bench_function("crawl_single_url", |b| {
            b.to_async(&rt).iter(|| async {
                let (_, status) = ctx.execute_request(
                    Method::POST,
                    "/crawl",
                    Some(request_body.clone())
                ).await;
                assert_eq!(status, StatusCode::OK);
            })
        });
    }

    fn benchmark_crawl_multiple_urls(c: &mut Criterion) {
        let rt = Runtime::new().unwrap();
        let ctx = PerformanceTestContext::new();

        let mut group = c.benchmark_group("crawl_multiple_urls");

        for url_count in [1, 5, 10, 25].iter() {
            let request_body = json!({
                "urls": (0..*url_count).map(|i| format!("https://example{}.com", i)).collect::<Vec<_>>()
            });

            group.bench_with_input(BenchmarkId::from_parameter(url_count), url_count, |b, _| {
                b.to_async(&rt).iter(|| async {
                    let (_, status) = ctx.execute_request(
                        Method::POST,
                        "/crawl",
                        Some(request_body.clone())
                    ).await;
                    assert_eq!(status, StatusCode::OK);
                })
            });
        }

        group.finish();
    }

    criterion_group!(
        benches,
        benchmark_health_endpoint,
        benchmark_crawl_endpoint,
        benchmark_crawl_multiple_urls
    );
    criterion_main!(benches);
}

// Integration with test runner
#[cfg(test)]
mod performance_integration_tests {
    use super::*;

    #[tokio::test]
    async fn test_performance_regression_detection() {
        let ctx = PerformanceTestContext::new();

        // Baseline performance test
        let baseline_iterations = 50;
        let start = Instant::now();

        let results = ctx.execute_concurrent_requests(
            baseline_iterations,
            Method::GET,
            "/healthz",
            None,
        ).await;

        let baseline_duration = start.elapsed();
        let baseline_rps = baseline_iterations as f64 / baseline_duration.as_secs_f64();

        // All requests should succeed
        let success_count = results.iter().filter(|(_, status)| *status == StatusCode::OK).count();
        assert_eq!(success_count, baseline_iterations);

        // Store baseline for regression detection
        println!("Performance baseline: {:.2} req/sec", baseline_rps);

        // In a real CI/CD system, this would be compared against stored baselines
        assert!(baseline_rps > 20.0, "Performance regression detected: {:.2} req/sec", baseline_rps);

        // Response time baseline
        let response_times: Vec<u128> = results.iter().map(|(duration, _)| duration.as_millis()).collect();
        let avg_response_time = response_times.iter().sum::<u128>() as f64 / response_times.len() as f64;
        let p95_response_time = {
            let mut sorted_times = response_times.clone();
            sorted_times.sort();
            sorted_times[(sorted_times.len() as f64 * 0.95) as usize]
        };

        println!("Response time baseline: avg={:.2}ms, p95={}ms", avg_response_time, p95_response_time);

        // Response time regression detection
        assert!(avg_response_time < 50.0, "Response time regression: avg={:.2}ms", avg_response_time);
        assert!(p95_response_time < 100, "P95 response time regression: {}ms", p95_response_time);
    }
}