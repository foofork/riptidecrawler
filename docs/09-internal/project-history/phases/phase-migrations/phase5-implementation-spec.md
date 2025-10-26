# Phase 5: Engine Selection - Implementation Specification

**Version:** 1.0
**Date:** 2025-10-23
**Status:** READY FOR IMPLEMENTATION

---

## 1. File Structure

### 1.1 New Files

```
/workspaces/eventmesh/crates/riptide-reliability/src/engine_selection.rs
/workspaces/eventmesh/crates/riptide-reliability/tests/engine_selection_tests.rs
```

### 1.2 Modified Files

```
/workspaces/eventmesh/crates/riptide-reliability/src/lib.rs
/workspaces/eventmesh/crates/riptide-cli/src/commands/extract.rs
/workspaces/eventmesh/crates/riptide-api/src/handlers/extract.rs (if exists)
```

### 1.3 Deprecated Files (Remove in Phase 2)

```
/workspaces/eventmesh/crates/riptide-cli/src/commands/engine_fallback.rs
```

---

## 2. Complete Implementation

### 2.1 Core Module: `engine_selection.rs`

```rust
//! # Engine Selection Module
//!
//! Intelligent engine selection with automatic fallback chains:
//! Raw → WASM → Headless
//!
//! This module consolidates engine selection logic previously duplicated
//! across CLI and API components.

use crate::gate::{score, Decision, GateFeatures};
use serde::{Deserialize, Serialize};

// ============================================================================
// Types
// ============================================================================

/// Engine types in fallback chain (ordered by speed)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Hash)]
pub enum Engine {
    /// Raw HTTP fetch with static HTML parsing (fastest, ~50ms)
    Raw,
    /// WASM-based extraction (balanced, ~100-200ms)
    Wasm,
    /// Headless browser with full JS execution (slowest, ~1-3s)
    Headless,
}

impl Engine {
    /// Get engine name as string
    pub fn as_str(&self) -> &'static str {
        match self {
            Engine::Raw => "raw",
            Engine::Wasm => "wasm",
            Engine::Headless => "headless",
        }
    }

    /// Get all engines in order of preference (speed)
    pub fn all() -> [Engine; 3] {
        [Engine::Raw, Engine::Wasm, Engine::Headless]
    }
}

/// JavaScript framework detection
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Framework {
    React,
    NextJs,
    Vue,
    Nuxt,
    Angular,
    Svelte,
    SvelteKit,
    Unknown,
}

/// SPA (Single Page Application) markers
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SpaMarker {
    NextData,              // __NEXT_DATA__ script tag
    ReactRoot,             // _reactRoot or data-reactroot
    WebpackRequire,        // __webpack_require__
    VueApp,                // v-app or createApp
    AngularApp,            // ng-app or platformBrowserDynamic
    InitialState,          // window.__INITIAL_STATE__
    ReactHelmet,           // data-react-helmet
    HydrationMarker,       // data-react-hydration
}

/// Anti-scraping technology detection
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AntiScraping {
    Cloudflare,
    CloudflareTurnstile,
    Recaptcha,
    RecaptchaV3,
    HCaptcha,
    PerimeterX,
    DataDome,
    Akamai,
    ImpervaIncapsula,
}

/// Content analysis result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContentAnalysis {
    /// Detected JavaScript framework
    pub framework: Option<Framework>,
    /// SPA markers found
    pub spa_markers: Vec<SpaMarker>,
    /// Anti-scraping technology detected
    pub anti_scraping: Option<AntiScraping>,
    /// Content quality score (0.0-1.0)
    pub quality_score: f32,
    /// Text-to-markup ratio
    pub text_ratio: f32,
    /// Has main content markers
    pub has_main_content: bool,
    /// Number of paragraphs
    pub paragraph_count: u32,
    /// Number of semantic elements
    pub semantic_element_count: u32,
}

/// Unified engine decision
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EngineDecision {
    /// Primary engine to try first
    pub primary: Engine,
    /// Fallback chain (ordered by preference)
    pub fallback_chain: Vec<Engine>,
    /// Confidence score (0.0-1.0)
    pub confidence: f32,
    /// Human-readable reasoning
    pub reasoning: String,
    /// Content analysis that informed decision
    pub analysis: ContentAnalysis,
}

// ============================================================================
// Framework Detection
// ============================================================================

/// Detect JavaScript framework from HTML content
pub fn detect_framework(html: &str) -> Option<Framework> {
    // Next.js detection (most specific first)
    if html.contains("__NEXT_DATA__") || html.contains("/_next/") {
        return Some(Framework::NextJs);
    }

    // React detection
    if html.contains("data-reactroot")
        || html.contains("data-react-helmet")
        || html.contains("_reactRoot")
        || html.contains("react-dom")
    {
        return Some(Framework::React);
    }

    // Nuxt.js detection
    if html.contains("__NUXT__") || html.contains("_nuxt/") {
        return Some(Framework::Nuxt);
    }

    // Vue detection
    if html.contains("v-app")
        || html.contains("v-cloak")
        || html.contains("createApp")
        || html.contains("vue.js")
    {
        return Some(Framework::Vue);
    }

    // Angular detection
    if html.contains("ng-app")
        || html.contains("ng-version")
        || html.contains("platformBrowserDynamic")
        || html.contains("angular.js")
    {
        return Some(Framework::Angular);
    }

    // SvelteKit detection
    if html.contains("__SVELTEKIT__") || html.contains("_app/") {
        return Some(Framework::SvelteKit);
    }

    // Svelte detection
    if html.contains("svelte") || html.contains("s-") {
        return Some(Framework::Svelte);
    }

    None
}

// ============================================================================
// SPA Marker Detection
// ============================================================================

/// Detect SPA markers in HTML
pub fn detect_spa_markers(html: &str) -> Vec<SpaMarker> {
    let mut markers = Vec::new();

    if html.contains("__NEXT_DATA__") {
        markers.push(SpaMarker::NextData);
    }
    if html.contains("data-reactroot") || html.contains("_reactRoot") {
        markers.push(SpaMarker::ReactRoot);
    }
    if html.contains("__webpack_require__") {
        markers.push(SpaMarker::WebpackRequire);
    }
    if html.contains("v-app") || html.contains("createApp") {
        markers.push(SpaMarker::VueApp);
    }
    if html.contains("ng-app") || html.contains("platformBrowserDynamic") {
        markers.push(SpaMarker::AngularApp);
    }
    if html.contains("window.__INITIAL_STATE__") || html.contains("__INITIAL_STATE__") {
        markers.push(SpaMarker::InitialState);
    }
    if html.contains("data-react-helmet") {
        markers.push(SpaMarker::ReactHelmet);
    }
    if html.contains("data-react-hydration") {
        markers.push(SpaMarker::HydrationMarker);
    }

    markers
}

// ============================================================================
// Anti-Scraping Detection
// ============================================================================

/// Detect anti-scraping technology
pub fn detect_anti_scraping(html: &str) -> Option<AntiScraping> {
    // Cloudflare Turnstile
    if html.contains("cf-turnstile") || html.contains("turnstile.js") {
        return Some(AntiScraping::CloudflareTurnstile);
    }

    // Cloudflare generic
    if html.contains("cf-browser-verification")
        || html.contains("__cf_chl_jschl_tk__")
        || html.contains("Cloudflare")
    {
        return Some(AntiScraping::Cloudflare);
    }

    // reCAPTCHA v3
    if html.contains("recaptcha/api.js") && html.contains("v3") {
        return Some(AntiScraping::RecaptchaV3);
    }

    // reCAPTCHA generic
    if html.contains("grecaptcha") || html.contains("recaptcha") {
        return Some(AntiScraping::Recaptcha);
    }

    // hCaptcha
    if html.contains("hcaptcha") || html.contains("h-captcha") {
        return Some(AntiScraping::HCaptcha);
    }

    // PerimeterX
    if html.contains("PerimeterX") || html.contains("_px") {
        return Some(AntiScraping::PerimeterX);
    }

    // DataDome
    if html.contains("DataDome") || html.contains("datadome") {
        return Some(AntiScraping::DataDome);
    }

    // Akamai
    if html.contains("akamai") || html.contains("_abck") {
        return Some(AntiScraping::Akamai);
    }

    // Imperva Incapsula
    if html.contains("incapsula") || html.contains("imperva") {
        return Some(AntiScraping::ImpervaIncapsula);
    }

    None
}

// ============================================================================
// Content Analysis
// ============================================================================

/// Calculate text-to-markup ratio
pub fn calculate_content_ratio(html: &str) -> f32 {
    let total_len = html.len() as f32;
    if total_len == 0.0 {
        return 0.0;
    }

    // Extract text content between tags
    let text_content: String = html
        .split('<')
        .filter_map(|s| s.split('>').nth(1))
        .collect();

    let content_len = text_content.trim().len() as f32;
    (content_len / total_len).clamp(0.0, 1.0)
}

/// Count specific HTML elements
fn count_elements(html: &str, tag: &str) -> u32 {
    let open_tag = format!("<{}", tag);
    html.matches(&open_tag).count() as u32
}

/// Count semantic content elements
fn count_semantic_elements(html: &str) -> u32 {
    count_elements(html, "article")
        + count_elements(html, "main")
        + count_elements(html, "section")
}

/// Check for main content markers
pub fn has_main_content_markers(html: &str) -> bool {
    html.contains("<article")
        || html.contains("<main")
        || html.contains("class=\"content\"")
        || html.contains("id=\"content\"")
        || html.contains("class=\"post\"")
        || html.contains("class=\"entry\"")
}

/// Perform comprehensive content analysis
pub fn analyze_content(url: &str, html: &str) -> ContentAnalysis {
    let framework = detect_framework(html);
    let spa_markers = detect_spa_markers(html);
    let anti_scraping = detect_anti_scraping(html);
    let text_ratio = calculate_content_ratio(html);
    let has_main_content = has_main_content_markers(html);
    let paragraph_count = count_elements(html, "p");
    let semantic_element_count = count_semantic_elements(html);

    // Build GateFeatures for quality scoring
    let features = GateFeatures {
        html_bytes: html.len(),
        visible_text_chars: (text_ratio * html.len() as f32) as usize,
        p_count: paragraph_count,
        article_count: semantic_element_count,
        h1h2_count: count_elements(html, "h1") + count_elements(html, "h2"),
        script_bytes: html.matches("<script").count() * 100, // Rough estimate
        has_og: html.contains("og:"),
        has_jsonld_article: html.contains("\"@type\":\"Article\""),
        spa_markers: spa_markers.len().min(255) as u8,
        domain_prior: 0.5, // Neutral prior (could be loaded from history)
    };

    let quality_score = score(&features);

    ContentAnalysis {
        framework,
        spa_markers,
        anti_scraping,
        quality_score,
        text_ratio,
        has_main_content,
        paragraph_count,
        semantic_element_count,
    }
}

// ============================================================================
// Engine Decision Logic
// ============================================================================

/// Decide optimal engine based on content analysis
pub fn decide_engine(url: &str, html: &str, content_type: Option<&str>) -> EngineDecision {
    // Check for PDF content (skip headless)
    if let Some(ct) = content_type {
        if ct.contains("application/pdf") {
            return EngineDecision {
                primary: Engine::Raw,
                fallback_chain: vec![],
                confidence: 1.0,
                reasoning: "PDF content type detected - using direct processing".to_string(),
                analysis: ContentAnalysis {
                    framework: None,
                    spa_markers: vec![],
                    anti_scraping: None,
                    quality_score: 1.0,
                    text_ratio: 0.0,
                    has_main_content: false,
                    paragraph_count: 0,
                    semantic_element_count: 0,
                },
            };
        }
    }

    let url_lower = url.to_lowercase();
    if url_lower.ends_with(".pdf") {
        return EngineDecision {
            primary: Engine::Raw,
            fallback_chain: vec![],
            confidence: 1.0,
            reasoning: "PDF URL detected - using direct processing".to_string(),
            analysis: ContentAnalysis {
                framework: None,
                spa_markers: vec![],
                anti_scraping: None,
                quality_score: 1.0,
                text_ratio: 0.0,
                has_main_content: false,
                paragraph_count: 0,
                semantic_element_count: 0,
            },
        };
    }

    // Analyze content
    let analysis = analyze_content(url, html);

    // Decision thresholds
    const HIGH_THRESHOLD: f32 = 0.7;
    const LOW_THRESHOLD: f32 = 0.3;

    // Strong headless indicators
    let force_headless = analysis.anti_scraping.is_some()
        || analysis.spa_markers.len() >= 3
        || matches!(
            analysis.framework,
            Some(Framework::React)
                | Some(Framework::NextJs)
                | Some(Framework::Vue)
                | Some(Framework::Angular)
        );

    // Build decision
    let (primary, fallback_chain, confidence, reasoning) = if force_headless {
        (
            Engine::Headless,
            vec![Engine::Wasm, Engine::Raw],
            0.8,
            format!(
                "Headless required: {}",
                if analysis.anti_scraping.is_some() {
                    "anti-scraping detected"
                } else if analysis.spa_markers.len() >= 3 {
                    "multiple SPA markers"
                } else {
                    format!("framework: {:?}", analysis.framework.unwrap()).to_lowercase()
                }
            ),
        )
    } else if analysis.quality_score >= HIGH_THRESHOLD {
        (
            Engine::Wasm,
            vec![Engine::Raw, Engine::Headless],
            analysis.quality_score,
            format!(
                "High-quality content (score: {:.2}), using WASM",
                analysis.quality_score
            ),
        )
    } else if analysis.quality_score <= LOW_THRESHOLD {
        (
            Engine::Headless,
            vec![Engine::Wasm, Engine::Raw],
            1.0 - analysis.quality_score,
            format!(
                "Low content quality (score: {:.2}), using headless",
                analysis.quality_score
            ),
        )
    } else {
        (
            Engine::Wasm,
            vec![Engine::Headless, Engine::Raw],
            analysis.quality_score,
            format!(
                "Mixed content (score: {:.2}), WASM with headless fallback",
                analysis.quality_score
            ),
        )
    };

    EngineDecision {
        primary,
        fallback_chain,
        confidence,
        reasoning,
        analysis,
    }
}

// ============================================================================
// Extraction Quality Validation
// ============================================================================

/// Validate extraction quality
pub fn validate_extraction_quality(
    content: &str,
    confidence: Option<f32>,
    min_content_length: usize,
    min_confidence: f32,
) -> bool {
    // Check minimum content length
    if content.len() < min_content_length {
        return false;
    }

    // Check confidence score
    if let Some(conf) = confidence {
        if conf < min_confidence {
            return false;
        }
    }

    // Check text ratio (avoid excessive whitespace)
    let word_count = content.split_whitespace().count();
    let text_ratio = if content.len() > 0 {
        word_count as f32 / content.len() as f32
    } else {
        0.0
    };

    // Reasonable text ratio (not all whitespace or markup)
    text_ratio >= 0.05
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_engine_as_str() {
        assert_eq!(Engine::Raw.as_str(), "raw");
        assert_eq!(Engine::Wasm.as_str(), "wasm");
        assert_eq!(Engine::Headless.as_str(), "headless");
    }

    #[test]
    fn test_detect_framework_nextjs() {
        let html = r#"<script>window.__NEXT_DATA__={}</script>"#;
        assert_eq!(detect_framework(html), Some(Framework::NextJs));
    }

    #[test]
    fn test_detect_framework_react() {
        let html = r#"<div data-reactroot></div>"#;
        assert_eq!(detect_framework(html), Some(Framework::React));
    }

    #[test]
    fn test_detect_framework_vue() {
        let html = r#"<div id="app" v-app></div>"#;
        assert_eq!(detect_framework(html), Some(Framework::Vue));
    }

    #[test]
    fn test_detect_spa_markers() {
        let html = r#"
            <script>window.__NEXT_DATA__={}</script>
            <div data-reactroot></div>
            <script>window.__INITIAL_STATE__={}</script>
        "#;
        let markers = detect_spa_markers(html);
        assert!(markers.contains(&SpaMarker::NextData));
        assert!(markers.contains(&SpaMarker::ReactRoot));
        assert!(markers.contains(&SpaMarker::InitialState));
        assert_eq!(markers.len(), 3);
    }

    #[test]
    fn test_detect_anti_scraping_cloudflare() {
        let html = r#"<div id="cf-browser-verification"></div>"#;
        assert_eq!(detect_anti_scraping(html), Some(AntiScraping::Cloudflare));
    }

    #[test]
    fn test_calculate_content_ratio() {
        let html = "<html><body>Hello World</body></html>";
        let ratio = calculate_content_ratio(html);
        assert!(ratio > 0.0 && ratio < 1.0);
    }

    #[test]
    fn test_has_main_content_markers() {
        let html = r#"<article><h1>Title</h1><p>Content</p></article>"#;
        assert!(has_main_content_markers(html));

        let html_no_markers = "<div><p>Content</p></div>";
        assert!(!has_main_content_markers(html_no_markers));
    }

    #[test]
    fn test_decide_engine_spa() {
        let html = r#"
            <html>
                <head><script>window.__NEXT_DATA__={}</script></head>
                <body><div id="__next"></div></body>
            </html>
        "#;
        let decision = decide_engine("https://app.com", html, None);
        assert_eq!(decision.primary, Engine::Headless);
        assert!(decision.confidence > 0.5);
    }

    #[test]
    fn test_decide_engine_static_article() {
        let html = r#"
            <html>
                <body>
                    <article>
                        <h1>Article Title</h1>
                        <p>Paragraph 1 with substantial content.</p>
                        <p>Paragraph 2 with substantial content.</p>
                        <p>Paragraph 3 with substantial content.</p>
                    </article>
                </body>
            </html>
        "#;
        let decision = decide_engine("https://blog.com/article", html, None);
        assert_eq!(decision.primary, Engine::Wasm);
        assert!(decision.confidence > 0.6);
    }

    #[test]
    fn test_decide_engine_pdf() {
        let decision = decide_engine("https://example.com/doc.pdf", "", None);
        assert_eq!(decision.primary, Engine::Raw);
        assert_eq!(decision.confidence, 1.0);

        let decision = decide_engine("https://example.com/doc", "", Some("application/pdf"));
        assert_eq!(decision.primary, Engine::Raw);
        assert_eq!(decision.confidence, 1.0);
    }

    #[test]
    fn test_validate_extraction_quality() {
        // Good extraction
        let content = "This is a good extraction with sufficient content length and quality.";
        assert!(validate_extraction_quality(content, Some(0.8), 50, 0.5));

        // Too short
        let content = "Short";
        assert!(!validate_extraction_quality(content, Some(0.8), 50, 0.5));

        // Low confidence
        let content = "This is enough content but low confidence.";
        assert!(!validate_extraction_quality(content, Some(0.3), 20, 0.5));
    }
}
```

---

## 3. Library Re-exports

### 3.1 Update `lib.rs`

```rust
// File: riptide-reliability/src/lib.rs

pub mod circuit;
pub mod circuit_breaker;
pub mod gate;
pub mod reliability;
pub mod engine_selection;  // ✨ NEW

// Re-export commonly used types
pub use circuit::{CircuitBreaker as AtomicCircuitBreaker, Clock, Config as CircuitConfig, State};
pub use circuit_breaker::{
    record_extraction_result, CircuitBreakerState, ExtractionResult as CircuitExtractionResult,
};
pub use gate::{decide, score, should_use_headless, Decision, GateFeatures};
pub use reliability::{
    ExtractionMode, ReliabilityConfig, ReliabilityMetrics, ReliableExtractor, WasmExtractor,
};

// ✨ NEW: Re-export engine selection types
pub use engine_selection::{
    analyze_content, calculate_content_ratio, decide_engine, detect_anti_scraping,
    detect_framework, detect_spa_markers, has_main_content_markers, validate_extraction_quality,
    AntiScraping, ContentAnalysis, Engine, EngineDecision, Framework, SpaMarker,
};
```

---

## 4. Integration Examples

### 4.1 CLI Integration

```rust
// File: riptide-cli/src/commands/extract.rs

use riptide_reliability::engine_selection::{decide_engine, Engine, validate_extraction_quality};
use tracing::{info, warn};

pub async fn execute_extract(url: &str, force_engine: Option<&str>) -> Result<ExtractResponse> {
    // Fetch initial HTML
    let html = fetch_html_with_cache(url).await?;

    // Decide engine (unless user forces specific engine)
    let decision = if let Some(forced) = force_engine {
        // User override
        info!("Using user-specified engine: {}", forced);
        create_forced_decision(forced)
    } else {
        // Intelligent decision
        let decision = decide_engine(url, &html, None);
        info!(
            "Engine decision: {} (confidence: {:.2})",
            decision.primary.as_str(),
            decision.confidence
        );
        info!("Reasoning: {}", decision.reasoning);
        decision
    };

    // Execute with fallback chain
    let mut last_error = None;

    for (attempt, engine) in std::iter::once(decision.primary)
        .chain(decision.fallback_chain.iter().copied())
        .enumerate()
    {
        info!("Attempt {}: Using {} engine", attempt + 1, engine.as_str());

        let result = match engine {
            Engine::Raw => execute_raw_extraction(url).await,
            Engine::Wasm => execute_wasm_extraction(url, &html).await,
            Engine::Headless => execute_headless_extraction(url).await,
        };

        match result {
            Ok(response) => {
                // Validate quality
                if validate_extraction_quality(
                    &response.content,
                    response.confidence,
                    100, // min_content_length
                    0.5, // min_confidence
                ) {
                    info!("Extraction successful with {} engine", engine.as_str());
                    return Ok(response);
                } else {
                    warn!(
                        "Extraction quality insufficient with {} engine, trying fallback",
                        engine.as_str()
                    );
                    last_error = Some(anyhow::anyhow!("Quality validation failed"));
                }
            }
            Err(e) => {
                warn!("Extraction failed with {} engine: {}", engine.as_str(), e);
                last_error = Some(e);
            }
        }
    }

    Err(last_error.unwrap_or_else(|| anyhow::anyhow!("All extraction attempts failed")))
}
```

### 4.2 API Integration

```rust
// File: riptide-api/src/handlers/extract.rs

use riptide_reliability::engine_selection::{decide_engine, Engine};

pub async fn handle_extract_request(req: ExtractRequest) -> Result<ExtractResponse> {
    let html = fetch_html(&req.url).await?;
    let decision = decide_engine(&req.url, &html, req.content_type.as_deref());

    // Log decision for monitoring
    tracing::info!(
        url = %req.url,
        engine = %decision.primary.as_str(),
        confidence = decision.confidence,
        "Engine decision made"
    );

    // Execute with same fallback logic as CLI
    execute_with_fallback(decision, &req).await
}
```

---

## 5. Testing Requirements

### 5.1 Unit Test Coverage

- ✅ Framework detection (React, Vue, Angular, etc.)
- ✅ SPA marker detection
- ✅ Anti-scraping detection
- ✅ Content ratio calculation
- ✅ Quality scoring
- ✅ Engine decision logic
- ✅ Extraction quality validation
- ✅ PDF detection
- ✅ Edge cases (empty HTML, malformed content)

### 5.2 Integration Test Coverage

- ✅ Full extraction workflow with fallback
- ✅ CLI integration (end-to-end)
- ✅ API integration (end-to-end)
- ✅ Behavior consistency between CLI and API
- ✅ Performance benchmarks

---

## 6. Success Criteria

1. ✅ All unit tests pass (90%+ coverage)
2. ✅ Integration tests verify identical CLI/API behavior
3. ✅ No circular dependencies introduced
4. ✅ Performance regression < 1ms
5. ✅ Documentation complete with examples
6. ✅ Code review approved by 2+ reviewers

---

## 7. Implementation Checklist

### Phase 1: Module Creation
- [ ] Create `engine_selection.rs` with complete implementation
- [ ] Create `engine_selection_tests.rs` with comprehensive tests
- [ ] Update `lib.rs` to re-export types
- [ ] Run `cargo test` - all tests pass
- [ ] Run `cargo clippy` - no warnings

### Phase 2: CLI Migration
- [ ] Update CLI `extract.rs` to use new module
- [ ] Remove deprecated `engine_fallback.rs`
- [ ] Update CLI tests
- [ ] Run CLI integration tests
- [ ] Manual smoke testing

### Phase 3: API Migration
- [ ] Update API handlers to use new module
- [ ] Update API tests
- [ ] Run API integration tests
- [ ] Manual API testing

### Phase 4: Validation
- [ ] Compare CLI and API behavior (should be identical)
- [ ] Performance benchmarks
- [ ] Documentation review
- [ ] Final code review
- [ ] Merge to main

---

**STATUS:** Ready for Coder agent to implement Phase 1.
