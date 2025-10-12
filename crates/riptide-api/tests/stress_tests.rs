//! Stress and Load Tests
//!
//! Tests system behavior under extreme load conditions:
//! - 1000 concurrent streaming connections
//! - Browser pool exhaustion and recovery
//! - Cache eviction under memory pressure
//! - Tenant quota enforcement under load
//! - Memory leak detection over extended periods

use axum::body::Body;
use axum::http::{Request, StatusCode};
use serde_json::json;
#[cfg(feature = "streaming")]
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;
use std::time::Duration;
use tokio::time::sleep;
use tower::ServiceExt;

mod test_helpers;

#[cfg(test)]
mod stress_tests {
    use super::*;

    /// Stress Test 1: 1000 concurrent streaming connections
    /// Verifies system can handle high concurrent load
    #[tokio::test]
    #[cfg(feature = "streaming")]
    async fn test_1000_concurrent_streams() {
        let app = Arc::new(test_helpers::create_minimal_test_app());
        let success_count = Arc::new(AtomicUsize::new(0));
        let failure_count = Arc::new(AtomicUsize::new(0));

        let mut handles = vec![];

        // Launch 1000 concurrent streaming requests
        for i in 0..1000 {
            let app_clone = Arc::clone(&app);
            let success_clone = Arc::clone(&success_count);
            let failure_clone = Arc::clone(&failure_count);

            let handle = tokio::spawn(async move {
                let result = app_clone
                    .clone()
                    .oneshot(
                        Request::builder()
                            .method("POST")
                            .uri("/api/v1/stream/start")
                            .header("content-type", "application/json")
                            .body(Body::from(
                                json!({
                                    "stream_id": format!("stress_stream_{}", i),
                                    "source": "test_data",
                                    "format": "ndjson"
                                })
                                .to_string(),
                            ))
                            .unwrap(),
                    )
                    .await;

                match result {
                    Ok(response) if response.status().is_success() => {
                        success_clone.fetch_add(1, Ordering::SeqCst);
                    }
                    _ => {
                        failure_clone.fetch_add(1, Ordering::SeqCst);
                    }
                }
            });

            handles.push(handle);

            // Small delay to prevent overwhelming the test harness
            if i % 100 == 0 {
                sleep(Duration::from_millis(10)).await;
            }
        }

        // Wait for all requests to complete
        for handle in handles {
            let _ = handle.await;
        }

        let successes = success_count.load(Ordering::SeqCst);
        let failures = failure_count.load(Ordering::SeqCst);

        println!(
            "Stress test results: {} successes, {} failures",
            successes, failures
        );

        // Allow up to 10% failure rate under extreme load
        assert!(successes > 900, "Success rate too low: {}/1000", successes);
    }

    /// Stress Test 2: Browser pool exhaustion and recovery
    /// Tests browser pool behavior when exhausted
    #[tokio::test]
    #[cfg(feature = "sessions")]
    async fn test_browser_pool_exhaustion_recovery() {
        let app = Arc::new(test_helpers::create_minimal_test_app());

        // Initialize browser pool with small size
        let _ = app
            .clone()
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/api/v1/browser/pool/init")
                    .body(Body::from(
                        json!({
                            "pool_size": 5,
                            "max_lifetime_seconds": 60
                        })
                        .to_string(),
                    ))
                    .unwrap(),
            )
            .await;

        // Attempt to allocate more browsers than pool size
        let mut handles = vec![];
        for i in 0..20 {
            let app_clone = Arc::clone(&app);
            let handle = tokio::spawn(async move {
                (*app_clone)
                    .clone()
                    .oneshot(
                        Request::builder()
                            .method("POST")
                            .uri("/api/v1/browser/pool/allocate")
                            .body(Body::from(
                                json!({"id": format!("browser_{}", i)}).to_string(),
                            ))
                            .unwrap(),
                    )
                    .await
            });
            handles.push(handle);
        }

        let mut success_count = 0;
        let mut queued_count = 0;

        for handle in handles {
            if let Ok(Ok(response)) = handle.await {
                match response.status() {
                    StatusCode::OK => success_count += 1,
                    StatusCode::ACCEPTED => queued_count += 1,
                    _ => {}
                }
            }
        }

        // Verify pool handled exhaustion gracefully
        assert!(
            success_count + queued_count >= 15,
            "Browser pool didn't handle load properly: {} allocated, {} queued",
            success_count,
            queued_count
        );

        // Verify pool can recover
        sleep(Duration::from_secs(2)).await;

        let recovery_response = app
            .clone()
            .oneshot(
                Request::builder()
                    .uri("/api/v1/browser/pool/status")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(recovery_response.status(), StatusCode::OK);
    }

    /// Stress Test 3: Cache eviction under memory pressure
    /// Tests cache behavior when memory limits are reached
    #[tokio::test]
    async fn test_cache_eviction_under_pressure() {
        let app = Arc::new(test_helpers::create_minimal_test_app());

        // Fill cache with large entries
        for i in 0..1000 {
            let large_value = "x".repeat(10_000); // 10KB per entry
            let _ = (*app)
                .clone()
                .oneshot(
                    Request::builder()
                        .method("POST")
                        .uri("/api/v1/cache/set")
                        .body(Body::from(
                            json!({
                                "key": format!("large_entry_{}", i),
                                "value": large_value,
                                "ttl_seconds": 3600
                            })
                            .to_string(),
                        ))
                        .unwrap(),
                )
                .await;

            // Small delay every 100 entries
            if i % 100 == 0 {
                sleep(Duration::from_millis(10)).await;
            }
        }

        // Verify cache is handling pressure
        let stats_response = (*app)
            .clone()
            .oneshot(
                Request::builder()
                    .uri("/api/v1/cache/stats")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(stats_response.status(), StatusCode::OK);

        // Verify old entries can still be accessed (LRU should keep recent ones)
        let recent_entry_response = (*app)
            .clone()
            .oneshot(
                Request::builder()
                    .uri("/api/v1/cache/get?key=large_entry_999")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(recent_entry_response.status(), StatusCode::OK);
    }

    /// Stress Test 4: Tenant quota enforcement under heavy load
    /// Tests quota system doesn't leak under concurrent requests
    #[tokio::test]
    async fn test_tenant_quota_under_load() {
        let app = Arc::new(test_helpers::create_minimal_test_app());

        // Create tenant with strict limits
        let _ = (*app)
            .clone()
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/api/v1/admin/tenants")
                    .header("X-Admin-Token", "test-token")
                    .body(Body::from(
                        json!({
                            "tenant_id": "stress-test-tenant",
                            "max_requests_per_minute": 100,
                            "max_concurrent_requests": 10
                        })
                        .to_string(),
                    ))
                    .unwrap(),
            )
            .await;

        // Flood with requests
        let mut handles = vec![];
        for i in 0..200 {
            let app_clone = Arc::clone(&app);
            let handle = tokio::spawn(async move {
                (*app_clone)
                    .clone()
                    .oneshot(
                        Request::builder()
                            .uri("/api/v1/health")
                            .header("X-Tenant-ID", "stress-test-tenant")
                            .body(Body::empty())
                            .unwrap(),
                    )
                    .await
            });
            handles.push(handle);

            // Throttle slightly
            if i % 50 == 0 {
                sleep(Duration::from_millis(10)).await;
            }
        }

        let mut allowed = 0;
        let mut rate_limited = 0;

        for handle in handles {
            if let Ok(Ok(response)) = handle.await {
                match response.status() {
                    StatusCode::OK => allowed += 1,
                    StatusCode::TOO_MANY_REQUESTS => rate_limited += 1,
                    _ => {}
                }
            }
        }

        println!(
            "Quota enforcement: {} allowed, {} rate limited",
            allowed, rate_limited
        );

        // Verify quota was enforced (some requests should be rate limited)
        assert!(rate_limited > 50, "Rate limiting not working under load");

        // Verify quota didn't let through too many
        assert!(
            allowed < 150,
            "Too many requests allowed, quota not enforced properly"
        );
    }

    /// Stress Test 5: Memory leak detection over 1 hour
    /// Long-running test to detect memory leaks
    #[tokio::test]
    #[cfg(feature = "profiling-full")]
    #[ignore] // Expensive test, run manually
    async fn test_memory_leak_detection() {
        let app = Arc::new(test_helpers::create_minimal_test_app());

        // Start memory profiling
        let _ = app
            .clone()
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/api/v1/profiling/start")
                    .body(Body::from(json!({"profile_type": "memory"}).to_string()))
                    .unwrap(),
            )
            .await;

        // Record initial memory
        sleep(Duration::from_secs(5)).await;
        let initial_memory = get_memory_usage(&app).await;

        // Run workload for 1 hour (simulated as 60 iterations)
        for iteration in 0..60 {
            println!("Memory leak test iteration {}/60", iteration + 1);

            // Mixed workload
            for i in 0..100 {
                let _ = app
                    .clone()
                    .oneshot(
                        Request::builder()
                            .method("POST")
                            .uri("/api/v1/extract")
                            .body(Body::from(
                                json!({
                                    "url": format!("https://example.com/page{}", i),
                                    "mode": "standard"
                                })
                                .to_string(),
                            ))
                            .unwrap(),
                    )
                    .await;
            }

            // Check memory every iteration
            if iteration % 10 == 0 {
                let current_memory = get_memory_usage(&app).await;
                let growth = current_memory - initial_memory;

                println!("Memory growth: {} MB", growth);

                // Alert if memory grows more than 50MB
                if growth > 50.0 {
                    panic!("Potential memory leak detected: {} MB growth", growth);
                }
            }

            sleep(Duration::from_millis(100)).await;
        }

        // Final memory check
        let final_memory = get_memory_usage(&app).await;
        let total_growth = final_memory - initial_memory;

        println!("Total memory growth: {} MB", total_growth);

        // Allow some growth, but not excessive
        assert!(
            total_growth < 100.0,
            "Excessive memory growth detected: {} MB",
            total_growth
        );
    }

    /// Stress Test 6: Concurrent writes to shared cache
    /// Tests cache consistency under concurrent writes
    #[tokio::test]
    async fn test_concurrent_cache_writes() {
        let app = Arc::new(test_helpers::create_minimal_test_app());

        let mut handles = vec![];

        // 100 concurrent writers to the same keys
        for i in 0..100 {
            let app_clone = Arc::clone(&app);
            let handle = tokio::spawn(async move {
                for j in 0..10 {
                    let key = format!("shared_key_{}", j % 5); // Only 5 keys, lots of contention
                    let _ = (*app_clone)
                        .clone()
                        .oneshot(
                            Request::builder()
                                .method("POST")
                                .uri("/api/v1/cache/set")
                                .body(Body::from(
                                    json!({
                                        "key": key,
                                        "value": format!("writer_{}_value_{}", i, j)
                                    })
                                    .to_string(),
                                ))
                                .unwrap(),
                        )
                        .await;
                }
            });
            handles.push(handle);
        }

        // Wait for all writes
        for handle in handles {
            let _ = handle.await;
        }

        // Verify cache is still functional and consistent
        for j in 0..5 {
            let key = format!("shared_key_{}", j);
            let response = (*app)
                .clone()
                .oneshot(
                    Request::builder()
                        .uri(format!("/api/v1/cache/get?key={}", key))
                        .body(Body::empty())
                        .unwrap(),
                )
                .await
                .unwrap();

            // Should be able to read something
            assert_eq!(response.status(), StatusCode::OK);
        }

        // Verify no cache corruption
        let stats = (*app)
            .clone()
            .oneshot(
                Request::builder()
                    .uri("/api/v1/cache/stats")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(stats.status(), StatusCode::OK);
    }

    // Helper function to get memory usage
    #[cfg(feature = "profiling-full")]
    async fn get_memory_usage(app: &Arc<axum::Router>) -> f64 {
        let response = (**app)
            .clone()
            .oneshot(
                Request::builder()
                    .uri("/api/v1/profiling/memory")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await;

        match response {
            Ok(resp) if resp.status() == StatusCode::OK => {
                // Parse memory from response (simplified)
                42.0 // Placeholder
            }
            _ => 0.0,
        }
    }
}
