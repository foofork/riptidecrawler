# Listings Extraction Test Report

**Test Date:** 2025-10-16
**Test Suite:** 30_listings.yml
**Engine:** raw (HTTP only, no JavaScript execution)
**WASM:** Disabled (`--no-wasm`)
**Mode:** Local extraction

## Test Configuration

```bash
riptide extract --url <URL> --engine raw --no-wasm --local
```

### Test URLs (from 30_listings.yml)

| Source | URL | Type |
|--------|-----|------|
| Hacker News | https://news.ycombinator.com/ | Listing page |
| GitHub Topics | https://github.com/topics/rust | Repository listing |
| Stack Overflow | https://stackoverflow.com/questions/tagged/rust | Question listing |
| Coolblue | https://www.coolblue.nl/en/laptops | Product listing |

## Results Summary

### Extraction Success

| Source | Items Extracted | Extraction Time | Status |
|--------|----------------|-----------------|--------|
| Hacker News | 10 | 685ms | ✓ Success |
| GitHub Topics | 0 | 1,089ms | ✗ Parsing failed |
| Stack Overflow | 0 | 131ms | ✗ Parsing failed |
| Coolblue | 0 | 787ms | ✗ Parsing failed |

**Total Items Extracted:** 10
**Average Extraction Time:** 685ms
**Overall Success Rate:** 25% (1/4 sources)

## Detailed Results

### 1. Hacker News ✓

**Status:** Successfully extracted 10 front-page stories
**Extraction Time:** 685ms
**Data Fields:** rank, title, URL, points, author, comments

**Sample Items:**

```csv
rank,title,points,author,comments
1,Liquibase continues to advertise itself as "open source" despite license switch,139,LaSombra,71
2,Upcoming Rust language features for kernel development,104,pykello,37
3,New coding models and integrations,107,meetpateltech,45
4,JustSketchMe – Digital Posing Tool,53,surprisetalk,18
5,Claude Haiku 4.5,634,adocomplete,233
```

**Observations:**
- HTML structure is simple and static (server-rendered)
- Regex patterns successfully matched story elements
- All metadata (points, author, comments) extracted accurately
- No JavaScript required for content rendering

### 2. GitHub Topics ✗

**Status:** Extraction succeeded but parsing failed
**Extraction Time:** 1,089ms
**Issue:** Modern React-based UI requires JavaScript execution

**Analysis:**
- Page uses client-side rendering (JavaScript required)
- Raw HTML contains mostly React placeholders
- Repository data loaded asynchronously via API calls
- Would require `--engine headless` for proper extraction

**Recommended Solution:**
```bash
riptide extract --url "https://github.com/topics/rust" --engine headless
```

### 3. Stack Overflow ✗

**Status:** Extraction succeeded but parsing failed
**Extraction Time:** 131ms (fastest)
**Issue:** HTML structure different from expected patterns

**Analysis:**
- Fast extraction time suggests server-rendered content
- HTML structure may have changed or differs from expected
- Regex patterns may need updating for current SO layout
- Possible that content is partially JavaScript-rendered

**Recommended Solution:**
- Update regex patterns for current Stack Overflow HTML structure
- May benefit from `--engine headless` if content is dynamic

### 4. Coolblue ✗

**Status:** Extraction succeeded but parsing failed
**Extraction Time:** 787ms
**Issue:** E-commerce site likely uses JavaScript for product rendering

**Analysis:**
- Moderate extraction time
- Product listings likely loaded via JavaScript
- May require authentication or cookie handling
- Anti-bot measures may be present

**Recommended Solution:**
```bash
riptide extract --url "https://www.coolblue.nl/en/laptops" \
  --engine headless \
  --stealth-level high
```

## Performance Metrics

### Extraction Time Analysis

| Metric | Value |
|--------|-------|
| Fastest | 131ms (Stack Overflow) |
| Slowest | 1,089ms (GitHub) |
| Average | 673ms |
| Median | 736ms |

### Success Factors

**What Worked (Hacker News):**
- Server-side rendered HTML
- Simple, semantic HTML structure
- No JavaScript execution required
- Stable CSS class names
- Clear content hierarchy

**What Didn't Work (Other Sites):**
- Client-side rendering (React/SPA)
- Dynamic content loading
- JavaScript-dependent UI
- Complex/obfuscated HTML
- Anti-scraping measures

## Recommendations

### 1. For JavaScript-Heavy Sites

Use headless browser engine instead of raw HTTP:

```bash
riptide extract \
  --url <URL> \
  --engine headless \
  --stealth-level medium
```

### 2. For E-Commerce Sites

Add stealth features and timing randomization:

```bash
riptide extract \
  --url <URL> \
  --engine headless \
  --stealth-level high \
  --randomize-timing \
  --simulate-behavior
```

### 3. For Listing Pages

Implement site-specific selectors or patterns:

```python
# Use CSS selectors for more reliable extraction
riptide extract \
  --url <URL> \
  --method css \
  --selector ".product-card"
```

### 4. Update Parsing Patterns

The current regex patterns work for Hacker News but need updates for:
- GitHub's React-rendered repository cards
- Stack Overflow's current question layout
- Coolblue's product listing structure

## Test Data Output

**CSV File:** `/workspaces/eventmesh/eval/results/listings_test.csv`
**Format:** source, rank, title, url, metadata, extraction_time_ms
**Rows:** 11 (1 header + 10 data rows)

### CSV Structure

```csv
source,rank,title,url,metadata,extraction_time_ms
hackernews,1,<title>,<url>,points:139|author:LaSombra|comments:71,685
```

## Conclusions

### Key Findings

1. **Raw engine works well for static sites** like Hacker News
2. **Modern web apps require headless engine** for JavaScript execution
3. **Extraction speed is good** (131ms - 1089ms range)
4. **Parsing reliability depends on** HTML structure stability
5. **Site-specific patterns are necessary** for different platforms

### Production Recommendations

1. **Implement engine auto-detection**
   - Try raw engine first (faster)
   - Fall back to headless if parsing fails

2. **Add retry logic**
   - Retry with different engines on failure
   - Implement exponential backoff

3. **Create site-specific extractors**
   - Maintain patterns for common sites
   - Use CSS selectors where possible

4. **Add validation**
   - Verify extracted item count
   - Check data quality metrics
   - Flag suspiciously empty results

### Next Steps

1. Test remaining URLs with headless engine
2. Update parsing patterns for failed extractions
3. Implement automated engine selection
4. Add extraction quality metrics
5. Create site-specific extractor plugins

## Files Generated

- `/workspaces/eventmesh/eval/results/listings_test.csv` - Extracted data
- `/workspaces/eventmesh/eval/scripts/extract_listings_v2.py` - Extraction script
- `/workspaces/eventmesh/eval/results/LISTINGS_EXTRACTION_TEST_REPORT.md` - This report

---

**Test Completed:** 2025-10-16
**Engine Version:** riptide (from /usr/local/bin/riptide)
**Total Execution Time:** ~3 seconds for all 4 URLs
