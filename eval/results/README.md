# Riptide Extract Command Test Results

## Overview

This directory contains comprehensive test results for the `riptide extract` command, testing various extraction engines and strategies on real-world URLs.

## Test Summary

- **Test Date:** 2025-10-16
- **Riptide Version:** 1.0.0
- **Total Tests:** 22
- **Success Rate:** 22.72% (5/22)
- **Critical Issue:** WASM module interface mismatch blocking 77.28% of features

## Files in This Directory

### Primary Reports

| File | Description | Format |
|------|-------------|--------|
| `extract_command_tests.csv` | Raw test data with all results | CSV |
| `extract_command_analysis.md` | Detailed technical analysis and findings | Markdown |
| `test_matrix.txt` | Visual test matrix with performance charts | Text |
| `working_examples.sh` | Executable examples of working commands | Shell Script |

### Test Data

| Directory | Contents |
|-----------|----------|
| `temp_extract/` | Individual test outputs and error logs |

### Supporting Files

| File | Description |
|------|-------------|
| `rust_lang_output.html` | Sample extracted content from rust-lang.org |
| `README.md` | This file |

## Quick Results

### Requested Tests

1. ✅ `https://example.com` with raw engine - **SUCCESS** (768 B, 555ms)
2. ✅ `https://en.wikipedia.org/wiki/Rust_(programming_language)` with raw engine - **SUCCESS** (581 KB, 150ms)
3. ✅ `https://news.ycombinator.com/` with raw engine - **SUCCESS** (35 KB, 763ms)
4. ❌ `https://www.rust-lang.org/` with auto engine + metadata - **FAILED** (WASM error)

### Engine Performance

| Engine | Tests | Success | Rate | Avg Time |
|--------|-------|---------|------|----------|
| raw | 5 | 5 | 100% | 437.6ms ✅ |
| auto | 12 | 0 | 0% | N/A ❌ |
| wasm | 7 | 0 | 0% | N/A ❌ |
| headless | 1 | 0 | 0% | N/A ❌ |

## Critical Issue

**WASM Module Interface Mismatch:**
```
Error: type-checking export func `health-check`
Caused by: expected record field named extractor-version, found trek-version
```

This error is blocking all non-raw engine functionality, including:
- Auto engine selection
- WASM-based extraction
- Headless browser extraction
- Advanced methods (css, llm, regex)
- Strategy compositions (chain, parallel, fallback)

## Working Features

✅ **Raw Engine** - 100% success rate
```bash
riptide extract --url "URL" --engine raw --local
```

✅ **File Operations**
```bash
# Save to file
riptide extract --url "URL" --engine raw --local -f output.html

# Read from file
riptide extract --input-file input.html --engine raw --local
```

## Performance Highlights

- **Fastest:** 130ms (rust-lang.org, 18KB)
- **Largest:** 581KB Wikipedia article in only 150ms!
- **Average:** 437.6ms across all successful tests
- **Size Range:** 768 bytes to 581KB handled reliably

## Recommendations

### Immediate (Critical Priority)

1. Fix WASM module interface mismatch
   - Update `health-check` function to use `extractor-version` field
   - Rebuild WASM module with correct interface

2. Verify WASM build process
   - Check compilation scripts
   - Ensure version consistency

3. Add interface compatibility tests to CI

### Workaround

Until the WASM module is fixed, use:
```bash
riptide extract --url "URL" --engine raw --local
```

## Test Scripts

Run the test suite again:
```bash
# Comprehensive tests
/workspaces/eventmesh/eval/test_extract_corrected.sh

# Working examples only
chmod +x /workspaces/eventmesh/eval/results/working_examples.sh
/workspaces/eventmesh/eval/results/working_examples.sh
```

## CSV Data Schema

The `extract_command_tests.csv` file contains:
- `URL` - Tested URL
- `Engine` - Extraction engine used (raw, auto, wasm, headless)
- `Method` - Extraction method (auto, wasm, css, llm, regex)
- `Strategy` - Strategy composition (chain, parallel, fallback, none)
- `Success` - true/false
- `Content_Length` - Bytes extracted
- `Time_ms` - Extraction time in milliseconds
- `Error` - Error message if failed

## Contact

For issues or questions about these test results, refer to:
- Main repository: `/workspaces/eventmesh/`
- Test scripts: `/workspaces/eventmesh/eval/`
