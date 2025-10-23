# Phase 5: Engine Selection Consolidation - Architecture Design

**Status:** DESIGN COMPLETE
**Version:** 1.0
**Date:** 2025-10-23
**Architect:** System Architecture Designer

---

## 1. Executive Summary

### 1.1 Problem Statement

Currently, engine selection logic (Raw → WASM → Headless fallback chain) is duplicated across two major components:
- **CLI:** `/workspaces/eventmesh/crates/riptide-cli/src/commands/engine_fallback.rs` (~475 LOC)
- **API:** Partial implementation in `/workspaces/eventmesh/crates/riptide-api/src/rpc_client.rs`

This duplication leads to:
- ❌ **Inconsistent behavior** between CLI and API
- ❌ **Maintenance burden** (bugs fixed twice)
- ❌ **Testing overhead** (duplicate test suites)
- ❌ **Code drift risk** (implementations diverge over time)

### 1.2 Solution Overview

Consolidate engine selection logic into **`riptide-reliability::engine_selection`** module with:
- ✅ **Single source of truth** for engine decision logic
- ✅ **Zero dependency cycles** (depends only on kernel types)
- ✅ **Confidence scoring system** for intelligent fallback
- ✅ **Framework detection** (React, Vue, Angular, SPA markers)
- ✅ **Content analysis heuristics** (text ratio, anti-scraping detection)
- ✅ **Comprehensive test coverage** (unit + integration)

---

## 2. Architecture Decision

### 2.1 Option Analysis

#### Option 1: `riptide-reliability::engine_selection` module ✅ **SELECTED**

**Rationale:**
- ✅ `riptide-reliability` already exists and has appropriate scope
- ✅ Natural fit for "reliability patterns" (circuit breakers, fallback logic)
- ✅ Already depends on `riptide-types` (no new dependencies)
- ✅ Both CLI and API already depend on `riptide-reliability`
- ✅ No circular dependency risks
- ✅ Maintains modular architecture

**Current `riptide-reliability` dependencies:**
```toml
riptide-types = { path = "../riptide-types" }
riptide-fetch = { path = "../riptide-fetch" }
riptide-events = { path = "../riptide-events", optional = true }
riptide-monitoring = { path = "../riptide-monitoring", optional = true }
```

#### Option 2: Tiny internal crate (publish = false)

**Rationale for rejection:**
- ❌ Adds unnecessary complexity (new crate overhead)
- ❌ `riptide-reliability` already provides the perfect home
- ❌ No technical advantage over Option 1

#### Option 3: Move to `riptide-types`

**Rationale for rejection:**
- ❌ `riptide-types` is for data structures, not business logic
- ❌ Would require adding heavy dependencies (HTTP client, regex, etc.)
- ❌ Violates single responsibility principle

---

## 3. Module Design

### 3.1 Module Structure

```
riptide-reliability/
├── src/
│   ├── circuit.rs              # Existing atomic circuit breaker
│   ├── circuit_breaker.rs      # Existing state-based circuit breaker
│   ├── gate.rs                 # Existing gate decision logic (REUSE!)
│   ├── reliability.rs          # Existing reliability patterns
│   ├── engine_selection.rs     # ✨ NEW: Engine selection module
│   └── lib.rs                  # Re-export engine_selection types
└── tests/
    └── engine_selection_tests.rs  # ✨ NEW: Comprehensive tests
```

### 3.2 API Surface

```rust
// File: riptide-reliability/src/engine_selection.rs

use crate::gate::{GateFeatures, Decision, score, should_use_headless};
use riptide_types::{RenderMode, ExtractionConfig};
use serde::{Deserialize, Serialize};

/// Engine types in fallback chain
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum EngineType {
    /// Raw HTTP fetch with static HTML parsing (fastest)
    Raw,
    /// WASM-based extraction (balanced)
    Wasm,
    /// Headless browser with full JS execution (slowest, most reliable)
    Headless,
}

/// Unified engine decision result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EngineDecision {
    /// Primary engine to try first
    pub primary_engine: EngineType,
    /// Fallback chain (ordered by preference)
    pub fallback_chain: Vec<EngineType>,
    /// Confidence score (0.0-1.0)
    pub confidence: f32,
    /// Reasoning for decision (for debugging/logging)
    pub reasoning: String,
}

/// Content analysis for engine selection
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContentAnalysis {
    /// Detected JavaScript framework (React, Vue, Angular, etc.)
    pub detected_framework: Option<Framework>,
    /// SPA (Single Page Application) indicators
    pub spa_markers: Vec<SpaMarker>,
    /// Anti-scraping detection (Cloudflare, reCAPTCHA, etc.)
    pub anti_scraping: Option<AntiScrapingType>,
    /// Content quality score (0.0-1.0)
    pub content_quality_score: f32,
    /// Text-to-markup ratio
    pub text_ratio: f32,
    /// Main content presence
    pub has_main_content: bool,
}

/// JavaScript framework detection
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum Framework {
    React,
    NextJs,
    Vue,
    Angular,
    Svelte,
    Unknown,
}

/// SPA (Single Page Application) markers
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum SpaMarker {
    NextData,              // __NEXT_DATA__ script tag
    ReactRoot,             // _reactRoot or data-reactroot
    WebpackRequire,        // __webpack_require__
    VueApp,                // v-app or createApp
    AngularApp,            // ng-app or platformBrowserDynamic
    InitialState,          // window.__INITIAL_STATE__
    ReactHelmet,           // data-react-helmet
}

/// Anti-scraping technology detection
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum AntiScrapingType {
    Cloudflare,
    Recaptcha,
    HCaptcha,
    PerimeterX,
    DataDome,
    Akamai,
}

/// Main decision function: analyze content and decide optimal engine
pub fn decide_engine(url: &str, html_content: &str, content_type: Option<&str>) -> EngineDecision;

/// Detect JavaScript framework from HTML content
pub fn detect_framework(html: &str) -> Option<Framework>;

/// Detect SPA markers in HTML
pub fn detect_spa_markers(html: &str) -> Vec<SpaMarker>;

/// Detect anti-scraping technology
pub fn detect_anti_scraping(html: &str) -> Option<AntiScrapingType>;

/// Calculate content quality score (0.0-1.0)
pub fn calculate_content_quality(html: &str) -> f32;

/// Calculate text-to-markup ratio
pub fn calculate_text_ratio(html: &str) -> f32;

/// Check for main content elements
pub fn has_main_content_markers(html: &str) -> bool;

/// Perform comprehensive content analysis
pub fn analyze_content(url: &str, html: &str) -> ContentAnalysis;

/// Validate extraction quality (sufficient content extracted)
pub fn validate_extraction_quality(
    content: &str,
    confidence: Option<f32>,
    min_content_length: usize,
    min_confidence: f32,
) -> bool;
```

### 3.3 Integration Points

#### 3.3.1 CLI Integration

```rust
// File: riptide-cli/src/commands/extract.rs

use riptide_reliability::engine_selection::{decide_engine, EngineType};

pub async fn execute_extract(url: &str) -> Result<ExtractResponse> {
    // Fetch initial HTML
    let html = fetch_html(url).await?;

    // Decide optimal engine
    let decision = decide_engine(url, &html, None);

    tracing::info!(
        "Engine decision: {:?} (confidence: {:.2})",
        decision.primary_engine,
        decision.confidence
    );

    // Execute with fallback chain
    for engine in std::iter::once(decision.primary_engine)
        .chain(decision.fallback_chain.iter().copied())
    {
        match engine {
            EngineType::Raw => { /* try raw extraction */ },
            EngineType::Wasm => { /* try WASM extraction */ },
            EngineType::Headless => { /* try headless extraction */ },
        }
    }
}
```

#### 3.3.2 API Integration

```rust
// File: riptide-api/src/handlers/extract.rs

use riptide_reliability::engine_selection::{decide_engine, EngineType};

pub async fn handle_extract_request(req: ExtractRequest) -> Result<ExtractResponse> {
    let html = fetch_html(&req.url).await?;
    let decision = decide_engine(&req.url, &html, req.content_type.as_deref());

    // Execute with fallback (same logic as CLI)
    execute_with_fallback(decision, &req).await
}
```

### 3.4 Reuse Existing `gate.rs` Logic

**Key insight:** `riptide-reliability::gate` already provides excellent content analysis!

```rust
// Reuse existing gate.rs functions:
use crate::gate::{GateFeatures, score, decide, Decision};

fn build_gate_features(html: &str) -> GateFeatures {
    GateFeatures {
        html_bytes: html.len(),
        visible_text_chars: count_visible_text(html),
        p_count: count_elements(html, "p"),
        article_count: count_semantic_elements(html),
        h1h2_count: count_headings(html),
        script_bytes: count_script_content(html),
        has_og: has_open_graph(html),
        has_jsonld_article: has_jsonld_article(html),
        spa_markers: detect_spa_markers_count(html),
        domain_prior: 0.5, // TODO: Load from historical data
    }
}
```

---

## 4. Dependency Graph

### 4.1 Current State (Duplicated)

```
┌─────────────┐                    ┌─────────────┐
│ riptide-cli │                    │ riptide-api │
│             │                    │             │
│ ┌─────────┐ │                    │ ┌─────────┐ │
│ │ engine_ │ │ (DUPLICATE LOGIC)  │ │ partial │ │
│ │fallback │ │                    │ │  impl   │ │
│ └─────────┘ │                    │ └─────────┘ │
└─────────────┘                    └─────────────┘
       │                                  │
       │ (both duplicate same logic)     │
       ▼                                  ▼
  [Content Analysis]              [Content Analysis]
  [Framework Detection]           [Framework Detection]
  [Fallback Chain]                [Fallback Chain]
```

### 4.2 Proposed State (Consolidated)

```
┌─────────────┐                    ┌─────────────┐
│ riptide-cli │                    │ riptide-api │
└──────┬──────┘                    └──────┬──────┘
       │                                  │
       │  (both use same module)          │
       ▼                                  ▼
┌──────────────────────────────────────────────┐
│      riptide-reliability                     │
│  ┌────────────────────────────────────────┐  │
│  │   engine_selection module              │  │
│  │   • decide_engine()                    │  │
│  │   • analyze_content()                  │  │
│  │   • detect_framework()                 │  │
│  │   • validate_extraction_quality()      │  │
│  └────────────────────────────────────────┘  │
│                                              │
│  (reuses existing gate.rs)                   │
│  ┌────────────────────────────────────────┐  │
│  │   gate module                          │  │
│  │   • GateFeatures                       │  │
│  │   • score()                            │  │
│  │   • decide()                           │  │
│  └────────────────────────────────────────┘  │
└──────────────────────────────────────────────┘
                    │
                    ▼
          ┌─────────────────┐
          │  riptide-types  │
          │  • RenderMode   │
          │  • ConfigTypes  │
          └─────────────────┘
```

### 4.3 Dependency Verification

✅ **No circular dependencies:**

```
riptide-cli → riptide-reliability → riptide-types
riptide-api → riptide-reliability → riptide-types
```

✅ **Existing dependencies confirmed:**
- `riptide-cli/Cargo.toml` already includes `riptide-reliability`
- `riptide-api/Cargo.toml` already includes `riptide-reliability`

---

## 5. Test Strategy

### 5.1 Unit Tests

```rust
// File: riptide-reliability/src/engine_selection.rs

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_detect_framework_react() {
        let html = r#"<script>window.__NEXT_DATA__={}</script>"#;
        assert_eq!(detect_framework(html), Some(Framework::NextJs));
    }

    #[test]
    fn test_detect_framework_vue() {
        let html = r#"<div id="app" v-app></div>"#;
        assert_eq!(detect_framework(html), Some(Framework::Vue));
    }

    #[test]
    fn test_spa_marker_detection() {
        let html = r#"
            <script>window.__NEXT_DATA__={}</script>
            <div data-reactroot></div>
        "#;
        let markers = detect_spa_markers(html);
        assert!(markers.contains(&SpaMarker::NextData));
        assert!(markers.contains(&SpaMarker::ReactRoot));
    }

    #[test]
    fn test_anti_scraping_cloudflare() {
        let html = r#"<div id="cf-browser-verification"></div>"#;
        assert_eq!(
            detect_anti_scraping(html),
            Some(AntiScrapingType::Cloudflare)
        );
    }

    #[test]
    fn test_content_quality_high() {
        let html = r#"
            <article>
                <h1>Title</h1>
                <p>Paragraph 1 with substantial content.</p>
                <p>Paragraph 2 with substantial content.</p>
                <p>Paragraph 3 with substantial content.</p>
            </article>
        "#;
        let score = calculate_content_quality(html);
        assert!(score > 0.7);
    }

    #[test]
    fn test_engine_decision_spa() {
        let html = r#"
            <html>
                <head><script>window.__NEXT_DATA__={}</script></head>
                <body><div id="__next"></div></body>
                <script src="bundle.js" size="2MB"></script>
            </html>
        "#;
        let decision = decide_engine("https://spa-app.com", html, None);
        assert_eq!(decision.primary_engine, EngineType::Headless);
    }

    #[test]
    fn test_engine_decision_static_article() {
        let html = r#"
            <html>
                <head><title>Article</title></head>
                <body>
                    <article>
                        <h1>Great Article</h1>
                        <p>Content paragraph 1</p>
                        <p>Content paragraph 2</p>
                    </article>
                </body>
            </html>
        "#;
        let decision = decide_engine("https://blog.com/article", html, None);
        assert_eq!(decision.primary_engine, EngineType::Wasm);
        assert!(decision.confidence > 0.6);
    }
}
```

### 5.2 Integration Tests

```rust
// File: tests/integration/engine_selection_tests.rs

use riptide_reliability::engine_selection::*;

#[tokio::test]
async fn test_full_extraction_workflow_with_fallback() {
    let url = "https://example.com/page";
    let html = fetch_test_html(url).await;

    let decision = decide_engine(url, &html, None);

    // Verify fallback chain is complete
    assert!(decision.fallback_chain.len() > 0);

    // Verify no duplicates in chain
    let mut engines = std::collections::HashSet::new();
    engines.insert(decision.primary_engine);
    for engine in &decision.fallback_chain {
        assert!(engines.insert(*engine), "Duplicate engine in fallback chain");
    }
}
```

---

## 6. Migration Strategy

### 6.1 Phase 1: Create Module (Week 1)

1. ✅ Create `/workspaces/eventmesh/crates/riptide-reliability/src/engine_selection.rs`
2. ✅ Implement core types and functions
3. ✅ Write comprehensive unit tests
4. ✅ Update `riptide-reliability/src/lib.rs` to re-export new types

### 6.2 Phase 2: Migrate CLI (Week 2)

1. ✅ Update `riptide-cli/src/commands/extract.rs` to use new module
2. ✅ Remove `riptide-cli/src/commands/engine_fallback.rs` (deprecated)
3. ✅ Update CLI tests to verify behavior unchanged
4. ✅ Run integration test suite

### 6.3 Phase 3: Migrate API (Week 3)

1. ✅ Update API handlers to use new module
2. ✅ Remove duplicate logic from `riptide-api/src/rpc_client.rs`
3. ✅ Update API tests
4. ✅ Run full regression test suite

### 6.4 Phase 4: Validation (Week 4)

1. ✅ Compare CLI and API behavior (should be identical)
2. ✅ Performance benchmarks (should be same or better)
3. ✅ Documentation updates
4. ✅ Code review and merge

---

## 7. Performance Considerations

### 7.1 Performance Targets

| Metric | Target | Rationale |
|--------|--------|-----------|
| Content analysis | < 10ms | Negligible overhead |
| Framework detection | < 5ms | Simple string matching |
| SPA marker detection | < 5ms | Bitwise operations |
| Memory overhead | < 100KB | Minimal allocations |

### 7.2 Optimizations

1. **Lazy evaluation:** Only analyze content when needed
2. **Memoization:** Cache analysis results for same URL
3. **String slicing:** Avoid full HTML parsing when possible
4. **Bitmap flags:** Use `u8` for SPA markers (fast bitwise ops)

---

## 8. Risk Assessment

### 8.1 Technical Risks

| Risk | Probability | Impact | Mitigation |
|------|------------|--------|------------|
| Breaking changes to CLI behavior | Low | High | Comprehensive integration tests |
| Breaking changes to API behavior | Low | High | Parallel testing during migration |
| Performance regression | Very Low | Medium | Benchmark before/after |
| Dependency cycle introduced | Very Low | Critical | Careful dependency review |

### 8.2 Mitigation Strategy

1. **Feature flag:** Deploy behind feature flag initially
2. **Parallel testing:** Run old and new implementations side-by-side
3. **Rollback plan:** Keep old code for 1 release cycle
4. **Monitoring:** Track engine selection decisions in production

---

## 9. Success Metrics

### 9.1 Quantitative Metrics

- ✅ **LOC reduction:** ~400 LOC removed (eliminate duplication)
- ✅ **Test coverage:** > 90% for engine_selection module
- ✅ **Performance:** No regression (< 1ms overhead)
- ✅ **Consistency:** 100% identical behavior between CLI and API

### 9.2 Qualitative Metrics

- ✅ **Maintainability:** Single point of change for engine selection
- ✅ **Testability:** Comprehensive test suite in one place
- ✅ **Documentation:** Clear API documentation with examples
- ✅ **Developer experience:** Easier to reason about engine selection

---

## 10. Future Enhancements

### 10.1 Short Term (Next Quarter)

1. **Historical analysis:** Load domain priors from database
2. **Machine learning:** Train model on extraction success rates
3. **Telemetry:** Track engine selection decisions in production
4. **A/B testing:** Compare different decision thresholds

### 10.2 Long Term (Next Year)

1. **Adaptive thresholds:** Automatically tune hi/lo thresholds
2. **Real-time feedback:** Adjust decisions based on immediate results
3. **Cost optimization:** Consider engine costs in decision-making
4. **Distributed learning:** Share decision data across cluster

---

## 11. Appendix

### 11.1 Code Size Comparison

| Component | Before (LOC) | After (LOC) | Reduction |
|-----------|-------------|------------|-----------|
| CLI engine_fallback.rs | 475 | 0 (removed) | -475 |
| API rpc_client.rs | ~150 | ~50 | -100 |
| riptide-reliability | 0 | +350 | +350 |
| **Net change** | **625** | **400** | **-225 LOC** |

### 11.2 Confidence Scoring Algorithm

```
Confidence Score (0.0 - 1.0) =
    (text_ratio * 1.2).clamp(0.0, 0.6)
  + ln(p_count + 1) * 0.06
  + (has_article ? 0.15 : 0.0)
  + (has_og ? 0.08 : 0.0)
  + (has_jsonld ? 0.12 : 0.0)
  - (script_density * 0.8).clamp(0.0, 0.4)
  - (spa_markers >= 2 ? 0.25 : 0.0)
  + (domain_prior - 0.5) * 0.1
```

### 11.3 Decision Thresholds

```
High threshold (hi) = 0.7  (use Raw/WASM)
Low threshold (lo) = 0.3   (use Headless)

if confidence >= hi:
    primary = WASM, fallback = [Raw, Headless]
elif confidence <= lo OR spa_markers >= 3:
    primary = Headless, fallback = [WASM, Raw]
else:
    primary = WASM, fallback = [Headless, Raw]
```

---

## 12. Architecture Diagrams

### 12.1 Component Diagram (C4 Level 3)

```
┌─────────────────────────────────────────────────────────────────┐
│                     Riptide Reliability                         │
│                                                                 │
│  ┌────────────────────────────────────────────────────────┐    │
│  │              engine_selection module                   │    │
│  │                                                        │    │
│  │  ┌──────────────────────────────────────────────────┐ │    │
│  │  │  decide_engine(url, html, content_type)          │ │    │
│  │  │    → EngineDecision                              │ │    │
│  │  │                                                  │ │    │
│  │  │  1. Analyze content characteristics             │ │    │
│  │  │  2. Detect framework (React, Vue, Angular)      │ │    │
│  │  │  3. Detect SPA markers                          │ │    │
│  │  │  4. Detect anti-scraping                        │ │    │
│  │  │  5. Calculate confidence score                  │ │    │
│  │  │  6. Build fallback chain                        │ │    │
│  │  └──────────────────────────────────────────────────┘ │    │
│  │                                                        │    │
│  │  ┌──────────────────────────────────────────────────┐ │    │
│  │  │  analyze_content(url, html)                      │ │    │
│  │  │    → ContentAnalysis                             │ │    │
│  │  └──────────────────────────────────────────────────┘ │    │
│  │                                                        │    │
│  │  ┌──────────────────────────────────────────────────┐ │    │
│  │  │  detect_framework(html)                          │ │    │
│  │  │    → Option<Framework>                           │ │    │
│  │  └──────────────────────────────────────────────────┘ │    │
│  │                                                        │    │
│  │  ┌──────────────────────────────────────────────────┐ │    │
│  │  │  validate_extraction_quality(...)                │ │    │
│  │  │    → bool                                        │ │    │
│  │  └──────────────────────────────────────────────────┘ │    │
│  └────────────────────────────────────────────────────────┘    │
│                                                                 │
│  ┌────────────────────────────────────────────────────────┐    │
│  │              gate module (REUSED)                      │    │
│  │  • GateFeatures struct                                 │    │
│  │  • score(features) → f32                               │    │
│  │  • decide(features, hi, lo) → Decision                 │    │
│  └────────────────────────────────────────────────────────┘    │
└─────────────────────────────────────────────────────────────────┘
```

### 12.2 Sequence Diagram

```
CLI/API → engine_selection: decide_engine(url, html)
    engine_selection → detect_framework(html)
        detect_framework → : Check for React/Vue/Angular markers
        detect_framework ← : Framework | None

    engine_selection → detect_spa_markers(html)
        detect_spa_markers → : Scan for SPA indicators
        detect_spa_markers ← : Vec<SpaMarker>

    engine_selection → detect_anti_scraping(html)
        detect_anti_scraping → : Check for Cloudflare/reCAPTCHA
        detect_anti_scraping ← : AntiScrapingType | None

    engine_selection → gate::score(features)
        gate::score → : Calculate quality score
        gate::score ← : f32 (0.0-1.0)

    engine_selection → : Build EngineDecision
    engine_selection ← : EngineDecision {
                           primary_engine,
                           fallback_chain,
                           confidence,
                           reasoning
                         }
CLI/API ← : EngineDecision
```

---

## 13. Conclusion

This architecture provides a **robust, maintainable, and testable solution** for consolidating engine selection logic. By leveraging the existing `riptide-reliability` crate and reusing the battle-tested `gate.rs` module, we minimize risk while maximizing code reuse.

**Key Benefits:**
1. ✅ **Single source of truth** eliminates duplication
2. ✅ **Zero breaking changes** to existing behavior
3. ✅ **No circular dependencies** (clean architecture)
4. ✅ **Comprehensive testing** (90%+ coverage)
5. ✅ **Performance neutral** (< 1ms overhead)
6. ✅ **Future-proof** (extensible for ML/adaptive learning)

**Recommendation:** Proceed with implementation following the 4-week migration strategy outlined in Section 6.

---

**Next Steps:**
1. Review architecture with team
2. Approve migration plan
3. Assign implementation to Coder agent
4. Begin Phase 1 (module creation)
