# Webpage Extraction Methods Comparison - Test Plan

## Executive Summary

This test plan defines a comprehensive evaluation framework for comparing multiple webpage content extraction methods available in the RipTide EventMesh system. The goal is to systematically test each extraction approach, identify strengths/weaknesses, and provide actionable recommendations.

**Test Duration:** 2-4 hours (automated execution)
**Methods Under Test:** 6 extraction approaches
**Test URLs:** 15-20 diverse real-world websites
**Success Criteria:** >80% extraction success rate, clear comparative analysis

---

## 1. Test Objectives

### Primary Objectives
1. **Functional Validation**: Verify all extraction methods work correctly
2. **Quality Comparison**: Compare extraction accuracy and completeness
3. **Performance Analysis**: Measure execution time and resource usage
4. **Gap Identification**: Document missing features and failure modes
5. **Recommendations**: Provide clear guidance on method selection

### Success Criteria
- All 6 extraction methods execute without critical failures
- Comparative metrics collected for all test URLs
- Clear documentation of strengths/weaknesses per method
- Actionable recommendations for production use
- Reproducible test harness for future validation

---

## 2. Extraction Methods Under Test

Based on codebase analysis, we will test the following extraction approaches:

### Method 1: Trek (WASM-based) Extraction
- **Implementation**: `TrekExtractor` in `extraction_strategies.rs`
- **Technology**: WASM component-based extraction using `CmExtractor`
- **Strengths**: High-quality extraction, semantic understanding
- **Configuration**: Default WASM path, configurable resource limits

### Method 2: CSS Selector Extraction
- **Implementation**: `CssExtractorStrategy` in `extraction_strategies.rs`
- **Technology**: CSS selector-based with JSON mapping (`CssJsonExtractor`)
- **Strengths**: Precise targeting, configurable selectors
- **Configuration**: Default selectors + custom patterns

### Method 3: Regex Pattern Extraction
- **Implementation**: `RegexExtractor` in `regex_extraction.rs`
- **Technology**: Pattern-based extraction with regex rules
- **Strengths**: Fast, rule-based, works without rendering
- **Configuration**: Default patterns from `default_patterns()`

### Method 4: Fallback Extraction
- **Implementation**: `fallback_extract()` in `extraction_strategies.rs`
- **Technology**: Simple selector-based extraction (no WASM)
- **Strengths**: Lightweight, no dependencies, always available
- **Configuration**: Built-in selector priorities

### Method 5: DOM Utilities Extraction
- **Implementation**: `DomTraverser` in `dom_utils.rs`
- **Technology**: Direct DOM traversal with element inspection
- **Strengths**: Fine-grained control, traversal capabilities
- **Configuration**: Custom traversal callbacks

### Method 6: Table-Specific Extraction
- **Implementation**: `TableExtractor` in `table_extraction/extractor.rs`
- **Technology**: Specialized table structure extraction
- **Strengths**: Structured data, export formats (CSV, Markdown)
- **Configuration**: `TableExtractionConfig` with detection options

---

## 3. Test Categories and Scenarios

### 3.1 Basic Content Extraction
**Test ID**: T001-T005
**Objective**: Verify basic text, title, and metadata extraction

| Test ID | Scenario | Expected Behavior | Priority |
|---------|----------|-------------------|----------|
| T001 | Simple article extraction | Extract title, content, metadata | High |
| T002 | Blog post with images | Extract text + media URLs | High |
| T003 | News article with byline | Extract author, date, content | High |
| T004 | Landing page | Extract headings, CTA text | Medium |
| T005 | Documentation page | Extract structured content | Medium |

### 3.2 Complex Structure Handling
**Test ID**: T010-T015
**Objective**: Test handling of complex HTML structures

| Test ID | Scenario | Expected Behavior | Priority |
|---------|----------|-------------------|----------|
| T010 | Multi-column layout | Extract all columns coherently | High |
| T011 | Nested divs and sections | Preserve content hierarchy | High |
| T012 | Dynamic content placeholders | Handle missing/loading content | Medium |
| T013 | Sidebar and footer noise | Extract main content only | High |
| T014 | Comment sections | Optionally include/exclude | Low |
| T015 | Ad-heavy page | Filter out advertisements | High |

### 3.3 Table and Structured Data
**Test ID**: T020-T024
**Objective**: Test extraction of structured data

| Test ID | Scenario | Expected Behavior | Priority |
|---------|----------|-------------------|----------|
| T020 | Simple HTML table | Extract rows/columns accurately | High |
| T021 | Table with merged cells | Handle colspan/rowspan | High |
| T022 | Nested tables | Extract recursively | Medium |
| T023 | Data table with headers | Preserve header relationships | High |
| T024 | Multiple tables on page | Extract all tables separately | Medium |

### 3.4 Media and Links
**Test ID**: T030-T034
**Objective**: Verify link and media extraction

| Test ID | Scenario | Expected Behavior | Priority |
|---------|----------|-------------------|----------|
| T030 | Image extraction | Extract all image URLs | High |
| T031 | Link extraction | Extract all hyperlinks | High |
| T032 | Relative URL resolution | Convert to absolute URLs | High |
| T033 | Media with attributes | Extract alt text, captions | Medium |
| T034 | Embedded media | Handle video/audio elements | Low |

### 3.5 Edge Cases and Error Handling
**Test ID**: T040-T045
**Objective**: Test robustness and error handling

| Test ID | Scenario | Expected Behavior | Priority |
|---------|----------|-------------------|----------|
| T040 | Malformed HTML | Graceful degradation | High |
| T041 | Empty page | Return empty result, no crash | High |
| T042 | Very large page (>1MB) | Handle within timeout | Medium |
| T043 | Unicode and special chars | Preserve encoding | High |
| T044 | JavaScript-heavy SPA | Extract available content | Medium |
| T045 | 404 or invalid URL | Proper error reporting | High |

---

## 4. Test URLs Selection

### Real-World Test Sites (15-20 URLs)

#### Category A: News and Articles (5 URLs)
- https://www.bbc.com/news (News article)
- https://techcrunch.com (Tech news)
- https://www.theguardian.com/international (International news)
- https://arstechnica.com (Technical journalism)
- https://www.reuters.com (Wire service)

#### Category B: Documentation and Blogs (5 URLs)
- https://docs.rs (Rust documentation)
- https://developer.mozilla.org/en-US/ (MDN docs)
- https://blog.rust-lang.org (Technical blog)
- https://aws.amazon.com/blogs/aws/ (Cloud blog)
- https://github.com/readme (README rendering)

#### Category C: Complex Layouts (3 URLs)
- https://stackoverflow.com/questions (Q&A format)
- https://www.reddit.com (Social media layout)
- https://medium.com (Article platform)

#### Category D: Structured Data (3 URLs)
- https://en.wikipedia.org/wiki/Rust_(programming_language) (Tables + infobox)
- https://www.imdb.com (Structured movie data)
- https://finance.yahoo.com (Financial tables)

#### Category E: Edge Cases (2-4 URLs)
- Data URL with inline content
- Simple HTML test page (controlled)
- Large HTML document (>500KB)
- Minimal HTML page

---

## 5. Test Execution Workflow

### 5.1 Pre-Test Validation Phase (15 min)

```bash
# Step 1: Environment setup
cargo build --release --all-features

# Step 2: Verify all extraction methods compile
cargo test --lib extraction_strategies -- --test-threads=1

# Step 3: Check test URLs accessibility
curl -I https://www.bbc.com/news
# ... (validate all URLs respond)

# Step 4: Create test directory structure
mkdir -p tests/webpage-extraction/{logs,results,scripts}
```

### 5.2 Test Execution Strategy

**Approach**: Sequential execution with parallel logging

```
For each TEST_URL in TEST_URLS:
  For each METHOD in EXTRACTION_METHODS:
    1. Start timer
    2. Execute extraction: METHOD.extract(TEST_URL)
    3. Stop timer
    4. Log results to: logs/{METHOD}_{URL_ID}_{timestamp}.json
    5. Capture errors to: logs/{METHOD}_{URL_ID}_error.log
    6. Store metrics: timing, success, content_length, quality_score
  End For
End For
```

**Timeout Policy**: 30 seconds per extraction attempt
**Retry Policy**: 1 retry on network failure, 0 retries on extraction failure
**Parallel Execution**: Not recommended (sequential for clear logging)

### 5.3 Execution Script Structure

```rust
// tests/webpage-extraction/scripts/run_comparison.rs

struct TestExecution {
    method: ExtractionMethod,
    url: String,
    url_id: String,
    timestamp: DateTime<Utc>,
}

async fn execute_test(test: TestExecution) -> TestResult {
    let start = Instant::now();

    let result = match test.method {
        Trek => execute_trek_extraction(&test.url).await,
        Css => execute_css_extraction(&test.url).await,
        Regex => execute_regex_extraction(&test.url).await,
        Fallback => execute_fallback_extraction(&test.url).await,
        Dom => execute_dom_extraction(&test.url).await,
        Table => execute_table_extraction(&test.url).await,
    };

    let duration = start.elapsed();

    // Log result
    log_test_result(&test, &result, duration).await;

    result
}
```

### 5.4 Log Aggregation Approach

**Log Format**: JSON Lines (JSONL) for easy parsing

```json
{
  "method": "trek",
  "url": "https://www.bbc.com/news",
  "url_id": "bbc_news",
  "timestamp": "2025-10-11T10:00:00Z",
  "success": true,
  "duration_ms": 1234,
  "content_length": 5678,
  "title": "Breaking News Article",
  "error": null,
  "metadata": {
    "quality_score": 95,
    "links_count": 45,
    "images_count": 12
  }
}
```

**Aggregation**: All logs collected in `logs/all_results.jsonl`

### 5.5 Post-Test Analysis Procedures

```bash
# Step 1: Parse all log files
python3 scripts/parse_logs.py logs/all_results.jsonl

# Step 2: Generate comparison report
python3 scripts/generate_report.py --input logs/all_results.jsonl \
                                    --output results/comparison_report.md

# Step 3: Create visualizations
python3 scripts/visualize_results.py --data logs/all_results.jsonl \
                                      --output results/charts/

# Step 4: Identify gaps and failures
grep '"success": false' logs/all_results.jsonl > results/failures.jsonl
```

---

## 6. Validation Checklist

### Pre-Execution Validation ✓

- [ ] All extraction methods compile successfully
- [ ] Test URLs are accessible (HTTP 200 responses)
- [ ] Output directories exist: `logs/`, `results/`, `scripts/`
- [ ] Required dependencies installed (WASM runtime, etc.)
- [ ] Network connectivity verified
- [ ] Timeout mechanisms configured
- [ ] Error handling tested with dummy data

### During Execution Validation ✓

- [ ] Each method invoked with correct parameters
- [ ] Logs capture all required data points
- [ ] Errors are caught and logged, not crashed
- [ ] Progress indicators working
- [ ] Timestamps are accurate
- [ ] Resource usage stays within limits

### Post-Execution Validation ✓

- [ ] All test URLs attempted for all methods
- [ ] Log files are well-formed JSON
- [ ] No missing or corrupted log entries
- [ ] Comparison tool produces readable diffs
- [ ] Statistical summary is accurate
- [ ] Failure cases are documented
- [ ] Test artifacts stored in git (optional)

---

## 7. Comparison Methodology

### 7.1 Metrics Collection

For each extraction, collect:

**Functional Metrics**:
- Success/failure status
- Content extracted (title, text, links, images)
- Content length (characters)
- Structure preservation (hierarchies, lists)

**Quality Metrics**:
- Extraction accuracy (manual spot-check)
- Noise level (unwanted content)
- Completeness (missing content)
- Formatting quality (Markdown rendering)

**Performance Metrics**:
- Execution time (milliseconds)
- Memory usage (if measurable)
- CPU utilization (if measurable)

**Reliability Metrics**:
- Error rate per method
- Timeout frequency
- Retry requirements

### 7.2 Comparison Dimensions

**Dimension 1: Extraction Quality**
- Title accuracy: Does extracted title match actual page title?
- Content completeness: Percentage of visible text extracted
- Noise filtering: How much unwanted content is included?
- Structure preservation: Are headings, lists, and hierarchies maintained?

**Dimension 2: Performance**
- Speed: Average execution time per method
- Scalability: Performance on large pages
- Resource usage: Memory/CPU consumption

**Dimension 3: Robustness**
- Error handling: Graceful degradation on malformed HTML
- Edge case handling: Success rate on unusual pages
- Reliability: Consistent results across runs

**Dimension 4: Feature Coverage**
- Media extraction: Images, videos
- Link extraction: Internal and external links
- Metadata extraction: Author, date, language
- Table extraction: Structured data
- Export formats: Markdown, plain text, JSON

### 7.3 Scoring System

Each method receives a score (0-100) in each dimension:

**Quality Score** = (Title Accuracy × 20) + (Content Completeness × 40) + (Noise Filtering × 20) + (Structure × 20)

**Performance Score** = 100 - (Normalized Execution Time × 50) - (Resource Usage × 50)

**Robustness Score** = (Success Rate × 60) + (Error Handling × 40)

**Feature Score** = (Features Supported / Total Features) × 100

**Overall Score** = (Quality × 0.4) + (Performance × 0.2) + (Robustness × 0.3) + (Features × 0.1)

---

## 8. Reporting Format

### 8.1 Summary Statistics Report

```markdown
# Webpage Extraction Methods - Comparison Report

## Executive Summary

**Test Date**: 2025-10-11
**URLs Tested**: 18
**Methods Compared**: 6
**Total Test Executions**: 108

### Overall Winner: [Method Name]
- **Quality Score**: 92/100
- **Performance Score**: 85/100
- **Robustness Score**: 95/100
- **Feature Score**: 88/100
- **Overall Score**: 90/100

---

## Summary Statistics

| Method | Success Rate | Avg Time (ms) | Quality Score | Overall Rank |
|--------|--------------|---------------|---------------|--------------|
| Trek   | 94%          | 1,234         | 92            | 1st          |
| CSS    | 89%          | 567           | 85            | 2nd          |
| Regex  | 100%         | 123           | 72            | 3rd          |
| ...    | ...          | ...           | ...           | ...          |
```

### 8.2 Method-by-Method Comparison

```markdown
## Method: Trek (WASM-based)

### Strengths
- Highest quality extraction (92/100)
- Excellent semantic understanding
- Best title and metadata extraction
- Good handling of complex layouts

### Weaknesses
- Slower execution (1,234ms average)
- Requires WASM runtime
- Higher memory usage
- 6% failure rate on edge cases

### Recommended Use Cases
- High-quality content extraction for articles
- Production systems with quality requirements
- Sites with complex HTML structures

### Failure Cases
- Very large pages (>1MB): Timeout
- Malformed HTML: WASM parser errors
```

### 8.3 Issue/Gap Identification Section

```markdown
## Identified Gaps and Issues

### Critical Issues (Must Fix)
1. **Trek WASM Timeouts**: Fails on pages >1MB (URLs: [list])
   - **Impact**: Cannot extract large documentation pages
   - **Recommendation**: Implement streaming or chunking

2. **CSS Selector Failures**: Fails when selectors don't match (URLs: [list])
   - **Impact**: Empty results on unexpected layouts
   - **Recommendation**: Add fallback selector chains

### Minor Issues (Should Fix)
1. **Regex Over-Extraction**: Includes navigation text in some cases
2. **Fallback Missing Metadata**: No author/date extraction
3. **DOM Traversal Performance**: Slow on deeply nested structures

### Enhancement Opportunities
1. Add confidence scoring to all methods
2. Implement automatic method selection based on page type
3. Create hybrid approach combining multiple methods
```

### 8.4 Recommendations Section

```markdown
## Recommendations for Production Use

### Recommended Strategy: Cascade Approach

1. **Primary**: Trek WASM Extraction
   - Use for most pages
   - High quality, good feature coverage
   - Fallback on timeout or error

2. **Fallback 1**: CSS Selector Extraction
   - Use when Trek fails or times out
   - Fast and reliable
   - Good for structured sites

3. **Fallback 2**: Regex Extraction
   - Use as last resort
   - Always succeeds
   - Acceptable quality for basic content

### Site-Specific Recommendations

- **News Sites**: Trek → CSS → Regex
- **Documentation**: CSS (custom selectors) → Trek
- **Social Media**: DOM Traversal → Trek
- **Structured Data**: Table Extractor → CSS
- **Unknown Sites**: Trek → Fallback

### Implementation Guidance

```rust
async fn extract_with_cascade(url: &str) -> Result<ExtractedContent> {
    // Try Trek first
    match trek_extract(url).await {
        Ok(content) if content.quality_score > 80 => return Ok(content),
        _ => {}
    }

    // Fallback to CSS
    match css_extract(url).await {
        Ok(content) => return Ok(content),
        _ => {}
    }

    // Last resort: Regex
    regex_extract(url).await
}
```
```

---

## 9. Test Artifacts and Deliverables

### Deliverables

1. **Test Plan Document** (this document)
   - Location: `tests/webpage-extraction/TEST-PLAN.md`

2. **Test Harness Script**
   - Location: `tests/webpage-extraction/scripts/run_comparison.rs`
   - Executable: `cargo run --bin webpage_extraction_test`

3. **Log Files**
   - Location: `tests/webpage-extraction/logs/`
   - Format: JSON Lines (JSONL)
   - Files: `all_results.jsonl`, `{method}_{url_id}.json`

4. **Comparison Report**
   - Location: `tests/webpage-extraction/results/comparison_report.md`
   - Format: Markdown with tables and statistics

5. **Visualization Charts** (optional)
   - Location: `tests/webpage-extraction/results/charts/`
   - Types: Bar charts, scatter plots, heatmaps

6. **Failure Analysis**
   - Location: `tests/webpage-extraction/results/failures.jsonl`
   - Format: JSON Lines with error details

### Reproducibility

All test artifacts should be version-controlled (except large log files):

```bash
git add tests/webpage-extraction/TEST-PLAN.md
git add tests/webpage-extraction/scripts/
git add tests/webpage-extraction/results/comparison_report.md
git add tests/webpage-extraction/results/failures.jsonl
# Exclude large logs
echo "tests/webpage-extraction/logs/*.jsonl" >> .gitignore
```

---

## 10. Timeline and Resource Allocation

### Estimated Timeline

**Phase 1: Setup and Validation** (30 min)
- Environment setup: 10 min
- Pre-test validation: 15 min
- URL accessibility check: 5 min

**Phase 2: Test Execution** (60-90 min)
- Per-URL testing: 3-5 min × 18 URLs = 54-90 min
- Includes: 6 methods × 10s = 60s per URL (best case)

**Phase 3: Analysis and Reporting** (45-60 min)
- Log parsing: 10 min
- Statistical analysis: 15 min
- Report generation: 20 min
- Review and validation: 10-15 min

**Phase 4: Documentation and Handoff** (15 min)
- Final report review
- Artifact storage
- Memory coordination

**Total Duration**: 2.5 - 3.5 hours

### Resource Requirements

- **Compute**: Single machine with 4GB RAM, 2 CPUs
- **Network**: Reliable internet for URL fetching
- **Storage**: ~500MB for logs and results
- **Dependencies**: Rust toolchain, WASM runtime, Python 3.x (for analysis)

---

## 11. Risk Assessment and Mitigation

### Risks

| Risk | Probability | Impact | Mitigation |
|------|-------------|--------|------------|
| URL becomes unavailable | Medium | Low | Have backup URLs, use cached versions |
| WASM runtime failure | Low | High | Test WASM separately, have fallback |
| Network timeout | Medium | Medium | Implement retry logic, increase timeout |
| Out of memory | Low | Medium | Test with resource limits, monitor usage |
| Log file corruption | Low | Low | Write atomically, validate after write |
| Incomplete test coverage | Low | Medium | Review test scenarios with team |

---

## 12. Appendices

### Appendix A: Sample Test Execution Output

```json
{
  "test_run_id": "run_2025_10_11_100000",
  "start_time": "2025-10-11T10:00:00Z",
  "end_time": "2025-10-11T11:30:00Z",
  "total_tests": 108,
  "successful_tests": 97,
  "failed_tests": 11,
  "methods": ["trek", "css", "regex", "fallback", "dom", "table"],
  "urls_tested": 18,
  "summary": {
    "trek": {"success": 17, "failed": 1, "avg_time_ms": 1234},
    "css": {"success": 16, "failed": 2, "avg_time_ms": 567},
    "regex": {"success": 18, "failed": 0, "avg_time_ms": 123}
  }
}
```

### Appendix B: Test Harness Usage

```bash
# Run full comparison
cargo run --release --bin webpage_extraction_test -- \
  --urls tests/webpage-extraction/urls.txt \
  --methods trek,css,regex,fallback,dom,table \
  --output tests/webpage-extraction/logs/all_results.jsonl

# Run single method
cargo run --release --bin webpage_extraction_test -- \
  --urls https://www.bbc.com/news \
  --methods trek \
  --output tests/webpage-extraction/logs/trek_bbc.json

# Generate report
cargo run --release --bin generate_report -- \
  --input tests/webpage-extraction/logs/all_results.jsonl \
  --output tests/webpage-extraction/results/report.md
```

### Appendix C: Memory Coordination Keys

**Hive Memory Keys for Agent Coordination:**

- `hive/test/execution-plan` → This test plan document
- `hive/test/status` → Test execution status updates
- `hive/test/results-summary` → Final test results summary
- `hive/test/failures` → Detailed failure analysis
- `hive/test/recommendations` → Final recommendations

**Usage:**
```bash
# Store test plan
npx claude-flow@alpha hooks post-edit \
  --file "tests/webpage-extraction/TEST-PLAN.md" \
  --memory-key "hive/test/execution-plan"

# Update status during execution
npx claude-flow@alpha memory store \
  --key "hive/test/status" \
  --value "In progress: 45/108 tests completed" \
  --namespace coordination
```

---

## 13. Approval and Sign-off

**Test Plan Created By**: Tester Agent (Hive Mind)
**Date**: 2025-10-11
**Version**: 1.0
**Status**: Ready for Execution

**Coordinator Approval**: Pending
**Execution Authorization**: Pending

---

## Quick Start Guide

For immediate execution:

```bash
# 1. Navigate to test directory
cd tests/webpage-extraction

# 2. Run comparison test
cargo run --release --bin webpage_extraction_test

# 3. View results
cat results/comparison_report.md

# 4. Check failures
cat results/failures.jsonl
```

**Expected Output**: Comprehensive comparison report with recommendations for production use.

---

**END OF TEST PLAN**
