# Extraction Strategies Guide (v0.9.0+)

## Overview

The RipTide Python SDK supports multiple extraction strategies for content extraction. In **v0.9.0**, the default strategy changed from `"multi"` to `"native"` for better performance.

## Available Strategies

### 1. Native Strategy (Default, Recommended)

**Pure Rust extraction using the `scraper` crate.**

```python
from riptide_sdk import RipTideClient
from riptide_sdk.models import ExtractOptions

async with RipTideClient() as client:
    # Default (native strategy)
    result = await client.extract.extract("https://example.com")

    # Explicit native
    options = ExtractOptions(strategy="native")
    result = await client.extract.extract("https://example.com", options=options)
```

**Characteristics:**
- ✅ **Speed**: 2-5ms per page
- ✅ **Availability**: Always available (default build)
- ✅ **Server requirement**: None (works out of the box)
- ✅ **Use case**: 99% of applications
- ❌ **Sandboxing**: No (processes HTML in main process)

### 2. WASM Strategy (Specialized)

**WASM-based extraction with sandboxing.**

```python
async with RipTideClient() as client:
    try:
        options = ExtractOptions(strategy="wasm")
        result = await client.extract.extract("https://example.com", options=options)
    except ExtractionError as e:
        print(f"WASM not available: {e}")
```

**Characteristics:**
- ⚠️ **Speed**: 10-20ms per page (4x slower than native)
- ⚠️ **Availability**: Only if server built with `--features wasm-extractor`
- ⚠️ **Server requirement**: Must have `WASM_EXTRACTOR_PATH` configured
- ✅ **Sandboxing**: Yes (isolated WASM execution)
- ✅ **Use case**: Untrusted HTML, strict resource limits

### 3. Multi Strategy (Auto-selection)

**Server automatically selects the best available strategy.**

```python
async with RipTideClient() as client:
    options = ExtractOptions(strategy="multi")
    result = await client.extract.extract("https://example.com", options=options)

    print(f"Server selected: {result.strategy_used}")
```

**Characteristics:**
- ⚙️ **Speed**: Varies (depends on server selection)
- ✅ **Availability**: Always
- ⚙️ **Server requirement**: Uses best available (native if WASM unavailable)
- ✅ **Use case**: When you don't care which strategy is used

## Server Compatibility Matrix

| Server Build | `strategy="native"` | `strategy="wasm"` | `strategy="multi"` |
|--------------|---------------------|-------------------|-------------------|
| **Native-only** (default)<br>`cargo build --release` | ✅ Works | ❌ Error | ✅ Falls back to native |
| **WASM-enabled**<br>`cargo build --release --features wasm-extractor` | ✅ Works | ✅ Works | ✅ Prefers WASM, falls back to native |

## Migration from v0.8.x to v0.9.0+

### What Changed

**Default Strategy Change:**
- **Before v0.9.0**: Default was `"multi"` (used WASM if available)
- **After v0.9.0**: Default is `"native"` (4x faster)

### Migration Examples

**No changes needed (automatic performance improvement):**
```python
# This code works in both v0.8.x and v0.9.0+
result = await client.extract.extract(url)

# v0.8.x: Might use WASM (slower)
# v0.9.0+: Uses native (faster)
```

**To restore old behavior (use WASM if available):**
```python
# Explicitly request WASM
options = ExtractOptions(strategy="wasm")
result = await client.extract.extract(url, options=options)
```

**To use auto-selection:**
```python
# Let server decide
options = ExtractOptions(strategy="multi")
result = await client.extract.extract(url, options=options)
```

## Strategy Selection Guide

### Decision Tree

```
Is performance critical?
├─ YES → Use "native" (2-5ms, default)
└─ NO
   └─ Is HTML from untrusted sources?
      ├─ YES → Use "wasm" (sandboxed, 10-20ms)
      └─ NO → Use "native" (default)

Don't care? → Use "multi" (server auto-selects)
```

### Use Case Examples

| Use Case | Recommended Strategy | Why |
|----------|---------------------|-----|
| Web scraping public sites | `native` | Fast, trusted sources |
| Internal content processing | `native` | Fast, controlled environment |
| User-submitted URLs | `wasm` | Sandboxing for untrusted HTML |
| Performance-critical apps | `native` | 4x faster than WASM |
| Plugin/extension system | `wasm` | Memory isolation |
| Don't care/unknown | `multi` | Server auto-selects best |

## Error Handling

### Strategy Not Available

```python
from riptide_sdk.exceptions import ExtractionError

async def safe_extract(client, url, preferred_strategy="wasm"):
    """Extract with fallback to native"""
    try:
        options = ExtractOptions(strategy=preferred_strategy)
        return await client.extract.extract(url, options=options)
    except ExtractionError as e:
        if "not available" in str(e).lower():
            print(f"⚠️  {preferred_strategy} unavailable, falling back to native")
            options = ExtractOptions(strategy="native")
            return await client.extract.extract(url, options=options)
        raise
```

### Graceful Fallback Pattern

```python
async def extract_with_fallback(client, url):
    """Try strategies in order of preference"""
    strategies = ["wasm", "native"]

    for strategy in strategies:
        try:
            options = ExtractOptions(strategy=strategy)
            result = await client.extract.extract(url, options=options)
            print(f"✅ Success with {strategy}")
            return result
        except ExtractionError as e:
            if "not available" in str(e).lower():
                print(f"⚠️  {strategy} unavailable, trying next...")
                continue
            raise

    raise RuntimeError("All strategies failed")
```

### Server Compatibility Check

```python
async def check_server_capabilities(client):
    """Check which strategies the server supports"""
    strategies = ["native", "wasm", "multi"]
    available = []

    for strategy in strategies:
        try:
            options = ExtractOptions(strategy=strategy)
            result = await client.extract.extract(
                "https://example.com",
                options=options
            )
            available.append(strategy)
            print(f"✅ {strategy}: Available (used: {result.strategy_used})")
        except ExtractionError as e:
            if "not available" in str(e).lower():
                print(f"❌ {strategy}: Not available")

    return available
```

## Performance Comparison

### Benchmarks

Based on typical HTML extraction (1000 pages):

| Strategy | Avg Time | Total Time | Use Case |
|----------|----------|------------|----------|
| `native` | 3ms | 3 seconds | General scraping ✅ |
| `wasm` | 12ms | 12 seconds | Untrusted HTML |
| `multi` | Varies | Depends | Auto-selection |

**Performance impact of WASM:**
- 4x slower than native (12ms vs 3ms)
- Memory overhead: ~50MB for WASM runtime
- Binary size: +50MB when WASM enabled

### When to Accept the Performance Cost

Use WASM despite the performance cost when:
1. Processing completely untrusted HTML from unknown sources
2. Strict memory/resource limits required (WASM isolation)
3. Sandboxing is a security requirement
4. Running in a plugin/extension architecture

## Checking Strategy Usage

### Inspect Strategy Used

```python
result = await client.extract.extract(url)

print(f"Strategy used: {result.strategy_used}")
# Output: "native" (in v0.9.0+ by default)

print(f"Extraction time: {result.extraction_time_ms}ms")
# Output: ~2-5ms for native, ~10-20ms for WASM
```

### Strategy vs Result

```python
# What you requested vs what was actually used
options = ExtractOptions(strategy="multi")
result = await client.extract.extract(url, options=options)

print(f"Requested: multi")
print(f"Actually used: {result.strategy_used}")
# Output: "native" or "wasm" depending on server
```

## Enabling WASM on Server

If you need WASM extraction, the server must be configured:

### 1. Build with WASM Feature

```bash
# Build server with WASM support
cargo build --release --features wasm-extractor

# Build WASM component
cd wasm/riptide-extractor-wasm
cargo build --release --target wasm32-wasip2
```

### 2. Configure Environment

```bash
export WASM_EXTRACTOR_PATH=/path/to/riptide_extractor_wasm.wasm
./target/release/riptide-api
```

### 3. Verify WASM Availability

```python
async with RipTideClient() as client:
    try:
        options = ExtractOptions(strategy="wasm")
        result = await client.extract.extract("https://example.com", options=options)
        print("✅ WASM is available")
    except ExtractionError as e:
        print(f"❌ WASM not available: {e}")
```

## Best Practices

### 1. Use Default (Native) Unless You Need WASM

```python
# ✅ Good: Use default
result = await client.extract.extract(url)

# ❌ Unnecessary: Explicitly requesting native
options = ExtractOptions(strategy="native")
result = await client.extract.extract(url, options=options)
```

### 2. Handle WASM Unavailability Gracefully

```python
# ✅ Good: Graceful fallback
try:
    options = ExtractOptions(strategy="wasm")
    result = await client.extract.extract(url, options=options)
except ExtractionError:
    options = ExtractOptions(strategy="native")
    result = await client.extract.extract(url, options=options)

# ❌ Bad: No error handling
options = ExtractOptions(strategy="wasm")
result = await client.extract.extract(url, options=options)  # Might fail!
```

### 3. Check strategy_used for Observability

```python
# ✅ Good: Log which strategy was used
result = await client.extract.extract(url)
logger.info(f"Extracted {url} using {result.strategy_used} in {result.extraction_time_ms}ms")
```

### 4. Use Multi-Strategy When Uncertain

```python
# ✅ Good: Let server decide
options = ExtractOptions(strategy="multi")
result = await client.extract.extract(url, options=options)
print(f"Server selected: {result.strategy_used}")
```

## FAQs

### Q: Will my code break if I upgrade to v0.9.0?

**A:** No. Existing code will work but will automatically use the faster native strategy by default (instead of multi/WASM). This is a performance improvement, not a breaking change.

### Q: How do I know which strategy my server supports?

**A:** Try extracting with each strategy and check for errors:

```python
capabilities = await check_server_capabilities(client)
print(f"Server supports: {capabilities}")
```

### Q: Can I force WASM even if native is available?

**A:** Yes, explicitly set `strategy="wasm"`:

```python
options = ExtractOptions(strategy="wasm")
result = await client.extract.extract(url, options=options)
```

### Q: What happens if I request WASM but it's not available?

**A:** The server returns an error. You should catch `ExtractionError` and fall back to native.

### Q: Is native extraction safe?

**A:** Yes, for trusted HTML sources (public websites, your own content). For completely untrusted HTML, use WASM for sandboxing.

### Q: How much slower is WASM?

**A:** Approximately 4x slower (10-20ms vs 2-5ms per page). This is acceptable for security-critical use cases.

### Q: Will the default change again in the future?

**A:** Unlikely. Native is now the recommended default. WASM is opt-in for specialized use cases.

## Examples

See `/workspaces/eventmesh/sdk/python/examples/extract_example.py` for complete working examples including:
- Basic extraction with default strategy
- Strategy comparison (native vs WASM)
- Graceful fallback patterns
- Server compatibility checks
- Performance benchmarking

## Summary

**Key Takeaways:**
1. **v0.9.0+ default**: `strategy="native"` (was `"multi"` in v0.8.x)
2. **Performance**: Native is 4x faster than WASM (2-5ms vs 10-20ms)
3. **Availability**: Native always available, WASM requires server feature
4. **Use native for**: 99% of applications (general web scraping)
5. **Use WASM for**: Untrusted HTML requiring sandboxing
6. **Migration**: No code changes needed (automatic improvement)
7. **Fallback**: Always handle WASM unavailability gracefully
