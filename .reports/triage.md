# Triage: Underscore Bindings & TODOs

**Generated:** 2025-10-07 07:46:22

Fill the **Decision** column using one of:

- `side_effect_only` · `promote_and_use` · `handle_result` · `guard_lifetime` · `detached_ok` · `wire_config` · `todo_tasked` · `axum_handler`

---

## Crate: `playground`

### TODOs

| ✔ | File | Line | Variable | Snippet / Context | Initial | Decision | Notes |
|:-:|:-----|-----:|:--------:|:------------------|:--------|:---------|:------|
|   | `./playground/src/pages/Examples.jsx` | 396 | `—` | `// TODO: Implement loading example into playground` | `todo` |  |  |

## Crate: `riptide-api`

### Underscore lets

| ✔ | File | Line | Variable | Snippet / Context | Initial | Decision | Notes |
|:-:|:-----|-----:|:--------:|:------------------|:--------|:---------|:------|
|   | `./crates/riptide-api/src/handlers/render/handlers.rs` | 22 | `_start_time` | `Instant::now()` | `review` |  |  |
|   | `./crates/riptide-api/src/handlers/render/processors.rs` | 83 | `_user_agent` | `stealth.next_user_agent()` | `review` |  |  |
|   | `./crates/riptide-api/src/handlers/render/processors.rs` | 84 | `_headers` | `stealth.generate_headers()` | `review` |  |  |
|   | `./crates/riptide-api/src/handlers/render/processors.rs` | 85 | `_delay` | `stealth.calculate_delay()` | `review` |  |  |
|   | `./crates/riptide-api/src/health.rs` | 151 | `_timestamp` | `chrono::Utc::now().to_rfc3339()` | `review` |  |  |
|   | `./crates/riptide-api/src/metrics.rs` | 683 | `_messages_sent_diff` | `streaming_metrics.total_messages_sent as f64` | `review` |  |  |
|   | `./crates/riptide-api/src/metrics.rs` | 684 | `_messages_dropped_diff` | `streaming_metrics.total_messages_dropped as f64` | `review` |  |  |
|   | `./crates/riptide-api/src/pipeline.rs` | 498 | `_pdf_config` | `self.options.pdf_config.clone().unwrap_or_default()` | `review` |  |  |
|   | `./crates/riptide-api/src/pipeline_enhanced.rs` | 97 | `_permit` | `semaphore.acquire().await.ok()?` | `handle_result?` |  |  |
|   | `./crates/riptide-api/src/routes/stealth.rs` | 35 | `_controller` | `StealthController::from_preset(StealthPreset::Medium)` | `review` |  |  |
|   | `./crates/riptide-api/tests/benchmarks/performance_tests.rs` | 155 | `_body` | `axum::body::to_bytes(response.into_body(), usize::MAX).await.unwrap()` | `review` |  |  |
|   | `./crates/riptide-api/tests/benchmarks/performance_tests.rs` | 178 | `_permit` | `semaphore.acquire().await.unwrap()` | `review` |  |  |
|   | `./crates/riptide-api/tests/benchmarks/performance_tests.rs` | 197 | `_body` | `axum::body::to_bytes(response.into_body(), usize::MAX).await.unwrap()` | `review` |  |  |
|   | `./crates/riptide-api/tests/pdf_integration_tests.rs` | 458 | `_memory_stream` | `monitor.start_monitoring(Duration::from_millis(100)).await` | `review` |  |  |
|   | `./crates/riptide-api/tests/unit/test_state.rs` | 218 | `_cloned` | `status.clone()` | `review` |  |  |
|   | `./crates/riptide-api/tests/unit/test_validation.rs` | 710 | `_result` | `validate_crawl_request(&body)` | `review` |  |  |
|   | `./crates/riptide-api/tests/unit/test_validation.rs` | 725 | `_result` | `validate_crawl_request(&body)` | `review` |  |  |

### TODOs

| ✔ | File | Line | Variable | Snippet / Context | Initial | Decision | Notes |
|:-:|:-----|-----:|:--------:|:------------------|:--------|:---------|:------|
|   | `./crates/riptide-api/src/errors.rs` | 30 | `—` | `#[allow(dead_code)] // TODO: Implement authentication middleware` | `todo` |  |  |
|   | `./crates/riptide-api/src/handlers/fetch.rs` | 15 | `—` | `// TODO: Fix method resolution issue with Arc<FetchEngine>` | `todo` |  |  |
|   | `./crates/riptide-api/src/handlers/monitoring.rs` | 219 | `—` | `// TODO: Implement memory profiling integration` | `todo` |  |  |
|   | `./crates/riptide-api/src/handlers/monitoring.rs` | 236 | `—` | `// TODO: Implement leak detection integration` | `todo` |  |  |
|   | `./crates/riptide-api/src/handlers/monitoring.rs` | 252 | `—` | `// TODO: Implement allocation analysis integration` | `todo` |  |  |
|   | `./crates/riptide-api/src/handlers/pdf.rs` | 462 | `—` | `#[allow(dead_code)] // TODO: Implement multipart PDF upload support` | `todo` |  |  |
|   | `./crates/riptide-api/src/handlers/render/processors.rs` | 86 | `—` | `// TODO: Apply these to the actual headless browser` | `todo` |  |  |
|   | `./crates/riptide-api/src/handlers/render/processors.rs` | 132 | `—` | `// TODO: Pass session context to RPC client for browser state persistence` | `todo` |  |  |
|   | `./crates/riptide-api/src/handlers/shared/mod.rs` | 103 | `—` | `// TODO: Apply CrawlOptions to spider config` | `todo` |  |  |
|   | `./crates/riptide-api/src/handlers/telemetry.rs` | 165 | `—` | `State(_state): State<AppState>, // TODO: Wire up to actual trace backend` | `todo` |  |  |
|   | `./crates/riptide-api/src/handlers/telemetry.rs` | 207 | `—` | `State(_state): State<AppState>, // TODO: Wire up to actual trace backend` | `todo` |  |  |
|   | `./crates/riptide-api/src/handlers/telemetry.rs` | 357 | `—` | `State(_state): State<AppState>, // TODO: Use state for runtime telemetry info` | `todo` |  |  |
|   | `./crates/riptide-api/src/health.rs` | 38 | `—` | `component_versions.insert("riptide-core".to_string(), "0.1.0".to_string()); /...` | `todo` |  |  |
|   | `./crates/riptide-api/src/health.rs` | 174 | `—` | `spider_engine: None, // TODO: Implement spider health check` | `todo` |  |  |
|   | `./crates/riptide-api/src/pipeline_enhanced.rs` | 1 | `—` | `// TODO: Enhanced pipeline orchestrator prepared for production use` | `todo` |  |  |
|   | `./crates/riptide-api/src/rpc_client.rs` | 55 | `—` | `session_id: None, // TODO: Implement session persistence` | `todo` |  |  |
|   | `./crates/riptide-api/src/state.rs` | 1015 | `—` | `// TODO: Publish alerts to event bus` | `todo` |  |  |
|   | `./crates/riptide-api/src/state.rs` | 1068 | `—` | `// TODO: Publish to event bus` | `todo` |  |  |
|   | `./crates/riptide-api/src/streaming/buffer.rs` | 1 | `—` | `// TODO: Streaming infrastructure - will be activated when routes are added` | `todo` |  |  |
|   | `./crates/riptide-api/src/streaming/config.rs` | 1 | `—` | `// TODO: Streaming infrastructure - will be activated when routes are added` | `todo` |  |  |
|   | `./crates/riptide-api/src/streaming/error.rs` | 1 | `—` | `// TODO: Streaming infrastructure - will be activated when routes are added` | `todo` |  |  |
|   | `./crates/riptide-api/src/streaming/lifecycle.rs` | 1 | `—` | `// TODO: Streaming infrastructure - will be activated when routes are added` | `todo` |  |  |
|   | `./crates/riptide-api/src/streaming/ndjson/streaming.rs` | 3 | `—` | `// TODO: Streaming infrastructure - will be activated when routes are added` | `todo` |  |  |
|   | `./crates/riptide-api/src/streaming/pipeline.rs` | 1 | `—` | `// TODO: Streaming pipeline infrastructure prepared but routes not yet activated` | `todo` |  |  |
|   | `./crates/riptide-api/src/streaming/processor.rs` | 1 | `—` | `// TODO: Streaming infrastructure prepared but routes not yet activated` | `todo` |  |  |
|   | `./crates/riptide-api/tests/integration_tests.rs` | 28 | `—` | `/// TODO: This will need to be implemented to create the actual app with all ...` | `todo` |  |  |
|   | `./crates/riptide-api/tests/integration_tests.rs` | 308 | `—` | `// TODO: Validate CSV content structure` | `todo` |  |  |
|   | `./crates/riptide-api/tests/integration_tests.rs` | 338 | `—` | `// TODO: Validate Markdown table format` | `todo` |  |  |
|   | `./crates/riptide-api/tests/integration_tests.rs` | 798 | `—` | `// TODO: Test actual failover behavior` | `todo` |  |  |

## Crate: `riptide-core`

### Underscore lets

| ✔ | File | Line | Variable | Snippet / Context | Initial | Decision | Notes |
|:-:|:-----|-----:|:--------:|:------------------|:--------|:---------|:------|
|   | `./crates/riptide-core/benches/performance_benches.rs` | 288 | `_span` | `tracer.start("benchmark_span")` | `review` |  |  |
|   | `./crates/riptide-core/src/ai_processor.rs` | 189 | `_permit` | `semaphore.acquire().await.unwrap()` | `review` |  |  |
|   | `./crates/riptide-core/src/benchmarks.rs` | 200 | `_extractor` | `CmExtractor::new(config.clone())` | `review` |  |  |
|   | `./crates/riptide-core/src/cache_warming_integration.rs` | 266 | `_engine` | `Engine::default()` | `review` |  |  |
|   | `./crates/riptide-core/src/cache_warming_integration.rs` | 267 | `_config` | `ExtractorConfig::default()` | `review` |  |  |
|   | `./crates/riptide-core/src/circuit.rs` | 281 | `_permit` | `cb.try_acquire().expect("should get permit")` | `review` |  |  |
|   | `./crates/riptide-core/src/events/handlers.rs` | 54 | `_metadata` | `event.metadata()` | `review` |  |  |
|   | `./crates/riptide-core/src/events/pool_integration.rs` | 490 | `_factory` | `EventAwarePoolFactory::new(event_bus)` | `review` |  |  |
|   | `./crates/riptide-core/src/fetch.rs` | 153 | `_overall_start` | `std::time::Instant::now()` | `review` |  |  |
|   | `./crates/riptide-core/src/fetch.rs` | 157 | `_robots_span` | `telemetry_span!("robots_check", url = %url)` | `review` |  |  |
|   | `./crates/riptide-core/src/fetch.rs` | 719 | `_client` | `http_client()` | `review` |  |  |
|   | `./crates/riptide-core/src/instance_pool_tests.rs` | 83 | `_final_permit` | `semaphore.try_acquire().unwrap()` | `review` |  |  |
|   | `./crates/riptide-core/src/monitoring/error.rs` | 57 | `_guard` | `poison_err.into_inner()` | `review` |  |  |
|   | `./crates/riptide-core/src/security.rs` | 444 | `_middleware` | `SecurityMiddleware::with_defaults().expect("Failed to create middleware")` | `review` |  |  |
|   | `./crates/riptide-core/src/spider/core.rs` | 387 | `_metrics` | `self.adaptive_stop_engine.analyze_result(&result).await?` | `handle_result?` |  |  |
|   | `./crates/riptide-core/src/spider/frontier.rs` | 474 | `_now` | `Instant::now()` | `review` |  |  |
|   | `./crates/riptide-core/src/spider/query_aware.rs` | 496 | `_url` | `Url::from_str("https://example.com/page").unwrap()` | `review` |  |  |
|   | `./crates/riptide-core/src/spider/query_aware_benchmark.rs` | 171 | `_score` | `analyzer.score(&url, i + 1)` | `review` |  |  |
|   | `./crates/riptide-core/src/spider/query_aware_benchmark.rs` | 577 | `_results` | `benchmark.run_full_benchmark()` | `review` |  |  |
|   | `./crates/riptide-core/src/spider/tests.rs` | 23 | `_seeds` | `[Url::from_str("https://example.com/").expect("Valid URL")]` | `review` |  |  |
|   | `./crates/riptide-core/src/strategies/manager.rs` | 134 | `_start` | `std::time::Instant::now()` | `review` |  |  |
|   | `./crates/riptide-core/tests/memory_manager_tests.rs` | 97 | `_stats` | `manager_clone.stats()` | `review` |  |  |
|   | `./crates/riptide-core/tests/memory_manager_tests.rs` | 153 | `_stats` | `manager.stats()` | `review` |  |  |
|   | `./crates/riptide-core/tests/memory_manager_tests.rs` | 240 | `_stats` | `manager.stats()` | `review` |  |  |
|   | `./crates/riptide-core/tests/pdf_pipeline_tests.rs` | 295 | `_updated_metrics` | `pipeline.get_metrics_snapshot()` | `review` |  |  |

### TODOs

| ✔ | File | Line | Variable | Snippet / Context | Initial | Decision | Notes |
|:-:|:-----|-----:|:--------:|:------------------|:--------|:---------|:------|
|   | `./crates/riptide-core/src/ai_processor.rs` | 406 | `—` | `/// TODO: Integrate with LLM client pool` | `todo` |  |  |
|   | `./crates/riptide-core/src/events/pool_integration.rs` | 452 | `—` | `pending_acquisitions: 0, // TODO: Get from pool if available` | `todo` |  |  |
|   | `./crates/riptide-core/src/events/pool_integration.rs` | 493 | `—` | `// TODO: Add actual test logic when WASM components are available` | `todo` |  |  |
|   | `./crates/riptide-core/src/fetch.rs` | 829 | `—` | `// TODO: Implement test_retryable_error_detection` | `todo` |  |  |
|   | `./crates/riptide-core/src/instance_pool/pool.rs` | 259 | `—` | `// TODO: Implement fallback to native extraction if needed` | `todo` |  |  |
|   | `./crates/riptide-core/src/instance_pool/pool.rs` | 874 | `—` | `pending_acquisitions: 0, // TODO: Track this if needed` | `todo` |  |  |
|   | `./crates/riptide-core/src/memory_manager.rs` | 192 | `—` | `#[allow(dead_code)] // TODO: wire into metrics` | `todo` |  |  |
|   | `./crates/riptide-core/src/memory_manager.rs` | 201 | `—` | `#[allow(dead_code)] // TODO: send stats summary at end-of-run` | `todo` |  |  |
|   | `./crates/riptide-core/src/monitoring/reports.rs` | 138 | `—` | `let _half_duration = duration / 2; // TODO: use for percentile calc or remove` | `todo` |  |  |
|   | `./crates/riptide-core/src/spider/sitemap.rs` | 153 | `—` | `// TODO: Check robots.txt for sitemap entries` | `todo` |  |  |
|   | `./crates/riptide-core/src/telemetry.rs` | 221 | `—` | `"${1}XXX".to_string(),` | `todo` |  |  |
|   | `./crates/riptide-core/src/telemetry.rs` | 399 | `—` | `// TODO: Implement proper percentile calculation with histogram` | `todo` |  |  |
|   | `./crates/riptide-core/src/telemetry.rs` | 549 | `—` | `disk_usage_bytes: 0, // TODO: Implement disk usage tracking` | `todo` |  |  |
|   | `./crates/riptide-core/src/telemetry.rs` | 552 | `—` | `open_file_descriptors: 0, // TODO: Implement FD tracking` | `todo` |  |  |
|   | `./crates/riptide-core/src/telemetry.rs` | 609 | `—` | `assert!(sanitized.contains("192.168.1.XXX"));` | `todo` |  |  |

## Crate: `riptide-headless`

### Underscore lets

| ✔ | File | Line | Variable | Snippet / Context | Initial | Decision | Notes |
|:-:|:-----|-----:|:--------:|:------------------|:--------|:---------|:------|
|   | `./crates/riptide-headless/src/cdp.rs` | 84 | `_timeout` | `Duration::from_millis(timeout_ms.unwrap_or(5000))` | `review` |  |  |
|   | `./crates/riptide-headless/src/launcher.rs` | 274 | `_stealth_controller` | `self.stealth_controller.read().await` | `review` |  |  |
|   | `./crates/riptide-headless/tests/headless_tests.rs` | 85 | `_browser_id` | `checkout.browser_id().to_string()` | `review` |  |  |
|   | `./crates/riptide-headless/tests/headless_tests.rs` | 218 | `_checkout` | `pool.checkout().await.unwrap()` | `review` |  |  |
|   | `./crates/riptide-headless/tests/headless_tests.rs` | 310 | `_events` | `launcher.pool_events()` | `review` |  |  |

## Crate: `riptide-html`

### Underscore lets

| ✔ | File | Line | Variable | Snippet / Context | Initial | Decision | Notes |
|:-:|:-----|-----:|:--------:|:------------------|:--------|:---------|:------|
|   | `./crates/riptide-html/tests/topic_chunking_integration.rs` | 318 | `_has_text` | `chunks.iter().any(\|c\| !c.content.contains('<'))` | `review` |  |  |

### TODOs

| ✔ | File | Line | Variable | Snippet / Context | Initial | Decision | Notes |
|:-:|:-----|-----:|:--------:|:------------------|:--------|:---------|:------|
|   | `./crates/riptide-html/src/table_extraction/extractor.rs` | 107 | `—` | `sub_headers: Vec::new(), // TODO: Implement multi-level headers` | `todo` |  |  |
|   | `./crates/riptide-html/src/wasm_extraction.rs` | 342 | `—` | `// TODO: Implement actual WASM component binding and invocation` | `todo` |  |  |

## Crate: `riptide-intelligence`

### Underscore lets

| ✔ | File | Line | Variable | Snippet / Context | Initial | Decision | Notes |
|:-:|:-----|-----:|:--------:|:------------------|:--------|:---------|:------|
|   | `./crates/riptide-intelligence/src/circuit_breaker.rs` | 322 | `_state` | `self.state.read()` | `review` |  |  |
|   | `./crates/riptide-intelligence/src/circuit_breaker.rs` | 331 | `_state` | `self.state.read()` | `review` |  |  |
|   | `./crates/riptide-intelligence/src/providers/aws_bedrock.rs` | 362 | `_payload` | `self.build_bedrock_payload(&request)?` | `handle_result?` |  |  |
|   | `./crates/riptide-intelligence/tests/integration_tests.rs` | 301 | `_response` | `client.complete(request.clone()).await.unwrap()` | `review` |  |  |

## Crate: `riptide-pdf`

### Underscore lets

| ✔ | File | Line | Variable | Snippet / Context | Initial | Decision | Notes |
|:-:|:-----|-----:|:--------:|:------------------|:--------|:---------|:------|
|   | `./crates/riptide-pdf/src/integration.rs` | 224 | `_start_time` | `std::time::Instant::now()` | `review` |  |  |
|   | `./crates/riptide-pdf/src/processor.rs` | 116 | `_resource_guard` | `ProcessingResourceGuard::new()` | `review` |  |  |
|   | `./crates/riptide-pdf/src/processor.rs` | 468 | `_permissions` | `document.permissions()` | `review` |  |  |

## Crate: `riptide-performance`

### Underscore lets

| ✔ | File | Line | Variable | Snippet / Context | Initial | Decision | Notes |
|:-:|:-----|-----:|:--------:|:------------------|:--------|:---------|:------|
|   | `./crates/riptide-performance/src/monitoring/monitor.rs` | 214 | `_performance_metrics` | `Arc::clone(&self.performance_metrics)` | `review` |  |  |
|   | `./crates/riptide-performance/src/optimization/mod.rs` | 432 | `_config` | `self.config.clone()` | `review` |  |  |
|   | `./crates/riptide-performance/tests/performance_tests.rs` | 70 | `_scope` | `ProfileScope::new(&profiler, "test_operation")` | `review` |  |  |
|   | `./crates/riptide-performance/tests/performance_tests.rs` | 83 | `_scope` | `ProfileScope::new(&profiler, "async_operation")` | `review` |  |  |
|   | `./crates/riptide-performance/tests/performance_tests.rs` | 97 | `_outer` | `ProfileScope::new(&profiler, "outer")` | `review` |  |  |
|   | `./crates/riptide-performance/tests/performance_tests.rs` | 99 | `_inner1` | `ProfileScope::new(&profiler, "inner1")` | `review` |  |  |
|   | `./crates/riptide-performance/tests/performance_tests.rs` | 103 | `_inner2` | `ProfileScope::new(&profiler, "inner2")` | `review` |  |  |

## Crate: `riptide-persistence`

### Underscore lets

| ✔ | File | Line | Variable | Snippet / Context | Initial | Decision | Notes |
|:-:|:-----|-----:|:--------:|:------------------|:--------|:---------|:------|
|   | `./crates/riptide-persistence/benches/persistence_benchmarks.rs` | 344 | `_session` | `state_manager.get_session(session_id).await.unwrap()` | `review` |  |  |
|   | `./crates/riptide-persistence/benches/persistence_benchmarks.rs` | 461 | `_tenant` | `tenant_manager.get_tenant(tenant_id).await.unwrap()` | `review` |  |  |
|   | `./crates/riptide-persistence/examples/integration_example.rs` | 48 | `_session_id` | `demonstrate_session_workflow(&state_manager, &tenant_id).await?` | `handle_result?` |  |  |
|   | `./crates/riptide-persistence/src/state.rs` | 238 | `_checkpoint_manager` | `Arc::clone(&self.checkpoint_manager)` | `review` |  |  |
|   | `./crates/riptide-persistence/tests/integration/performance_tests.rs` | 283 | `_session` | `state_manager.get_session(session_id).await?` | `handle_result?` |  |  |
|   | `./crates/riptide-persistence/tests/integration/performance_tests.rs` | 412 | `_cleared` | `cache_manager.clear().await?` | `handle_result?` |  |  |

### TODOs

| ✔ | File | Line | Variable | Snippet / Context | Initial | Decision | Notes |
|:-:|:-----|-----:|:--------:|:------------------|:--------|:---------|:------|
|   | `./crates/riptide-persistence/src/metrics.rs` | 349 | `—` | `eviction_count: 0, // TODO: Implement eviction tracking` | `todo` |  |  |

## Crate: `riptide-search`

### Underscore lets

| ✔ | File | Line | Variable | Snippet / Context | Initial | Decision | Notes |
|:-:|:-----|-----:|:--------:|:------------------|:--------|:---------|:------|
|   | `./crates/riptide-search/src/circuit_breaker.rs` | 352 | `_fail1` | `circuit.search("no urls here", 1, "us", "en").await` | `review` |  |  |
|   | `./crates/riptide-search/src/circuit_breaker.rs` | 353 | `_fail2` | `circuit.search("still no urls", 1, "us", "en").await` | `review` |  |  |
|   | `./crates/riptide-search/src/circuit_breaker.rs` | 379 | `_fail1` | `circuit.search("no urls", 1, "us", "en").await` | `review` |  |  |
|   | `./crates/riptide-search/src/circuit_breaker.rs` | 380 | `_fail2` | `circuit.search("no urls", 1, "us", "en").await` | `review` |  |  |

## Crate: `riptide-stealth`

### Underscore lets

| ✔ | File | Line | Variable | Snippet / Context | Initial | Decision | Notes |
|:-:|:-----|-----:|:--------:|:------------------|:--------|:---------|:------|
|   | `./crates/riptide-stealth/tests/integration_test.rs` | 55 | `_stealth_config` | `StealthConfig::default()` | `review` |  |  |
|   | `./crates/riptide-stealth/tests/integration_test.rs` | 56 | `_user_agent_config` | `UserAgentConfig::default()` | `review` |  |  |
|   | `./crates/riptide-stealth/tests/integration_test.rs` | 57 | `_fingerprinting_config` | `FingerprintingConfig::default()` | `review` |  |  |
|   | `./crates/riptide-stealth/tests/integration_test.rs` | 58 | `_request_randomization` | `RequestRandomization::default()` | `review` |  |  |
|   | `./crates/riptide-stealth/tests/integration_test.rs` | 61 | `_rotation_strategy` | `RotationStrategy::Random` | `review` |  |  |
|   | `./crates/riptide-stealth/tests/integration_test.rs` | 62 | `_browser_type` | `BrowserType::Chrome` | `review` |  |  |
|   | `./crates/riptide-stealth/tests/integration_test.rs` | 63 | `_locale_strategy` | `LocaleStrategy::Random` | `review` |  |  |
|   | `./crates/riptide-stealth/tests/integration_test.rs` | 64 | `_stealth_preset` | `StealthPreset::Medium` | `review` |  |  |

## Crate: `riptide-streaming`

### Underscore lets

| ✔ | File | Line | Variable | Snippet / Context | Initial | Decision | Notes |
|:-:|:-----|-----:|:--------:|:------------------|:--------|:---------|:------|
|   | `./crates/riptide-streaming/src/progress.rs` | 420 | `_rx` | `tracker.start_tracking(stream_id).await.unwrap()` | `review` |  |  |
|   | `./crates/riptide-streaming/src/progress.rs` | 437 | `_rx` | `tracker.start_tracking(stream_id).await.unwrap()` | `review` |  |  |
|   | `./crates/riptide-streaming/src/progress.rs` | 460 | `_rx` | `tracker.start_tracking(stream_id).await.unwrap()` | `review` |  |  |
|   | `./crates/riptide-streaming/src/progress.rs` | 488 | `_rx` | `tracker.start_tracking(stream_id).await.unwrap()` | `review` |  |  |
|   | `./crates/riptide-streaming/tests/streaming_integration_tests.rs` | 51 | `_rx` | `tracker.start_tracking(stream_id).await.unwrap()` | `review` |  |  |
|   | `./crates/riptide-streaming/tests/streaming_integration_tests.rs` | 109 | `_permit` | `controller.acquire(stream_id, 1024).await.unwrap()` | `review` |  |  |
|   | `./crates/riptide-streaming/tests/streaming_integration_tests.rs` | 159 | `_permit` | `controller.acquire(stream_id, 1024).await.unwrap()` | `review` |  |  |
|   | `./crates/riptide-streaming/tests/streaming_integration_tests.rs` | 173 | `_rx` | `tracker.start_tracking(stream_id).await.unwrap()` | `review` |  |  |

## Crate: `riptide-workers`

### Underscore lets

| ✔ | File | Line | Variable | Snippet / Context | Initial | Decision | Notes |
|:-:|:-----|-----:|:--------:|:------------------|:--------|:---------|:------|
|   | `./crates/riptide-workers/src/processors.rs` | 184 | `_permit` | `semaphore.acquire().await.expect("Semaphore closed")` | `review` |  |  |
|   | `./crates/riptide-workers/src/service.rs` | 128 | `_queue` | `self.queue.lock().await` | `review` |  |  |
|   | `./crates/riptide-workers/src/service.rs` | 155 | `_worker_pool_clone` | `Arc::new(worker_pool)` | `review` |  |  |
|   | `./crates/riptide-workers/src/worker.rs` | 234 | `_permit` | `self.semaphore.acquire().await?` | `handle_result?` |  |  |

## Crate: `unknown`

### Underscore lets

| ✔ | File | Line | Variable | Snippet / Context | Initial | Decision | Notes |
|:-:|:-----|-----:|:--------:|:------------------|:--------|:---------|:------|
|   | `./tests/feature_flags/feature_flag_tests.rs` | 588 | `_converter` | `error_conversions::CoreErrorConverter::new()` | `review` |  |  |
|   | `./tests/golden/search/search_provider_golden.rs` | 498 | `_results` | `boxed.search("trait object test", 5, "us", "en").await` | `review` |  |  |
|   | `./tests/golden/search_provider_golden_test.rs` | 270 | `_error_type_name` | `expected_type` | `review` |  |  |
|   | `./tests/golden_test_cli.rs` | 268 | `_fix` | `sub_matches.get_flag("fix")` | `review` |  |  |
|   | `./tests/golden_test_cli.rs` | 292 | `_create_baseline` | `sub_matches.get_flag("create-baseline")` | `review` |  |  |
|   | `./tests/golden_test_cli.rs` | 301 | `_benchmark_name` | `sub_matches.get_one::<String>("benchmark-name")` | `review` |  |  |
|   | `./tests/golden_test_cli.rs` | 304 | `_save_baseline` | `sub_matches.get_flag("save-baseline")` | `review` |  |  |
|   | `./tests/performance/performance_baseline_tests.rs` | 218 | `_permit` | `sem.acquire().await.unwrap()` | `review` |  |  |
|   | `./tests/performance/performance_baseline_tests.rs` | 602 | `_permit` | `sem.acquire().await.unwrap()` | `review` |  |  |
|   | `./tests/streaming/deepsearch_stream_tests.rs` | 280 | `_serper_mock` | `framework.setup_serper_mock("streaming technology", search_results.clone())` | `review` |  |  |
|   | `./tests/streaming/deepsearch_stream_tests.rs` | 349 | `_serper_mock` | `framework.setup_serper_mock("fast search", search_results)` | `review` |  |  |
|   | `./tests/streaming/deepsearch_stream_tests.rs` | 390 | `_serper_mock` | `framework.setup_serper_mock("cached query", search_results)` | `review` |  |  |
|   | `./tests/streaming/deepsearch_stream_tests.rs` | 438 | `_serper_mock` | `framework.setup_serper_mock("mixed results", search_results)` | `review` |  |  |
|   | `./tests/streaming/deepsearch_stream_tests.rs` | 521 | `_serper_mock` | `framework.setup_serper_mock("large query", search_results)` | `review` |  |  |
|   | `./tests/streaming/deepsearch_stream_tests.rs` | 534 | `_working_mocks` | `framework.setup_content_mocks(&working_refs)` | `review` |  |  |
|   | `./tests/streaming/deepsearch_stream_tests.rs` | 535 | `_failing_mocks` | `framework.setup_failing_content_mocks(&failing_refs)` | `review` |  |  |
|   | `./tests/streaming/deepsearch_stream_tests.rs` | 602 | `_serper_mock1` | `framework.setup_serper_mock("query one", search_results1)` | `review` |  |  |
|   | `./tests/streaming/deepsearch_stream_tests.rs` | 603 | `_serper_mock2` | `framework.setup_serper_mock("query two", search_results2)` | `review` |  |  |
|   | `./tests/streaming/deepsearch_stream_tests.rs` | 646 | `_serper_mock` | `framework.setup_serper_mock("rate limit test", search_results)` | `review` |  |  |
|   | `./tests/unit/spider_handler_tests.rs` | 537 | `_debug_str` | `format!("{:?}", cloned)` | `handle_result?` |  |  |
|   | `./tests/unit/strategies_pipeline_tests.rs` | 180 | `_cloned` | `config.clone()` | `review` |  |  |
|   | `./tests/unit/strategies_pipeline_tests.rs` | 181 | `_debug_str` | `format!("{:?}", config)` | `handle_result?` |  |  |
|   | `./tests/unit/strategies_pipeline_tests.rs` | 471 | `_debug_str` | `format!("{:?}", cloned)` | `handle_result?` |  |  |
|   | `./tests/unit/strategies_pipeline_tests.rs` | 541 | `_debug_str` | `format!("{:?}", config)` | `handle_result?` |  |  |
|   | `./tests/unit/telemetry_opentelemetry_test.rs` | 15 | `_tracer` | `telemetry_system.tracer()` | `review` |  |  |
|   | `./tests/week3/benchmark_suite.rs` | 218 | `_chunks` | `chunk_content(&test_text, &config).await?` | `handle_result?` |  |  |
|   | `./tests/week3/benchmark_suite.rs` | 260 | `_links` | `extract_links(&html_clone)?` | `handle_result?` |  |  |
|   | `./tests/week3/benchmark_suite.rs` | 271 | `_images` | `extract_images(&html_clone)?` | `handle_result?` |  |  |
|   | `./tests/week3/benchmark_suite.rs` | 283 | `_stats` | `traverser.get_stats()` | `review` |  |  |
|   | `./tests/week3/benchmark_suite.rs` | 284 | `_elements` | `traverser.get_elements_info("div, p, a, img")?` | `handle_result?` |  |  |
|   | `./tests/week3/benchmark_suite.rs` | 296 | `_tables` | `processor.extract_tables(&html_clone, TableExtractionMode::All).await?` | `handle_result?` |  |  |
|   | `./tests/week3/benchmark_suite.rs` | 326 | `_result` | `processor.extract_with_css(&html_clone, "https://example.com", &selectors_clo...` | `handle_result?` |  |  |
|   | `./tests/week3/benchmark_suite.rs` | 342 | `_chunks` | `processor.chunk_content(&extracted.content, HtmlChunkingMode::default()).await?` | `handle_result?` |  |  |
|   | `./tests/week3/benchmark_suite.rs` | 345 | `_tables` | `processor.extract_tables(&html_clone, TableExtractionMode::All).await?` | `handle_result?` |  |  |
|   | `./tests/week3/benchmark_suite.rs` | 348 | `_links` | `extract_links(&html_clone)?` | `handle_result?` |  |  |
|   | `./tests/week3/benchmark_suite.rs` | 349 | `_images` | `extract_images(&html_clone)?` | `handle_result?` |  |  |
|   | `./tests/week3/performance_report.rs` | 250 | `_chunks` | `chunk_content(&test_text, &config).await.unwrap()` | `review` |  |  |
|   | `./wasm/riptide-extractor-wasm/src/lib.rs` | 175 | `_start_time` | `std::time::Instant::now()` | `review` |  |  |
|   | `./wasm/riptide-extractor-wasm/src/lib_clean.rs` | 118 | `_start_time` | `std::time::Instant::now()` | `review` |  |  |

### TODOs

| ✔ | File | Line | Variable | Snippet / Context | Initial | Decision | Notes |
|:-:|:-----|-----:|:--------:|:------------------|:--------|:---------|:------|
|   | `./tests/golden/behavior_capture.rs` | 140 | `—` | `initial_heap: AtomicU64::new(0), // TODO: Get heap info if available` | `todo` |  |  |
|   | `./tests/golden/behavior_capture.rs` | 292 | `—` | `bytes_processed_per_second: 0, // TODO: Calculate from functional outputs` | `todo` |  |  |
|   | `./tests/golden/memory_monitor.rs` | 126 | `—` | `heap_bytes: 0, // TODO: Get heap info if available` | `todo` |  |  |
|   | `./tests/golden/memory_monitor.rs` | 347 | `—` | `heap_bytes: 0, // TODO: Implement heap tracking` | `todo` |  |  |
|   | `./tests/golden_test_cli.rs` | 235 | `—` | `// TODO: Print detailed report` | `todo` |  |  |
|   | `./tests/golden_test_cli.rs` | 253 | `—` | `// TODO: Implement JSON output` | `todo` |  |  |
|   | `./tests/golden_test_cli.rs` | 257 | `—` | `// TODO: Implement YAML output` | `todo` |  |  |
|   | `./tests/golden_test_cli.rs` | 296 | `—` | `// TODO: Implement single test execution` | `todo` |  |  |
|   | `./tests/golden_test_cli.rs` | 308 | `—` | `// TODO: Implement benchmark execution` | `todo` |  |  |
|   | `./tests/golden_test_cli.rs` | 320 | `—` | `// TODO: Implement memory-specific tests` | `todo` |  |  |
|   | `./tests/unit/telemetry_opentelemetry_test.rs` | 23 | `—` | `assert!(sanitized.contains("XXX"));` | `todo` |  |  |
|   | `./tests/unit/telemetry_opentelemetry_test.rs` | 70 | `—` | `("192.168.1.100", "192.168.1.XXX"),` | `todo` |  |  |
|   | `./tests/unit/telemetry_opentelemetry_test.rs` | 76 | `—` | `assert!(sanitized.contains("***REDACTED***") \|\| sanitized.contains("***EMAI...` | `todo` |  |  |
|   | `./wasm/riptide-extractor-wasm/src/lib_clean.rs` | 293 | `—` | `links: vec![], // TODO: Extract links from content` | `todo` |  |  |
|   | `./wasm/riptide-extractor-wasm/src/lib_clean.rs` | 294 | `—` | `media: vec![], // TODO: Extract media URLs` | `todo` |  |  |
|   | `./wasm/riptide-extractor-wasm/src/lib_clean.rs` | 295 | `—` | `language: None, // TODO: Language detection` | `todo` |  |  |
|   | `./wasm/riptide-extractor-wasm/src/lib_clean.rs` | 299 | `—` | `categories: vec![], // TODO: Category extraction` | `todo` |  |  |
|   | `./wasm/riptide-extractor-wasm/tests/mod.rs` | 14 | `—` | `// TODO: Create integration module` | `todo` |  |  |
|   | `./wasm/riptide-extractor-wasm/tests/mod.rs` | 80 | `—` | `// TODO: Re-enable when integration module is implemented` | `todo` |  |  |
|   | `./wasm/riptide-extractor-wasm/tests/mod.rs` | 291 | `—` | `/// TODO: Re-enable when integration module is implemented` | `todo` |  |  |
|   | `./wasm/riptide-extractor-wasm/tests/mod.rs` | 296 | `—` | `// TODO: Enable when integration module exists` | `todo` |  |  |
|   | `./wasm/riptide-extractor-wasm/tests/test_runner.rs` | 35 | `—` | `// TODO: Re-enable full test suite when modules are properly accessible` | `todo` |  |  |
|   | `./wasm/riptide-extractor-wasm/tests/test_runner.rs` | 38 | `—` | `// TODO: Re-enable individual test category runners when modules are accessible` | `todo` |  |  |
|   | `./wasm/riptide-extractor-wasm/tests/test_runner.rs` | 128 | `—` | `// TODO: Re-enable when integration module is implemented` | `todo` |  |  |
|   | `./xtask/src/main.rs` | 12 | `—` | `Lazy::new(\|\| Regex::new(r#"(?i)\b(TODO\|FIXME\|HACK\|XXX)\b"#).unwrap());` | `todo` |  |  |
|   | `./xtask/src/main.rs` | 134 | `—` | `// TODO-like markers` | `todo` |  |  |
|   | `./xtask/src/main.rs` | 143 | `—` | `initial_guess: "todo",` | `todo` |  |  |

---

## Summary

- **Total findings:** 206
- **Underscore bindings:** 131
- **TODOs/FIXMEs:** 75
- **Crates analyzed:** 14
