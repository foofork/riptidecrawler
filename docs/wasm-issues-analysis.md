# WASM Extractor Issues - Root Cause Analysis

**Date**: 2025-10-28
**Status**: Critical issues identified, solutions proposed

---

## 🔍 Issues Identified

### 1. **CRITICAL: WASM HTML Parser Crash (tendril/html5ever)**

**Location**: `/wasm/riptide-extractor-wasm/src/extraction.rs:9,74`

**Root Cause**:
```rust
// This line crashes in WASM environment
let document = Html::parse_document(html);
```

**Stack Trace**:
```
WASM runtime error: error while executing at wasm backtrace:
  0: 0x205b18 - tendril::Tendril<F,A>::unsafe_pop_front
  1: 0x20506e - tendril::Tendril<F,A>::pop_front_char
  2: 0x204e2c - <markup5ever::util::buffer_queue::BufferQueue as Iterator>::next
  3: 0x1c3a01 - html5ever::tokenizer::Tokenizer<Sink>::get_char
  4: 0x1bf43d - html5ever::tokenizer::Tokenizer<Sink>::step
  5: 0x1bdc0b - html5ever::tokenizer::Tokenizer<Sink>::run
  6: 0x1d4c68 - tendril::stream::TendrilSink::one
  7: 0x1ddbda - scraper::html::Html::parse_document
```

**Impact**:
- **100% extraction failure rate**
- WASM extraction fails
- Headless rendering works (POST fix successful), but WASM extraction of rendered HTML also fails
- All fallback methods fail because they all depend on WASM parser

**Technical Details**:
- `scraper 0.20` uses `html5ever` for HTML parsing
- `html5ever` uses `tendril` for UTF-8 buffer management
- `tendril` memory operations fail in WASM Component Model
- Issue likely related to WASM memory growth or UTF-8 validation

**Dependencies** (from Cargo.toml):
```toml
scraper = "0.20"  # Uses html5ever internally
```

---

### 2. **Binaryen WASM Optimization Failure**

**Location**: `infra/docker/Dockerfile.api:231`

**Issue**:
```
[parse exception: this looks like a wasm component, which Binaryen does not support yet]
Fatal: error parsing wasm
```

**Root Cause**:
- WASM extractor uses **WASM Component Model** (new standard)
- `binaryen/wasm-opt` only supports **classic WASM modules**
- Component Model support tracked: https://github.com/WebAssembly/binaryen/issues/6728

**Current Handling**:
```dockerfile
RUN wasm-opt -Oz target/wasm32-wasip2/ci/riptide_extractor_wasm.wasm \
    -o target/wasm32-wasip2/ci/riptide_extractor_wasm.optimized.wasm || \
    cp target/wasm32-wasip2/ci/riptide_extractor_wasm.wasm \
       target/wasm32-wasip2/ci/riptide_extractor_wasm.optimized.wasm
```

**Impact**:
- **Non-critical** - gracefully handled with fallback
- Slightly larger WASM file (~5-10% bigger unoptimized)
- No functional impact

**Solutions**:
1. ✅ **Current**: Graceful fallback (already implemented)
2. **Alternative**: Use `wasm-tools` for Component optimization
3. **Future**: Wait for binaryen Component Model support

---

### 3. **swagger-ui in Default Docker Compose**

**Location**: `docker-compose.yml:225-242`

**Issue**:
- Swagger UI included by default in docker-compose
- Adds 132MB image + container overhead
- Optional documentation tool, not core functionality

**Current State**:
```yaml
swagger-ui:
  image: swaggerapi/swagger-ui:latest
  container_name: riptide-swagger-ui
  restart: unless-stopped
  ports:
    - "${SWAGGER_PORT:-8081}:8080"
```

**Impact**:
- **Minor** - unnecessary resource usage
- Most users don't need live API docs
- Should be opt-in, not default

**Solution**: Comment out or move to separate compose file

---

### 4. **spider_engine Causing "Degraded" Health Status**

**Location**: `crates/riptide-api/src/state.rs:648-690`

**Issue**:
```json
{
  "status": "degraded",
  "dependencies": {
    "spider_engine": null  // ← Causes degraded status
  }
}
```

**Root Cause**:
- `spider_engine` is **OPTIONAL** - only initialized if `spider_config` is provided
- Health check treats `null` as unhealthy
- Actually NOT an error - it's just not configured

**Technical Details**:
```rust
// Only initialized if config present
let spider = if let Some(ref spider_config) = config.spider_config {
    // Initialize spider...
} else {
    None  // ← This is NORMAL, not an error
};
```

**What is spider_engine?**
- Advanced deep-crawling engine (riptide-spider crate)
- Memory-efficient crawling with intelligent scheduling
- **NOT required** - headless Chrome provides all needed functionality

**Solutions**:
1. ✅ **Simple**: Treat `null` spider_engine as "OK" in health check
2. **Alternative**: Remove from health dependencies entirely
3. **Documentation**: Clarify it's optional in health response

---

## 🔬 WASM Crash Investigation

### Memory Layout Issues

The crash happens in `tendril` buffer operations, specifically:
1. `unsafe_pop_front` - removing UTF-8 bytes from front of buffer
2. `pop_front_char` - extracting Unicode character
3. Buffer queue operations in html5ever

**Hypothesis**: WASM memory growth or UTF-8 validation fails

### WASM Environment Details

From Cargo.toml:
```toml
wit-bindgen = "0.34"  # Component Model bindings
wasmtime = { version = "34", features = ["component-model"] }
```

WASM memory limits (from logs):
```
Current: 0 bytes (0 pages)
Desired: 2359296 bytes (36 pages)
Maximum: Some(4294967296)
Our limit: 8192 pages (512 MB)
ALLOWED: Within limit
```

Memory growth IS allowed, so this isn't a limit issue.

---

## 💡 Proposed Solutions

### Option 1: Replace scraper with Alternative Parser

**Replace**:
```rust
use scraper::{Html, Selector};  // Uses html5ever
```

**With**:
```rust
use html_parser::{Dom, Node};  // Pure Rust, simpler
// OR
use html2text;  // Text extraction without full parsing
// OR
use quick-xml;  // Faster, more WASM-friendly
```

**Pros**:
- Avoid tendril/html5ever complexity
- Better WASM compatibility
- Potentially faster

**Cons**:
- May need to rewrite extraction logic
- Different API
- Testing needed

---

### Option 2: Fix tendril in WASM

**Investigate**:
```bash
# Enable detailed backtraces
WASMTIME_BACKTRACE_DETAILS=1

# Check tendril version
# Current: latest from scraper dependency
```

**Potential Fixes**:
- Update scraper/html5ever/tendril versions
- Apply WASM-specific patches
- Use alternative tendril features

---

### Option 3: Use Native Parser After Headless

**Current Flow**:
```
Headless (POST) → Rendered HTML → WASM Parser (crashes)
```

**Proposed Flow**:
```
Headless (POST) → Rendered HTML → Native Rust Parser (in API)
```

**Implementation**:
```rust
// In reliability.rs after headless render
// Instead of:
let doc = wasm_extractor.extract(rendered_html.as_bytes(), url, "article")?;

// Use native parser:
let doc = native_html_parser::parse(&rendered_html, url)?;
```

**Pros**:
- Bypass WASM entirely for headless-rendered content
- Native Rust parser is faster and more reliable
- WASM only needed for direct extraction (which we can disable)

**Cons**:
- Need to implement native parser
- Duplicate parsing logic

---

### Option 4: Return Raw HTML (Short-term Workaround)

**Add option to skip extraction**:
```json
{
  "urls": ["https://example.com"],
  "skip_extraction": true,
  "render_mode": "headless"
}
```

**Returns**:
```json
{
  "success": true,
  "content": {
    "html": "<!DOCTYPE html>...",  // Raw rendered HTML
    "text": "",  // Empty - not extracted
    "links": []  // Empty - not extracted
  }
}
```

**Use Case**: Users can extract on their end with their own tools

---

## 🎯 Recommended Immediate Actions

### 1. **Quick Wins** (Can implement now)

- [x] ~~Fix headless POST endpoint~~ (DONE)
- [ ] Comment out swagger-ui in docker-compose.yml
- [ ] Fix health check to treat spider_engine=null as OK
- [ ] Document binaryen limitation (already handled)
- [ ] Add `skip_extraction` option for raw HTML returns

### 2. **Medium-term** (This week)

- [ ] Implement native HTML parser in API for headless-rendered content
- [ ] Add detailed WASM debugging with WASMTIME_BACKTRACE_DETAILS=1
- [ ] Test alternative parsers (html_parser, quick-xml)
- [ ] Create configuration option to disable WASM extraction

### 3. **Long-term** (Next sprint)

- [ ] Replace scraper with WASM-friendly parser
- [ ] Comprehensive WASM testing suite
- [ ] Memory profiling of WASM operations
- [ ] Consider native-only extraction mode

---

## 📊 Current System Status

### ✅ Working
- Docker health checks (healthcheck script, curl, endpoints)
- Redis connectivity
- HTTP client
- Headless service (POST fix successful)
- Container orchestration

### ❌ Broken
- WASM HTML parsing (100% failure rate)
- All extraction methods (all depend on WASM)

### ⚠️ Degraded
- Overall health status (due to optional spider_engine=null)

---

## 🔗 Related Issues

1. **RUSTSEC-2025-0046**: wasmtime security update (already updated to v34)
2. **WebAssembly/binaryen#6728**: Component Model support in binaryen
3. **tendril memory safety**: Potential WASM memory management issue

---

## 📝 Testing Checklist

Once fixed, test:
- [ ] Simple HTML (example.com)
- [ ] Complex HTML (news sites)
- [ ] JavaScript-heavy (SPAs)
- [ ] Large HTML (>1MB)
- [ ] International characters (UTF-8)
- [ ] Malformed HTML
- [ ] Batch requests
- [ ] Performance benchmarks

---

**Next Steps**: Implement Option 3 (native parser after headless) as it provides immediate relief while preserving all functionality.
