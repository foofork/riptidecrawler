# Riptide Extract Command Test Results

## Executive Summary

**Test Date:** 2025-10-16
**Riptide Binary:** `/workspaces/eventmesh/target/x86_64-unknown-linux-gnu/release/riptide`
**Riptide Version:** 1.0.0
**Total Tests:** 22
**Successful:** 5 (22.72%)
**Failed:** 17 (77.28%)

## Critical Finding

**WASM Module Compatibility Issue Detected:**
```
Error: type-checking export func `health-check`
Caused by:
    expected record field named extractor-version, found trek-version
```

All non-raw engine tests (`auto`, `wasm`, `headless`) are failing due to a WASM interface version mismatch between the Rust binary and the WASM extraction module.

## Performance Summary

### Successfully Working Engine: `raw`

- **Success Rate:** 100% (5/5 tests)
- **Average Extraction Time:** 437.6ms
- **Content Length Range:** 768 bytes - 581,423 bytes
- **Median Content Length:** 18,846 bytes

### Failed Engines

- **auto:** 0% success (0/12 tests)
- **wasm:** 0% success (0/7 tests)
- **headless:** 0% success (0/1 test)

## Detailed Test Results

### Test 1: example.com with raw engine (local) ✓
- **URL:** https://example.com
- **Engine:** raw
- **Status:** SUCCESS
- **Content Length:** 768 bytes
- **Extraction Time:** 555ms
- **Notes:** Clean HTML extraction with proper formatting

### Test 2: Wikipedia Rust page with raw engine (local) ✓
- **URL:** https://en.wikipedia.org/wiki/Rust_(programming_language)
- **Engine:** raw
- **Status:** SUCCESS
- **Content Length:** 581,423 bytes
- **Extraction Time:** 150ms
- **Notes:** Large page extraction, excellent performance

### Test 3: Hacker News with raw engine (local) ✓
- **URL:** https://news.ycombinator.com/
- **Engine:** raw
- **Status:** SUCCESS
- **Content Length:** 35,599 bytes
- **Extraction Time:** 763ms
- **Notes:** Successfully extracted complex HTML structure

### Test 4: Rust-lang.org with auto engine and metadata (local) ✗
- **URL:** https://www.rust-lang.org/
- **Engine:** auto
- **Status:** FAILED
- **Content Length:** 0 bytes
- **Extraction Time:** 1,186ms
- **Error:** WASM module version mismatch (extractor-version vs trek-version)

### Additional Tests: Rust-lang.org with raw engine ✓
- **URL:** https://www.rust-lang.org/
- **Engine:** raw
- **Status:** SUCCESS
- **Content Length:** 18,846 bytes
- **Extraction Time:** 130ms
- **Notes:** Raw extraction works perfectly

## Engine-Specific Performance

### Raw Engine (100% Success)
| URL | Content Size | Time | Notes |
|-----|--------------|------|-------|
| example.com | 768 B | 555ms | Simple HTML |
| example.com (repeat) | 768 B | 590ms | Consistent performance |
| Wikipedia Rust | 581 KB | 150ms | Large page, fast |
| Hacker News | 35 KB | 763ms | Complex structure |
| rust-lang.org | 18 KB | 130ms | Modern website |

**Raw Engine Statistics:**
- Min Time: 130ms
- Max Time: 763ms
- Avg Time: 437.6ms
- Very reliable for basic content extraction

### Auto Engine (0% Success)
All 12 tests failed with WASM version mismatch:
- Methods tested: auto, wasm, css, llm, regex
- Strategies tested: chain, parallel, fallback
- Average failure time: ~1,450ms
- All failures show same root cause

### WASM Engine (0% Success)
All 7 tests failed with same version mismatch:
- No successful extractions
- Average failure time: ~1,248ms
- Indicates core WASM module issue

### Headless Engine (0% Success)
Single test failed:
- Same WASM version mismatch
- Failure time: 1,606ms

## Method Testing Results

All method variations failed when using `auto` engine:
- `--method auto`: FAILED (1,880ms)
- `--method wasm`: FAILED (1,437ms)
- `--method css`: FAILED (1,458ms)
- `--method llm`: FAILED (1,469ms)
- `--method regex`: FAILED (1,565ms)

## Strategy Testing Results

All strategy compositions failed:
- `--strategy chain:wasm,css`: FAILED (1,655ms)
- `--strategy parallel:all`: FAILED (1,318ms)
- `--strategy fallback:wasm,css,regex`: FAILED (1,486ms)

## Sample Raw Output

```html
<!doctype html>
<html lang="en">
<head>
  <title>Example Domain</title>
  <meta name="viewport" content="width=device-width, initial-scale=1">
  <style>body{background:#eee;width:60vw;margin:15vh auto;font-family:system-ui,sans-serif}...</style>
</head>
<body>
  <div>
    <h1>Example Domain</h1>
    <p>This domain is for use in documentation examples without needing permission...</p>
    <p><a href="https://iana.org/domains/example">Learn more</a></p>
  </div>
</body>
</html>
```

## Recommendations

### Immediate Actions Required

1. **Fix WASM Module Version Mismatch**
   - Update WASM module interface to match expected `extractor-version` field
   - Currently shows `trek-version` instead of `extractor-version`
   - This is blocking all advanced extraction features

2. **Rebuild WASM Module**
   - Ensure consistency between Rust binary version and WASM module
   - Run build scripts to regenerate WASM with correct interface

3. **Update Health Check Function**
   - Fix the `health-check` export function signature
   - Ensure proper struct field naming

### Testing Recommendations

1. **WASM Module Validation**
   ```bash
   # Verify WASM module exports
   wasm-objdump -x <wasm-file> | grep health-check
   ```

2. **Interface Compatibility Test**
   ```bash
   # Test WASM loading directly
   riptide extract --url "https://example.com" --engine wasm --local -vv
   ```

3. **Regression Testing**
   - Once fixed, rerun full test suite
   - Verify all engines work correctly
   - Test with various strategies and methods

## Positive Findings

Despite the WASM issues, the `raw` engine demonstrates:
- ✓ 100% reliability
- ✓ Fast performance (130-763ms range)
- ✓ Handles various website sizes (768B - 581KB)
- ✓ Clean HTML extraction
- ✓ Consistent results across multiple runs

## Technical Details

### Test Environment
- **Platform:** Linux x86_64
- **Binary Path:** `/workspaces/eventmesh/target/x86_64-unknown-linux-gnu/release/riptide`
- **Test Script:** `/workspaces/eventmesh/eval/test_extract_corrected.sh`
- **Results CSV:** `/workspaces/eventmesh/eval/results/extract_command_tests.csv`

### URLs Tested
1. https://example.com - Simple reference site
2. https://en.wikipedia.org/wiki/Rust_(programming_language) - Large complex page
3. https://news.ycombinator.com/ - Dynamic content site
4. https://www.rust-lang.org/ - Modern framework site

### Available Engines
- `auto` - Automatic engine selection (BROKEN)
- `raw` - Pure HTTP fetch (WORKING)
- `wasm` - WASM-based extraction (BROKEN)
- `headless` - Browser-based extraction (BROKEN)

### Available Methods
- `auto`, `wasm`, `css`, `llm`, `regex` (ALL BROKEN when engine ≠ raw)

### Available Strategies
- `chain`, `parallel`, `fallback` (ALL BROKEN due to WASM issues)

## Conclusion

The `riptide extract` command has a **critical WASM module compatibility issue** that prevents all advanced extraction features from working. However, the `raw` engine provides reliable basic HTML extraction with good performance.

**Priority:** HIGH - Fix WASM module interface to restore full functionality.

**Immediate Workaround:** Use `--engine raw --local` for reliable content extraction until WASM module is updated.
