# Triage: Underscore Bindings & TODOs

**Generated:** 2025-10-07 07:31:47

Fill the **Decision** column using one of:

- `side_effect_only` · `promote_and_use` · `handle_result` · `guard_lifetime` · `detached_ok` · `wire_config` · `todo_tasked` · `axum_handler`

---

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

---

## Summary

- **Total findings:** 40
- **Underscore bindings:** 25
- **TODOs/FIXMEs:** 15
- **Crates analyzed:** 1
