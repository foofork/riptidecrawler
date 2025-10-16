# RipTide Real-World Test Infrastructure

## Overview

This directory contains a comprehensive test suite for validating RipTide's extraction capabilities against real-world URLs across different content types and structures.

## Test Suites

### 00_static_docs
**Static Documentation Pages**
- Clean HTML with stable layout
- Technical documentation from MDN, Rust, PostgreSQL, Kubernetes
- Wikipedia reference pages
- **Expected Success Rate**: >90%

### 10_news_articles
**News Article Pages**
- Reuters graphics and special reports
- NOS Dutch news articles
- Article structure with bylines and dates
- **Expected Success Rate**: >85%

### 20_product_pages
**E-commerce Product Pages**
- Coolblue and B&H Photo products
- Rich schema.org data
- Product specifications and pricing
- **Expected Success Rate**: >80%

### 30_listings
**Listing and Hub Pages**
- Hacker News front page
- GitHub topics
- Stack Overflow questions
- E-commerce category pages
- **Expected Success Rate**: >75%

### 40_tables_pdfs
**PDF Documents with Tables**
- UK government budget documents
- OECD statistics
- Dutch municipal documents
- Complex financial tables
- **Expected Success Rate**: >70%

### 50_events_hilversum_music
**Event Listings**
- Live Hilversum events (Dutch/English)
- De Vorstin venue listings
- Songkick concert aggregator
- **Expected Success Rate**: >75%

## Quick Start

```bash
# Verify all URLs are accessible
make verify

# Run quick accessibility tests
make quick

# Run full extraction tests (requires built CLI)
make test

# Run specific suite
make test-static
make test-news
make test-products

# Clean results
make clean
```

## Test Scripts

### verify_urls.sh
Checks that all URLs in test suites are accessible using HTTP HEAD requests.

### quick_test.sh
Performs basic content verification using curl to ensure pages load and contain expected content.

### run_tests.sh
Full extraction test runner that:
- Executes RipTide CLI on each URL
- Captures results in JSON format
- Generates HTML report
- Tracks success/failure statistics

## Results Structure

```
eval/results/
└── YYYYMMDD_HHMMSS/
    ├── test_run.log           # Detailed execution log
    ├── summary.json            # Test results in JSON
    ├── report.html             # HTML report
    └── [suite]_[test].json     # Individual extraction results
```

## Success Criteria

Based on the RipTide CLI specification:
- **Static content**: >90% success rate
- **News sites**: >85% success rate
- **E-commerce**: >70% success rate
- **SPA (with headless)**: >80% post-integration
- **Overall errors**: <1% (non-policy)

## Performance Gates

- **P95 latency**:
  - Static: <500ms
  - News: <1s
  - Complex: <3s
- **Memory ceiling**: <100MB/extraction (avg)
- **Watchdog**: 2GB/job max

## Adding New Tests

1. Create a new YAML file in `eval/suites/`
2. Follow the structure:

```yaml
suite: XX_suite_name
description: "Description of test suite"
targets:
  - name: "Test Name"
    url: "https://example.com/page"
    type: article|product|listing|pdf
    expected:
      - content_present: true
      - specific_check: true
```

3. Update this README with the new suite information
4. Add a make target if needed

## Troubleshooting

### URLs Not Accessible
- Check network connectivity
- Verify URLs haven't changed
- Some sites may block automated requests

### Extraction Failures
- Check WASM module is available
- Verify Redis is running (if required)
- Review extraction strategy settings

### Memory Issues
- Monitor memory usage during tests
- Adjust timeout settings
- Run suites individually

## CI/CD Integration

```yaml
# GitHub Actions example
- name: Run RipTide Tests
  run: |
    make verify
    make test
  continue-on-error: true

- name: Upload Test Results
  uses: actions/upload-artifact@v2
  with:
    name: test-results
    path: eval/results/
```

## Test Coverage

Current coverage as of implementation:
- **6 test suites**
- **24 unique URLs**
- **6 content types** (docs, articles, products, listings, PDFs, events)
- **3 languages** (English, Dutch, mixed)
- **Multiple extraction strategies** tested

## Next Steps

1. Build and verify RipTide CLI binary
2. Run full extraction tests
3. Analyze results and identify improvements
4. Add more edge cases and difficult sites
5. Implement continuous monitoring

---

For more information, see the main implementation plan at `/workspaces/eventmesh/docs/riptide-cli-revised-implementation-plan.md`