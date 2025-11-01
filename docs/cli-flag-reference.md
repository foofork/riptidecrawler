# CLI Flag Reference - Extract Command

## Quick Reference

### Extraction Method Flags

| Flag | Status | Default | Description |
|------|--------|---------|-------------|
| `--with-wasm` | **NEW** | OFF | Opt-in to WASM-enhanced extraction |
| `--no-wasm` | **DEPRECATED** | N/A | Native is now default (shows warning) |

### Engine Selection

| Flag | Default | Behavior |
|------|---------|----------|
| `--engine auto` | ✓ | Auto-select engine (native by default) |
| `--engine raw` | | Native parser for basic HTML |
| `--engine wasm` | | Requires `--with-wasm` flag |
| `--engine headless` | | Browser + native parser (or WASM with `--with-wasm`) |

## Usage Examples

### Default Behavior (Native Extraction)

```bash
# URL-based extraction (native parser)
riptide extract --url https://example.com --local

# File-based extraction (native parser)
riptide extract --input-file article.html

# Stdin extraction (native parser)
cat article.html | riptide extract --stdin

# Headless extraction with native parsing
riptide extract --url https://spa.example.com --local --engine headless
```

### WASM Enhancement (Opt-in)

```bash
# Enable WASM enhancement
riptide extract --url https://example.com --local --with-wasm

# Headless with WASM parsing
riptide extract --url https://example.com --local --engine headless --with-wasm

# Specify custom WASM path
riptide extract --url https://example.com --local --with-wasm \
  --wasm-path /path/to/riptide-extractor.wasm
```

### Backward Compatibility

```bash
# Deprecated: --no-wasm (shows warning)
riptide extract --url https://example.com --no-wasm
# Output: Warning: --no-wasm flag is deprecated (native is now default)

# Conflicting flags (--with-wasm takes precedence)
riptide extract --url https://example.com --with-wasm --no-wasm
# Result: Uses native (--no-wasm overrides --with-wasm)
```

## Extraction Flow

### Without `--with-wasm` (Default)

```
URL/File/Stdin
    ↓
HTTP Fetch (if URL)
    ↓
Native Rust Parser
    ↓
ExtractedDoc
    ↓
Output (JSON/Text/Table)
```

### With `--with-wasm`

```
URL/File/Stdin
    ↓
HTTP Fetch (if URL)
    ↓
WASM Extractor Initialization
    ↓
WASM Extraction
    ↓ (on failure)
Fallback to Native Parser
    ↓
ExtractedDoc
    ↓
Output (JSON/Text/Table)
```

## Method Names in Output

| Method | Engine | Parser | Description |
|--------|--------|--------|-------------|
| `native` | raw | native | Direct native extraction |
| `headless-native` | headless | native | Browser render + native parse |
| `local-wasm` | wasm | WASM | Local WASM extraction |
| `headless-wasm` | headless | WASM | Browser render + WASM parse |

## Environment Variables

| Variable | Purpose | Example |
|----------|---------|---------|
| `RIPTIDE_WASM_PATH` | Override WASM module path | `/opt/riptide/riptide-extractor.wasm` |

## Feature Flags

| Feature | Purpose | Required For |
|---------|---------|--------------|
| `wasm-extractor` | Enable WASM extraction support | `--with-wasm` flag |

Build with WASM support:
```bash
cargo build --features wasm-extractor
```

## Common Scenarios

### Scenario 1: Simple Article Extraction
```bash
# Just extract the article (native is fast and reliable)
riptide extract --url https://blog.example.com/article --local
```

### Scenario 2: JavaScript-Heavy Site
```bash
# Use headless browser with native parser
riptide extract --url https://spa.example.com --local --engine headless

# Or with WASM for extra quality
riptide extract --url https://spa.example.com --local --engine headless --with-wasm
```

### Scenario 3: Batch Processing
```bash
# Process multiple files with native parser (fast)
for file in *.html; do
  riptide extract --input-file "$file" -f "output/${file%.html}.txt"
done
```

### Scenario 4: High-Quality Extraction
```bash
# Use WASM for potentially better quality
riptide extract --url https://complex-site.com --local --with-wasm \
  --show-confidence --metadata
```

### Scenario 5: Stealth Extraction
```bash
# Headless with stealth + native parsing
riptide extract --url https://protected-site.com --local \
  --engine headless --stealth-level high --simulate-behavior
```

## Migration Checklist

- [ ] Review scripts using `--no-wasm` flag
- [ ] Remove `--no-wasm` flags (no longer needed)
- [ ] Add `--with-wasm` flag where WASM enhancement is desired
- [ ] Test extraction quality with native parser
- [ ] Update documentation referencing old flag behavior
- [ ] Update CI/CD pipelines if using extraction commands

## Troubleshooting

### "Native extraction failed"
```bash
# Try with WASM enhancement
riptide extract --url <URL> --local --with-wasm
```

### "WASM module not found"
```bash
# Build WASM module
cargo build --release --target wasm32-wasip2

# Or use native parser (remove --with-wasm)
riptide extract --url <URL> --local
```

### "WASM initialization timed out"
```bash
# Increase timeout
riptide extract --url <URL> --local --with-wasm --init-timeout-ms 10000

# Or use native parser
riptide extract --url <URL> --local
```

### Low quality score with native parser
```bash
# Try WASM enhancement
riptide extract --url <URL> --local --with-wasm

# Or use headless for JavaScript-heavy sites
riptide extract --url <URL> --local --engine headless
```

## Performance Tips

1. **Use native parser by default**: It's fast and reliable for most sites
2. **Use `--with-wasm` for complex sites**: When you need extra extraction quality
3. **Use headless for SPAs**: When content requires JavaScript execution
4. **Combine headless + WASM**: For maximum quality on complex JavaScript sites
5. **Batch operations**: Native parser is faster for processing many files

## Output Formats

All extraction methods support the same output formats:

```bash
# JSON output (default)
riptide extract --url <URL> --local -o json

# Plain text
riptide extract --url <URL> --local -o text

# Table format
riptide extract --url <URL> --local -o table
```

## Additional Flags

These flags work with all extraction methods:

| Flag | Description |
|------|-------------|
| `--show-confidence` | Show confidence score |
| `--metadata` | Include extraction metadata |
| `--file <path>` | Save output to file |
| `--stealth-level <level>` | Anti-detection level (none/low/medium/high) |
| `--user-agent <ua>` | Custom user agent string |
| `--proxy <url>` | Use proxy server |
| `--selector <css>` | CSS selector for extraction |
| `--pattern <regex>` | Regex pattern for extraction |

## More Information

- [Full Documentation](/docs/native-extraction-update.md)
- [Native Parser Details](/crates/riptide-extraction/src/native_parser/README.md)
- [WASM Extractor Details](/crates/riptide-extraction/src/wasm_extraction/README.md)
