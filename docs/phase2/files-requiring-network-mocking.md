# Phase 2: Files Requiring Network Mocking

## Summary Statistics

- **Total Files with Network Calls**: 293
- **Files with Wiremock**: 12
- **Files Needing Migration**: 281
- **Test Files**: ~120
- **Example/Doc Files**: ~100
- **Source Files**: ~60

---

## High-Priority Test Files (Immediate Migration)

### API Integration Tests

| File Path | Network Calls | Services |
|-----------|---------------|----------|
| `/workspaces/eventmesh/tests/api/complete_api_coverage_tests.rs` | 9 | httpbin.org |
| `/workspaces/eventmesh/tests/api/dynamic_rendering_tests.rs` | 7 | example.com, mock URLs |
| `/workspaces/eventmesh/crates/riptide-api/tests/integration_tests.rs` | 3 | various |
| `/workspaces/eventmesh/crates/riptide-api/tests/api_tests.rs` | 2 | httpbin.org |

**Priority**: P0 (Week 1)
**Effort**: 6 hours total

---

### Core Integration Tests

| File Path | Network Calls | Services |
|-----------|---------------|----------|
| `/workspaces/eventmesh/crates/riptide-core/tests/integration_tests.rs` | 2 | various |
| `/workspaces/eventmesh/tests/integration_tests.rs` | 3 | example.com |
| `/workspaces/eventmesh/tests/integration_test.rs` | 2 | various |
| `/workspaces/eventmesh/tests/e2e_tests.rs` | 4 | httpbin.org, example.com |

**Priority**: P0 (Week 1)
**Effort**: 5 hours total

---

### HTML Extraction Tests

| File Path | Network Calls | Services |
|-----------|---------------|----------|
| `/workspaces/eventmesh/tests/golden_tests.rs` | 8 | example.com (inline HTML) |
| `/workspaces/eventmesh/tests/html_extraction_tests.rs` | 2 | example.com |
| `/workspaces/eventmesh/crates/riptide-html/tests/integration_tests.rs` | 4 | example.com |
| `/workspaces/eventmesh/crates/riptide-html/tests/table_extraction_comprehensive_tests.rs` | 1 | example.com |
| `/workspaces/eventmesh/crates/riptide-html/tests/css_merge_policy_performance_tests.rs` | 1 | example.com |
| `/workspaces/eventmesh/crates/riptide-html/tests/css_has_text_tests.rs` | 1 | example.com |

**Priority**: P1 (Week 2)
**Effort**: 8 hours total

---

### Intelligence Provider Tests

| File Path | Network Calls | Services |
|-----------|---------------|----------|
| `/workspaces/eventmesh/crates/riptide-intelligence/tests/provider_tests.rs` | 5 | OpenAI, Azure, Google Vertex APIs |
| `/workspaces/eventmesh/crates/riptide-intelligence/tests/integration_tests.rs` | 3 | AI provider APIs |
| `/workspaces/eventmesh/crates/riptide-intelligence/examples/multi_provider_usage.rs` | 2 | AI APIs |
| `/workspaces/eventmesh/crates/riptide-intelligence/src/providers/base.rs` | 1 | reqwest::Client |
| `/workspaces/eventmesh/crates/riptide-intelligence/src/providers/local.rs` | 1 | reqwest::Client |
| `/workspaces/eventmesh/crates/riptide-intelligence/src/providers/google_vertex.rs` | 1 | Google Vertex API |
| `/workspaces/eventmesh/crates/riptide-intelligence/src/providers/azure.rs` | 1 | Azure OpenAI API |

**Priority**: P0 (Week 1)
**Effort**: 8 hours total

---

### Search Provider Tests

| File Path | Network Calls | Services |
|-----------|---------------|----------|
| `/workspaces/eventmesh/tests/unit/search_provider_test.rs` | 2 | Serper API |
| `/workspaces/eventmesh/tests/unit/serper_provider_test.rs` | 1 | Serper API |
| `/workspaces/eventmesh/tests/unit/riptide_search_providers_tests.rs` | 1 | Search APIs |
| `/workspaces/eventmesh/tests/integration/riptide_search_integration_tests.rs` | 2 | Search APIs |
| `/workspaces/eventmesh/tests/integration/search_provider_integration_test.rs` | 1 | Search APIs |
| `/workspaces/eventmesh/tests/search_provider_integration_test.rs` | 1 | Search APIs |
| `/workspaces/eventmesh/tests/search_provider_test.rs` | 1 | Search APIs |
| `/workspaces/eventmesh/crates/riptide-search/tests/integration_tests.rs` | 2 | Search APIs |
| `/workspaces/eventmesh/crates/riptide-search/src/providers.rs` | 1 | reqwest::Client |

**Priority**: P1 (Week 2)
**Effort**: 7 hours total

---

### Spider/Crawler Tests

| File Path | Network Calls | Services |
|-----------|---------------|----------|
| `/workspaces/eventmesh/tests/integration/spider_integration_tests.rs` | 3 | httpbin.org |
| `/workspaces/eventmesh/tests/spider_query_aware_integration_test.rs` | 2 | httpbin.org |
| `/workspaces/eventmesh/crates/riptide-core/src/spider/tests.rs` | 2 | httpbin.org |
| `/workspaces/eventmesh/crates/riptide-core/src/spider/query_aware_tests.rs` | 1 | httpbin.org |
| `/workspaces/eventmesh/crates/riptide-html/src/spider/tests.rs` | 2 | example.com |
| `/workspaces/eventmesh/tests/week3/spider_tests.rs` | 1 | httpbin.org |
| `/workspaces/eventmesh/tests/week3/dom_spider_tests.rs` | 1 | example.com |

**Priority**: P1 (Week 2)
**Effort**: 6 hours total

---

### Streaming/WebSocket Tests

| File Path | Network Calls | Services |
|-----------|---------------|----------|
| `/workspaces/eventmesh/crates/riptide-api/tests/streaming_sse_ws_tests.rs` | 1 | reqwest::Client |
| `/workspaces/eventmesh/tests/streaming/deepsearch_stream_tests.rs` | 2 | httpbin.org |
| `/workspaces/eventmesh/tests/streaming/ndjson_stream_tests.rs` | 1 | httpbin.org |
| `/workspaces/eventmesh/tests/integration/streaming_validation_tests.rs` | 1 | httpbin.org |

**Priority**: P1 (Week 2)
**Effort**: 4 hours total

---

### Chaos/Edge Case Tests

| File Path | Network Calls | Services |
|-----------|---------------|----------|
| `/workspaces/eventmesh/tests/chaos/edge_cases_tests.rs` | 15 | example.com, invalid URLs |
| `/workspaces/eventmesh/tests/chaos/error_resilience_tests.rs` | 5 | example.com |

**Priority**: P2 (Week 3)
**Effort**: 4 hours total

---

### Security Tests

| File Path | Network Calls | Services |
|-----------|---------------|----------|
| `/workspaces/eventmesh/tests/security/comprehensive_security_tests.rs` | 3 | example.com |

**Priority**: P2 (Week 3)
**Effort**: 2 hours total

---

## Medium-Priority Files (Examples & Documentation)

### Example Files

| Category | File Count | Typical Services |
|----------|------------|------------------|
| Core Examples | 8 | example.com, httpbin.org |
| Intelligence Examples | 3 | AI provider APIs |
| Spider Examples | 5 | various URLs |
| Search Examples | 2 | Search APIs |
| Strategy Examples | 6 | example.com |

**Priority**: P2 (Week 3-4)
**Effort**: 12 hours total

---

### Documentation Files

| Category | File Count | Content Type |
|----------|------------|--------------|
| API Documentation | 15 | curl examples with URLs |
| Architecture Docs | 8 | URL references |
| Testing Guides | 12 | Test examples |
| Configuration Docs | 10 | Example endpoints |

**Priority**: P3 (Week 5-6)
**Effort**: 8 hours total

---

## Low-Priority Files (Source Code with Network Calls)

### Production Code with HTTP Clients

| File Path | Purpose | Mock Needed? |
|-----------|---------|--------------|
| `/workspaces/eventmesh/crates/riptide-core/src/fetch.rs` | HTTP fetching | Yes (integration tests) |
| `/workspaces/eventmesh/crates/riptide-api/src/rpc_client.rs` | RPC client | Yes (unit tests) |
| `/workspaces/eventmesh/crates/riptide-api/src/state.rs` | State management | No (integration only) |
| `/workspaces/eventmesh/crates/riptide-headless/src/lib.rs` | Headless browser | Yes (integration tests) |

**Priority**: P2 (Week 3)
**Effort**: 6 hours total

---

## Benchmark Files

| File Path | Network Calls | Services |
|-----------|---------------|----------|
| `/workspaces/eventmesh/benches/performance_benchmarks.rs` | 2 | example.com |
| `/workspaces/eventmesh/crates/riptide-api/tests/benchmarks/performance_tests.rs` | 1 | httpbin.org |
| `/workspaces/eventmesh/crates/riptide-persistence/benches/persistence_benchmarks.rs` | 1 | example.com |

**Priority**: P3 (Week 5)
**Effort**: 3 hours total

---

## WASM Test Files

| File Path | Network Calls | Services |
|-----------|---------------|----------|
| `/workspaces/eventmesh/wasm/riptide-extractor-wasm/tests/test_wasm_extractor.rs` | 2 | example.com |
| `/workspaces/eventmesh/wasm/riptide-extractor-wasm/tests/integration/mod.rs` | 1 | example.com |
| `/workspaces/eventmesh/wasm/riptide-extractor-wasm/tests/golden/mod.rs` | 1 | example.com |

**Priority**: P2 (Week 3)
**Effort**: 3 hours total

---

## Frontend/Playground Files

| File Path | Network Calls | Services |
|-----------|---------------|----------|
| `/workspaces/eventmesh/playground/src/pages/Streaming.jsx` | 1 | API endpoints |
| `/workspaces/eventmesh/playground/src/components/streaming/LiveProgressWidget.jsx` | 1 | API endpoints |
| `/workspaces/eventmesh/playground/src/components/workers/JobSubmitForm.jsx` | 1 | API endpoints |
| `/workspaces/eventmesh/playground/src/utils/endpoints.js` | 5 | API base URLs |

**Priority**: P3 (Week 6)
**Effort**: 4 hours total

---

## Files Already Using Wiremock ✅

These files demonstrate correct patterns and don't need migration:

1. `/workspaces/eventmesh/tests/integration_fetch_reliability.rs` ✅
2. `/workspaces/eventmesh/crates/riptide-api/tests/integration/test_handlers.rs` ✅
3. `/workspaces/eventmesh/tests/integration_headless_cdp.rs` ✅
4. `/workspaces/eventmesh/tests/integration_pipeline_orchestration.rs` ✅

---

## Migration Priority Summary

| Priority | Phase | File Count | Effort (hours) | Week |
|----------|-------|------------|----------------|------|
| P0 | Critical Tests | 35 | 19 | Week 1 |
| P1 | Integration Tests | 45 | 21 | Week 2 |
| P2 | Source & Examples | 60 | 23 | Week 3-4 |
| P3 | Docs & Benchmarks | 141 | 15 | Week 5-6 |

**Total Effort**: 78 hours across 6 weeks

---

## Migration Tracking

### Week 1 Goals (P0 - Critical)
- [ ] API Integration Tests (6 hours)
- [ ] Core Integration Tests (5 hours)
- [ ] Intelligence Provider Tests (8 hours)

### Week 2 Goals (P1 - High Priority)
- [ ] HTML Extraction Tests (8 hours)
- [ ] Search Provider Tests (7 hours)
- [ ] Spider/Crawler Tests (6 hours)
- [ ] Streaming Tests (4 hours)

### Week 3 Goals (P2 - Medium Priority Part 1)
- [ ] Chaos/Edge Tests (4 hours)
- [ ] Security Tests (2 hours)
- [ ] Production Code Tests (6 hours)
- [ ] WASM Tests (3 hours)

### Week 4 Goals (P2 - Medium Priority Part 2)
- [ ] Example Files (12 hours)

### Week 5-6 Goals (P3 - Low Priority)
- [ ] Documentation (8 hours)
- [ ] Benchmarks (3 hours)
- [ ] Frontend/Playground (4 hours)

---

## Success Metrics

- **Test Execution Time**: Reduce from ~45s to ~0.5s (90x improvement)
- **CI/CD Pipeline**: Reduce from ~15min to ~2min (7.5x improvement)
- **Test Reliability**: Reduce flakiness from 5-10% to <0.1%
- **Offline Capability**: 100% of tests run without internet
- **API Cost**: $0 spent on external API calls during testing

---

**Status**: Ready for implementation
**Last Updated**: 2025-10-10
**Maintained By**: RipTide v1.0 Hive Mind
