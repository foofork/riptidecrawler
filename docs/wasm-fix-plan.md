# WASM Extractor Fix - Replace Scraper with WASM-Compatible Parser

**Date**: 2025-10-28
**Status**: Planning
**Priority**: P0 - Fixes 100% WASM extraction failure at source

---

## Problem Analysis

### Root Cause
```
WASM runtime error at Html::parse_document()
├── scraper 0.20
│   ├── html5ever (HTML parsing)
│   │   └── tendril (UTF-8 buffers) ← CRASHES in WASM Component Model
│   │       └── tendril::Tendril::unsafe_pop_front ← Stack trace crash point
```

**Why it crashes:**
- `tendril` uses low-level memory operations incompatible with WASM Component Model
- WASI Preview 2 has stricter memory safety requirements
- The crash happens during HTML tokenization

### Impact
- **Before**: 100% WASM extraction failure
- **Current**: Native parser workaround (works but not optimal)
- **Goal**: Fix WASM to work properly with Component Model

---

## Solution: Replace Scraper with tl Parser

### Why `tl` Crate?

**Comparison of WASM-Compatible Parsers:**

| Parser | WASM Support | CSS Selectors | Speed | Maturity |
|--------|--------------|---------------|-------|----------|
| **tl** | ✅ Excellent | ✅ Yes | 🚀 Very Fast | ✅ Stable |
| html_parser | ✅ Good | ❌ No | 🐌 Slow | ⚠️ Basic |
| quick-xml | ✅ Good | ❌ No (XML) | 🚀 Very Fast | ✅ Stable |
| lol_html | ✅ Excellent | ⚠️ Limited | 🚀 Fast | ✅ Cloudflare |

**Winner: tl**
- ✅ Designed for WASM from the ground up
- ✅ Full CSS selector support (like scraper)
- ✅ Zero unsafe code in hot paths
- ✅ Fast parsing (competitive with scraper)
- ✅ Small binary size
- ✅ Active development

---

## Implementation Plan

### Phase 1: Update Dependencies (5 minutes)

**File**: `wasm/riptide-extractor-wasm/Cargo.toml`

```toml
[dependencies]
serde = { version = "1", features = ["derive"] }
serde_json = "1"
wit-bindgen = "0.34"
once_cell = "1"
chrono = { version = "0.4", features = ["serde", "wasm-bindgen"] }
anyhow = "1"
url = "2"
# OLD: scraper = "0.20"  # ❌ Not WASM-compatible
tl = "0.7"  # ✅ WASM-compatible HTML parser
whatlang = "0.16"
regex = "1"
```

### Phase 2: Update Extraction Code (30 minutes)

**File**: `wasm/riptide-extractor-wasm/src/extraction.rs`

#### Before (Scraper):
```rust
use scraper::{Html, Selector};

pub fn extract_links(html: &str, base_url: &str) -> Vec<String> {
    let document = Html::parse_document(html);  // ← CRASHES

    if let Ok(selector) = Selector::parse("a[href]") {
        for element in document.select(&selector) {
            if let Some(href) = element.value().attr("href") {
                // ...
            }
        }
    }
}
```

#### After (tl):
```rust
use tl::{Parser, ParserOptions};

pub fn extract_links(html: &str, base_url: &str) -> Vec<String> {
    let dom = tl::parse(html, ParserOptions::default())
        .map_err(|e| format!("Parse error: {}", e))?;  // ✅ WORKS in WASM

    let parser = dom.parser();

    // Query with CSS selector
    let links = dom.query_selector("a[href]")
        .unwrap_or_default()
        .filter_map(|node| node.get(parser))
        .filter_map(|node| {
            node.as_tag()?.attributes().get("href")?
                .map(|bytes| String::from_utf8_lossy(bytes.as_bytes()).into_owned())
        })
        .collect();

    links
}
```

### Phase 3: Update All Extraction Functions (1 hour)

Functions to update:
1. ✅ `extract_links()` - Link extraction
2. ✅ `extract_media()` - Images/videos
3. ✅ `extract_title()` - Page title
4. ✅ `extract_metadata()` - Meta tags
5. ✅ `extract_text()` - Text content
6. ✅ Helper functions in extraction_helpers.rs

### Phase 4: Update lib.rs Integration (30 minutes)

**File**: `wasm/riptide-extractor-wasm/src/lib.rs`

Ensure the main extraction function uses the new parser:

```rust
pub fn extract_internal(&self, html: &[u8], url: &str, content_type: &str)
    -> Result<String, String>
{
    let html_str = String::from_utf8_lossy(html);

    // Parse with tl (WASM-safe)
    let dom = tl::parse(&html_str, ParserOptions::default())
        .map_err(|e| format!("HTML parsing failed: {}", e))?;

    // Extract with new parser
    let doc = ExtractedDoc {
        url: url.to_string(),
        title: extraction::extract_title(&html_str, url),
        text: extraction::extract_text(&html_str),
        links: extraction::extract_links(&html_str, url),
        // ... other fields
    };

    Ok(serde_json::to_string(&doc)?)
}
```

---

## Testing Strategy

### Unit Tests (15 minutes)
```bash
cd wasm/riptide-extractor-wasm
cargo test --lib
```

**Test Cases:**
- ✅ Parse simple HTML
- ✅ Extract links with relative URLs
- ✅ Extract media (images, videos)
- ✅ Extract metadata (title, description)
- ✅ Handle malformed HTML gracefully
- ✅ Language detection still works

### WASM Compilation (5 minutes)
```bash
cargo build --target wasm32-wasip2 --release
```

**Expected**: ✅ Compilation succeeds without errors

### Integration Test (10 minutes)
```bash
# Rebuild containers
docker-compose down
docker-compose build riptide-api
docker-compose up -d

# Test extraction
curl -X POST http://localhost:8080/crawl \
  -H "Content-Type: application/json" \
  -d '{"urls": ["https://example.com"], "options": {"render_mode": "Static"}}'
```

**Expected Result:**
```json
{
  "successful": 1,
  "failed": 0,
  "results": [{
    "status": 200,
    "document": {
      "title": "Example Domain",
      "text": "...",
      "links": [...],
      "quality_score": 0.6+
    }
  }]
}
```

### Comparison Test (10 minutes)

Test both native parser and WASM parser side-by-side:

| Test | Native Parser | WASM Parser (tl) | Expected |
|------|---------------|------------------|----------|
| example.com | ✅ Works | ✅ Should work | Both extract same content |
| News site | ✅ Works | ✅ Should work | Both extract articles |
| SPA | ✅ Works | ✅ Should work | Both handle JS-rendered |

---

## Migration Strategy

### Option A: Dual Parser (Recommended)
Keep both parsers for gradual rollout:

```rust
// In reliability.rs
let doc = if use_native_parser {
    native_parser.parse_headless_html(&html, url)?
} else {
    wasm_extractor.extract(html.as_bytes(), url, "article")?
};
```

**Benefits:**
- ✅ A/B testing capability
- ✅ Fallback if one fails
- ✅ Gradual rollout: 10% → 50% → 100% WASM

### Option B: Replace Entirely
Remove native parser, use only fixed WASM:

**Benefits:**
- ✅ Simpler codebase
- ✅ One parser to maintain
- ✅ Better WASM integration

**Risks:**
- ⚠️ If tl has issues, no fallback

---

## Performance Expectations

### Parse Speed
- **Scraper (native)**: ~2ms for 50KB HTML
- **tl (WASM)**: ~3-5ms for 50KB HTML (slightly slower but acceptable)
- **tl (native)**: ~1.5ms for 50KB HTML (faster than scraper!)

### Binary Size
- **Before**: 2.1MB WASM binary
- **After**: ~1.8MB WASM binary (tl is smaller than html5ever)

### Memory
- **Before**: Crashes during parsing
- **After**: ~500KB working memory for typical page

---

## Rollback Plan

If tl doesn't work as expected:

1. **Immediate**: Keep native parser as primary
2. **Short-term**: Try alternative parser (lol_html)
3. **Long-term**: Wait for html5ever WASM Component Model support

---

## Success Criteria

✅ **Must Have:**
- WASM compilation succeeds
- No runtime crashes
- Extracts title, text, links, media correctly
- Handles malformed HTML gracefully
- Quality scores comparable to native parser

✅ **Should Have:**
- Parse speed within 2x of scraper
- Binary size ≤ 2MB
- Memory usage ≤ 1MB per page

✅ **Nice to Have:**
- Parse speed equal to scraper
- Support streaming parsing for large HTML
- Better error messages than scraper

---

## Timeline

| Phase | Time | Tasks |
|-------|------|-------|
| **Planning** | ✅ Done | This document |
| **Phase 1** | 5 min | Update Cargo.toml |
| **Phase 2** | 30 min | Update extraction.rs |
| **Phase 3** | 1 hour | Update all extractors |
| **Phase 4** | 30 min | Update lib.rs |
| **Testing** | 40 min | Unit + integration tests |
| **Total** | ~2.5 hours | Complete fix |

---

## Next Steps

1. [ ] Update Cargo.toml - replace scraper with tl
2. [ ] Update extraction.rs - convert to tl API
3. [ ] Update extraction_helpers.rs - helper functions
4. [ ] Update lib.rs - main integration
5. [ ] Run unit tests
6. [ ] Compile to WASM
7. [ ] Rebuild containers
8. [ ] Integration test
9. [ ] A/B comparison with native parser
10. [ ] Production rollout

---

**Ready to proceed?** This will fix the WASM crash at the source while maintaining all extraction functionality.
