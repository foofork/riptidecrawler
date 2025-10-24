# Technical Feasibility Assessment - Engine Selection Enhancements

**Assessment Date:** 2025-10-23
**Assessed By:** Coder Agent (Hive Mind Swarm)
**Codebase Version:** main @ 9b67ccc

---

## Executive Summary

This assessment evaluates 5 proposed enhancements to the RipTide extraction engine selection system. The proposals aim to reduce headless browser usage by adding probe-first escalation, JSON-LD short-circuits, WASM micro-network capabilities, improved signal detection, and domain warm-start caching.

**Overall Risk:** Medium (blocking dependencies on WASM runtime modifications)
**Overall Effort:** 2-3 weeks (320-480 LOC total)
**Breaking Changes:** No (all enhancements are additive)

---

## 1. Probe-First Escalation (SPA → WASM → Headless)

### Current Implementation
- **Location:** `crates/riptide-reliability/src/engine_selection.rs:137-192`
- **Current Logic:** Lines 181-183 directly jump to `Engine::Headless` for SPA/React/Vue/Angular frameworks
- **Problem:** No intermediate WASM probe step, causing unnecessary headless usage

### Proposed Changes

#### File: `engine_selection.rs`
```rust
// Lines 181-186 (CURRENT):
} else if has_react || has_vue || has_angular || has_spa_markers {
    Engine::Headless
} else if content_ratio < 0.1 {
    Engine::Headless
}

// PROPOSED:
} else if has_react || has_vue || has_angular || has_spa_markers {
    Engine::Wasm  // Try WASM probe first
} else if content_ratio < 0.1 {
    Engine::Wasm  // Try WASM probe first
}
```

#### New Module: `crates/riptide-extraction/src/probe_engine.rs`
```rust
pub enum ProbeOutcome {
    Populated { dom_growth: usize, network_needed: bool },
    NeedsNetwork,
    NoGrowth,
    Blocked,
}

pub struct ProbeOpts {
    timeout_ms: u32,
    detect_mutations: bool,
}

pub async fn probe_wasm(html: &str, opts: ProbeOpts) -> Result<ProbeOutcome>;
```

#### Integration Point: `crates/riptide-cli/src/commands/extract.rs`
Add probe logic after engine selection, before WASM extraction.

### Feasibility Assessment

| Metric | Value | Notes |
|--------|-------|-------|
| **Estimated LOC** | 80-120 | New probe module + integration |
| **Affected Modules** | 3 | `engine_selection.rs`, new `probe_engine.rs`, `extract.rs` |
| **Breaking Changes** | N | Additive only, backward compatible |
| **Test Coverage Required** | 85%+ | Critical path, needs extensive tests |
| **Risk Level** | **Low** | Simple control flow change |

### Implementation Complexity
- **Easy:** Engine selection modification (5 LOC)
- **Medium:** Probe module creation (~80 LOC)
- **Easy:** CLI integration (~20 LOC)

### Dependencies
- Requires WASM runtime metrics (already exists: `WasmResourceTracker`)
- Needs DOM mutation detection (can use `scraper` crate diffing)

### Test Requirements
1. Unit tests for probe logic (10 tests)
2. Integration tests for escalation path (5 scenarios)
3. Performance benchmarks (WASM vs Headless on SPA sites)

### Risks & Mitigations

| Risk | Impact | Likelihood | Mitigation |
|------|--------|------------|-----------|
| False positives (WASM fails on valid SPAs) | Medium | Low | Tunable thresholds, fast fallback |
| Increased latency (probe + extraction) | Low | Medium | 50ms timeout, parallel probe |
| WASM OOM on large DOMs | High | Low | Existing memory limits (8192 pages) |

---

## 2. JSON-LD Short-Circuit for Events/Articles

### Current Implementation
- **Location:** `crates/riptide-extraction/src/strategies/metadata.rs:168-271`
- **Current Logic:** Extracts JSON-LD but doesn't check completeness or early-return
- **Problem:** Always proceeds to full extraction even when JSON-LD has complete data

### Proposed Changes

#### File: `metadata.rs` (after line 186)
```rust
fn extract_json_ld(...) -> Result<()> {
    let selector = Selector::parse("script[type='application/ld+json']").unwrap();

    for element in document.select(&selector) {
        let json_text = element.text().collect::<String>();

        if let Ok(json_value) = serde_json::from_str::<serde_json::Value>(&json_text) {
            // NEW: Check for complete Event schema
            if let Some(complete_event) = validate_complete_event(&json_value) {
                return Ok(/* Early return with high confidence */);
            }

            // NEW: Check for complete Article schema
            if let Some(complete_article) = validate_complete_article(&json_value) {
                return Ok(/* Early return with high confidence */);
            }

            extract_from_json_ld(&json_value, metadata)?;
            method.json_ld = true;
        }
    }

    Ok(())
}

// NEW functions:
fn validate_complete_event(json: &Value) -> Option<Event> { /* ... */ }
fn validate_complete_article(json: &Value) -> Option<Article> { /* ... */ }
```

### Feasibility Assessment

| Metric | Value | Notes |
|--------|-------|-------|
| **Estimated LOC** | 60-80 | Validation functions + early return logic |
| **Affected Modules** | 1 | `metadata.rs` only |
| **Breaking Changes** | N | Additive, improves confidence scoring |
| **Test Coverage Required** | 90%+ | Schema validation is critical |
| **Risk Level** | **Low** | Isolated change, high value |

### Implementation Complexity
- **Easy:** Early return logic (10 LOC)
- **Medium:** Schema validation (50-70 LOC)

### Schema Completeness Criteria

**Event (schema.org/Event):**
- Required: `name`, `startDate`, `location.name` OR `location.address`
- Optional but boosting: `url`, `organizer`, `description`
- Confidence: 0.90 if all required present

**Article (schema.org/Article):**
- Required: `headline`, `datePublished`, `author.name`
- Optional but boosting: `articleBody`, `image`, `publisher`
- Confidence: 0.85 if all required present

### Test Requirements
1. Unit tests for validation (15 tests covering edge cases)
2. Golden file tests with real-world JSON-LD
3. Performance regression tests (no slowdown on non-JSON-LD pages)

### Risks & Mitigations

| Risk | Impact | Likelihood | Mitigation |
|------|--------|------------|-----------|
| False completeness detection | High | Medium | Strict schema validation |
| Missing non-JSON-LD content | Medium | Low | Fallback to full extraction if low confidence |
| Schema.org version drift | Low | Low | Regular schema updates |

---

## 3. WASM Micro-Network (Same-Origin Budgeted Fetch)

### Current Implementation
- **Location:** `crates/riptide-extraction/src/wasm_extraction.rs:29-56`
- **Current Logic:** `HostExtractionMode` has 4 modes: Article, Full, Metadata, Custom
- **Problem:** No network-enabled mode for WASM (always network-free or headless)

### Proposed Changes

#### File: `wasm_extraction.rs` (line 43)
```rust
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum HostExtractionMode {
    Article,
    Full,
    Metadata,
    Custom(Vec<String>),

    // NEW: Network-enabled mode with budget
    FullWithNetwork(NetworkBudget),
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct NetworkBudget {
    pub max_requests: u32,      // e.g., 4
    pub max_total_bytes: u64,   // e.g., 300KB
    pub same_origin_only: bool, // Always true for security
    pub allowed_paths: Vec<String>, // e.g., ["/api/*", "/__NEXT_DATA__"]
}
```

#### New Module: `crates/riptide-extraction/src/wasm_network.rs`
```rust
pub struct WasmNetworkGuard {
    budget: NetworkBudget,
    requests_made: AtomicU32,
    bytes_consumed: AtomicU64,
    page_origin: Url,
}

impl WasmNetworkGuard {
    pub fn allow_request(&self, url: &Url) -> Result<bool, NetworkError>;
    pub fn track_response(&self, url: &Url, size: u64) -> Result<()>;
}
```

#### WIT Interface Changes (WASM-host boundary)
**File:** `wasm/riptide-extractor-wasm/wit/extractor.wit`
```wit
variant extraction-mode {
    article,
    full,
    metadata,
    custom(list<string>),
    full-with-network(network-budget),  // NEW
}

record network-budget {
    max-requests: u32,
    max-total-bytes: u64,
    same-origin-only: bool,
}
```

### Feasibility Assessment

| Metric | Value | Notes |
|--------|-------|-------|
| **Estimated LOC** | 150-200 | Network guard + WIT changes + host integration |
| **Affected Modules** | 4 | `wasm_extraction.rs`, new `wasm_network.rs`, `extractor.wit`, WASM guest |
| **Breaking Changes** | N | New variant, existing modes unchanged |
| **Test Coverage Required** | 95%+ | Security-critical, needs fuzzing |
| **Risk Level** | **High** | Security + WASM runtime complexity |

### Implementation Complexity
- **Medium:** Host-side network guard (~80 LOC)
- **Hard:** WIT interface changes (~30 LOC WIT + 50 LOC bindings)
- **Hard:** WASM guest fetch interception (~100 LOC in guest)

### Security Considerations

**CRITICAL RISKS:**
1. **Origin bypass:** Must validate ALL URLs against page origin
2. **SSRF attacks:** No localhost/private IP access
3. **Budget exhaustion:** Hard limits, no retries
4. **Timing attacks:** Constant-time origin validation

**Mitigations:**
- Whitelist approach (deny by default)
- URL parsing with `url` crate (battle-tested)
- Atomic counters for concurrency safety
- Extensive fuzzing tests

### Test Requirements
1. Security tests (15 tests): origin bypass, SSRF, budget attacks
2. Unit tests (10 tests): budget tracking, URL validation
3. Integration tests (8 tests): real-world API endpoints
4. Fuzz tests (3 campaigns): URL parsing, budget logic

### Risks & Mitigations

| Risk | Impact | Likelihood | Mitigation |
|------|--------|------------|-----------|
| Origin bypass vulnerability | **Critical** | Low | Strict URL parsing, audit |
| SSRF to internal services | **Critical** | Low | Block RFC1918/loopback |
| Budget bypass (race conditions) | High | Medium | Atomic operations |
| WASM-host communication overhead | Medium | High | Preallocated buffers |
| Guest code complexity | Medium | High | Simplify to HTTP GET only |

### Blocking Dependencies
- **WASM guest modifications:** Requires changes to `wasm/riptide-extractor-wasm/src/lib.rs`
- **WIT version compatibility:** Wasmtime 37 component model constraints
- **HTTP client in WASM:** Use `reqwest` with WASI HTTP (experimental)

---

## 4. Improved Signal Detection (Text Density + Placeholder Detection)

### Current Implementation
- **Location:** `crates/riptide-reliability/src/engine_selection.rs:307-322`
- **Current Logic:** Lines 315-321 use naive `split('<')` for content ratio
- **Problem:** Counts script tags, style tags, and empty tags as content

### Proposed Changes

#### File: `engine_selection.rs` (replace lines 307-322)
```rust
pub fn calculate_content_ratio(html: &str) -> f64 {
    let total_len = html.len() as f64;
    if total_len == 0.0 {
        return 0.0;
    }

    // NEW: Parse and extract only visible text
    let document = Html::parse_document(html);

    // Remove non-visible elements
    let blocklist = ["script", "style", "noscript", "head", "meta", "link"];
    let visible_text = extract_visible_text(&document, &blocklist);

    // Detect placeholder patterns
    let placeholder_ratio = detect_placeholders(&document);

    // Calculate adjusted ratio
    let text_density = visible_text.len() as f64 / total_len;
    let adjusted_ratio = text_density * (1.0 - placeholder_ratio);

    adjusted_ratio
}

fn extract_visible_text(doc: &Html, blocklist: &[&str]) -> String { /* ... */ }
fn detect_placeholders(doc: &Html) -> f64 { /* ... */ }
```

#### New Struct: `DetectorMetrics`
```rust
#[derive(Debug, Clone)]
pub struct DetectorMetrics {
    pub text_density: f64,
    pub placeholder_hits: usize,
    pub spa_framework: Option<String>,
    pub noscript_ratio: f64,
    pub confidence_score: f64,
}

impl DetectorMetrics {
    pub fn analyze(html: &str) -> Self { /* ... */ }
    pub fn recommend_engine(&self) -> Engine { /* ... */ }
}
```

### Feasibility Assessment

| Metric | Value | Notes |
|--------|-------|-------|
| **Estimated LOC** | 100-150 | Text extraction + placeholder detection + metrics |
| **Affected Modules** | 1 | `engine_selection.rs` (refactor) |
| **Breaking Changes** | N | Internal calculation change only |
| **Test Coverage Required** | 90%+ | Core heuristic, needs extensive test cases |
| **Risk Level** | **Low** | Deterministic logic, low risk |

### Implementation Complexity
- **Medium:** Visible text extraction (~50 LOC, uses `scraper`)
- **Medium:** Placeholder detection (~40 LOC, regex patterns)
- **Easy:** Metrics struct (~30 LOC)

### Placeholder Detection Patterns
```rust
const PLACEHOLDER_CLASSES: &[&str] = &[
    "skeleton", "shimmer", "placeholder", "loading",
    "spinner", "lazy-load", "content-loader"
];

const PLACEHOLDER_IDS: &[&str] = &[
    "root", "app", "__next", "react-root"
];

const SPA_MARKERS: &[&str] = &[
    "__NEXT_DATA__", "__NUXT__", "ng-version",
    "data-reactroot", "v-app"
];
```

### Test Requirements
1. Unit tests for text extraction (12 tests)
2. Placeholder detection tests (10 tests)
3. Regression tests against old ratio calculation (20 golden files)
4. Performance benchmarks (no slowdown on large HTML)

### Risks & Mitigations

| Risk | Impact | Likelihood | Mitigation |
|------|--------|------------|-----------|
| Slowdown on large HTML | Medium | Medium | Cache parsed DOM, incremental parsing |
| False placeholder detection | Low | Medium | Tunable thresholds, confidence scoring |
| Regex backtracking (ReDoS) | Low | Low | Use `regex` crate (NFA-based) |

---

## 5. Domain Warm-Start (Redis Engine Preference Cache)

### Current Implementation
- **Location:** `crates/riptide-intelligence/src/domain_profiling/profiler.rs:17-27`
- **Current Structure:** `DomainProfile` has config, baseline, metadata, patterns
- **Problem:** No `preferred_engine` field or TTL-based caching

### Proposed Changes

#### File: `profiler.rs` (add to struct at line 27)
```rust
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct DomainProfile {
    pub name: String,
    pub domain: String,
    pub version: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub config: DomainConfig,
    pub baseline: Option<SiteBaseline>,
    pub metadata: DomainMetadata,
    pub patterns: DomainPatterns,

    // NEW: Engine preference tracking
    pub preferred_engine: Option<Engine>,
    pub last_success_confidence: Option<f32>,
    pub engine_success_rate: Option<f32>,
    pub last_engine_update: Option<DateTime<Utc>>,
}
```

#### New Module: `crates/riptide-cache/src/engine_cache.rs`
```rust
pub struct EngineCacheManager {
    redis_client: RedisClient,
    ttl_seconds: u64,
}

impl EngineCacheManager {
    pub async fn get_preferred_engine(&self, domain: &str) -> Option<Engine>;
    pub async fn update_engine_preference(&self, domain: &str, engine: Engine, confidence: f32);
    pub async fn increment_success_rate(&self, domain: &str, engine: Engine);
}
```

#### Integration: `engine_selection.rs`
```rust
pub async fn decide_engine_with_cache(
    html: &str,
    url: &Url,
    cache: &EngineCacheManager,
) -> Engine {
    // Check cache first
    if let Some(cached_engine) = cache.get_preferred_engine(url.host_str()).await {
        return cached_engine;
    }

    // Fallback to analysis
    decide_engine(html, url.as_str())
}
```

### Feasibility Assessment

| Metric | Value | Notes |
|--------|-------|-------|
| **Estimated LOC** | 120-180 | Redis cache layer + profile extension + integration |
| **Affected Modules** | 3 | `profiler.rs`, new `engine_cache.rs`, `engine_selection.rs` |
| **Breaking Changes** | N | Additive fields, backward compatible |
| **Test Coverage Required** | 80%+ | Cache logic + TTL expiration |
| **Risk Level** | **Medium** | Redis dependency + cache invalidation |

### Implementation Complexity
- **Easy:** Struct field additions (~10 LOC)
- **Medium:** Redis cache manager (~80 LOC)
- **Easy:** Integration with engine selection (~30 LOC)

### Cache Strategy
- **Key Format:** `engine:pref:{domain}`
- **Value:** JSON-serialized `{ engine: "wasm", confidence: 0.85, success_rate: 0.92 }`
- **TTL:** 7 days (configurable)
- **Update Trigger:** On successful extraction with confidence >= 0.75

### Test Requirements
1. Unit tests for cache CRUD (8 tests)
2. TTL expiration tests (4 tests)
3. Integration tests with Redis (6 tests)
4. Performance benchmarks (cache hit latency)

### Risks & Mitigations

| Risk | Impact | Likelihood | Mitigation |
|------|--------|------------|-----------|
| Redis unavailability | Medium | Low | Graceful fallback to analysis |
| Stale cache (site changes) | Medium | Medium | 7-day TTL, manual invalidation API |
| Cache poisoning (bad engines cached) | Low | Low | Confidence threshold (>=0.75) |
| Redis memory exhaustion | Low | Low | LRU eviction, max 10K domains |

### Dependencies
- **Redis:** Already integrated in `riptide-cache` crate
- **Serialization:** `serde_json` for engine storage

---

## Cross-Cutting Concerns

### 1. Testing Strategy
- **Unit Tests:** 60+ new tests
- **Integration Tests:** 25+ scenarios
- **Performance Tests:** 10+ benchmarks
- **Golden File Tests:** 30+ real-world pages

### 2. Performance Impact
- **Probe-first:** +50ms initial probe, -2000ms headless savings = **NET -1950ms**
- **JSON-LD short-circuit:** -500ms full extraction = **NET -500ms**
- **WASM micro-network:** +200ms network requests, -2000ms headless = **NET -1800ms**
- **Signal improvements:** +10ms parsing overhead = **NET +10ms**
- **Domain warm-start:** +5ms Redis lookup, -50ms analysis = **NET -45ms**

**Total Estimated Speedup:** 35-50% on SPA/event sites

### 3. Monitoring & Observability
- Add metrics for each enhancement:
  - `riptide_probe_success_rate`
  - `riptide_jsonld_shortcircuits_total`
  - `riptide_wasm_network_requests_total`
  - `riptide_signal_confidence_score`
  - `riptide_engine_cache_hit_rate`

### 4. Configuration
All enhancements should be feature-gated:
```toml
[features]
default = ["probe-first", "jsonld-shortcircuit", "signal-improvements", "domain-warmstart"]
full = ["default", "wasm-network"]  # High-risk feature
```

---

## Implementation Roadmap

### Phase 1: Low-Risk Quick Wins (Week 1)
1. **Signal Improvements** (2 days)
   - Refactor content ratio calculation
   - Add placeholder detection
   - Test on 30 golden files

2. **JSON-LD Short-Circuit** (2 days)
   - Add validation functions
   - Early return logic
   - Schema completeness tests

3. **Probe-First Escalation** (3 days)
   - Create probe module
   - Update engine selection
   - Integration tests

### Phase 2: Medium-Risk Cache Layer (Week 2)
4. **Domain Warm-Start** (5 days)
   - Extend DomainProfile
   - Redis cache manager
   - Integration + TTL tests

### Phase 3: High-Risk Network Feature (Week 3)
5. **WASM Micro-Network** (7-10 days)
   - NetworkBudget struct
   - WIT interface changes
   - Security audit + fuzzing
   - WASM guest modifications

**Note:** Phase 3 may require 1-2 additional weeks for thorough security review.

---

## Blockers & Dependencies

### Critical Blockers
1. **WASM Guest Codebase Access** (Enhancement #3)
   - Need write access to `wasm/riptide-extractor-wasm/src/`
   - Requires Wasmtime component model expertise

2. **Security Audit Resources** (Enhancement #3)
   - Network-enabled WASM needs external security review
   - Est. 3-5 days from security team

### Non-Blocking Dependencies
- Redis server for local testing (Enhancement #5)
- Golden file corpus for testing (Enhancements #1, #2, #4)

---

## Risk Matrix

| Enhancement | Tech Risk | Security Risk | Timeline Risk | Overall Risk |
|-------------|-----------|---------------|---------------|--------------|
| Probe-First | Low | Low | Low | **Low** |
| JSON-LD Short-Circuit | Low | Low | Low | **Low** |
| WASM Micro-Network | High | **Critical** | High | **High** |
| Signal Improvements | Low | Low | Low | **Low** |
| Domain Warm-Start | Medium | Low | Low | **Medium** |

---

## Recommendations

### Ship Immediately (Low Risk, High Value)
1. **JSON-LD Short-Circuit** - 60-80 LOC, huge win for events/articles
2. **Signal Improvements** - 100-150 LOC, better heuristics
3. **Probe-First Escalation** - 80-120 LOC, reduces headless usage

### Ship Next Sprint (Medium Risk, High Value)
4. **Domain Warm-Start** - 120-180 LOC, requires Redis but low risk

### Defer for Security Review (High Risk, High Complexity)
5. **WASM Micro-Network** - 150-200 LOC, needs thorough security audit

---

## Post-Task Coordination

Storing assessment in Hive Mind memory:

```bash
npx claude-flow@alpha hooks post-task \
  --task-id "coder-feasibility" \
  --memory-key "hive/coder/feasibility" \
  --summary "Completed feasibility assessment: 5 enhancements, 320-480 LOC total, 3 low-risk/1 med/1 high-risk"
```

**Next Steps:**
1. Planner agent to prioritize enhancements based on risk/value
2. Tester agent to design comprehensive test strategy
3. Architect agent to review WASM security boundaries
4. Reviewer agent to validate implementation approach

---

**Assessment Status:** ✅ Complete
**Confidence:** High (code-level analysis completed)
**Blocking Issues:** WASM guest codebase access for Enhancement #3
