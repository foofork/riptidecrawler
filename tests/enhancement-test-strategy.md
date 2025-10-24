# Comprehensive Test Strategy for WASM-First Enhancement

**Swarm:** hive-1761237761270-0brzzczvo
**Agent:** Tester
**Date:** 2025-10-23
**Scope:** Validation of 5 priority fixes from temp.md

---

## Executive Summary

This test strategy covers validation for:
1. JSON-LD short-circuit (Events/Articles)
2. Probe-first escalation (SPA detection → WASM → Headless)
3. WASM micro-network (budgeted same-origin fetches)
4. Enhanced content signals (text density, placeholders)
5. Domain warm-start (Redis-backed engine preferences)

**Total Estimated Tests:** 147 (68 unit, 42 integration, 21 performance, 16 chaos/edge)

---

## 1. JSON-LD Short-Circuit Testing

### 1.1 Unit Tests (18 tests)

**Test File:** `tests/unit/jsonld_shortcircuit_test.rs`

#### Event Schema Tests (9 tests)
```rust
#[test]
fn test_complete_event_with_all_fields() {
    // Complete: name, startDate, location.name, location.address, url, organizer
    // Expected: High confidence (0.9), early return
}

#[test]
fn test_complete_event_minimal_required() {
    // Required only: name, startDate, location.name OR location.address
    // Expected: High confidence (0.9), early return
}

#[test]
fn test_incomplete_event_missing_start_date() {
    // Has name, location, but no startDate
    // Expected: No short-circuit, continue to full extraction
}

#[test]
fn test_incomplete_event_missing_location() {
    // Has name, startDate, but no location data
    // Expected: No short-circuit
}

#[test]
fn test_event_with_partial_location() {
    // Has location.name but missing address, or vice versa
    // Expected: Short-circuit if at least one is present
}

#[test]
fn test_event_array_multiple_valid() {
    // JSON-LD with 3 events, 2 complete, 1 incomplete
    // Expected: Return first complete event
}

#[test]
fn test_event_with_virtual_location() {
    // Online event with virtual location URL
    // Expected: Accept virtualLocation as valid location
}

#[test]
fn test_event_with_nested_organizer() {
    // Deep organizer: {type: Organization, name, url, contactPoint}
    // Expected: Parse and validate nested structure
}

#[test]
fn test_malformed_jsonld_event() {
    // Invalid JSON syntax, missing @type, wrong schema.org URL
    // Expected: Graceful fallback, no panic
}
```

#### Article Schema Tests (9 tests)
```rust
#[test]
fn test_complete_article_news() {
    // NewsArticle: headline, author, datePublished, articleBody
    // Expected: High confidence (0.85), early return
}

#[test]
fn test_complete_article_blog_post() {
    // BlogPosting: headline, author, datePublished, text
    // Expected: Confidence 0.85
}

#[test]
fn test_incomplete_article_missing_body() {
    // Has headline, author, date, but no articleBody
    // Expected: No short-circuit
}

#[test]
fn test_incomplete_article_missing_author() {
    // Has headline, date, body, but no author
    // Expected: Configurable - may still short-circuit if body is substantial
}

#[test]
fn test_article_minimal_required() {
    // headline + (datePublished OR dateModified) + articleBody (>200 chars)
    // Expected: Short-circuit with confidence 0.80
}

#[test]
fn test_article_with_image_and_publisher() {
    // Rich metadata: image, publisher, mainEntityOfPage
    // Expected: Boost confidence to 0.90
}

#[test]
fn test_article_array_mixed_types() {
    // JSON-LD with Article, Event, Product types
    // Expected: Find and return first complete Article
}

#[test]
fn test_article_with_truncated_body() {
    // articleBody present but <100 chars (likely incomplete)
    // Expected: No short-circuit, flag as low quality
}

#[test]
fn test_multiple_jsonld_scripts() {
    // Page has 3 <script type="application/ld+json"> blocks
    // Expected: Parse all, find first complete schema
}
```

### 1.2 Integration Tests (8 tests)

**Test File:** `tests/integration/jsonld_extraction_test.rs`

```rust
#[tokio::test]
async fn test_event_page_short_circuit_vs_full() {
    // Compare: short-circuit result vs full WASM extraction
    // Validate: field parity, performance delta (>80% faster)
}

#[tokio::test]
async fn test_article_page_short_circuit_confidence() {
    // Real news article with complete JSON-LD
    // Validate: confidence >= 0.85, all core fields present
}

#[tokio::test]
async fn test_incomplete_jsonld_fallback_to_wasm() {
    // Event page with partial JSON-LD (missing location)
    // Validate: Falls through to WASM, extracts from DOM
}

#[tokio::test]
async fn test_no_jsonld_standard_flow() {
    // Page without JSON-LD
    // Validate: No delay, proceeds to normal engine selection
}

#[tokio::test]
async fn test_jsonld_with_multiple_events_pagination() {
    // Event listing page (EventSeries or itemListElement)
    // Validate: Extract all events, not just first
}

#[tokio::test]
async fn test_jsonld_cache_hit() {
    // Same URL extracted twice with Redis cache enabled
    // Validate: Second extraction uses cached short-circuit result
}

#[tokio::test]
async fn test_jsonld_schema_version_tolerance() {
    // JSON-LD using schema.org v3.9, v15.0, https vs http
    // Validate: All versions parse correctly
}

#[tokio::test]
async fn test_telemetry_short_circuit_metrics() {
    // Extract page with JSON-LD short-circuit
    // Validate: Telemetry logs engine=JsonLD, confidence, time_saved_ms
}
```

---

## 2. Probe-First Escalation Testing

### 2.1 Unit Tests (14 tests)

**Test File:** `tests/unit/probe_escalation_test.rs`

#### SPA Detection (5 tests)
```rust
#[test]
fn test_spa_react_detection() {
    let html = r#"<div id="root"></div><script src="/_next/static/..."></script>"#;
    // Expected: is_spa_like = true, probe_first = true
}

#[test]
fn test_spa_vue_detection() {
    let html = r#"<div id="app" data-v-app=""></div>"#;
    // Expected: is_spa_like = true
}

#[test]
fn test_spa_angular_detection() {
    let html = r#"<app-root></app-root><script src="main.js"></script>"#;
    // Expected: is_spa_like = true
}

#[test]
fn test_static_site_not_spa() {
    let html = r#"<body><h1>Title</h1><p>Lots of visible text here...</p></body>"#;
    // Expected: is_spa_like = false, content_ratio > 0.1
}

#[test]
fn test_hybrid_ssr_spa() {
    // Server-rendered React with hydration script
    let html = r#"<div id="root"><article>Pre-rendered content</article></div><script>__NEXT_DATA__</script>"#;
    // Expected: is_spa_like = true, but content_ratio > 0.5
}
```

#### Probe Outcome Handling (9 tests)
```rust
#[test]
fn test_probe_success_no_network() {
    // ProbeOutcome::Populated { dom_growth: 150%, network_needed: false }
    // Expected: Proceed to full WASM extraction
}

#[test]
fn test_probe_success_network_needed() {
    // ProbeOutcome::NeedsNetwork { xhr_blocked: 3, fetch_blocked: 2 }
    // Expected: Attempt WASM with micro-network budget
}

#[test]
fn test_probe_no_growth() {
    // ProbeOutcome::NoGrowth (DOM unchanged after 500ms)
    // Expected: Escalate to headless
}

#[test]
fn test_probe_blocked_by_wasm() {
    // ProbeOutcome::Blocked (WASM module crashed or OOM)
    // Expected: Skip WASM, go direct to headless
}

#[test]
fn test_probe_timeout() {
    // Probe exceeds 1s fuel limit
    // Expected: Treat as NoGrowth, escalate
}

#[test]
fn test_probe_anti_scraping_bypass() {
    // analyze() detects Cloudflare/Akamai before probe
    // Expected: Skip probe, direct to headless (priority 1)
}

#[test]
fn test_probe_content_ratio_threshold() {
    // content_ratio = 0.08 (below 0.1 threshold)
    // Expected: Trigger probe-first
}

#[test]
fn test_probe_content_ratio_borderline() {
    // content_ratio = 0.12 (just above threshold)
    // Expected: Still use WASM, but no probe (direct full)
}

#[test]
fn test_escalation_chain() {
    // SPA detected → WASM probe → NeedsNetwork → micro-network fails → headless
    // Expected: Correct engine progression, telemetry for each hop
}
```

### 2.2 Integration Tests (10 tests)

**Test File:** `tests/integration/probe_escalation_test.rs`

```rust
#[tokio::test]
async fn test_nextjs_page_probe_success() {
    // Real Next.js page with __NEXT_DATA__ containing pageProps
    // Expected: WASM probe succeeds, extracts from inline JSON
}

#[tokio::test]
async fn test_react_spa_needs_network() {
    // React app that fetches /api/data on mount
    // Expected: Probe detects network need, attempts micro-network
}

#[tokio::test]
async fn test_vue_spa_escalates_to_headless() {
    // Vue SPA with WebSocket dependency (can't be budgeted)
    // Expected: Probe → micro-network fails → headless succeeds
}

#[tokio::test]
async fn test_static_site_skips_probe() {
    // Plain HTML blog post with visible content
    // Expected: analyze() sees high content_ratio, uses WASM direct
}

#[tokio::test]
async fn test_angular_with_lazy_loading() {
    // Angular app with lazy-loaded modules
    // Expected: Probe sees DOM growth but incomplete, escalates
}

#[tokio::test]
async fn test_cloudflare_challenge_skips_probe() {
    // Page with cf-challenge-form
    // Expected: analyze() detects anti-scraping, goes direct to headless
}

#[tokio::test]
async fn test_probe_performance_vs_direct_headless() {
    // Benchmark: SPA page extracted with probe-first vs old direct-headless
    // Expected: 60-80% time savings on successful probe
}

#[tokio::test]
async fn test_mixed_domain_batch() {
    // Extract 5 URLs: 2 SPAs (probe), 2 static (direct), 1 Cloudflare
    // Expected: Correct engine per URL, no cross-contamination
}

#[tokio::test]
async fn test_probe_retry_on_transient_failure() {
    // WASM probe fails once (OOM), retry with 2x memory
    // Expected: Second probe succeeds or escalates correctly
}

#[tokio::test]
async fn test_telemetry_escalation_path() {
    // Extract URL that goes: probe → network → headless
    // Expected: Telemetry logs each decision point with reasons
}
```

---

## 3. WASM Micro-Network Testing

### 3.1 Unit Tests (12 tests)

**Test File:** `tests/unit/wasm_micronetwork_test.rs`

#### Budget Enforcement (6 tests)
```rust
#[test]
fn test_network_budget_max_requests() {
    let budget = NetworkBudget { max_requests: 4, max_kb: 300 };
    // Simulate 5 fetch calls
    // Expected: First 4 succeed, 5th blocked with NetworkBudgetExceeded
}

#[test]
fn test_network_budget_max_bytes() {
    let budget = NetworkBudget { max_requests: 10, max_kb: 100 };
    // Simulate 3 fetches: 40KB, 40KB, 30KB (total 110KB)
    // Expected: Third fetch blocked mid-stream
}

#[test]
fn test_network_budget_zero_budget() {
    let budget = NetworkBudget { max_requests: 0, max_kb: 0 };
    // Expected: All network calls blocked immediately
}

#[test]
fn test_network_budget_reset_per_extraction() {
    // Extract URL 1 with 4 requests, then URL 2
    // Expected: URL 2 gets fresh budget of 4 requests
}

#[test]
fn test_network_budget_partial_response() {
    // Fetch returns 50KB response, budget = 30KB
    // Expected: Truncate at 30KB, mark as incomplete
}

#[test]
fn test_network_budget_telemetry() {
    // Make 3 requests (2 succeed, 1 blocked)
    // Expected: Telemetry reports used_requests=3, blocked=1, bytes_transferred
}
```

#### Same-Origin Checks (6 tests)
```rust
#[test]
fn test_same_origin_allowed() {
    let page_url = "https://example.com/events";
    let fetch_url = "https://example.com/api/events";
    // Expected: Allowed (same origin)
}

#[test]
fn test_same_origin_subdomain_blocked() {
    let page_url = "https://example.com/page";
    let fetch_url = "https://api.example.com/data";
    // Expected: Blocked (different subdomain)
}

#[test]
fn test_same_origin_protocol_mismatch() {
    let page_url = "https://example.com/page";
    let fetch_url = "http://example.com/data";
    // Expected: Blocked (http vs https)
}

#[test]
fn test_same_origin_port_mismatch() {
    let page_url = "https://example.com:443/page";
    let fetch_url = "https://example.com:8080/api";
    // Expected: Blocked (different port)
}

#[test]
fn test_third_party_cdn_blocked() {
    let page_url = "https://example.com/page";
    let fetch_url = "https://cdn.cloudflare.com/lib.js";
    // Expected: Blocked (third-party)
}

#[test]
fn test_relative_url_resolution() {
    let page_url = "https://example.com/events/123";
    let fetch_url = "/api/event/123";
    // Expected: Resolve to https://example.com/api/event/123, allowed
}
```

### 3.2 Integration Tests (8 tests)

**Test File:** `tests/integration/wasm_micronetwork_test.rs`

```rust
#[tokio::test]
async fn test_nextjs_api_route_fetch() {
    // Next.js page that calls getServerSideProps via /api/props
    // Budget: 2 requests, 100KB
    // Expected: WASM fetches API, extracts data, avoids headless
}

#[tokio::test]
async fn test_react_multiple_api_calls() {
    // React app makes 3 API calls: /user, /settings, /notifications
    // Budget: 4 requests, 200KB
    // Expected: All succeed, extraction completes
}

#[tokio::test]
async fn test_budget_exceeded_escalation() {
    // SPA makes 10 small API calls
    // Budget: 3 requests, 500KB
    // Expected: After 3, network blocked, escalate to headless
}

#[tokio::test]
async fn test_large_json_response() {
    // API returns 500KB JSON blob
    // Budget: 10 requests, 300KB
    // Expected: Response truncated, extraction partial, flag low confidence
}

#[tokio::test]
async fn test_mixed_same_third_party() {
    // Page fetches /api/data (same-origin) + https://analytics.com/track
    // Expected: /api/data succeeds, analytics blocked
}

#[tokio::test]
async fn test_cors_preflight_handling() {
    // WASM fetch triggers OPTIONS preflight
    // Expected: Preflight not counted against request budget
}

#[tokio::test]
async fn test_network_timeout() {
    // API endpoint takes 5s to respond
    // Expected: WASM times out after 2s, escalates
}

#[tokio::test]
async fn test_network_success_vs_headless_comparison() {
    // Extract page with micro-network enabled vs headless
    // Expected: Field parity >95%, performance 50% faster
}
```

---

## 4. Enhanced Signal Testing

### 4.1 Unit Tests (16 tests)

**Test File:** `tests/unit/content_signals_test.rs`

#### Text Density Calculation (6 tests)
```rust
#[test]
fn test_text_density_high_content() {
    let html = r#"<body><article><p>500 words of visible text...</p></article></body>"#;
    // Expected: density > 0.7, high confidence
}

#[test]
fn test_text_density_low_spa() {
    let html = r#"<div id="root"></div><script>bundle.js</script>"#;
    // Expected: density < 0.05
}

#[test]
fn test_text_density_strip_scripts_styles() {
    let html = r#"<script>1000 lines of JS</script><body>10 words</body>"#;
    // Expected: Exclude script content from density calc
}

#[test]
fn test_text_density_noscript_fallback() {
    let html = r#"<div id="root"></div><noscript><h1>Enable JS</h1></noscript>"#;
    // Expected: Low density, flag noscript presence
}

#[test]
fn test_text_density_hidden_elements() {
    let html = r#"<div style="display:none">Hidden text</div><p>Visible</p>"#;
    // Expected: Exclude display:none, visibility:hidden
}

#[test]
fn test_text_density_whitespace_collapse() {
    let html = r#"<p>   Multiple    spaces   </p>"#;
    // Expected: Collapse to single spaces, accurate char count
}
```

#### Placeholder Detection (5 tests)
```rust
#[test]
fn test_placeholder_skeleton_classes() {
    let html = r#"<div class="skeleton-loader"><div class="shimmer"></div></div>"#;
    // Expected: placeholder_hits = 2, flag as SPA-loading
}

#[test]
fn test_placeholder_loading_text() {
    let html = r#"<div>Loading...</div><span>Please wait</span>"#;
    // Expected: Detect loading indicators, low confidence
}

#[test]
fn test_placeholder_empty_containers() {
    let html = r#"<div id="content"></div><main></main>"#;
    // Expected: Empty semantic containers, placeholder_hits++
}

#[test]
fn test_placeholder_aria_busy() {
    let html = r#"<div aria-busy="true" aria-live="polite">Loading content</div>"#;
    // Expected: ARIA loading states detected
}

#[test]
fn test_no_placeholders_real_content() {
    let html = r#"<article><h1>Title</h1><p>Full paragraph of content</p></article>"#;
    // Expected: placeholder_hits = 0
}
```

#### SPA Framework Detection (5 tests)
```rust
#[test]
fn test_framework_nextjs() {
    let html = r#"<script id="__NEXT_DATA__" type="application/json">{...}</script>"#;
    // Expected: spa_framework = "Next.js"
}

#[test]
fn test_framework_nuxt() {
    let html = r#"<div id="__nuxt"><script>window.__NUXT__={...}</script></div>"#;
    // Expected: spa_framework = "Nuxt.js"
}

#[test]
fn test_framework_vite() {
    let html = r#"<script type="module" src="/@vite/client"></script>"#;
    // Expected: spa_framework = "Vite"
}

#[test]
fn test_framework_angular() {
    let html = r#"<app-root ng-version="17.0.0"></app-root>"#;
    // Expected: spa_framework = "Angular"
}

#[test]
fn test_framework_none() {
    let html = r#"<body><h1>Static HTML</h1></body>"#;
    // Expected: spa_framework = None
}
```

### 4.2 Integration Tests (6 tests)

**Test File:** `tests/integration/content_signals_test.rs`

```rust
#[tokio::test]
async fn test_signal_score_threshold_raw_ok() {
    // Static blog: high density, no placeholders, no framework
    // Expected: score < 0.25, engine = RawOK (rare, but possible)
}

#[tokio::test]
async fn test_signal_score_threshold_wasm() {
    // Hybrid SSR: medium density, Next.js detected
    // Expected: score 0.25-0.6, engine = Wasm
}

#[tokio::test]
async fn test_signal_score_threshold_headless() {
    // Pure SPA: low density, Vue + placeholders
    // Expected: score >= 0.6, engine = Headless (after probe fails)
}

#[tokio::test]
async fn test_signal_evolution_during_extraction() {
    // Initial HTML: low density → WASM probe → post-probe: high density
    // Expected: Re-score after probe, decide on full WASM
}

#[tokio::test]
async fn test_noscript_ratio_fallback() {
    // SPA with substantial noscript content
    // Expected: Use noscript as fallback extraction source
}

#[tokio::test]
async fn test_telemetry_signal_metrics() {
    // Extract page, log all signals: density, placeholders, framework
    // Expected: Telemetry includes detector_metrics struct
}
```

---

## 5. Domain Warm-Start Testing

### 5.1 Unit Tests (8 tests)

**Test File:** `tests/unit/domain_warmstart_test.rs`

```rust
#[test]
fn test_domain_profile_initial_state() {
    let profile = DomainProfile::new("example.com");
    // Expected: preferred_engine = None, success_confidence = None
}

#[test]
fn test_domain_profile_update_success() {
    let mut profile = DomainProfile::new("example.com");
    profile.record_success(Engine::Wasm, 0.88);
    // Expected: preferred_engine = Some(Wasm), last_success_confidence = 0.88
}

#[test]
fn test_domain_profile_update_failure() {
    let mut profile = DomainProfile::new("example.com");
    profile.record_failure(Engine::Wasm);
    // Expected: Decrement wasm_success_rate, may clear preferred_engine
}

#[test]
fn test_domain_profile_ttl_expiry() {
    let profile = DomainProfile::with_ttl("example.com", Duration::from_secs(1));
    sleep(2);
    // Expected: is_stale() = true, should re-profile
}

#[test]
fn test_domain_profile_confidence_threshold() {
    let mut profile = DomainProfile::new("example.com");
    profile.record_success(Engine::Wasm, 0.65); // Low confidence
    // Expected: preferred_engine only set if confidence > 0.75
}

#[test]
fn test_domain_profile_multi_engine_tracking() {
    let mut profile = DomainProfile::new("example.com");
    profile.record_success(Engine::Wasm, 0.80);
    profile.record_success(Engine::Headless, 0.90);
    // Expected: preferred_engine = Headless (higher confidence)
}

#[test]
fn test_domain_profile_serialization() {
    let profile = DomainProfile::new("example.com");
    let json = serde_json::to_string(&profile).unwrap();
    // Expected: Round-trip serialization preserves all fields
}

#[test]
fn test_subdomain_inheritance() {
    // Profile for blog.example.com inherits from example.com
    // Expected: Check parent domain if subdomain has no profile
}
```

### 5.2 Integration Tests (10 tests)

**Test File:** `tests/integration/domain_warmstart_test.rs`

```rust
#[tokio::test]
async fn test_warm_start_cache_hit() {
    // Extract example.com/page1, then example.com/page2
    // Expected: Second extraction uses cached preferred_engine
}

#[tokio::test]
async fn test_warm_start_cache_miss() {
    // Extract newsite.com for first time
    // Expected: Full profiling, store result in Redis
}

#[tokio::test]
async fn test_warm_start_ttl_expiry() {
    // Profile stored with 1s TTL, extract after 2s
    // Expected: Cache miss, re-profile, update cache
}

#[tokio::test]
async fn test_warm_start_stale_on_failure() {
    // Cached preferred_engine = Wasm, but extraction fails
    // Expected: Invalidate cache, try Headless, update profile
}

#[tokio::test]
async fn test_warm_start_cross_subdomain() {
    // Profile example.com, then extract api.example.com
    // Expected: Inherit base domain profile as starting point
}

#[tokio::test]
async fn test_warm_start_confidence_decay() {
    // 5 successful extractions, then 3 failures
    // Expected: Confidence decays, may clear preferred_engine
}

#[tokio::test]
async fn test_warm_start_redis_persistence() {
    // Extract, store profile, restart service, extract again
    // Expected: Profile loaded from Redis, used immediately
}

#[tokio::test]
async fn test_warm_start_multi_tenant() {
    // Two users extract same domain concurrently
    // Expected: No race condition, consistent profile updates
}

#[tokio::test]
async fn test_warm_start_eviction_policy() {
    // Cache reaches max size (1000 domains)
    // Expected: LRU eviction, least-used domains dropped
}

#[tokio::test]
async fn test_telemetry_cache_hit_rate() {
    // Extract 20 URLs from 5 domains
    // Expected: Telemetry reports cache_hit_rate, avg_confidence
}
```

---

## 6. Performance Benchmarks (21 tests)

**Test File:** `tests/benchmarks/enhancement_perf_test.rs`

### 6.1 JSON-LD Performance (4 benchmarks)
```rust
#[bench]
fn bench_jsonld_parse_small() {
    // Single Event JSON-LD (1KB)
    // Target: <1ms
}

#[bench]
fn bench_jsonld_parse_large() {
    // Multiple schemas (50KB JSON-LD)
    // Target: <10ms
}

#[bench]
fn bench_jsonld_vs_full_extraction() {
    // Event page: short-circuit vs WASM full
    // Target: 80% time reduction
}

#[bench]
fn bench_jsonld_cache_hit() {
    // Second extraction with cached JSON-LD
    // Target: <0.5ms
}
```

### 6.2 Probe-First Performance (5 benchmarks)
```rust
#[bench]
fn bench_wasm_probe_success() {
    // SPA probe with DOM growth
    // Target: <200ms
}

#[bench]
fn bench_probe_vs_direct_headless() {
    // Time savings: probe-first vs old direct-headless
    // Target: 60-80% reduction on successful probe
}

#[bench]
fn bench_probe_escalation_overhead() {
    // Failed probe + escalation latency
    // Target: <100ms overhead
}

#[bench]
fn bench_spa_detection_signals() {
    // analyze() execution time on complex HTML
    // Target: <5ms
}

#[bench]
fn bench_parallel_probe_batch() {
    // 10 URLs probed concurrently
    // Target: Linear scaling (10x single probe time)
}
```

### 6.3 Micro-Network Performance (5 benchmarks)
```rust
#[bench]
fn bench_same_origin_check() {
    // URL origin validation per request
    // Target: <10µs
}

#[bench]
fn bench_network_budget_accounting() {
    // Track bytes/requests per fetch
    // Target: <50µs overhead
}

#[bench]
fn bench_micro_network_success() {
    // WASM with 3 API calls vs headless
    // Target: 40-60% faster
}

#[bench]
fn bench_budget_exceeded_handling() {
    // Block request + escalation decision
    // Target: <5ms
}

#[bench]
fn bench_network_vs_no_network_wasm() {
    // WASM Full (network-free) vs FullWithNetwork
    // Target: <20% overhead with network
}
```

### 6.4 Signal Computation Performance (4 benchmarks)
```rust
#[bench]
fn bench_text_density_calculation() {
    // Visible text extraction on 100KB HTML
    // Target: <20ms
}

#[bench]
fn bench_placeholder_detection() {
    // Scan HTML for skeleton/shimmer classes
    // Target: <10ms
}

#[bench]
fn bench_framework_detection() {
    // Regex + string matching for SPA frameworks
    // Target: <5ms
}

#[bench]
fn bench_signal_score_computation() {
    // Full DetectorMetrics struct generation
    // Target: <30ms total
}
```

### 6.5 Domain Warm-Start Performance (3 benchmarks)
```rust
#[bench]
fn bench_redis_cache_read() {
    // Load DomainProfile from Redis
    // Target: <5ms
}

#[bench]
fn bench_redis_cache_write() {
    // Store DomainProfile to Redis
    // Target: <10ms
}

#[bench]
fn bench_warm_start_vs_cold_start() {
    // Extraction with cached profile vs first-time
    // Target: 50-100ms time savings
}
```

---

## 7. Chaos & Edge Case Testing (16 tests)

**Test File:** `tests/chaos/edge_cases_test.rs`

### 7.1 JSON-LD Edge Cases (4 tests)
```rust
#[tokio::test]
async fn chaos_malformed_jsonld_no_panic() {
    // Invalid JSON, missing brackets, null bytes
    // Expected: Graceful error, no process crash
}

#[tokio::test]
async fn chaos_huge_jsonld_oom() {
    // 10MB JSON-LD blob
    // Expected: Memory limit enforced, truncate or skip
}

#[tokio::test]
async fn chaos_recursive_jsonld_schema() {
    // Self-referencing @graph structure
    // Expected: Cycle detection, max depth limit
}

#[tokio::test]
async fn chaos_unicode_in_jsonld() {
    // Event with emoji, RTL text, zero-width chars
    // Expected: Correct UTF-8 handling, no corruption
}
```

### 7.2 Probe-First Edge Cases (4 tests)
```rust
#[tokio::test]
async fn chaos_wasm_oom_during_probe() {
    // Probe triggers OOM in WASM module
    // Expected: Catch, log, escalate to headless
}

#[tokio::test]
async fn chaos_infinite_loop_in_spa_js() {
    // SPA JS enters infinite loop during probe
    // Expected: Fuel limit stops execution, escalate
}

#[tokio::test]
async fn chaos_dom_growth_explosion() {
    // Malicious page generates 100k DOM nodes
    // Expected: Growth cap enforced, flag as suspicious
}

#[tokio::test]
async fn chaos_anti_scraping_mid_probe() {
    // Page triggers Cloudflare after probe starts
    // Expected: Detect, abort probe, escalate
}
```

### 7.3 Micro-Network Edge Cases (4 tests)
```rust
#[tokio::test]
async fn chaos_network_request_loop() {
    // SPA makes circular fetch calls (A→B→A)
    // Expected: Budget exhausted, escalate
}

#[tokio::test]
async fn chaos_network_timeout_cascade() {
    // All 4 allowed requests time out
    // Expected: Escalate after 4 timeouts, no indefinite hang
}

#[tokio::test]
async fn chaos_large_chunked_response() {
    // API returns 5MB in Transfer-Encoding: chunked
    // Expected: Truncate at budget, flag incomplete
}

#[tokio::test]
async fn chaos_cors_error_handling() {
    // Browser CORS blocked (should not happen in WASM, but test)
    // Expected: Graceful handling, no fetch panic
}
```

### 7.4 Signal & Warm-Start Edge Cases (4 tests)
```rust
#[tokio::test]
async fn chaos_text_density_all_whitespace() {
    // HTML with 10KB of spaces and newlines
    // Expected: density ≈ 0, flag as empty
}

#[tokio::test]
async fn chaos_placeholder_false_positives() {
    // Real content with "skeleton" in class name (not loading state)
    // Expected: Context-aware detection, minimal false positives
}

#[tokio::test]
async fn chaos_redis_connection_lost() {
    // Redis unavailable during profile lookup
    // Expected: Fallback to no-cache mode, proceed with extraction
}

#[tokio::test]
async fn chaos_domain_profile_corruption() {
    // Corrupted JSON in Redis cache
    // Expected: Parse error caught, invalidate, re-profile
}
```

---

## 8. Test Data Requirements

### 8.1 Sample HTML Corpus (30 files)
**Location:** `tests/fixtures/html/`

- `event_complete_jsonld.html` - Event with full schema
- `event_incomplete_jsonld.html` - Event missing location
- `article_news.html` - NewsArticle with complete metadata
- `article_blog.html` - BlogPosting schema
- `nextjs_ssr.html` - Next.js SSR page with __NEXT_DATA__
- `react_spa_client.html` - Pure client-side React
- `vue_spa.html` - Vue 3 application shell
- `angular_app.html` - Angular with lazy loading
- `static_blog.html` - Plain HTML blog post
- `cloudflare_challenge.html` - Cloudflare interstitial
- `skeleton_loader.html` - Loading state with placeholders
- `malformed_jsonld.html` - Invalid JSON syntax
- `huge_jsonld.html` - 10MB JSON-LD blob
- `empty_spa.html` - Div with id=root only
- `hybrid_ssr.html` - SSR + client hydration
- (15 more variations covering edge cases)

### 8.2 API Mock Responses (20 files)
**Location:** `tests/fixtures/api/`

- `events_list.json` - Array of events for /api/events
- `event_detail.json` - Single event detail
- `user_profile.json` - User data
- `settings.json` - App settings
- `large_response.json` - 500KB JSON blob
- `chunked_response.txt` - Simulated chunked transfer
- (14 more API mocks for various scenarios)

### 8.3 Expected Extraction Results (30 files)
**Location:** `tests/fixtures/expected/`

- JSON files with expected field values for each HTML fixture
- Used for assertion in integration tests

### 8.4 Performance Baseline Data
**Location:** `tests/benchmarks/baseline.json`

```json
{
  "jsonld_parse_small": { "mean_ms": 0.8, "std_dev": 0.1 },
  "probe_success": { "mean_ms": 180, "std_dev": 20 },
  "micro_network_3_requests": { "mean_ms": 450, "std_dev": 50 },
  "text_density_calc": { "mean_ms": 18, "std_dev": 3 }
  // ... more baselines
}
```

---

## 9. Test Execution Plan

### 9.1 Local Development
```bash
# Unit tests (fast, run frequently)
cargo test --lib

# Integration tests (slower, run before commit)
cargo test --test '*_integration_test'

# Benchmarks (run weekly or on perf-critical changes)
cargo bench

# Chaos tests (run before release)
cargo test --test chaos -- --nocapture
```

### 9.2 CI/CD Pipeline
```yaml
stages:
  - fast_feedback:    # Unit tests (< 2 min)
  - integration:      # Integration tests (< 10 min)
  - performance:      # Benchmarks + regression check
  - chaos:            # Edge cases (< 5 min)
  - e2e:              # Full system test (< 20 min)

performance:
  script:
    - cargo bench --no-fail-fast
    - python scripts/compare_baselines.py  # Fail if >10% regression
```

### 9.3 Test Coverage Targets
- **Unit tests:** 85% line coverage
- **Integration tests:** 75% scenario coverage
- **Benchmarks:** 100% critical path coverage
- **Chaos tests:** 90% error path coverage

---

## 10. Success Criteria

### 10.1 Functional Validation
✅ JSON-LD short-circuit: Reduces extraction time by >80% for complete schemas
✅ Probe-first escalation: 60-80% time savings on SPAs vs old direct-headless
✅ WASM micro-network: Handles 90% of API-dependent SPAs without headless
✅ Enhanced signals: <5% false positive rate on SPA detection
✅ Domain warm-start: Cache hit rate >70% in steady-state

### 10.2 Performance Validation
✅ No regression: All benchmarks within 10% of baseline
✅ Memory efficiency: <50MB heap growth per extraction
✅ Scalability: Linear performance degradation up to 10x load

### 10.3 Reliability Validation
✅ Chaos tests: 0 panics, all errors gracefully handled
✅ Edge cases: 100% test pass rate
✅ Telemetry: All decision points logged with reasons

---

## 11. Test Maintenance

### 11.1 Test Review Cadence
- **Weekly:** Review flaky tests (>5% failure rate)
- **Monthly:** Update fixtures with real-world samples
- **Quarterly:** Refresh performance baselines

### 11.2 Test Debt Management
- **Max tech debt:** 10% of tests marked `#[ignore]`
- **Flaky test SLA:** Fix or quarantine within 1 week
- **Coverage decay:** Alert if drops below 80%

---

## Appendix: Test Execution Summary

| Test Category | Unit | Integration | Performance | Chaos | Total |
|--------------|------|-------------|-------------|-------|-------|
| JSON-LD | 18 | 8 | 4 | 4 | 34 |
| Probe-First | 14 | 10 | 5 | 4 | 33 |
| Micro-Network | 12 | 8 | 5 | 4 | 29 |
| Signals | 16 | 6 | 4 | 2 | 28 |
| Warm-Start | 8 | 10 | 3 | 2 | 23 |
| **TOTAL** | **68** | **42** | **21** | **16** | **147** |

**Estimated Test Execution Time:**
- Unit: 30 seconds
- Integration: 8 minutes
- Performance: 15 minutes
- Chaos: 4 minutes
- **Total:** ~27 minutes (parallel execution: ~12 minutes)

---

**End of Test Strategy Document**
