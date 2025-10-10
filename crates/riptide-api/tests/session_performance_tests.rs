//! Performance benchmarks for session middleware
//!
//! Tests measure:
//! - Session middleware overhead
//! - Rate limiter performance
//! - Concurrent session handling
//! - Memory usage patterns

#[cfg(test)]
mod performance_tests {
    use axum::{
        body::Body,
        extract::Request,
        http::{header, StatusCode},
        Router,
    };
    use riptide_api::sessions::{
        middleware::{SecurityConfig, SessionLayer, SessionRateLimiter},
        SessionConfig, SessionManager,
    };
    use std::sync::Arc;
    use std::time::{Duration, Instant};
    use tempfile::TempDir;
    use tower::ServiceExt;

    /// Helper to create test session manager
    async fn create_test_manager() -> (Arc<SessionManager>, TempDir) {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let config = SessionConfig {
            base_data_dir: temp_dir.path().to_path_buf(),
            ..Default::default()
        };

        let manager = Arc::new(
            SessionManager::new(config)
                .await
                .expect("Failed to create manager"),
        );
        (manager, temp_dir)
    }

    /// Helper to create test app
    async fn create_test_app(
        manager: Arc<SessionManager>,
        security_config: SecurityConfig,
    ) -> Router {
        use axum::routing::get;

        Router::new()
            .route("/test", get(|| async { "OK" }))
            .layer(SessionLayer::with_security_config(manager, security_config))
    }

    #[tokio::test]
    async fn benchmark_session_middleware_overhead() {
        let (manager, _temp_dir) = create_test_manager().await;

        // Create session
        let session = manager
            .create_session()
            .await
            .expect("Failed to create session");
        let session_id = session.session_id.clone();

        // Test with minimal security (no rate limiting, no expiration check)
        let security_config = SecurityConfig {
            validate_expiration: false,
            enable_rate_limiting: false,
            ..Default::default()
        };

        let app = create_test_app(manager.clone(), security_config).await;

        // Warm up
        for _ in 0..10 {
            let request = Request::builder()
                .uri("/test")
                .header(header::COOKIE, format!("riptide_session_id={}", session_id))
                .body(Body::empty())
                .unwrap();

            let _ = app.clone().oneshot(request).await;
        }

        // Benchmark 1000 requests
        let start = Instant::now();
        let iterations = 1000;

        for _ in 0..iterations {
            let request = Request::builder()
                .uri("/test")
                .header(header::COOKIE, format!("riptide_session_id={}", session_id))
                .body(Body::empty())
                .unwrap();

            let response = app.clone().oneshot(request).await.unwrap();
            assert_eq!(response.status(), StatusCode::OK);
        }

        let elapsed = start.elapsed();
        let avg_latency = elapsed.as_micros() / iterations;

        println!(
            "Session middleware overhead: {} requests in {:?}",
            iterations, elapsed
        );
        println!("Average latency: {} µs per request", avg_latency);

        // Assert reasonable performance (< 5ms p95)
        assert!(
            avg_latency < 5000,
            "Average latency should be under 5ms, got {} µs",
            avg_latency
        );
    }

    #[tokio::test]
    async fn benchmark_rate_limiter_performance() {
        let mut rate_limiter = SessionRateLimiter::new();

        let session_id = "benchmark_session";
        let max_requests = 100;
        let window = Duration::from_secs(60);

        // Benchmark rate limiter check performance
        let start = Instant::now();
        let iterations = 10_000;

        for _ in 0..iterations {
            // Reset by creating new limiter to avoid hitting limit
            let mut limiter = SessionRateLimiter::new();
            let _ = limiter.check_rate_limit(session_id, max_requests, window);
        }

        let elapsed = start.elapsed();
        let avg_latency = elapsed.as_nanos() / iterations;

        println!(
            "Rate limiter check: {} iterations in {:?}",
            iterations, elapsed
        );
        println!("Average latency: {} ns per check", avg_latency);

        // Should be very fast (< 10 µs)
        assert!(
            avg_latency < 10_000,
            "Rate limiter should be under 10µs, got {} ns",
            avg_latency
        );
    }

    #[tokio::test]
    async fn benchmark_concurrent_session_requests() {
        let (manager, _temp_dir) = create_test_manager().await;

        // Create 10 sessions
        let mut session_ids = Vec::new();
        for _ in 0..10 {
            let session = manager
                .create_session()
                .await
                .expect("Failed to create session");
            session_ids.push(session.session_id);
        }

        let security_config = SecurityConfig {
            enable_rate_limiting: false,
            ..Default::default()
        };

        // Spawn 100 concurrent tasks (10 per session)
        let start = Instant::now();
        let mut handles = vec![];

        for session_id in &session_ids {
            for _ in 0..10 {
                let app = create_test_app(manager.clone(), security_config.clone()).await;
                let sid = session_id.clone();
                let handle = tokio::spawn(async move {
                    let request = Request::builder()
                        .uri("/test")
                        .header(header::COOKIE, format!("riptide_session_id={}", sid))
                        .body(Body::empty())
                        .unwrap();

                    app.oneshot(request).await
                });
                handles.push(handle);
            }
        }

        // Wait for all to complete
        for handle in handles {
            let response = handle.await.expect("Task failed").expect("Request failed");
            assert_eq!(response.status(), StatusCode::OK);
        }

        let elapsed = start.elapsed();
        println!(
            "100 concurrent requests across 10 sessions completed in {:?}",
            elapsed
        );

        // Should complete reasonably fast (< 5 seconds)
        assert!(
            elapsed < Duration::from_secs(5),
            "Concurrent requests took too long: {:?}",
            elapsed
        );
    }

    #[tokio::test]
    async fn benchmark_session_creation() {
        let (manager, _temp_dir) = create_test_manager().await;

        let start = Instant::now();
        let iterations = 100;

        for _ in 0..iterations {
            let _ = manager
                .create_session()
                .await
                .expect("Failed to create session");
        }

        let elapsed = start.elapsed();
        let avg_latency = elapsed.as_micros() / iterations;

        println!("Session creation: {} sessions in {:?}", iterations, elapsed);
        println!("Average latency: {} µs per session", avg_latency);

        // Session creation should be fast (< 10ms each)
        assert!(
            avg_latency < 10_000,
            "Session creation should be under 10ms, got {} µs",
            avg_latency
        );
    }

    #[tokio::test]
    async fn benchmark_cookie_operations() {
        use riptide_api::sessions::Cookie;

        let (manager, _temp_dir) = create_test_manager().await;

        let session = manager
            .create_session()
            .await
            .expect("Failed to create session");
        let session_id = session.session_id.clone();

        // Benchmark cookie setting
        let start = Instant::now();
        let iterations = 100;

        for i in 0..iterations {
            let cookie = Cookie::new(format!("cookie_{}", i), format!("value_{}", i));
            manager
                .set_cookie(&session_id, "example.com", cookie)
                .await
                .expect("Failed to set cookie");
        }

        let set_elapsed = start.elapsed();
        let set_avg = set_elapsed.as_micros() / iterations;

        println!(
            "Cookie setting: {} cookies in {:?}",
            iterations, set_elapsed
        );
        println!("Average latency: {} µs per cookie set", set_avg);

        // Benchmark cookie retrieval
        let start = Instant::now();

        for i in 0..iterations {
            let _ = manager
                .get_cookie(&session_id, "example.com", &format!("cookie_{}", i))
                .await
                .expect("Failed to get cookie");
        }

        let get_elapsed = start.elapsed();
        let get_avg = get_elapsed.as_micros() / iterations;

        println!(
            "Cookie retrieval: {} cookies in {:?}",
            iterations, get_elapsed
        );
        println!("Average latency: {} µs per cookie get", get_avg);

        // Should be reasonably fast
        assert!(
            set_avg < 5000,
            "Cookie set should be under 5ms, got {} µs",
            set_avg
        );
        assert!(
            get_avg < 5000,
            "Cookie get should be under 5ms, got {} µs",
            get_avg
        );
    }

    #[tokio::test]
    async fn benchmark_rate_limiter_cleanup() {
        let mut rate_limiter = SessionRateLimiter::new();

        // Add many sessions to trigger cleanup
        let window = Duration::from_millis(100);

        for i in 0..1000 {
            rate_limiter.check_rate_limit(&format!("session_{}", i), 100, window);
        }

        let stats_before = rate_limiter.get_stats();
        println!("Before cleanup: {:?}", stats_before);

        // Wait for window to expire
        tokio::time::sleep(Duration::from_millis(150)).await;

        // Trigger cleanup by checking a session
        let start = Instant::now();
        rate_limiter.check_rate_limit("cleanup_trigger", 100, window);
        let cleanup_time = start.elapsed();

        let stats_after = rate_limiter.get_stats();
        println!("After cleanup: {:?}", stats_after);
        println!("Cleanup took: {:?}", cleanup_time);

        // Cleanup should be fast even with many sessions
        assert!(
            cleanup_time < Duration::from_millis(100),
            "Cleanup took too long: {:?}",
            cleanup_time
        );
    }

    #[tokio::test]
    async fn stress_test_session_middleware() {
        let (manager, _temp_dir) = create_test_manager().await;

        // Create 50 sessions
        let mut session_ids = Vec::new();
        for _ in 0..50 {
            let session = manager
                .create_session()
                .await
                .expect("Failed to create session");
            session_ids.push(session.session_id);
        }

        let security_config = SecurityConfig {
            enable_rate_limiting: true,
            max_requests_per_window: 50,
            rate_limit_window: Duration::from_secs(10),
            ..Default::default()
        };

        // Spawn 500 concurrent requests (10 per session)
        let start = Instant::now();
        let mut handles = vec![];

        for session_id in &session_ids {
            for _ in 0..10 {
                let app = create_test_app(manager.clone(), security_config.clone()).await;
                let sid = session_id.clone();
                let handle = tokio::spawn(async move {
                    let request = Request::builder()
                        .uri("/test")
                        .header(header::COOKIE, format!("riptide_session_id={}", sid))
                        .body(Body::empty())
                        .unwrap();

                    app.oneshot(request).await
                });
                handles.push(handle);
            }
        }

        // Collect results
        let mut success_count = 0;
        let mut failure_count = 0;

        for handle in handles {
            match handle.await {
                Ok(Ok(response)) => {
                    if response.status() == StatusCode::OK {
                        success_count += 1;
                    } else {
                        failure_count += 1;
                    }
                }
                _ => failure_count += 1,
            }
        }

        let elapsed = start.elapsed();

        println!(
            "Stress test: 500 requests across 50 sessions in {:?}",
            elapsed
        );
        println!("Success: {}, Failed: {}", success_count, failure_count);

        // Most requests should succeed
        assert!(
            success_count >= 450,
            "Expected at least 450 successful requests, got {}",
            success_count
        );
    }
}
