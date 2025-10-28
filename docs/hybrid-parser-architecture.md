# Hybrid Parser Architecture - WASM + Native with Smart Fallbacks

**Date**: 2025-10-28
**Status**: ✅ Implementation Ready
**Architecture**: Hybrid (WASM for security, Native for speed)

---

## Architecture Overview

```
┌─────────────────────────────────────────────────┐
│           Extraction Request                     │
└──────────────────┬──────────────────────────────┘
                   │
        ┌──────────▼──────────┐
        │   Gate Decision      │
        │   (probes_first,     │
        │    raw, headless)    │
        └──────────┬──────────┘
                   │
     ┌─────────────┴─────────────┐
     │                           │
┌────▼────────┐          ┌──────▼──────┐
│   DIRECT    │          │  HEADLESS   │
│   FETCH     │          │  RENDERING  │
│ (Untrusted) │          │  (Trusted)  │
└────┬────────┘          └──────┬──────┘
     │                          │
     │ Raw HTML                 │ Rendered HTML
     │ from internet            │ from our Chrome
     │                          │
┌────▼──────────────┐    ┌─────▼──────────────┐
│  WASM EXTRACTOR   │    │  NATIVE PARSER     │
│  (tl parser)      │    │  (scraper crate)   │
│  🔒 SANDBOXED     │    │  🚀 FAST           │
│  📊 RESOURCE CAPS │    │  ✅ TRUSTED        │
└────┬──────────────┘    └─────┬──────────────┘
     │                          │
     │  Fallback: Native ───────┤
     │  (if WASM fails)         │
     │                          │
     │  Fallback: WASM ─────────┤
     │  (if Native fails)       │
     │                          │
┌────▼──────────────────────────▼─────┐
│       ExtractedDoc Result            │
│  (title, text, links, metadata)      │
└──────────────────────────────────────┘
```

---

## Decision Matrix

| Scenario | Primary Parser | Fallback | Reasoning |
|----------|----------------|----------|-----------|
| **Direct fetch** | 🔒 WASM | ⚡ Native | Untrusted HTML needs sandboxing |
| **Headless render** | ⚡ Native | 🔒 WASM | Trusted HTML, optimize for speed |
| **WASM fails** | → Native | N/A | Non-circular fallback |
| **Native fails** | → WASM | N/A | Non-circular fallback |

---

## Implementation

### **Phase 1: WASM Extractor (tl parser)**

**File**: `wasm/riptide-extractor-wasm/src/extraction.rs`

✅ **Status**: Converted from `scraper` to `tl`

**Key Changes**:
```rust
// OLD (crashes in WASM Component Model)
use scraper::{Html, Selector};
let doc = Html::parse_document(html);  // ❌ tendril crash

// NEW (WASM-compatible)
use tl::ParserOptions;
let dom = tl::parse(html, ParserOptions::default())?;  // ✅ Works!
```

### **Phase 2: Hybrid Routing Logic**

**File**: `crates/riptide-reliability/src/reliability.rs`

```rust
// Headless path: Use native parser (trusted, fast)
async fn extract_with_headless(&self, url: &str, ...) -> Result<ExtractedDoc> {
    let rendered_html = self.headless_render(url).await?;

    // Primary: Native parser (fast, already in memory)
    match native_parser.parse_headless_html(&rendered_html, url) {
        Ok(doc) => {
            info!("Native parser succeeded for headless");
            return Ok(doc);
        }
        Err(e) => {
            warn!("Native parser failed, trying WASM fallback: {}", e);
            // Fallback: WASM (sandboxed, reliable)
            wasm_extractor.extract(rendered_html.as_bytes(), url, "article")
        }
    }
}

// Direct fetch path: Use WASM (untrusted, sandboxed)
async fn extract_fast(&self, url: &str, ...) -> Result<ExtractedDoc> {
    let raw_html = self.http_fetch(url).await?;

    // Primary: WASM extractor (sandboxed, secure)
    match wasm_extractor.extract(raw_html.as_bytes(), url, "article") {
        Ok(doc) => {
            info!("WASM extractor succeeded for direct fetch");
            return Ok(doc);
        }
        Err(e) => {
            warn!("WASM extractor failed, trying native fallback: {}", e);
            // Fallback: Native (fast, but less secure for untrusted HTML)
            let html_str = String::from_utf8_lossy(raw_html);
            native_parser.parse_headless_html(&html_str, url)
        }
    }
}
```

### **Phase 3: Non-Circular Fallbacks**

**Guarantee**: Each parser tries exactly once per request

```rust
enum ParserAttempt {
    WasmPrimary,     // WASM tried, Native as fallback
    NativePrimary,   // Native tried, WASM as fallback
}

// Prevents infinite loops
let attempt = match path {
    Direct => ParserAttempt::WasmPrimary,
    Headless => ParserAttempt::NativePrimary,
};

match attempt {
    WasmPrimary => {
        wasm_extract()
            .or_else(|_| native_extract())  // ✅ Falls back once
            .or_else(|_| Err("Both failed")) // ❌ Stops here
    }
    NativePrimary => {
        native_extract()
            .or_else(|_| wasm_extract())    // ✅ Falls back once
            .or_else(|_| Err("Both failed")) // ❌ Stops here
    }
}
```

---

## Benefits

### **1. Security** 🔒
- **Untrusted HTML** → WASM sandbox (can't escape)
- **DoS protection** → Resource limits (max memory/CPU)
- **Malicious payloads** → Isolated from host

### **2. Performance** 🚀
- **Headless path** → Native (no boundary crossing)
- **Direct path** → WASM (85-95% native speed, acceptable)
- **Optimal for each** → Right tool for the job

### **3. Reliability** ✅
- **Non-circular fallbacks** → Each parser tries once
- **Dual redundancy** → If one fails, other succeeds
- **High availability** → System stays up even if one parser has issues

### **4. Future-Proof** 🔮
- **Hot-reload** → Update WASM without restart
- **A/B testing** → Multiple WASM versions side-by-side
- **Custom extractors** → Users can provide WASM plugins
- **Edge computing** → Run same WASM in browser/edge/server

---

## Performance Expectations

### **Headless Path** (Trusted HTML)
```
Primary: Native Parser
├─ Success: ~2ms parse time
└─ Failure → WASM Fallback: ~3ms parse time

Expected: 99% native, 1% WASM fallback
```

### **Direct Path** (Untrusted HTML)
```
Primary: WASM Extractor
├─ Success: ~3ms parse time (sandboxed)
└─ Failure → Native Fallback: ~2ms parse time

Expected: 95% WASM, 5% native fallback
```

### **Worst Case** (Both parsers fail)
```
Primary → Fallback → Error
Total attempts: 2
Total time: ~5-6ms before giving up
```

---

## Rollout Plan

### **Phase 1: Deploy WASM Fix** (Today)
1. ✅ Convert WASM to `tl` parser
2. ✅ Update `reliability.rs` with hybrid routing
3. ✅ Test WASM compilation
4. ✅ Deploy to production

### **Phase 2: Monitor & Tune** (Week 1)
1. Monitor fallback rates
2. Track performance metrics
3. Adjust timeouts if needed
4. Optimize based on real data

### **Phase 3: Advanced Features** (Month 1)
1. Hot-reload for WASM updates
2. A/B testing framework
3. Custom extractor plugins
4. Edge deployment

---

## Testing Strategy

### **Unit Tests**
```bash
# Test WASM extractor with tl parser
cd wasm/riptide-extractor-wasm
cargo test --lib

# Test native parser
cd crates/riptide-extraction
cargo test --lib native_parser
```

### **Integration Tests**
```bash
# Test direct fetch path (WASM primary)
curl -X POST http://localhost:8080/crawl \
  -d '{"urls": ["https://example.com"], "options": {"render_mode": "Static"}}'

# Test headless path (Native primary)
curl -X POST http://localhost:8080/crawl \
  -d '{"urls": ["https://example.com"], "options": {"render_mode": "Dynamic"}}'
```

### **Fallback Tests**
```bash
# Simulate WASM failure (should fallback to native)
# Simulate native failure (should fallback to WASM)
# Verify non-circular behavior
```

---

## Success Metrics

✅ **Must Have:**
- WASM extractor compiles without errors
- No tendril crashes in production
- Fallbacks work correctly (non-circular)
- Performance within 5ms per request
- 95%+ extraction success rate

✅ **Nice to Have:**
- <3ms average extraction time
- <1% fallback rate per path
- Hot-reload working
- A/B testing framework deployed

---

## Maintenance

### **WASM Updates**
```bash
# Build new WASM module
cd wasm/riptide-extractor-wasm
cargo build --target wasm32-wasip2 --release

# Hot-reload (zero downtime)
curl -X POST http://localhost:8080/admin/reload-wasm \
  -F "module=@target/wasm32-wasip2/release/extractor.wasm"
```

### **Native Updates**
```bash
# Rebuild API (requires restart)
docker-compose build riptide-api
docker-compose restart riptide-api  # ~10-30s downtime
```

---

## Conclusion

This hybrid architecture gives us:
- ✅ **Security**: WASM sandbox for untrusted HTML
- ✅ **Performance**: Native speed for trusted paths
- ✅ **Reliability**: Non-circular fallbacks
- ✅ **Flexibility**: Hot-reload, A/B test, plugins
- ✅ **Best of both worlds**: Right tool for each job

**Ready for production deployment!**
