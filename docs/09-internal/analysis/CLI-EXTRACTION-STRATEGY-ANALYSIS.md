# RipTide CLI - Extraction Strategy Analysis

**Question**: Does the API automatically figure out extraction strategy, or do we need extraction flags on spider/crawl commands?

**Answer**: **It depends on the endpoint!**

---

## The Truth: Two Different Paths

### Path 1: Automatic Extraction (Simple)

```
POST /crawl
POST /spider/crawl
POST /deepsearch

Strategy Selection: AUTOMATIC (no user control)
Uses: Gate analysis → Automatic strategy selection
Options: Can disable extraction (skip_extraction: true) but can't choose strategy
```

**CrawlOptions fields:**
```rust
pub struct CrawlOptions {
    pub concurrency: usize,
    pub cache_mode: String,
    pub render_mode: RenderMode,         // ← Static/Dynamic/Adaptive
    pub use_spider: Option<bool>,
    pub chunking_config: Option<...>,
    pub skip_extraction: Option<bool>,   // ← Can skip, but can't control HOW
    // NO extraction_strategy field!
}
```

**How it works:**
```
User → POST /crawl
  → Pipeline: fetch → gate analysis → AUTO extract
  → Gate decides: Raw, ProbesFirst, or Headless
  → Extraction happens with default strategy
  → User has NO control over extraction strategy
```

---

### Path 2: Controlled Extraction (Advanced)

```
POST /extract

Strategy Selection: USER CONTROLLED
Options: strategy, quality_threshold, timeout
Strategies: "auto", "css", "wasm", "llm", "multi"
```

**ExtractOptions fields:**
```rust
pub struct ExtractOptions {
    pub strategy: String,              // ← "auto", "css", "wasm", "llm", "multi"
    pub quality_threshold: f64,        // ← 0.0-1.0
    pub timeout_ms: u64,
}
```

**How it works:**
```
User → POST /extract { url, options: { strategy: "css" } }
  → Direct extraction with specified strategy
  → User has FULL control
```

---

## Available Extraction Strategies

### Core Strategies (Built-in)

```rust
pub enum ExtractionStrategyType {
    /// WASM-based extraction (fastest, default)
    Wasm,

    /// CSS selector-based extraction
    Css,

    /// Regular expression-based extraction
    Regex,

    /// Automatic strategy selection
    Auto,
}
```

### Extended Strategies (via /extract endpoint)

From `/extract` endpoint docs:
- `"auto"` - Automatic selection
- `"css"` - CSS selectors
- `"wasm"` - WebAssembly extraction
- `"llm"` - LLM-powered extraction
- `"multi"` - Multi-strategy with fallback (DEFAULT)

---

## Comparison: What API Endpoints Expose

| Endpoint | Extraction | Strategy Control | Quality Control |
|----------|-----------|------------------|-----------------|
| POST /crawl | ✅ Automatic | ❌ No | ❌ No |
| POST /spider/crawl | ✅ Automatic | ❌ No | ❌ No |
| POST /deepsearch | ✅ Automatic | ❌ No | ❌ No |
| POST /extract | ✅ Controlled | ✅ Yes | ✅ Yes |

---

## CLI Command Design: The Answer

### ❌ WRONG: Add Extraction Flags to spider/crawl

```bash
# DON'T do this - API doesn't support it!
riptide spider URL --extraction-strategy css  # ← Not supported by /spider/crawl!
riptide crawl URL --strategy wasm             # ← Not supported by /crawl!
```

**Why this is wrong:**
- API endpoints `/crawl` and `/spider/crawl` don't accept strategy parameter
- Would create confusion (CLI accepts flag but API ignores it)
- Violates thin-client principle (CLI shouldn't do what API can't)

---

### ✅ CORRECT: Separate Commands Match API Capabilities

```bash
# Simple workflows - automatic extraction (no control needed)
riptide spider https://docs.site.com --depth 5
  → POST /spider/crawl { seed_urls, max_depth: 5 }
  → Extraction happens automatically
  → No strategy control (by design)

riptide search "web scraping" --limit 20
  → POST /deepsearch { query, limit: 20 }
  → Extraction happens automatically
  → No strategy control (by design)

# Advanced extraction - full control
riptide extract https://example.com --strategy css --selector "article"
  → POST /extract { url, options: { strategy: "css" } }
  → User controls extraction strategy

riptide extract https://example.com --strategy llm --quality-threshold 0.9
  → POST /extract { url, options: { strategy: "llm", quality_threshold: 0.9 } }
  → User controls extraction + quality requirements
```

---

## Why This Design Makes Sense

### Use Case 1: "I want to deep crawl a documentation site"
```bash
riptide spider https://docs.rust-lang.org --depth 5 --max-pages 1000
```
**Expectation:** Just crawl and extract, I don't care HOW
**API Behavior:** Automatic extraction with sensible defaults
**User Experience:** ✅ Simple, no extraction knowledge required

---

### Use Case 2: "I need to extract specific content using CSS selectors"
```bash
riptide extract https://blog.example.com \
  --strategy css \
  --selector "article.post-content" \
  --quality-threshold 0.8
```
**Expectation:** Precise extraction control
**API Behavior:** Uses CSS strategy with specified selector
**User Experience:** ✅ Full control, advanced usage

---

### Use Case 3: "I want to extract but need high quality LLM results"
```bash
riptide extract https://research-paper.com \
  --strategy llm \
  --quality-threshold 0.95
```
**Expectation:** High-quality AI-powered extraction
**API Behavior:** Uses LLM strategy with quality filtering
**User Experience:** ✅ Advanced features when needed

---

## CLI Implementation Plan

### Command 1: `spider` - NO Extraction Flags

```rust
#[derive(Args)]
pub struct SpiderArgs {
    url: String,

    #[arg(long, default_value = "5")]
    depth: u32,

    #[arg(long, default_value = "100")]
    max_pages: u32,

    // NO extraction strategy flags!
    // API handles it automatically
}

pub async fn spider(args: SpiderArgs) -> Result<()> {
    let body = json!({
        "seed_urls": [args.url],
        "max_depth": args.depth,
        "max_pages": args.max_pages
    });

    // POST to /spider/crawl
    // Extraction happens automatically in API
    let response = client.post("/spider/crawl", body).await?;
    println!("{}", format_results(response));
}
```

---

### Command 2: `extract` - WITH Extraction Flags

```rust
#[derive(Args)]
pub struct ExtractArgs {
    url: String,

    /// Extraction strategy: auto, css, wasm, llm, multi
    #[arg(long, default_value = "multi")]
    strategy: String,

    /// CSS selector (for css strategy)
    #[arg(long)]
    selector: Option<String>,

    /// Minimum quality threshold (0.0-1.0)
    #[arg(long, default_value = "0.7")]
    quality_threshold: f64,

    /// Timeout in milliseconds
    #[arg(long, default_value = "30000")]
    timeout: u64,
}

pub async fn extract(args: ExtractArgs) -> Result<()> {
    let body = json!({
        "url": args.url,
        "mode": "standard",
        "options": {
            "strategy": args.strategy,
            "quality_threshold": args.quality_threshold,
            "timeout_ms": args.timeout
        }
    });

    // POST to /extract
    let response = client.post("/extract", body).await?;
    println!("{}", format_results(response));
}
```

---

## What About Hybrid Use Cases?

### Q: "I want to spider crawl BUT use a specific extraction strategy"

**Current API**: Not supported directly

**Workarounds:**

**Option A**: Two-step process (CLI orchestration)
```bash
# Step 1: Spider without extraction (get URLs)
riptide spider https://docs.site.com --depth 5 --no-extract > urls.txt

# Step 2: Extract each URL with your strategy
cat urls.txt | while read url; do
  riptide extract "$url" --strategy css --selector "article"
done
```

**Option B**: Future API enhancement (v1.1+)
```yaml
# Potential future feature for /spider/crawl
POST /spider/crawl
{
  "seed_urls": ["..."],
  "max_depth": 5,
  "extraction_options": {  ← NEW in v1.1
    "strategy": "css",
    "selector": "article"
  }
}
```

**For v1.0**: Don't add this to CLI (API doesn't support it yet)

---

## Summary: Architecture Decision

### API Layer (Current State)

```
Simple Endpoints (Automatic):
  /crawl              → Auto extraction, no control
  /spider/crawl       → Auto extraction, no control
  /deepsearch         → Auto extraction, no control

Advanced Endpoint (Manual Control):
  /extract            → User-controlled extraction
```

### CLI Layer (Recommended)

```
Simple Commands (Mirror API):
  riptide spider      → No extraction flags (API doesn't support)
  riptide search      → No extraction flags (API doesn't support)

Advanced Command (Mirror API):
  riptide extract     → Full extraction flags (strategy, quality, etc.)
```

---

## Decision Matrix

| Scenario | Command | Extraction Control | Why |
|----------|---------|-------------------|-----|
| Crawl docs site | `spider` | ❌ Automatic | User wants simplicity |
| Search web | `search` | ❌ Automatic | User wants results fast |
| Extract specific content | `extract` | ✅ Manual | User needs control |
| Extract with CSS | `extract` | ✅ Manual | User knows structure |
| Extract with LLM | `extract` | ✅ Manual | User wants AI quality |

---

## Final Answer to Your Question

### Q: Does it automatically figure out what extraction to use?

**Answer**:
- **YES** for `/crawl`, `/spider/crawl`, `/deepsearch` - automatic via gate analysis
- **NO** for `/extract` - user specifies strategy

### Q: Should spider/crawl have extraction flags?

**Answer**: **NO** - The API endpoints don't support it.

**Recommendation**: Keep commands separate:
- `spider` / `search` = Simple, automatic extraction
- `extract` = Advanced, controlled extraction

If users need advanced extraction during crawling, they can:
1. Use two-step workflow (spider to get URLs, then extract with control)
2. Wait for v1.1 API enhancement to add extraction options to spider endpoint

---

## Future Enhancement (v1.1+)

### Potential API Change

Add extraction options to spider/crawl endpoints:

```rust
// Enhanced CrawlOptions (v1.1)
pub struct CrawlOptions {
    // ... existing fields ...

    /// NEW: Extraction strategy override
    pub extraction_strategy: Option<String>,

    /// NEW: CSS selector for extraction
    pub css_selector: Option<String>,

    /// NEW: Quality threshold
    pub quality_threshold: Option<f64>,
}
```

**Then** CLI could add:
```bash
riptide spider URL --depth 5 --extraction-strategy css --selector "article"
```

**But for v1.0**: Don't add these flags - API doesn't support it yet.

---

## Conclusion

**For v1.0 CLI:**

1. ✅ `spider` - NO extraction flags (automatic only)
2. ✅ `extract` - WITH extraction flags (full control)
3. ❌ Don't add `--strategy` to spider (API doesn't support)
4. ✅ Keep separation of concerns:
   - Simple commands = automatic
   - Advanced command = manual control

**This matches the API capabilities exactly.**
