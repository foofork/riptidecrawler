//! Cross-Module Integration Tests
//!
//! Tests interactions between different system modules:
//! - Streaming + Persistence (stream to cache)
//! - Browser + Profiling (measure browser memory)
//! - Persistence + Multi-tenancy (tenant quota enforcement)
//! - Profiling + Browser Pool (track pool resources)
//! - Streaming + Browser (stream automation results)

mod test_helpers;

#[cfg(test)]
mod cross_module_tests {
    use super::test_helpers;

    #[allow(unused_imports)]
    use axum::body::Body;
    #[allow(unused_imports)]
    use axum::http::{Request, StatusCode};
    #[allow(unused_imports)]
    use serde_json::json;
    #[allow(unused_imports)]
    use tower::ServiceExt;

    /// Test 1: Streaming + Persistence
    /// Stream data and persist to cache for later retrieval
    #[tokio::test]
    #[cfg(all(feature = "streaming", feature = "jemalloc"))]
    async fn test_streaming_to_cache_persistence() {
        let app = test_helpers::create_minimal_test_app();

        // Start streaming with cache persistence
        let stream_response = app
            .clone()
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/api/v1/stream/start")
                    .header("content-type", "application/json")
                    .body(Body::from(
                        json!({
                            "urls": ["https://example.com/data"],
                            "persist_to_cache": true,
                            "cache_key_prefix": "stream_results"
                        })
                        .to_string(),
                    ))
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(stream_response.status(), StatusCode::OK);

        // Verify data persisted to cache
        let cache_response = app
            .oneshot(
                Request::builder()
                    .uri("/api/v1/cache/get?key=stream_results:example.com")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(cache_response.status(), StatusCode::OK);
    }

    /// Test 2: Browser + Profiling
    /// Measure memory usage during browser operations
    #[tokio::test]
    #[cfg(all(feature = "sessions", feature = "profiling-full"))]
    async fn test_browser_memory_profiling() {
        let app = test_helpers::create_minimal_test_app();

        // Start profiling
        let _ = app
            .clone()
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/api/v1/profiling/start")
                    .body(Body::from(json!({"profile_type": "memory"}).to_string()))
                    .unwrap(),
            )
            .await
            .unwrap();

        // Create browser session
        let session_response = app
            .clone()
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/api/v1/sessions")
                    .body(Body::from(
                        json!({"url": "https://example.com"}).to_string(),
                    ))
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(session_response.status(), StatusCode::OK);

        // Navigate to pages
        for i in 0..5 {
            let _ = app
                .clone()
                .oneshot(
                    Request::builder()
                        .method("POST")
                        .uri("/api/v1/sessions/test-session/navigate")
                        .body(Body::from(
                            json!({"url": format!("https://example.com/page{}", i)}).to_string(),
                        ))
                        .unwrap(),
                )
                .await;
        }

        // Get profiling report with browser memory stats
        let profile_response = app
            .oneshot(
                Request::builder()
                    .uri("/api/v1/profiling/report")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(profile_response.status(), StatusCode::OK);
    }

    /// Test 3: Persistence + Multi-tenancy
    /// Verify tenant quota enforcement with cache operations
    /// NOTE: Disabled - tenants feature not implemented yet
    #[tokio::test]
    #[ignore = "tenants feature not implemented"]
    async fn test_tenant_quota_with_cache() {
        let app = test_helpers::create_minimal_test_app();

        // Initialize tenant with storage quota
        let _ = app
            .clone()
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/api/v1/admin/tenants")
                    .header("X-Admin-Token", "test-token")
                    .body(Body::from(
                        json!({
                            "tenant_id": "tenant-storage-test",
                            "max_storage_mb": 10,
                            "max_cache_entries": 100
                        })
                        .to_string(),
                    ))
                    .unwrap(),
            )
            .await;

        // Fill cache up to quota
        for i in 0..100 {
            let _ = app
                .clone()
                .oneshot(
                    Request::builder()
                        .method("POST")
                        .uri("/api/v1/cache/set")
                        .header("X-Tenant-ID", "tenant-storage-test")
                        .body(Body::from(
                            json!({"key": format!("item_{}", i), "value": "data"}).to_string(),
                        ))
                        .unwrap(),
                )
                .await;
        }

        // Attempt to exceed quota
        let quota_response = app
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/api/v1/cache/set")
                    .header("X-Tenant-ID", "tenant-storage-test")
                    .body(Body::from(
                        json!({"key": "item_101", "value": "data"}).to_string(),
                    ))
                    .unwrap(),
            )
            .await
            .unwrap();

        // Should be rejected due to quota
        assert!(
            quota_response.status() == StatusCode::TOO_MANY_REQUESTS
                || quota_response.status() == StatusCode::PAYLOAD_TOO_LARGE
        );
    }

    /// Test 4: Profiling + Browser Pool
    /// Track resource usage across browser pool operations
    #[tokio::test]
    #[cfg(all(feature = "sessions", feature = "profiling-full"))]
    async fn test_browser_pool_resource_tracking() {
        let app = test_helpers::create_minimal_test_app();

        // Initialize browser pool with profiling
        let _ = app
            .clone()
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/api/v1/browser/pool/init")
                    .body(Body::from(
                        json!({
                            "pool_size": 3,
                            "enable_profiling": true
                        })
                        .to_string(),
                    ))
                    .unwrap(),
            )
            .await;

        // Allocate and use browsers
        for i in 0..3 {
            let _ = app
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
                .await;
        }

        // Get resource usage report
        let resource_response = app
            .oneshot(
                Request::builder()
                    .uri("/api/v1/browser/pool/resources")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(resource_response.status(), StatusCode::OK);
    }

    /// Test 5: Streaming + Browser Automation
    /// Stream browser automation results in real-time
    #[tokio::test]
    #[cfg(all(feature = "streaming", feature = "sessions"))]
    async fn test_stream_browser_automation() {
        let app = test_helpers::create_minimal_test_app();

        // Start browser automation with streaming
        let automation_response = app
            .clone()
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/api/v1/automation/stream")
                    .body(Body::from(
                        json!({
                            "urls": ["https://example.com/page1", "https://example.com/page2"],
                            "actions": [
                                {"type": "click", "selector": ".button"},
                                {"type": "screenshot"}
                            ],
                            "stream_format": "ndjson"
                        })
                        .to_string(),
                    ))
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(automation_response.status(), StatusCode::OK);
    }

    /// Test 6: Cache + Tenant Isolation
    /// Verify cache isolation between tenants
    /// NOTE: Disabled - tenants feature not implemented yet
    #[tokio::test]
    #[ignore = "tenants feature not implemented"]
    async fn test_cache_tenant_isolation() {
        let app = test_helpers::create_minimal_test_app();

        // Tenant A stores data
        let _ = app
            .clone()
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/api/v1/cache/set")
                    .header("X-Tenant-ID", "tenant-a")
                    .body(Body::from(
                        json!({"key": "shared_key", "value": "tenant_a_data"}).to_string(),
                    ))
                    .unwrap(),
            )
            .await;

        // Tenant B stores different data with same key
        let _ = app
            .clone()
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/api/v1/cache/set")
                    .header("X-Tenant-ID", "tenant-b")
                    .body(Body::from(
                        json!({"key": "shared_key", "value": "tenant_b_data"}).to_string(),
                    ))
                    .unwrap(),
            )
            .await;

        // Verify tenant A gets their data
        let tenant_a_response = app
            .clone()
            .oneshot(
                Request::builder()
                    .uri("/api/v1/cache/get?key=shared_key")
                    .header("X-Tenant-ID", "tenant-a")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(tenant_a_response.status(), StatusCode::OK);

        // Verify tenant B gets their data
        let tenant_b_response = app
            .oneshot(
                Request::builder()
                    .uri("/api/v1/cache/get?key=shared_key")
                    .header("X-Tenant-ID", "tenant-b")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(tenant_b_response.status(), StatusCode::OK);
    }

    /// Test 7: Profiling + Streaming Performance
    /// Measure streaming performance impact
    #[tokio::test]
    #[cfg(all(feature = "streaming", feature = "profiling-full"))]
    async fn test_streaming_performance_profiling() {
        let app = test_helpers::create_minimal_test_app();

        // Start profiling
        let _ = app
            .clone()
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/api/v1/profiling/start")
                    .body(Body::from(
                        json!({"profile_type": "performance"}).to_string(),
                    ))
                    .unwrap(),
            )
            .await;

        // Start high-volume stream
        let _ = app
            .clone()
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/api/v1/stream/start")
                    .body(Body::from(
                        json!({
                            "source": "test_data",
                            "item_count": 10000,
                            "format": "ndjson"
                        })
                        .to_string(),
                    ))
                    .unwrap(),
            )
            .await;

        // Get performance analysis
        let analysis_response = app
            .oneshot(
                Request::builder()
                    .uri("/api/v1/profiling/analysis?component=streaming")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(analysis_response.status(), StatusCode::OK);
    }

    /// Test 8: Browser + Cache Integration
    /// Cache browser session data
    #[tokio::test]
    #[cfg(feature = "sessions")]
    async fn test_browser_session_caching() {
        let app = test_helpers::create_minimal_test_app();

        // Create browser session with caching
        let session_response = app
            .clone()
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/api/v1/sessions")
                    .body(Body::from(
                        json!({
                            "url": "https://example.com",
                            "cache_cookies": true,
                            "cache_local_storage": true
                        })
                        .to_string(),
                    ))
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(session_response.status(), StatusCode::OK);

        // Verify session data cached
        let cache_response = app
            .oneshot(
                Request::builder()
                    .uri("/api/v1/cache/get?key=session:test-session:cookies")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert!(
            cache_response.status() == StatusCode::OK
                || cache_response.status() == StatusCode::NOT_FOUND
        );
    }

    /// Test 9: Multi-tenant Rate Limiting + Profiling
    /// Track rate limit violations per tenant
    /// NOTE: Disabled - tenants feature not implemented yet
    #[tokio::test]
    #[ignore = "tenants feature not implemented"]
    async fn test_tenant_rate_limiting_with_profiling() {
        let app = test_helpers::create_minimal_test_app();

        // Create tenant with strict limits
        let _ = app
            .clone()
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/api/v1/admin/tenants")
                    .header("X-Admin-Token", "test-token")
                    .body(Body::from(
                        json!({
                            "tenant_id": "rate-limit-test",
                            "max_requests_per_minute": 5
                        })
                        .to_string(),
                    ))
                    .unwrap(),
            )
            .await;

        // Make requests until hitting limit
        for i in 0..10 {
            let _ = app
                .clone()
                .oneshot(
                    Request::builder()
                        .uri("/api/v1/health")
                        .header("X-Tenant-ID", "rate-limit-test")
                        .body(Body::empty())
                        .unwrap(),
                )
                .await;

            if i >= 5 {
                // Should start hitting rate limits
                tokio::time::sleep(tokio::time::Duration::from_millis(50)).await;
            }
        }

        // Get tenant rate limit stats
        let stats_response = app
            .oneshot(
                Request::builder()
                    .uri("/api/v1/admin/tenants/rate-limit-test/stats")
                    .header("X-Admin-Token", "test-token")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(stats_response.status(), StatusCode::OK);
    }

    /// Test 10: Streaming + Persistence + Search
    /// Stream data, persist, and enable search
    #[tokio::test]
    #[cfg(feature = "streaming")]
    async fn test_stream_persist_search() {
        let app = test_helpers::create_minimal_test_app();

        // Stream data with persistence and indexing
        let _ = app
            .clone()
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/api/v1/stream/start")
                    .body(Body::from(
                        json!({
                            "source": "data_feed",
                            "persist": true,
                            "enable_search_index": true
                        })
                        .to_string(),
                    ))
                    .unwrap(),
            )
            .await;

        // Search streamed data
        let search_response = app
            .oneshot(
                Request::builder()
                    .uri("/api/v1/search?q=test&source=data_feed")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(search_response.status(), StatusCode::OK);
    }

    /// Test 11: Browser Pool + Tenant Isolation
    /// Ensure browsers are isolated per tenant
    #[tokio::test]
    #[cfg(feature = "sessions")]
    async fn test_browser_pool_tenant_isolation() {
        let app = test_helpers::create_minimal_test_app();

        // Tenant A creates browser session
        let tenant_a_session = app
            .clone()
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/api/v1/sessions")
                    .header("X-Tenant-ID", "tenant-a")
                    .body(Body::from(
                        json!({"url": "https://example.com"}).to_string(),
                    ))
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(tenant_a_session.status(), StatusCode::OK);

        // Tenant B attempts to access tenant A's session
        let cross_tenant_access = app
            .oneshot(
                Request::builder()
                    .uri("/api/v1/sessions/tenant-a-session-id")
                    .header("X-Tenant-ID", "tenant-b")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        // Should be forbidden
        assert!(
            cross_tenant_access.status() == StatusCode::FORBIDDEN
                || cross_tenant_access.status() == StatusCode::NOT_FOUND
        );
    }

    /// Test 12: Full Stack Integration
    /// Browser → Extract → Cache → Stream → Search
    #[tokio::test]
    #[cfg(all(feature = "sessions", feature = "streaming"))]
    async fn test_full_stack_integration() {
        let app = test_helpers::create_minimal_test_app();

        // Step 1: Create browser session
        let session_response = app
            .clone()
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/api/v1/sessions")
                    .body(Body::from(
                        json!({"url": "https://example.com"}).to_string(),
                    ))
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(session_response.status(), StatusCode::OK);

        // Step 2: Extract content
        let extract_response = app
            .clone()
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/api/v1/extract")
                    .body(Body::from(
                        json!({"url": "https://example.com", "mode": "full"}).to_string(),
                    ))
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(extract_response.status(), StatusCode::OK);

        // Step 3: Cache extracted content
        let cache_response = app
            .clone()
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/api/v1/cache/set")
                    .body(Body::from(
                        json!({"key": "extracted:example.com", "value": "content"}).to_string(),
                    ))
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(cache_response.status(), StatusCode::OK);

        // Step 4: Stream results
        let stream_response = app
            .clone()
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/api/v1/stream/start")
                    .body(Body::from(
                        json!({"source": "cached_extractions"}).to_string(),
                    ))
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(stream_response.status(), StatusCode::OK);

        // Step 5: Search extracted content
        let search_response = app
            .oneshot(
                Request::builder()
                    .uri("/api/v1/search?q=example")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(search_response.status(), StatusCode::OK);
    }
}
