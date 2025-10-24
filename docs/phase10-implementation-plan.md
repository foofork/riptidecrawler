# Phase 10: Engine Selection Quick Wins - Implementation Plan

**Status:** Research Complete
**Date:** 2025-10-24
**Researcher:** Claude Code Research Agent
**Estimated Effort:** 3-5 days (~290 LOC total)
**Expected Impact:** 60-80% reduction in headless browser usage

---

## Executive Summary

Phase 10 introduces three surgical optimizations to the engine selection system that dramatically reduce headless browser usage (60-80% cost savings) with minimal code changes (~290 LOC). All changes use feature flags for gradual rollout (0→10→50→100%) based on the proven pattern from `riptide-intelligence/runtime_switch.rs`.

**Key Insight:** Current system auto-jumps to headless browsers for SPAs. By introducing a WASM probe-first pattern and short-circuiting on complete JSON-LD, we can avoid expensive headless rendering for most pages.

---

## Current System Analysis

### 1. File Locations

| Component | File Path | LOC |
|-----------|-----------|-----|
| **Engine Selection** | `/workspaces/eventmesh/crates/riptide-reliability/src/engine_selection.rs` | 450 |
| **Metadata Extraction** | `/workspaces/eventmesh/crates/riptide-extraction/src/strategies/metadata.rs` | 733 |
| **CLI Orchestration** | `/workspaces/eventmesh/crates/riptide-cli/src/commands/extract.rs` | 882 |

### 2. Current Engine Selection Flow

**Location:** `riptide-reliability/src/engine_selection.rs` (lines 137-192)

```rust
pub fn decide_engine(html: &str, _url: &str) -> Engine {
    // Priority 1: Anti-scraping → Headless
    if has_anti_scraping { return Engine::Headless; }

    // Priority 2: JavaScript frameworks (React/Vue/Angular) → Headless
    if has_react || has_vue || has_angular || has_spa_markers {
        return Engine::Headless;  // ⚠️ NO PROBE ATTEMPT
    }

    // Priority 3: Low content ratio → Headless
    if content_ratio < 0.1 { return Engine::Headless; }

    // Default: WASM
    Engine::Wasm
}
```

**Problem:** Lines 181-183 immediately jump to headless for SPA markers without attempting WASM extraction first.

### 3. Current JSON-LD Extraction

**Location:** `riptide-extraction/src/strategies/metadata.rs` (lines 168-186)

```rust
fn extract_json_ld(document: &Html, metadata: &mut DocumentMetadata,
                   method: &mut ExtractionMethod) -> Result<()> {
    for element in document.select(&selector) {
        let json_value = serde_json::from_str::<serde_json::Value>(&json_text)?;
        extract_from_json_ld(&json_value, metadata)?;
        method.json_ld = true;
        // ⚠️ NO SHORT-CIRCUIT - continues to extract Open Graph, meta tags, heuristics
    }
    Ok(())
}
```

**Problem:** Even with complete JSON-LD (Event/Article schema), continues expensive extraction.

### 4. Current Content Ratio Calculation

**Location:** `riptide-reliability/src/engine_selection.rs` (lines 307-322)

```rust
pub fn calculate_content_ratio(html: &str) -> f64 {
    let total_len = html.len() as f64;
    let text_content: String = html
        .split('<')
        .filter_map(|s| s.split('>').nth(1))
        .collect();
    let content_len = text_content.trim().len() as f64;
    content_len / total_len
}
```

**Problems:**
1. Includes `<script>` and `<style>` content in ratio
2. Counts placeholder text ("Loading...", skeleton screens)
3. No detection of lazy-load markers

---

## Phase 10 Changes

### 10.1: Probe-First Escalation (1 day, ~100 LOC)

**Objective:** Try WASM probe before escalating to headless for SPA pages.

#### Files to Modify

**File 1:** `crates/riptide-reliability/src/engine_selection.rs`

**Current Code (lines 181-183):**
```rust
} else if has_react || has_vue || has_angular || has_spa_markers {
    // Priority 2: JavaScript frameworks typically require headless
    Engine::Headless
```

**New Code:**
```rust
} else if has_react || has_vue || has_angular || has_spa_markers {
    // Priority 2: SPA detected - try probe-first escalation if enabled
    #[cfg(feature = "probe-first-escalation")]
    {
        // Return ProbeFirst signal for orchestrator to handle
        Engine::ProbeFirst  // NEW: try WASM probe before headless
    }
    #[cfg(not(feature = "probe-first-escalation"))]
    {
        Engine::Headless  // Legacy behavior
    }
```

**Changes Required:**

1. **Add new Engine variant** (line 36):
```rust
pub enum Engine {
    Auto,
    Raw,
    Wasm,
    Headless,
    ProbeFirst,  // NEW: try WASM, escalate to headless if needed
}
```

2. **Update FromStr impl** (lines 48-64):
```rust
fn from_str(s: &str) -> Result<Self, Self::Err> {
    match s.to_lowercase().as_str() {
        "auto" => Ok(Engine::Auto),
        "raw" => Ok(Engine::Raw),
        "wasm" => Ok(Engine::Wasm),
        "headless" => Ok(Engine::Headless),
        "probe-first" => Ok(Engine::ProbeFirst),  // NEW
        _ => anyhow::bail!("Invalid engine: {}", s),
    }
}
```

3. **Update Display/name methods** (lines 66-82):
```rust
pub fn name(&self) -> &'static str {
    match self {
        Engine::Auto => "auto",
        Engine::Raw => "raw",
        Engine::Wasm => "wasm",
        Engine::Headless => "headless",
        Engine::ProbeFirst => "probe-first",  // NEW
    }
}
```

4. **Update decide_engine logic** (lines 181-183):
```rust
} else if has_react || has_vue || has_angular || has_spa_markers {
    #[cfg(feature = "probe-first-escalation")]
    return Engine::ProbeFirst;

    #[cfg(not(feature = "probe-first-escalation"))]
    return Engine::Headless;
}
```

**File 2:** `crates/riptide-cli/src/commands/extract.rs`

**Current Code (lines 496-530):**
```rust
// Auto-detect engine if set to auto
if engine == Engine::Auto {
    engine = riptide_reliability::engine_selection::decide_engine(&html, url);
    output::print_info(&format!("Auto-detected engine: {}", engine.name()));
}

// Handle different engines
match engine {
    Engine::Raw => { /* ... */ }
    Engine::Headless => { /* ... */ }
    Engine::Wasm | Engine::Auto => { /* ... */ }
}
```

**New Code:**
```rust
// Auto-detect engine if set to auto
if engine == Engine::Auto {
    engine = riptide_reliability::engine_selection::decide_engine(&html, url);
    output::print_info(&format!("Auto-detected engine: {}", engine.name()));
}

// Handle probe-first escalation
if engine == Engine::ProbeFirst {
    output::print_info("SPA detected - attempting WASM probe before headless escalation");

    // Try WASM extraction first
    match try_wasm_extraction(&html, url, &args).await {
        Ok(result) if result.quality_score.unwrap_or(0) >= 50 => {
            // WASM probe succeeded with good quality
            output::print_info("✓ WASM probe successful - skipping headless");
            return output_extraction_result(result, &args, output_format, url);
        }
        Ok(_) | Err(_) => {
            // WASM probe failed or low quality - escalate to headless
            output::print_info("→ WASM probe insufficient - escalating to headless");
            engine = Engine::Headless;
        }
    }
}

// Handle different engines
match engine {
    Engine::Raw => { /* ... */ }
    Engine::Headless => { /* ... */ }
    Engine::Wasm | Engine::Auto => { /* ... */ }
    Engine::ProbeFirst => {
        unreachable!("ProbeFirst should be handled above")
    }
}
```

**File 3:** `crates/riptide-reliability/Cargo.toml`

Add feature flag:
```toml
[features]
default = []
probe-first-escalation = []
```

#### Testing Strategy

**Unit Tests** (`crates/riptide-reliability/tests/engine_selection_tests.rs`):
```rust
#[test]
#[cfg(feature = "probe-first-escalation")]
fn test_spa_probe_first_escalation() {
    let html = r#"<html><script>window.__NEXT_DATA__={}</script><body>Content</body></html>"#;
    let engine = decide_engine(html, "https://example.com");
    assert_eq!(engine, Engine::ProbeFirst);
}

#[test]
#[cfg(not(feature = "probe-first-escalation"))]
fn test_spa_legacy_headless() {
    let html = r#"<html><script>window.__NEXT_DATA__={}</script><body>Content</body></html>"#;
    let engine = decide_engine(html, "https://example.com");
    assert_eq!(engine, Engine::Headless);
}
```

**Integration Tests** (new file: `tests/integration/probe_first_escalation_tests.rs`):
```rust
#[tokio::test]
#[cfg(feature = "probe-first-escalation")]
async fn test_spa_wasm_probe_success() {
    // SPA page with good content in HTML
    let html = include_str!("../../fixtures/spa_with_ssr.html");
    let result = extract_with_probe_first(html).await.unwrap();
    assert!(result.extracted_with_wasm);
    assert!(!result.escalated_to_headless);
}

#[tokio::test]
#[cfg(feature = "probe-first-escalation")]
async fn test_spa_wasm_probe_escalation() {
    // SPA page with client-only content
    let html = r#"<html><div id="root"></div><script>/* React app */</script></html>"#;
    let result = extract_with_probe_first(html).await.unwrap();
    assert!(result.escalated_to_headless);
}
```

**Metrics to Track:**
- `engine_selection.probe_first.attempts` (counter)
- `engine_selection.probe_first.wasm_success` (counter)
- `engine_selection.probe_first.headless_escalation` (counter)
- `engine_selection.probe_first.quality_score` (histogram)

#### Rollout Plan

**Phase 1 (0%):** Feature disabled, legacy behavior
```bash
cargo build --release
```

**Phase 2 (10%):** Enable for 10% of traffic via environment variable
```bash
export RIPTIDE_PROBE_FIRST_ROLLOUT=0.10
cargo build --release --features probe-first-escalation
```

**Phase 3 (50%):** Increase to 50% after 48h monitoring
```bash
export RIPTIDE_PROBE_FIRST_ROLLOUT=0.50
```

**Phase 4 (100%):** Full rollout after 7 days
```bash
export RIPTIDE_PROBE_FIRST_ROLLOUT=1.0
```

**Rollback Trigger:** Headless escalation rate > 80% (indicates probe is failing)

---

### 10.2: JSON-LD Short-Circuit (0.5 day, ~70 LOC)

**Objective:** Early return when JSON-LD contains complete Event/Article schema.

#### Files to Modify

**File 1:** `crates/riptide-extraction/src/strategies/metadata.rs`

**Current Code (lines 168-186):**
```rust
fn extract_json_ld(document: &Html, metadata: &mut DocumentMetadata,
                   method: &mut ExtractionMethod) -> Result<()> {
    for element in document.select(&selector) {
        let json_value = serde_json::from_str::<serde_json::Value>(&json_text)?;
        extract_from_json_ld(&json_value, metadata)?;
        method.json_ld = true;
    }
    Ok(())
}
```

**New Code:**
```rust
fn extract_json_ld(document: &Html, metadata: &mut DocumentMetadata,
                   method: &mut ExtractionMethod) -> Result<()> {
    for element in document.select(&selector) {
        let json_value = serde_json::from_str::<serde_json::Value>(&json_text)?;
        extract_from_json_ld(&json_value, metadata)?;
        method.json_ld = true;

        // Phase 10: Short-circuit if complete Event/Article schema found
        #[cfg(feature = "jsonld-shortcircuit")]
        if is_jsonld_complete(&json_value, metadata) {
            tracing::debug!(
                "JSON-LD short-circuit: Complete {} schema detected",
                get_schema_type(&json_value).unwrap_or("unknown")
            );
            return Ok(());  // Skip Open Graph, meta tags, heuristics
        }
    }
    Ok(())
}

/// Check if JSON-LD schema is complete enough to skip other extraction
#[cfg(feature = "jsonld-shortcircuit")]
fn is_jsonld_complete(json: &serde_json::Value, metadata: &DocumentMetadata) -> bool {
    let items = if json.is_array() {
        json.as_array().unwrap()
    } else {
        std::slice::from_ref(json)
    };

    for item in items {
        if let Some(obj) = item.as_object() {
            let schema_type = obj.get("@type").and_then(|v| v.as_str());

            match schema_type {
                Some("Event") => {
                    // Event must have: name, startDate, location
                    let has_required = metadata.title.is_some()
                        && obj.get("startDate").is_some()
                        && obj.get("location").is_some();
                    if has_required { return true; }
                }
                Some("Article") | Some("NewsArticle") | Some("BlogPosting") => {
                    // Article must have: headline, author, datePublished, description
                    let has_required = metadata.title.is_some()
                        && metadata.author.is_some()
                        && metadata.published_date.is_some()
                        && metadata.description.is_some();
                    if has_required { return true; }
                }
                _ => continue,
            }
        }
    }
    false
}

/// Get schema type for logging
#[cfg(feature = "jsonld-shortcircuit")]
fn get_schema_type(json: &serde_json::Value) -> Option<String> {
    let items = if json.is_array() {
        json.as_array()?
    } else {
        std::slice::from_ref(json)
    };

    for item in items {
        if let Some(schema_type) = item.get("@type").and_then(|v| v.as_str()) {
            return Some(schema_type.to_string());
        }
    }
    None
}
```

**File 2:** `crates/riptide-extraction/Cargo.toml`

Add feature flag:
```toml
[features]
default = []
jsonld-shortcircuit = ["tracing"]
```

#### Testing Strategy

**Unit Tests** (`crates/riptide-extraction/tests/metadata_extraction_tests.rs`):
```rust
#[tokio::test]
#[cfg(feature = "jsonld-shortcircuit")]
async fn test_jsonld_event_shortcircuit() {
    let html = r#"
    <html>
    <head>
        <script type="application/ld+json">
        {
            "@type": "Event",
            "name": "Tech Conference 2025",
            "startDate": "2025-11-01",
            "location": "San Francisco"
        }
        </script>
        <meta property="og:title" content="Different Title">
    </head>
    </html>"#;

    let metadata = extract_metadata(html, "https://example.com").await.unwrap();

    // Should use JSON-LD data, not Open Graph
    assert_eq!(metadata.title, Some("Tech Conference 2025".to_string()));
    assert!(metadata.extraction_method.json_ld);
    assert!(!metadata.extraction_method.open_graph);  // Skipped
}

#[tokio::test]
#[cfg(feature = "jsonld-shortcircuit")]
async fn test_jsonld_article_shortcircuit() {
    let html = r#"
    <script type="application/ld+json">
    {
        "@type": "Article",
        "headline": "Breaking News",
        "author": {"name": "Jane Doe"},
        "datePublished": "2025-10-24",
        "description": "Important story"
    }
    </script>"#;

    let metadata = extract_metadata(html, "https://example.com").await.unwrap();

    assert_eq!(metadata.title, Some("Breaking News".to_string()));
    assert_eq!(metadata.author, Some("Jane Doe".to_string()));
    assert!(metadata.extraction_method.json_ld);
    assert!(!metadata.extraction_method.heuristics);  // Skipped
}

#[tokio::test]
#[cfg(feature = "jsonld-shortcircuit")]
async fn test_jsonld_incomplete_fallback() {
    let html = r#"
    <script type="application/ld+json">
    {
        "@type": "Article",
        "headline": "Breaking News"
    }
    </script>
    <meta property="og:author" content="John Smith">
    "#;

    let metadata = extract_metadata(html, "https://example.com").await.unwrap();

    // Incomplete JSON-LD, should fall back to Open Graph
    assert!(metadata.extraction_method.json_ld);
    assert!(metadata.extraction_method.open_graph);
    assert_eq!(metadata.author, Some("John Smith".to_string()));
}
```

**Metrics to Track:**
- `metadata.jsonld_shortcircuit.events` (counter)
- `metadata.jsonld_shortcircuit.articles` (counter)
- `metadata.jsonld_shortcircuit.incomplete` (counter - fallback to normal flow)
- `metadata.extraction_time_ms` (histogram - should decrease)

#### Rollout Plan

**Phase 1 (0%):** Feature disabled
```bash
cargo build --release
```

**Phase 2 (10%):** Enable for 10% of traffic
```bash
cargo build --release --features jsonld-shortcircuit
export RIPTIDE_JSONLD_SHORTCIRCUIT_ROLLOUT=0.10
```

**Phase 3 (100%):** Full rollout after 48h monitoring (low risk change)
```bash
export RIPTIDE_JSONLD_SHORTCIRCUIT_ROLLOUT=1.0
```

**Quality Check:** Compare extracted metadata quality (title, author, date accuracy) between short-circuit and full extraction on 1000 random pages.

---

### 10.3: Refined Content Signals (1 day, ~120 LOC)

**Objective:** Improve content ratio calculation to reduce false positives (mis-classification as client-rendered).

#### Files to Modify

**File 1:** `crates/riptide-reliability/src/engine_selection.rs`

**Current Code (lines 307-322):**
```rust
pub fn calculate_content_ratio(html: &str) -> f64 {
    let total_len = html.len() as f64;
    if total_len == 0.0 { return 0.0; }

    let text_content: String = html
        .split('<')
        .filter_map(|s| s.split('>').nth(1))
        .collect();

    let content_len = text_content.trim().len() as f64;
    content_len / total_len
}
```

**New Code:**
```rust
pub fn calculate_content_ratio(html: &str) -> f64 {
    #[cfg(feature = "refined-content-signals")]
    return calculate_content_ratio_refined(html);

    #[cfg(not(feature = "refined-content-signals"))]
    {
        // Legacy behavior
        let total_len = html.len() as f64;
        if total_len == 0.0 { return 0.0; }

        let text_content: String = html
            .split('<')
            .filter_map(|s| s.split('>').nth(1))
            .collect();

        let content_len = text_content.trim().len() as f64;
        content_len / total_len
    }
}

/// Refined content ratio calculation with script/style stripping and placeholder detection
#[cfg(feature = "refined-content-signals")]
fn calculate_content_ratio_refined(html: &str) -> f64 {
    use scraper::{Html, Selector};

    let document = Html::parse_document(html);
    let total_len = html.len() as f64;
    if total_len == 0.0 { return 0.0; }

    // 1. Strip <script> and <style> content
    let mut stripped_html = html.to_string();

    // Remove <script> blocks
    if let Ok(script_selector) = Selector::parse("script") {
        for element in document.select(&script_selector) {
            let script_html = element.html();
            stripped_html = stripped_html.replace(&script_html, "");
        }
    }

    // Remove <style> blocks
    if let Ok(style_selector) = Selector::parse("style") {
        for element in document.select(&style_selector) {
            let style_html = element.html();
            stripped_html = stripped_html.replace(&style_html, "");
        }
    }

    // 2. Extract visible text
    let text_content: String = stripped_html
        .split('<')
        .filter_map(|s| s.split('>').nth(1))
        .collect();

    // 3. Filter out placeholder/skeleton text
    let visible_text = filter_placeholder_text(&text_content);

    // 4. Calculate ratio
    let content_len = visible_text.trim().len() as f64;
    content_len / total_len
}

/// Filter out common placeholder/skeleton text patterns
#[cfg(feature = "refined-content-signals")]
fn filter_placeholder_text(text: &str) -> String {
    let placeholder_patterns = [
        "loading",
        "please wait",
        "skeleton",
        "shimmer",
        "...",
        "⋯",
        "●●●",
    ];

    text.lines()
        .filter(|line| {
            let line_lower = line.trim().to_lowercase();

            // Skip empty lines
            if line_lower.is_empty() { return false; }

            // Skip lines that are only placeholder patterns
            if placeholder_patterns.iter().any(|p| line_lower == *p) {
                return false;
            }

            // Skip lines that are very short (likely placeholders)
            if line_lower.len() < 5 { return false; }

            true
        })
        .collect::<Vec<_>>()
        .join("\n")
}

/// Detect placeholder/skeleton classes in HTML
#[cfg(feature = "refined-content-signals")]
pub fn has_placeholder_markers(html: &str) -> bool {
    let html_lower = html.to_lowercase();

    let placeholder_classes = [
        "skeleton",
        "shimmer",
        "placeholder",
        "loading-skeleton",
        "content-loader",
        "skeleton-loader",
    ];

    placeholder_classes.iter().any(|class| {
        html_lower.contains(&format!("class=\"{}\"", class))
            || html_lower.contains(&format!("class=\"{}\"", class))
            || html_lower.contains(&format!("class=\"{} ", class))
    })
}
```

**Update decide_engine logic** (lines 177-192):
```rust
pub fn decide_engine(html: &str, _url: &str) -> Engine {
    let html_lower = html.to_lowercase();

    // ... existing detection code ...

    // Calculate content ratio (uses refined calculation if feature enabled)
    let content_ratio = calculate_content_ratio(html);

    // Check for placeholder markers (refined signals)
    #[cfg(feature = "refined-content-signals")]
    let has_placeholders = has_placeholder_markers(html);

    #[cfg(not(feature = "refined-content-signals"))]
    let has_placeholders = false;

    // Decision logic
    if has_anti_scraping {
        Engine::Headless
    } else if has_react || has_vue || has_angular || has_spa_markers {
        #[cfg(feature = "probe-first-escalation")]
        return Engine::ProbeFirst;

        #[cfg(not(feature = "probe-first-escalation"))]
        return Engine::Headless;
    } else if content_ratio < 0.1 || has_placeholders {
        // Low content OR placeholder markers → likely client-rendered
        Engine::Headless
    } else {
        Engine::Wasm
    }
}
```

**File 2:** `crates/riptide-reliability/Cargo.toml`

Add dependencies and feature flag:
```toml
[dependencies]
scraper = "0.17"  # Already present

[features]
default = []
refined-content-signals = []
```

#### Testing Strategy

**Unit Tests** (`crates/riptide-reliability/tests/content_signal_tests.rs`):
```rust
#[test]
#[cfg(feature = "refined-content-signals")]
fn test_content_ratio_strips_scripts() {
    let html = r#"
    <html>
    <head>
        <script>
        // Large React bundle (1000 chars)
        const app = { ... };
        </script>
    </head>
    <body>
        <article>Short article text</article>
    </body>
    </html>"#;

    let legacy_ratio = calculate_content_ratio_legacy(html);
    let refined_ratio = calculate_content_ratio_refined(html);

    assert!(refined_ratio > legacy_ratio, "Refined should ignore script content");
    assert!(refined_ratio > 0.1, "Should pass content threshold");
}

#[test]
#[cfg(feature = "refined-content-signals")]
fn test_placeholder_detection() {
    let html = r#"<div class="skeleton-loader">Loading...</div>"#;
    assert!(has_placeholder_markers(html));

    let html2 = r#"<div class="content">Real content here</div>"#;
    assert!(!has_placeholder_markers(html2));
}

#[test]
#[cfg(feature = "refined-content-signals")]
fn test_placeholder_text_filtering() {
    let text = "Loading...\nReal content here\nPlease wait\nMore real content";
    let filtered = filter_placeholder_text(text);

    assert!(!filtered.contains("Loading"));
    assert!(!filtered.contains("Please wait"));
    assert!(filtered.contains("Real content"));
}

#[test]
#[cfg(feature = "refined-content-signals")]
fn test_engine_decision_with_placeholders() {
    let html = r#"
    <html>
    <body>
        <div class="skeleton-loader"></div>
        <div class="shimmer"></div>
    </body>
    </html>"#;

    let engine = decide_engine(html, "https://example.com");
    assert_eq!(engine, Engine::Headless, "Skeleton markers should trigger headless");
}

#[test]
#[cfg(feature = "refined-content-signals")]
fn test_engine_decision_with_ssr_content() {
    let html = r#"
    <html>
    <head>
        <script src="next.js"></script>
        <script>window.__NEXT_DATA__ = {...}</script>
    </head>
    <body>
        <article>
            This is substantial server-side rendered content that should
            be extractable with WASM even though Next.js markers are present.
            Lorem ipsum dolor sit amet, consectetur adipiscing elit.
        </article>
    </body>
    </html>"#;

    #[cfg(feature = "probe-first-escalation")]
    {
        let engine = decide_engine(html, "https://example.com");
        assert_eq!(engine, Engine::ProbeFirst, "Should try WASM probe first");
    }
}
```

**Integration Tests** (new file: `tests/integration/content_signal_tests.rs`):
```rust
#[tokio::test]
#[cfg(feature = "refined-content-signals")]
async fn test_ssr_nextjs_extraction() {
    // Real-world Next.js page with SSR content
    let html = include_str!("../../fixtures/nextjs_ssr.html");

    let legacy_ratio = calculate_content_ratio_legacy(html);
    let refined_ratio = calculate_content_ratio_refined(html);

    assert!(refined_ratio > legacy_ratio);
    assert!(refined_ratio > 0.15, "SSR Next.js should have good content ratio");
}

#[tokio::test]
#[cfg(feature = "refined-content-signals")]
async fn test_skeleton_screen_detection() {
    let html = include_str!("../../fixtures/skeleton_screen.html");

    assert!(has_placeholder_markers(html));
    let engine = decide_engine(html, "https://example.com");
    assert_eq!(engine, Engine::Headless);
}
```

**Metrics to Track:**
- `content_signals.refined_ratio` (histogram - distribution of refined ratios)
- `content_signals.legacy_ratio` (histogram - for comparison)
- `content_signals.placeholder_detected` (counter)
- `content_signals.misclassification_avoided` (counter - when refined prevents headless)

#### Rollout Plan

**Phase 1 (0%):** Feature disabled
```bash
cargo build --release
```

**Phase 2 (10%):** Enable for 10% of traffic
```bash
cargo build --release --features refined-content-signals
export RIPTIDE_REFINED_SIGNALS_ROLLOUT=0.10
```

**Phase 3 (50%):** Increase to 50% after 48h monitoring
```bash
export RIPTIDE_REFINED_SIGNALS_ROLLOUT=0.50
```

**Phase 4 (100%):** Full rollout after 7 days
```bash
export RIPTIDE_REFINED_SIGNALS_ROLLOUT=1.0
```

**Quality Validation:**
1. Sample 1000 pages classified as "needs headless" by legacy
2. Reclassify with refined signals
3. For pages now classified as "WASM-capable":
   - Extract with WASM
   - Measure quality score (should be >60)
4. Target: 20-30% reclassification with <5% quality regression

---

## Combined Integration

### Feature Flag Coordination

All three features can be enabled independently or together:

**Cargo.toml (workspace level):**
```toml
[workspace.features]
# Phase 10 Quick Wins
probe-first-escalation = ["riptide-reliability/probe-first-escalation"]
jsonld-shortcircuit = ["riptide-extraction/jsonld-shortcircuit"]
refined-content-signals = ["riptide-reliability/refined-content-signals"]

# All Phase 10 features
phase10-all = [
    "probe-first-escalation",
    "jsonld-shortcircuit",
    "refined-content-signals"
]
```

### Gradual Rollout Implementation

**Pattern from `riptide-intelligence/runtime_switch.rs`** (lines 112-121):

```rust
/// Gradual rollout configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GradualRolloutConfig {
    pub enabled: bool,
    pub from_provider: String,
    pub to_provider: String,
    pub duration: Duration,
    pub traffic_increment: f64,  // percentage per step
    pub step_duration: Duration,
    pub success_threshold: f64,  // minimum success rate to continue
    pub auto_rollback: bool,
}
```

**Apply to Phase 10** (new file: `crates/riptide-reliability/src/phase10_rollout.rs`):

```rust
use serde::{Deserialize, Serialize};
use std::time::Duration;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Phase10RolloutConfig {
    pub probe_first_enabled: bool,
    pub probe_first_traffic: f64,  // 0.0 to 1.0

    pub jsonld_shortcircuit_enabled: bool,
    pub jsonld_shortcircuit_traffic: f64,

    pub refined_signals_enabled: bool,
    pub refined_signals_traffic: f64,

    pub auto_rollback_on_quality_drop: bool,
    pub quality_threshold: f64,  // minimum quality score (0.6 = 60%)
}

impl Default for Phase10RolloutConfig {
    fn default() -> Self {
        Self {
            probe_first_enabled: false,
            probe_first_traffic: 0.0,
            jsonld_shortcircuit_enabled: false,
            jsonld_shortcircuit_traffic: 0.0,
            refined_signals_enabled: false,
            refined_signals_traffic: 0.0,
            auto_rollback_on_quality_drop: true,
            quality_threshold: 0.6,
        }
    }
}

pub fn should_use_probe_first(config: &Phase10RolloutConfig) -> bool {
    if !config.probe_first_enabled { return false; }
    rand::random::<f64>() < config.probe_first_traffic
}

pub fn should_use_jsonld_shortcircuit(config: &Phase10RolloutConfig) -> bool {
    if !config.jsonld_shortcircuit_enabled { return false; }
    rand::random::<f64>() < config.jsonld_shortcircuit_traffic
}

pub fn should_use_refined_signals(config: &Phase10RolloutConfig) -> bool {
    if !config.refined_signals_enabled { return false; }
    rand::random::<f64>() < config.refined_signals_traffic
}
```

### Environment Variables

```bash
# Phase 10.1: Probe-First Escalation
RIPTIDE_PROBE_FIRST_ENABLED=true
RIPTIDE_PROBE_FIRST_TRAFFIC=0.10  # 10% rollout

# Phase 10.2: JSON-LD Short-Circuit
RIPTIDE_JSONLD_SHORTCIRCUIT_ENABLED=true
RIPTIDE_JSONLD_SHORTCIRCUIT_TRAFFIC=0.10

# Phase 10.3: Refined Content Signals
RIPTIDE_REFINED_SIGNALS_ENABLED=true
RIPTIDE_REFINED_SIGNALS_TRAFFIC=0.10

# Quality monitoring
RIPTIDE_PHASE10_AUTO_ROLLBACK=true
RIPTIDE_PHASE10_QUALITY_THRESHOLD=0.60
```

---

## Testing Strategy Summary

### Unit Tests (40 tests total)

| Component | Tests | File |
|-----------|-------|------|
| Engine Selection | 15 | `crates/riptide-reliability/tests/engine_selection_tests.rs` |
| Metadata Extraction | 12 | `crates/riptide-extraction/tests/metadata_extraction_tests.rs` |
| Content Signals | 13 | `crates/riptide-reliability/tests/content_signal_tests.rs` |

### Integration Tests (12 tests)

| Feature | Tests | File |
|---------|-------|------|
| Probe-First | 4 | `tests/integration/probe_first_escalation_tests.rs` |
| JSON-LD Short-Circuit | 4 | `tests/integration/jsonld_shortcircuit_tests.rs` |
| Content Signals | 4 | `tests/integration/content_signal_tests.rs` |

### Performance Tests (3 benchmarks)

| Benchmark | Metric | Target |
|-----------|--------|--------|
| JSON-LD Short-Circuit | Extraction time | 70% reduction |
| Probe-First vs Direct Headless | Engine decision time | <50ms overhead |
| Content Ratio Refined | Calculation time | <5ms |

### Quality Validation

**Automated Validation Suite:**
1. **Extraction Quality** (1000 pages):
   - Sample pages from production traffic
   - Compare extraction results (legacy vs Phase 10)
   - Measure: title accuracy, author accuracy, content completeness
   - Threshold: >95% quality retention

2. **Cost Metrics** (7-day window):
   - Headless browser launches (expect 60-80% reduction)
   - Average extraction time (expect 40-60% reduction)
   - Resource usage (CPU, memory)

3. **Edge Cases:**
   - SSR React/Next.js pages (probe-first should succeed)
   - Client-only SPAs (probe-first should escalate)
   - Skeleton screens (refined signals should detect)
   - Complete JSON-LD (short-circuit should activate)

---

## Implementation Pseudocode

### 10.1: Probe-First Escalation

```rust
// engine_selection.rs
pub enum Engine {
    Auto, Raw, Wasm, Headless,
    ProbeFirst,  // NEW
}

pub fn decide_engine(html: &str, url: &str) -> Engine {
    if has_anti_scraping(html) {
        return Engine::Headless;
    }

    if has_spa_markers(html) {
        #[cfg(feature = "probe-first-escalation")]
        return Engine::ProbeFirst;

        #[cfg(not(feature = "probe-first-escalation"))]
        return Engine::Headless;
    }

    if content_ratio < 0.1 {
        return Engine::Headless;
    }

    Engine::Wasm
}

// extract.rs
async fn execute_local_extraction(args, output_format, engine) -> Result<()> {
    // ... fetch HTML ...

    if engine == Engine::ProbeFirst {
        match try_wasm_extraction(&html, url, &args).await {
            Ok(result) if result.quality_score >= 50 => {
                log("WASM probe succeeded");
                return output_extraction_result(result, ...);
            }
            _ => {
                log("WASM probe failed, escalating to headless");
                engine = Engine::Headless;
            }
        }
    }

    // ... continue with engine selection ...
}
```

### 10.2: JSON-LD Short-Circuit

```rust
// metadata.rs
fn extract_json_ld(document: &Html, metadata: &mut DocumentMetadata,
                   method: &mut ExtractionMethod) -> Result<()> {
    for element in document.select(&selector) {
        let json_value = parse_json_ld(element)?;
        extract_from_json_ld(&json_value, metadata)?;
        method.json_ld = true;

        #[cfg(feature = "jsonld-shortcircuit")]
        if is_complete_schema(&json_value, metadata) {
            log("JSON-LD short-circuit: complete schema");
            return Ok(());  // Skip other extraction methods
        }
    }
    Ok(())
}

fn is_complete_schema(json: &Value, metadata: &DocumentMetadata) -> bool {
    match json.get("@type").as_str() {
        Some("Event") => {
            has_fields(metadata, ["title", "startDate", "location"])
        }
        Some("Article") | Some("NewsArticle") => {
            has_fields(metadata, ["title", "author", "published_date", "description"])
        }
        _ => false
    }
}
```

### 10.3: Refined Content Signals

```rust
// engine_selection.rs
pub fn calculate_content_ratio(html: &str) -> f64 {
    #[cfg(feature = "refined-content-signals")]
    {
        let document = Html::parse_document(html);

        // Strip <script> and <style>
        let stripped = strip_non_content_tags(&document);

        // Extract visible text
        let visible_text = extract_text(&stripped);

        // Filter placeholders
        let real_content = filter_placeholder_text(&visible_text);

        real_content.len() as f64 / html.len() as f64
    }

    #[cfg(not(feature = "refined-content-signals"))]
    {
        // Legacy calculation
        simple_text_ratio(html)
    }
}

pub fn has_placeholder_markers(html: &str) -> bool {
    let placeholder_classes = ["skeleton", "shimmer", "placeholder", "loading-skeleton"];
    placeholder_classes.iter().any(|class| html.contains(class))
}

pub fn decide_engine(html: &str, url: &str) -> Engine {
    let content_ratio = calculate_content_ratio(html);

    #[cfg(feature = "refined-content-signals")]
    let has_placeholders = has_placeholder_markers(html);

    #[cfg(not(feature = "refined-content-signals"))]
    let has_placeholders = false;

    if content_ratio < 0.1 || has_placeholders {
        return Engine::Headless;
    }

    // ... other checks ...
}
```

---

## Success Metrics

### Primary Metrics (Week 1)

| Metric | Baseline | Target | Measurement |
|--------|----------|--------|-------------|
| **Headless Usage** | 100% for SPAs | 20-40% for SPAs | `engine_selection.headless.count` |
| **WASM Probe Success** | N/A | 60-80% | `engine_selection.probe_first.wasm_success / probe_first.attempts` |
| **JSON-LD Short-Circuit** | 0% | 30-40% for news sites | `metadata.jsonld_shortcircuit.events / total_extractions` |
| **Extraction Quality** | Baseline | >95% retention | Compare title/author/date accuracy |

### Secondary Metrics (Week 2-4)

| Metric | Target | Measurement |
|--------|--------|-------------|
| **Cost Reduction** | 60-80% | Headless browser seconds / total extractions |
| **Average Extraction Time** | 40-60% faster | `extraction_time_ms` histogram (p50, p95) |
| **False Positive Rate** | <5% | Pages incorrectly classified as Headless |
| **False Negative Rate** | <2% | Pages incorrectly classified as WASM |

### Rollback Triggers

**Automatic Rollback If:**
1. Extraction quality drops below 90% (5% regression tolerance)
2. Headless escalation rate > 85% (indicates probe is failing)
3. JSON-LD short-circuit quality < 85%
4. Fatal errors increase > 10%

**Manual Review If:**
1. False positive rate 5-10%
2. New edge cases discovered (exotic SPAs, unusual JSON-LD)
3. Performance degradation in content ratio calculation

---

## Risk Assessment

### 10.1: Probe-First Escalation

**Risk Level:** MEDIUM

**Risks:**
- WASM probe adds latency (50-200ms)
- Quality assessment may be inaccurate (quality_score threshold tuning)
- Some SPAs may work partially with WASM (incomplete content)

**Mitigation:**
- Fast timeout for WASM probe (1-2s max)
- Conservative quality threshold (50-60)
- Feature flag for instant rollback
- A/B test with 10% traffic first

### 10.2: JSON-LD Short-Circuit

**Risk Level:** LOW

**Risks:**
- Some JSON-LD may be incomplete (missing fields)
- Schema validation may have edge cases

**Mitigation:**
- Strict completeness checks (require all critical fields)
- Log all short-circuit cases for review
- Compare quality with full extraction in first week
- Easy rollback via feature flag

### 10.3: Refined Content Signals

**Risk Level:** LOW

**Risks:**
- Content ratio calculation slightly slower (HTML parsing)
- May over-filter legitimate short content
- Placeholder detection may have false positives

**Mitigation:**
- Cache parsed HTML document (already parsed in metadata extraction)
- Validate placeholder patterns on production data
- Whitelist known false positives
- Compare legacy vs refined ratios for 1000 pages

---

## Implementation Checklist

### Pre-Implementation (Day 0)

- [ ] Create feature branches:
  - `feature/phase10-probe-first`
  - `feature/phase10-jsonld-shortcircuit`
  - `feature/phase10-refined-signals`
- [ ] Set up metrics dashboards (Grafana/Prometheus)
- [ ] Create test fixtures (SSR pages, skeleton screens, JSON-LD examples)
- [ ] Review existing runtime_switch.rs pattern

### Day 1: Probe-First Escalation

- [ ] Add `ProbeFirst` variant to `Engine` enum
- [ ] Update `decide_engine` logic with feature flag
- [ ] Implement `try_wasm_extraction` helper
- [ ] Update CLI orchestration in `extract.rs`
- [ ] Write 8 unit tests
- [ ] Write 4 integration tests
- [ ] Manual testing with 10 SPA URLs

### Day 2: JSON-LD Short-Circuit

- [ ] Implement `is_jsonld_complete` function
- [ ] Add short-circuit logic to `extract_json_ld`
- [ ] Implement `get_schema_type` helper
- [ ] Write 8 unit tests
- [ ] Write 4 integration tests
- [ ] Validate on 100 news sites with JSON-LD

### Day 3: Refined Content Signals

- [ ] Implement `calculate_content_ratio_refined`
- [ ] Implement `has_placeholder_markers`
- [ ] Implement `filter_placeholder_text`
- [ ] Update `decide_engine` with placeholder check
- [ ] Write 10 unit tests
- [ ] Write 4 integration tests
- [ ] Validate on 100 skeleton screen examples

### Day 4: Integration & Testing

- [ ] Run full test suite (`cargo test --workspace`)
- [ ] Run benchmarks (extraction time, content ratio)
- [ ] Manual QA on 50 diverse URLs
- [ ] Performance profiling (flamegraph)
- [ ] Documentation updates

### Day 5: Deployment & Monitoring

- [ ] Deploy with all features disabled (0% rollout)
- [ ] Enable 10% rollout for each feature
- [ ] Monitor metrics for 24 hours
- [ ] Validate quality on sampled extractions
- [ ] Adjust thresholds based on data
- [ ] Prepare rollback plan

### Week 2-3: Gradual Rollout

- [ ] Day 7: Increase to 50% rollout
- [ ] Day 14: Increase to 100% rollout
- [ ] Day 21: Remove feature flags (make default)
- [ ] Week 4: Cleanup legacy code paths

---

## Appendix A: Code Locations Reference

### Key Files

```
/workspaces/eventmesh/
├── crates/
│   ├── riptide-reliability/
│   │   ├── src/
│   │   │   ├── engine_selection.rs (450 LOC) ← MODIFY (10.1, 10.3)
│   │   │   └── phase10_rollout.rs ← NEW (rollout config)
│   │   ├── tests/
│   │   │   ├── engine_selection_tests.rs ← ADD TESTS
│   │   │   └── content_signal_tests.rs ← NEW
│   │   └── Cargo.toml ← ADD FEATURES
│   │
│   ├── riptide-extraction/
│   │   ├── src/
│   │   │   └── strategies/
│   │   │       └── metadata.rs (733 LOC) ← MODIFY (10.2)
│   │   ├── tests/
│   │   │   └── metadata_extraction_tests.rs ← ADD TESTS
│   │   └── Cargo.toml ← ADD FEATURES
│   │
│   └── riptide-cli/
│       └── src/
│           └── commands/
│               └── extract.rs (882 LOC) ← MODIFY (10.1 orchestration)
│
└── tests/
    └── integration/
        ├── probe_first_escalation_tests.rs ← NEW
        ├── jsonld_shortcircuit_tests.rs ← NEW
        └── content_signal_tests.rs ← NEW
```

### Line Numbers for Modifications

**engine_selection.rs:**
- Line 36: Add `ProbeFirst` to `Engine` enum
- Lines 52-64: Update `from_str` implementation
- Lines 72-82: Update `name()` method
- Lines 181-183: Add probe-first logic (10.1)
- Lines 307-322: Replace with refined calculation (10.3)
- After line 322: Add helper functions for 10.3

**metadata.rs:**
- Lines 168-186: Add short-circuit logic (10.2)
- After line 186: Add `is_jsonld_complete` function
- After line 270: Add `get_schema_type` helper

**extract.rs:**
- Lines 496-530: Add probe-first orchestration (10.1)
- After line 530: Add `try_wasm_extraction` helper

---

## Appendix B: Example Test Fixtures

### SSR Next.js Page (nextjs_ssr.html)

```html
<!DOCTYPE html>
<html>
<head>
    <script id="__NEXT_DATA__" type="application/json">
    {"props":{"pageProps":{"article":{"title":"Server Rendered"}}}}
    </script>
</head>
<body>
    <article>
        <h1>Server Rendered Article</h1>
        <p>This content is rendered server-side and should be extractable
           with WASM even though Next.js markers are present.</p>
        <p>Lorem ipsum dolor sit amet, consectetur adipiscing elit. Sed do
           eiusmod tempor incididunt ut labore et dolore magna aliqua.</p>
    </article>
</body>
</html>
```

### Skeleton Screen (skeleton_screen.html)

```html
<!DOCTYPE html>
<html>
<body>
    <div class="skeleton-loader">
        <div class="skeleton shimmer"></div>
        <div class="skeleton shimmer"></div>
        <div class="skeleton shimmer"></div>
    </div>
    <script>
    // Content will be loaded client-side
    fetch('/api/content').then(r => r.json()).then(renderContent);
    </script>
</body>
</html>
```

### Complete JSON-LD Event (jsonld_event.html)

```html
<!DOCTYPE html>
<html>
<head>
    <script type="application/ld+json">
    {
        "@context": "https://schema.org",
        "@type": "Event",
        "name": "Tech Conference 2025",
        "startDate": "2025-11-01T09:00:00-07:00",
        "endDate": "2025-11-03T17:00:00-07:00",
        "location": {
            "@type": "Place",
            "name": "Moscone Center",
            "address": {
                "@type": "PostalAddress",
                "streetAddress": "747 Howard St",
                "addressLocality": "San Francisco",
                "addressRegion": "CA",
                "postalCode": "94103"
            }
        },
        "description": "Annual technology conference",
        "image": "https://example.com/event.jpg",
        "offers": {
            "@type": "Offer",
            "price": "299.00",
            "priceCurrency": "USD"
        }
    }
    </script>
</head>
<body>
    <h1>Tech Conference 2025</h1>
</body>
</html>
```

---

## Next Steps

**Immediate Actions:**
1. Review this plan with team
2. Create feature branches
3. Set up metrics dashboards
4. Begin Day 1 implementation (Probe-First Escalation)

**Questions to Resolve:**
1. Quality score threshold for WASM probe (50 vs 60 vs 70?)
2. WASM probe timeout (1s vs 2s?)
3. Metrics backend (Prometheus/Grafana vs DataDog?)
4. Rollout schedule (aggressive vs conservative?)

**Dependencies:**
- [ ] `scraper` crate (already present)
- [ ] `tracing` crate (already present)
- [ ] `rand` crate (add for traffic splitting)
- [ ] Metrics infrastructure (existing monitoring crate)

---

**End of Implementation Plan**
