# CLI Test Scenarios - Spider-Chrome Migration

**Purpose:** Test scenarios for validating CLI functionality with spider-chrome engine

---

## 1. Basic Extraction Tests

### Scenario 1.1: Simple URL Extraction
```bash
# Command
riptide extract --url "https://example.com" --engine spider

# Expected Output
{
  "url": "https://example.com",
  "engine": "spider",
  "status": "success",
  "content": "Example Domain\n\nThis domain is for use in illustrative examples...",
  "metadata": {
    "title": "Example Domain",
    "extraction_time_ms": 1245,
    "method_used": "spider"
  }
}

# Validation
✓ Exit code: 0
✓ JSON output valid
✓ Content contains "Example Domain"
✓ Engine reported as "spider"
✓ Extraction time < 5000ms
```

### Scenario 1.2: Multiple URLs
```bash
# Create URL list
cat > test-urls.txt <<EOF
https://example.com
https://example.org
https://example.net
EOF

# Command
riptide batch extract --urls test-urls.txt --engine spider --concurrency 3

# Expected Output
[
  {
    "url": "https://example.com",
    "status": "success",
    "content": "...",
    "extraction_time_ms": 1123
  },
  {
    "url": "https://example.org",
    "status": "success",
    "content": "...",
    "extraction_time_ms": 1087
  },
  {
    "url": "https://example.net",
    "status": "success",
    "content": "...",
    "extraction_time_ms": 1201
  }
]

# Validation
✓ Exit code: 0
✓ All 3 URLs processed
✓ All statuses: "success"
✓ Concurrent execution (total time < sum of individual times)
```

---

## 2. Screenshot Tests

### Scenario 2.1: PNG Screenshot
```bash
# Command
riptide extract --url "https://example.com" --engine spider --screenshot --output example.png

# Expected Output
Screenshot saved to: example.png

# Validation
✓ Exit code: 0
✓ File exists: example.png
✓ File size > 10KB
✓ PNG header valid: [137, 80, 78, 71, 13, 10, 26, 10]
✓ Image dimensions >= 800x600
```

### Scenario 2.2: Full Page Screenshot
```bash
# Command
riptide extract --url "https://example.com" --engine spider --screenshot --full-page --output example-full.png

# Expected Output
Full page screenshot saved to: example-full.png

# Validation
✓ Exit code: 0
✓ File size > 20KB (full page larger than viewport)
✓ Height > 2000px (captures beyond fold)
```

### Scenario 2.3: JPEG Screenshot with Quality
```bash
# Command
riptide extract --url "https://example.com" --engine spider --screenshot --format jpeg --quality 80 --output example.jpg

# Expected Output
Screenshot saved to: example.jpg

# Validation
✓ Exit code: 0
✓ JPEG header valid: [0xFF, 0xD8, 0xFF]
✓ File size appropriate for quality=80
```

---

## 3. PDF Generation Tests

### Scenario 3.1: Basic PDF
```bash
# Command
riptide extract --url "https://example.com" --engine spider --pdf --output example.pdf

# Expected Output
PDF saved to: example.pdf

# Validation
✓ Exit code: 0
✓ File exists: example.pdf
✓ PDF header valid: "%PDF-"
✓ File size > 5KB
✓ Can be opened with PDF reader
```

### Scenario 3.2: PDF with Custom Options
```bash
# Command
riptide extract --url "https://example.com" --engine spider --pdf \
    --pdf-landscape \
    --pdf-scale 0.8 \
    --output example-landscape.pdf

# Expected Output
PDF saved to: example-landscape.pdf

# Validation
✓ Exit code: 0
✓ PDF is in landscape orientation
✓ Content is scaled appropriately
```

---

## 4. Wait Strategy Tests

### Scenario 4.1: Network Idle Wait
```bash
# Command
riptide extract --url "https://example.com" --engine spider --wait-until networkidle

# Validation
✓ Exit code: 0
✓ Content fully loaded
✓ No pending network requests
✓ Extraction time reasonable (< 10s)
```

### Scenario 4.2: DOM Content Loaded Wait
```bash
# Command
riptide extract --url "https://example.com" --engine spider --wait-until domcontentloaded

# Validation
✓ Exit code: 0
✓ DOM content loaded
✓ Faster than networkidle (< 3s)
```

### Scenario 4.3: Load Event Wait
```bash
# Command
riptide extract --url "https://example.com" --engine spider --wait-until load

# Validation
✓ Exit code: 0
✓ Load event fired
✓ Basic resources loaded
```

---

## 5. Stealth Mode Tests

### Scenario 5.1: Low Stealth
```bash
# Command
riptide extract --url "https://example.com" --engine spider --stealth low

# Validation
✓ Exit code: 0
✓ Basic stealth features enabled
✓ User agent modified
```

### Scenario 5.2: Medium Stealth
```bash
# Command
riptide extract --url "https://example.com" --engine spider --stealth medium

# Validation
✓ Exit code: 0
✓ WebDriver detection bypassed
✓ Canvas fingerprinting mitigated
```

### Scenario 5.3: High Stealth
```bash
# Command
riptide extract --url "https://example.com" --engine spider --stealth high

# Validation
✓ Exit code: 0
✓ All stealth features enabled
✓ Bot detection bypassed
```

---

## 6. Timeout Tests

### Scenario 6.1: Custom Timeout
```bash
# Command
riptide extract --url "https://example.com" --engine spider --timeout 60

# Validation
✓ Exit code: 0
✓ Completes within 60s
```

### Scenario 6.2: Timeout Exceeded
```bash
# Command (slow loading site)
riptide extract --url "https://httpstat.us/200?sleep=70000" --engine spider --timeout 5

# Expected Output
{
  "url": "https://httpstat.us/200?sleep=70000",
  "status": "error",
  "error": {
    "code": "TIMEOUT",
    "message": "Navigation timeout after 5000ms"
  }
}

# Validation
✓ Exit code: 1
✓ Error code: TIMEOUT
✓ Fails within ~5s (not 70s)
```

---

## 7. Error Handling Tests

### Scenario 7.1: Invalid URL
```bash
# Command
riptide extract --url "not-a-valid-url" --engine spider

# Expected Output
{
  "url": "not-a-valid-url",
  "status": "error",
  "error": {
    "code": "INVALID_URL",
    "message": "Invalid URL format"
  }
}

# Validation
✓ Exit code: 1
✓ Error code: INVALID_URL
✓ Helpful error message
```

### Scenario 7.2: Network Error
```bash
# Command
riptide extract --url "https://invalid-domain-12345.com" --engine spider

# Expected Output
{
  "url": "https://invalid-domain-12345.com",
  "status": "error",
  "error": {
    "code": "NAVIGATION_FAILED",
    "message": "Failed to navigate: DNS resolution failed"
  }
}

# Validation
✓ Exit code: 1
✓ Error code: NAVIGATION_FAILED
✓ DNS error detected
```

### Scenario 7.3: 404 Error
```bash
# Command
riptide extract --url "https://example.com/nonexistent-page" --engine spider

# Expected Output
{
  "url": "https://example.com/nonexistent-page",
  "status": "success",
  "content": "404 Not Found...",
  "metadata": {
    "http_status": 404,
    "title": "404 Not Found"
  }
}

# Validation
✓ Exit code: 0 (page loaded, even if 404)
✓ HTTP status: 404
✓ Content indicates 404
```

### Scenario 7.4: SSL Error
```bash
# Command
riptide extract --url "https://expired.badssl.com" --engine spider

# Expected Output
{
  "url": "https://expired.badssl.com",
  "status": "error",
  "error": {
    "code": "SSL_ERROR",
    "message": "SSL certificate expired"
  }
}

# Validation
✓ Exit code: 1
✓ Error code: SSL_ERROR
✓ Certificate error detected
```

---

## 8. Output Format Tests

### Scenario 8.1: JSON Output
```bash
# Command
riptide extract --url "https://example.com" --engine spider --format json

# Validation
✓ Valid JSON output
✓ All required fields present
✓ Can be parsed with jq
```

### Scenario 8.2: Plain Text Output
```bash
# Command
riptide extract --url "https://example.com" --engine spider --format text

# Expected Output
Example Domain

This domain is for use in illustrative examples in documents...

# Validation
✓ Plain text (no JSON)
✓ Content only (no metadata)
✓ Clean formatting
```

### Scenario 8.3: Verbose Output
```bash
# Command
riptide extract --url "https://example.com" --engine spider --verbose

# Expected Output
[INFO] Initializing spider engine...
[INFO] Launching browser...
[INFO] Navigating to https://example.com...
[INFO] Waiting for page load...
[INFO] Extracting content...
[INFO] Extraction complete in 1.2s

{...content...}

# Validation
✓ Logging messages present
✓ Timestamps included
✓ Execution steps visible
```

---

## 9. Performance Tests

### Scenario 9.1: Rapid Sequential Requests
```bash
# Command
for i in {1..10}; do
    riptide extract --url "https://example.com" --engine spider
done

# Validation
✓ All 10 requests succeed
✓ No resource exhaustion
✓ Consistent performance
✓ Average time < 2s per request
```

### Scenario 9.2: Memory Usage
```bash
# Command (monitor memory)
/usr/bin/time -v riptide extract --url "https://example.com" --engine spider

# Validation
✓ Peak memory < 1GB
✓ No memory leaks
✓ Clean shutdown
```

---

## 10. Integration Tests

### Scenario 10.1: Extract + Screenshot + PDF
```bash
# Command
riptide extract --url "https://example.com" --engine spider \
    --screenshot --output screenshot.png \
    --pdf --output page.pdf \
    --format json --output content.json

# Validation
✓ All 3 outputs created
✓ Screenshot valid PNG
✓ PDF valid and openable
✓ JSON contains content
```

### Scenario 10.2: Pipeline with Multiple Steps
```bash
# Command
riptide extract --url "https://example.com" --engine spider --format json | \
    jq '.content' | \
    wc -w

# Validation
✓ Pipeline works
✓ Content extracted
✓ Word count > 10
```

---

## Test Execution Order

1. **Smoke Tests** (2-3 minutes)
   - Basic extraction
   - Screenshot
   - PDF generation

2. **Feature Tests** (5-10 minutes)
   - Wait strategies
   - Stealth modes
   - Timeout handling

3. **Error Tests** (3-5 minutes)
   - Invalid URLs
   - Network errors
   - Timeout scenarios

4. **Performance Tests** (10-15 minutes)
   - Sequential requests
   - Memory usage
   - Concurrent execution

5. **Integration Tests** (5-10 minutes)
   - Multi-output scenarios
   - Pipelines

**Total Estimated Time:** 25-45 minutes

---

## Test Data

### Safe Test URLs
```
https://example.com           # Basic HTML
https://example.org           # Basic HTML
https://httpbin.org/html      # Test HTML
https://httpbin.org/delay/2   # Slow response
https://httpstat.us/404       # 404 error
https://httpstat.us/500       # 500 error
```

### Expected Results Database
Location: `/workspaces/eventmesh/tests/webpage-extraction/test-urls.json`

Contains expected outputs for various test scenarios.

---

## Success Criteria

For CLI tests to pass:
✓ All basic extraction tests pass
✓ Screenshot generation works
✓ PDF generation works
✓ Error handling works correctly
✓ Performance is acceptable (< 5s average)
✓ Memory usage is reasonable (< 1GB peak)

---

## Next Steps

1. Wait for build to complete
2. Run smoke test: `./docs/testing/smoke-test.sh`
3. Run full CLI tests: `./docs/testing/run-spider-chrome-tests.sh`
4. Execute manual CLI scenarios from this document
5. Document any failures or issues
