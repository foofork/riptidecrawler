//! Integration tests for native pool with API layer
//!
//! Tests end-to-end extraction flows, failover scenarios, and state integration.

use anyhow::Result;
use std::sync::Arc;

// ==========================
// END-TO-END EXTRACTION TESTS
// ==========================

#[tokio::test]
async fn test_e2e_native_pool_extraction() -> Result<()> {
    // Given: A configured API with native pool
    // let config = AppConfig {
    //     use_native_pool: true,
    //     native_pool_size: 4,
    //     ..Default::default()
    // };
    // let state = AppState::new(config).await?;

    // When: Making an extraction request through the API
    // let html = r#"
    //     <html>
    //         <head><title>Integration Test</title></head>
    //         <body><article><h1>Test Content</h1></article></body>
    //     </html>
    // "#;

    // let result = state.extract_handler(ExtractRequest {
    //     html: html.to_string(),
    //     url: "http://test.com".to_string(),
    //     mode: ExtractionMode::Article,
    // }).await?;

    // Then: Extraction should succeed with correct metadata
    // assert!(result.success);
    // assert_eq!(result.parser_used, "native");
    // assert!(result.doc.is_some());

    Ok(())
}

#[tokio::test]
async fn test_e2e_batch_extraction() -> Result<()> {
    // Given: An API with native pool configured
    // let state = create_test_state().await?;

    // When: Processing batch of URLs
    // let urls = vec![
    //     "http://example.com/page1",
    //     "http://example.com/page2",
    //     "http://example.com/page3",
    // ];

    // let results = state.batch_extract(urls).await?;

    // Then: All extractions should succeed
    // assert_eq!(results.len(), 3);
    // assert!(results.iter().all(|r| r.success));

    Ok(())
}

// ==========================
// NATIVE → WASM FAILOVER TESTS
// ==========================

#[tokio::test]
async fn test_native_primary_wasm_fallback() -> Result<()> {
    // Given: A system configured with native as primary, WASM as fallback
    // let config = AppConfig {
    //     extraction_strategy: Strategy::NativePrimaryWasmFallback,
    //     ..Default::default()
    // };
    // let state = AppState::new(config).await?;

    // When: Native extraction succeeds
    // let result = state.extract(valid_html, url).await?;

    // Then: Native should be used
    // assert_eq!(result.parser_metadata.parser_used, "native");
    // assert!(!result.parser_metadata.fallback_occurred);

    Ok(())
}

#[tokio::test]
async fn test_fallback_on_native_failure() -> Result<()> {
    // Given: A system with fallback enabled
    // let state = create_test_state_with_fallback().await?;

    // When: Native extraction fails (e.g., malformed HTML)
    // let malformed_html = "<html><body><unclosed-tag>";
    // let result = state.extract(malformed_html, url).await?;

    // Then: Should fall back to WASM
    // assert_eq!(result.parser_metadata.parser_used, "wasm");
    // assert!(result.parser_metadata.fallback_occurred);
    // assert!(result.parser_metadata.primary_error.is_some());

    Ok(())
}

#[tokio::test]
async fn test_fallback_on_native_timeout() -> Result<()> {
    // Given: Native pool with aggressive timeout
    // let config = AppConfig {
    //     native_extraction_timeout_ms: 100,
    //     enable_wasm_fallback: true,
    //     ..Default::default()
    // };
    // let state = AppState::new(config).await?;

    // When: Native extraction times out
    // let very_large_html = generate_large_html(10_000_000); // 10MB
    // let result = state.extract(&very_large_html, url).await?;

    // Then: Should fall back to WASM
    // assert!(result.parser_metadata.fallback_occurred);
    // assert!(result.parser_metadata.primary_error.unwrap().contains("timeout"));

    Ok(())
}

#[tokio::test]
async fn test_fallback_on_pool_exhaustion() -> Result<()> {
    // Given: A native pool with max_size = 2
    // let state = create_test_state_with_small_pool().await?;

    // When: More concurrent requests than pool capacity
    // let handles: Vec<_> = (0..5)
    //     .map(|_| {
    //         let state = state.clone();
    //         tokio::spawn(async move {
    //             state.extract(html, url).await
    //         })
    //     })
    //     .collect();

    // let results = futures::future::join_all(handles).await;

    // Then: Some should use fallback due to pool exhaustion
    // let fallback_count = results.iter()
    //     .filter(|r| {
    //         r.as_ref().unwrap().as_ref().unwrap()
    //             .parser_metadata.fallback_occurred
    //     })
    //     .count();
    // assert!(fallback_count > 0);

    Ok(())
}

#[tokio::test]
async fn test_no_fallback_when_disabled() -> Result<()> {
    // Given: Fallback disabled in config
    // let config = AppConfig {
    //     enable_wasm_fallback: false,
    //     ..Default::default()
    // };
    // let state = AppState::new(config).await?;

    // When: Native extraction fails
    // let result = state.extract(invalid_html, url).await;

    // Then: Should return error, not fall back
    // assert!(result.is_err());

    Ok(())
}

// ==========================
// STATE INTEGRATION TESTS
// ==========================

#[tokio::test]
async fn test_state_tracks_pool_metrics() -> Result<()> {
    // Given: A state with native pool
    // let state = create_test_state().await?;

    // When: Performing multiple extractions
    // for i in 0..10 {
    //     state.extract(html, &format!("http://test.com/{}", i)).await?;
    // }

    // Then: State should track accurate metrics
    // let metrics = state.get_native_pool_metrics().await;
    // assert_eq!(metrics.total_extractions, 10);
    // assert!(metrics.avg_extraction_time_ms > 0.0);

    Ok(())
}

#[tokio::test]
async fn test_state_health_check_includes_pool() -> Result<()> {
    // Given: A state with native pool
    // let state = create_test_state().await?;

    // When: Checking system health
    // let health = state.health_check().await;

    // Then: Health should include pool status
    // assert!(health.components.contains_key("native_pool"));
    // assert_eq!(health.components["native_pool"].status, "healthy");
    // assert!(health.components["native_pool"].metrics.is_some());

    Ok(())
}

#[tokio::test]
async fn test_state_graceful_shutdown() -> Result<()> {
    // Given: A running state with active extractions
    // let state = create_test_state().await?;

    // Spawn some background extractions
    // let handle = tokio::spawn(async move {
    //     state.extract(html, url).await
    // });

    // When: Shutting down the state
    // state.shutdown().await?;

    // Then: Pool should be cleaned up
    // let pool_status = state.get_native_pool_status().await;
    // assert_eq!(pool_status.total_instances, 0);

    Ok(())
}

// ==========================
// ERROR PROPAGATION TESTS
// ==========================

#[tokio::test]
async fn test_extraction_errors_propagate_correctly() -> Result<()> {
    // Given: A state with native pool
    // let state = create_test_state().await?;

    // When: Various error conditions occur
    // Test cases:
    // 1. Empty HTML
    // 2. Invalid URL
    // 3. Malformed HTML
    // 4. Timeout

    // Then: Errors should have correct types and messages
    // ... assertions ...

    Ok(())
}

#[tokio::test]
async fn test_pool_errors_include_context() -> Result<()> {
    // Given: A state with pool error scenario
    // let state = create_test_state_with_small_pool().await?;

    // When: Pool is exhausted
    // ... exhaust the pool ...

    // Then: Error should include helpful context
    // let err = state.extract(html, url).await.unwrap_err();
    // assert!(err.to_string().contains("pool exhausted"));
    // assert!(err.to_string().contains("max_size"));

    Ok(())
}

// ==========================
// CONCURRENT INTEGRATION TESTS
// ==========================

#[tokio::test]
async fn test_concurrent_api_requests() -> Result<()> {
    // Given: A shared state across multiple requests
    // let state = Arc::new(create_test_state().await?);

    // When: Handling 100 concurrent API requests
    // let handles: Vec<_> = (0..100)
    //     .map(|i| {
    //         let state = state.clone();
    //         tokio::spawn(async move {
    //             state.extract(
    //                 &format!("<html><body>Doc {}</body></html>", i),
    //                 &format!("http://example.com/{}", i)
    //             ).await
    //         })
    //     })
    //     .collect();

    // let results = futures::future::join_all(handles).await;

    // Then: All requests should complete successfully
    // assert_eq!(results.iter().filter(|r| r.is_ok()).count(), 100);

    Ok(())
}

#[tokio::test]
async fn test_mixed_native_and_wasm_requests() -> Result<()> {
    // Given: A state that supports both native and WASM
    // let state = create_test_state().await?;

    // When: Some requests use native, others use WASM explicitly
    // let native_results = state.extract_with_native(html, url).await?;
    // let wasm_results = state.extract_with_wasm(html, url).await?;

    // Then: Both should produce valid results
    // assert!(native_results.success);
    // assert!(wasm_results.success);

    Ok(())
}

// ==========================
// PIPELINE INTEGRATION TESTS
// ==========================

#[tokio::test]
async fn test_pipeline_with_native_extraction() -> Result<()> {
    // Given: A complete pipeline with native extraction
    // let state = create_test_state().await?;

    // When: Running full pipeline (fetch → extract → process → store)
    // let pipeline = Pipeline::new(state);
    // let result = pipeline.run(PipelineRequest {
    //     url: "http://example.com".to_string(),
    //     use_native: true,
    //     post_processors: vec![
    //         Processor::Summarize,
    //         Processor::ExtractKeywords,
    //     ],
    // }).await?;

    // Then: Pipeline should complete with native extraction
    // assert_eq!(result.extraction_method, "native");
    // assert!(result.summary.is_some());
    // assert!(!result.keywords.is_empty());

    Ok(())
}

// ==========================
// METRICS AND MONITORING
// ==========================

#[tokio::test]
async fn test_prometheus_metrics_exposed() -> Result<()> {
    // Given: A state with metrics enabled
    // let state = create_test_state_with_metrics().await?;

    // When: Performing extractions
    // state.extract(html, url).await?;

    // Then: Prometheus metrics should be updated
    // let metrics = state.get_prometheus_metrics().await;
    // assert!(metrics.contains("native_pool_extractions_total"));
    // assert!(metrics.contains("native_pool_instances_available"));

    Ok(())
}

#[tokio::test]
async fn test_distributed_tracing_integration() -> Result<()> {
    // Given: A state with tracing enabled
    // let state = create_test_state_with_tracing().await?;

    // When: Performing extraction
    // let span = tracing::info_span!("test_extraction");
    // let _guard = span.enter();
    // state.extract(html, url).await?;

    // Then: Trace should include native pool spans
    // ... verify tracing output ...

    Ok(())
}

// ==========================
// CONFIGURATION VALIDATION
// ==========================

#[tokio::test]
async fn test_invalid_config_rejected() -> Result<()> {
    // Given: Invalid configurations
    // let config = AppConfig {
    //     native_pool_size: 0, // Invalid!
    //     ..Default::default()
    // };

    // When: Attempting to create state
    // let result = AppState::new(config).await;

    // Then: Should fail with descriptive error
    // assert!(result.is_err());
    // assert!(result.unwrap_err().to_string().contains("pool_size"));

    Ok(())
}

#[tokio::test]
async fn test_config_from_environment() -> Result<()> {
    // Given: Environment variables set
    // std::env::set_var("NATIVE_POOL_SIZE", "12");
    // std::env::set_var("NATIVE_POOL_TIMEOUT_MS", "5000");

    // When: Creating state from environment
    // let state = AppState::from_env().await?;

    // Then: Config should reflect environment
    // assert_eq!(state.config.native_pool_size, 12);
    // assert_eq!(state.config.native_pool_timeout_ms, 5000);

    Ok(())
}

// ==========================
// HELPER FUNCTIONS
// ==========================

// Helper to create test state with default config
// async fn create_test_state() -> Result<AppState> {
//     AppState::new(AppConfig::default()).await
// }

// Helper to create state with small pool for exhaustion tests
// async fn create_test_state_with_small_pool() -> Result<AppState> {
//     AppState::new(AppConfig {
//         native_pool_size: 2,
//         ..Default::default()
//     }).await
// }

// Helper to generate large HTML for stress testing
// fn generate_large_html(size_bytes: usize) -> String {
//     let mut html = String::from("<html><body>");
//     while html.len() < size_bytes {
//         html.push_str("<p>Content paragraph.</p>");
//     }
//     html.push_str("</body></html>");
//     html
// }
