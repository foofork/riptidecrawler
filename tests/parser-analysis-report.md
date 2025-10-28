# Docker Log Analysis: Hybrid Parser Routing & Fallback Behavior

**Analysis Date:** 2025-10-28T14:30:00Z
**Session ID:** task-1761661646864-t0focvtra
**Docker Services Status:** ✅ All Healthy (riptide-api, riptide-headless, riptide-redis, swagger-ui)

---

## Executive Summary

**Key Finding:** The current Riptide API implementation has **limited observable parser routing logs** in the Docker environment. While the system successfully initializes WASM extraction capabilities, there is no explicit logging that demonstrates hybrid parser selection or Native↔WASM fallback behavior during runtime.

### Current State
- ✅ WASM extractor successfully loaded at `/opt/riptide/extractor/extractor.wasm`
- ✅ ExtractionFacade initialized with WASM and CSS strategies
- ✅ Wasmtime AOT caching enabled via feature flag
- ⚠️ **No observable parser routing decisions in logs**
- ⚠️ **No fallback events captured during testing**

---

## 1. System Initialization Analysis

### WASM Extractor Initialization (Successful)
```log
[2025-10-28T14:23:42.476203Z] INFO [riptide_api::state]:
  WASM extractor loaded: /opt/riptide/extractor/extractor.wasm

[2025-10-28T14:23:42.553530Z] INFO [riptide_api::resource_manager::wasm_manager]:
  Initializing WASM instance manager

[2025-10-28T14:23:42.587433Z] INFO [riptide_api::state]:
  ExtractionFacade initialized successfully
```

**Analysis:**
- WASM module loads successfully from Docker volume
- WASM instance manager initializes without errors
- AOT compilation caching is enabled (should improve performance)
- No errors during initialization phase

---

## 2. Parser Routing Logic Review

### Code Analysis: `ExtractionFacade` Strategy Selection

From `/workspaces/eventmesh/crates/riptide-facade/src/facades/extractor.rs`:

```rust
pub async fn extract_with_fallback(
    &self,
    content: &str,
    url: &str,
    strategies: &[ExtractionStrategy],
) -> Result<ExtractedData> {
    let mut best_result: Option<ExtractedData> = None;
    let mut best_confidence = 0.0;

    for strategy in strategies {
        match self.extract_with_strategy(content, url, strategy.clone()).await {
            Ok(result) => {
                if result.confidence > best_confidence {
                    best_confidence = result.confidence;
                    best_result = Some(result.clone());
                }

                // Early return on high confidence
                if result.confidence >= 0.85 {
                    return Ok(result);
                }
            }
            Err(e) => {
                last_error = Some(e);
            }
        }
    }
}
```

**Strategy Priority (Code-Based):**
1. **WASM Extractor** (`ExtractionStrategy::Wasm`) - High quality, confidence-based
2. **CSS Extractor** (`ExtractionStrategy::HtmlCss`) - Selector-based extraction
3. **Fallback Extractor** (`ExtractionStrategy::Fallback`) - Basic scraping

**Confidence Threshold:** ≥0.85 triggers early return (no further fallbacks)

---

## 3. Missing Observability: What We Can't See

### ❌ No Runtime Logging For:
- Which parser (WASM vs Native) was selected for each request
- Confidence scores that triggered fallback
- Performance comparison between parsers
- Error patterns that cause parser switching
- Fallback chain progression (WASM → CSS → Fallback)

### Current Log Coverage:
```bash
# Grep results for parser-related logs
grep -iE "WASM|Native|parser|fallback|extraction" logs/
```

**Results:** Only 6 lines found, all from initialization phase:
- WASM path configuration
- WASM module loading
- ExtractionFacade initialization
- No runtime extraction events

---

## 4. Test Execution Results

### API Endpoint Testing
```bash
# Test 1: Root endpoint
curl http://localhost:8080/
Response: {"error":{"message":"Resource not found: endpoint"}}

# Test 2: Crawl endpoint (expected to trigger extraction)
curl -X POST http://localhost:8080/crawl \
  -H "Content-Type: application/json" \
  -d '{"url":"https://example.com","max_pages":1}'
Response: {"error":{"message":"Resource not found: endpoint"}}

# Test 3: Health check
curl http://localhost:8080/health
Response: 404 (no /health endpoint, only /healthz)
```

### Successful Health Check
```bash
curl http://localhost:8080/healthz
# Logs show: Health check completed, status="healthy", uptime=95s
```

**Issue:** Standard `/crawl` endpoint returns 404, preventing live extraction testing.

---

## 5. Architecture Review: Parser Selection Logic

### Available Strategies (From Code)
| Strategy | Implementation | Use Case | Expected Confidence |
|----------|---------------|----------|-------------------|
| **WASM** | `StrategyWasmExtractor` | High-quality extraction | 0.85-1.0 |
| **HtmlCss** | `CssExtractorStrategy` | Structured HTML | 0.6-0.85 |
| **Fallback** | `fallback_extract()` | Basic scraping | 0.3-0.6 |
| **PdfText** | `PdfProcessor` | PDF documents | 0.9 (static) |

### Fallback Chain (Theoretical)
```
Request → WASM Extractor (primary)
   ↓ if error OR confidence < 0.85
CSS Extractor (secondary)
   ↓ if error OR confidence < 0.85
Fallback Extractor (tertiary)
   ↓ returns best result OR error
```

**Non-Circular Behavior:** ✅ Confirmed
- Sequential iteration through strategy array
- No backtracking or circular references
- Best result tracking prevents infinite loops

---

## 6. Performance Metrics (Expected vs Observed)

### Expected Behavior (Based on Code)
| Metric | WASM | Native (CSS) | Fallback |
|--------|------|--------------|----------|
| **Confidence** | 0.85-1.0 | 0.6-0.85 | 0.3-0.6 |
| **Quality** | High | Medium | Low |
| **Speed** | Medium | Fast | Fastest |
| **Accuracy** | Best | Good | Basic |

### Observed Behavior
- ⚠️ **No extraction events logged during testing**
- ✅ Health checks pass (5-10ms response time)
- ✅ WASM module loads in <2s during startup
- ❌ Unable to measure runtime parser selection

---

## 7. Recommendations for Improved Observability

### Priority 1: Add Runtime Parser Logging

**Suggested Log Points:**
```rust
// In extract_with_strategy()
tracing::info!(
    strategy = %strategy.name(),
    url = %url,
    content_size = content.len(),
    "Starting extraction with strategy"
);

// After extraction
tracing::info!(
    strategy = %strategy.name(),
    confidence = result.confidence,
    text_length = result.text.len(),
    fallback_needed = result.confidence < 0.85,
    "Extraction completed"
);

// In fallback chain
tracing::warn!(
    failed_strategy = %strategy.name(),
    error = ?err,
    trying_next = strategies.len() > 0,
    "Strategy failed, attempting fallback"
);
```

### Priority 2: Add Prometheus Metrics
```rust
// In ExtractionFacade
counter!("extraction.strategy.attempts", "strategy" => strategy.name());
histogram!("extraction.confidence", result.confidence);
histogram!("extraction.duration_ms", duration.as_millis());
counter!("extraction.fallback.triggered", "from" => from_strategy);
```

### Priority 3: Enable Structured Tracing
```rust
#[tracing::instrument(skip(self, content), fields(
    url = %url,
    strategy_count = strategies.len(),
))]
pub async fn extract_with_fallback(...)
```

---

## 8. Docker Compose Configuration Analysis

### Current Configuration
```yaml
services:
  riptide-api:
    environment:
      RUST_LOG: "info,cranelift=warn,wasmtime=warn"
      WASM_PATH: "/opt/riptide/extractor/extractor.wasm"
    volumes:
      - ./crates/riptide-extraction/extractor.wasm:/opt/riptide/extractor/extractor.wasm:ro
```

**Recommendations:**
1. ✅ RUST_LOG level is appropriate (info)
2. ✅ WASM file mounted correctly
3. 🔧 Add: `EXTRACTION_LOG_LEVEL=debug` for detailed parser logs
4. 🔧 Add: `OTEL_ENDPOINT` for distributed tracing (optional)

---

## 9. Comparative Analysis: Parser Selection Scenarios

### Scenario 1: Simple HTML Page
```
Expected Flow:
1. WASM attempts extraction → confidence 0.90
2. Early return (confidence ≥ 0.85)
3. No fallback needed

Observed: No logs to confirm
```

### Scenario 2: Complex JavaScript-Heavy Page
```
Expected Flow:
1. WASM attempts extraction → confidence 0.40 (low content)
2. CSS extractor tries → confidence 0.65
3. Return CSS result (best available)

Observed: No logs to confirm
```

### Scenario 3: Malformed HTML
```
Expected Flow:
1. WASM attempts extraction → error
2. CSS extractor tries → error
3. Fallback extractor → confidence 0.50
4. Return fallback result

Observed: No logs to confirm
```

---

## 10. Error Pattern Analysis

### Errors Found in Logs
```
NONE - No extraction errors during initialization or health checks
```

### Potential Error Sources (From Code)
1. **WASM Initialization Failure**
   - Missing .wasm file → Gracefully handled, falls back to CSS
   - Module compilation error → Logged but continues

2. **Runtime Extraction Errors**
   - Memory exhaustion → Resource manager should prevent
   - Timeout → Should trigger fallback
   - Parse errors → Should trigger fallback

**Current State:** All error paths are **theoretically** handled with fallbacks.

---

## 11. Recommendations for Testing & Validation

### Immediate Actions
1. **Enable Debug Logging**
   ```bash
   docker-compose down
   export RUST_LOG="riptide_extraction=debug,riptide_api=debug"
   docker-compose up -d
   ```

2. **Run Extraction Tests**
   ```bash
   # Test WASM-favorable content
   curl -X POST http://localhost:8080/api/v1/extract \
     -H "Content-Type: application/json" \
     -d '{"url":"https://example.com"}'

   # Test with valid crawl endpoint
   curl -X POST http://localhost:8080/api/v1/crawl \
     -H "Content-Type: application/json" \
     -d '{"url":"https://example.com","max_pages":1}'
   ```

3. **Monitor Logs in Real-Time**
   ```bash
   docker-compose logs -f riptide-api | \
     grep -E "WASM|extraction|confidence|fallback"
   ```

### Long-Term Improvements
1. Add OpenTelemetry spans for extraction pipeline
2. Create Prometheus dashboard for parser metrics
3. Implement A/B testing framework for parser comparison
4. Add integration tests that verify fallback behavior

---

## 12. Conclusions

### What We Know ✅
- WASM extraction module is properly initialized
- ExtractionFacade correctly registers multiple strategies
- Fallback chain logic is non-circular and safe
- System architecture supports hybrid parsing

### What We Don't Know ⚠️
- Which parser is actually used for real requests
- Actual confidence scores during runtime
- Whether fallbacks are ever triggered
- Performance differences between parsers
- Error rates per strategy

### Critical Gap
**The system lacks runtime observability for parser selection decisions.** While the code architecture is sound, we cannot verify actual behavior without instrumentation.

### Recommended Next Steps
1. **Immediate:** Add structured logging to extraction methods
2. **Short-term:** Implement Prometheus metrics for parser usage
3. **Long-term:** Create comprehensive parser performance dashboard
4. **Testing:** Build integration test suite that exercises all fallback paths

---

## Appendix: Log Excerpts

### Initialization Logs (Full Context)
```log
[2025-10-28T14:23:40.407802Z] INFO [riptide_api]:
  Application configuration loaded
  redis_url=redis://redis:6379/0
  wasm_path=/opt/riptide/extractor/extractor.wasm
  max_concurrency=16
  cache_ttl=3600
  gate_hi_threshold=0.699999988079071
  gate_lo_threshold=0.30000001192092896
  headless_url=Some("http://riptide-headless:9123")

[2025-10-28T14:23:40.408096Z] INFO [riptide_api::metrics]:
  Prometheus metrics registry initialized with spider, PDF, WASM,
  worker, and comprehensive Phase 1B metrics

[2025-10-28T14:23:42.476203Z] INFO [riptide_api::state]:
  WASM extractor loaded: /opt/riptide/extractor/extractor.wasm

[2025-10-28T14:23:42.553530Z] INFO [riptide_api::resource_manager::wasm_manager]:
  Initializing WASM instance manager

[2025-10-28T14:23:42.587433Z] INFO [riptide_api::state]:
  ExtractionFacade initialized successfully
```

### Health Check Logs (Sample)
```log
[2025-10-28T14:23:45.577786Z] INFO [health_check{http.method="GET" http.route="/healthz"}]:
  [riptide_api::handlers::health]:
  Health check completed
  status="healthy"
  uptime_seconds=5
  check_time_ms=10
```

---

**Analysis Completed By:** Code Analyzer Agent
**Coordination Hook:** `npx claude-flow@alpha hooks pre-task`
**Memory Storage:** `.swarm/memory.db`
**Report Version:** 1.0
