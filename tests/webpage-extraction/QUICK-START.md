# Webpage Extraction Comparison - Quick Start Guide

## TL;DR

This test suite compares 6 webpage extraction methods in RipTide EventMesh across 18 real-world URLs.

**Goal**: Identify the best extraction method(s) for production use.

---

## Quick Execution

```bash
# Navigate to test directory
cd tests/webpage-extraction

# Run full comparison (when test harness is implemented)
cargo run --release --bin webpage_extraction_test

# View results
cat results/comparison_report.md
```

---

## What's Being Tested

### 6 Extraction Methods:
1. **Trek (WASM)** - High-quality semantic extraction
2. **CSS Selectors** - Precise selector-based extraction
3. **Regex Patterns** - Fast pattern-matching extraction
4. **Fallback** - Lightweight no-dependency extraction
5. **DOM Traversal** - Fine-grained DOM inspection
6. **Table Extractor** - Specialized structured data extraction

### 18 Test URLs Across 5 Categories:
- **News Sites**: BBC, TechCrunch, Guardian, Ars Technica, Reuters
- **Documentation**: docs.rs, MDN, Rust blog, AWS blog, GitHub
- **Complex Layouts**: StackOverflow, Reddit, Medium
- **Structured Data**: Wikipedia, IMDB, Yahoo Finance
- **Edge Cases**: Data URLs, simple pages

---

## Test Dimensions

Each method is scored on:
- **Quality** (40%): Accuracy, completeness, noise filtering
- **Performance** (20%): Execution speed, resource usage
- **Robustness** (30%): Error handling, edge cases
- **Features** (10%): Media, links, metadata, exports

---

## Expected Outputs

### 1. Comparison Report
Location: `results/comparison_report.md`

Contains:
- Summary statistics table
- Method-by-method analysis
- Strengths and weaknesses
- Recommended use cases
- Production implementation guidance

### 2. Detailed Logs
Location: `logs/all_results.jsonl`

JSON Lines format with:
- Extraction results per method per URL
- Timing and performance metrics
- Error details and stack traces

### 3. Failure Analysis
Location: `results/failures.jsonl`

Lists all failed extractions with:
- URL and method combination
- Error message and type
- Recommendations for fixing

---

## Pre-Execution Checklist

Before running tests, verify:

- [ ] Rust toolchain installed (`cargo --version`)
- [ ] All dependencies built (`cargo build --release`)
- [ ] WASM runtime available (for Trek method)
- [ ] Network connectivity (for URL fetching)
- [ ] Output directories exist (`logs/`, `results/`)

---

## Interpreting Results

### Success Criteria:
- ✅ **>80% overall success rate** across all methods
- ✅ **Clear performance rankings** with timing data
- ✅ **Actionable recommendations** for production use
- ✅ **Documented failure patterns** with mitigation strategies

### Expected Outcome:
A **cascade strategy** recommendation, e.g.:
1. Try Trek (high quality)
2. Fallback to CSS (fast, reliable)
3. Last resort: Regex (always works)

---

## Next Steps

1. **Implement Test Harness**: Create `scripts/run_comparison.rs`
2. **Execute Tests**: Run comparison suite
3. **Analyze Results**: Review comparison report
4. **Implement Recommendations**: Update production extraction logic
5. **Monitor Production**: Track real-world performance

---

## Support

- Full test plan: `TEST-PLAN.md`
- Test URLs: `urls.txt`
- Script templates: `scripts/README.md`

For questions or issues, review the comprehensive test plan or coordinate via hive memory:
- Key: `hive/test/execution-plan`
- Namespace: `coordination`

---

**Status**: ✅ Test plan ready for execution
**Created**: 2025-10-11
**Agent**: Tester (Hive Mind)
