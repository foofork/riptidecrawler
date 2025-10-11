# Webpage Extraction Test Suite

Comprehensive testing infrastructure for evaluating webpage extraction methods across diverse real-world URLs.

## Directory Structure

```
tests/webpage-extraction/
├── test-urls.json          # 30 diverse test URLs across categories
├── cli-test-harness.rs     # Rust test harness for CLI invocation
├── comparison-tool.rs      # Result comparison and analysis
├── lib.rs                  # Module exports
├── scripts/
│   ├── run-all-tests.sh    # Execute all tests
│   └── compare-results.sh  # Compare test sessions
├── logs/                   # Test execution logs
└── results/                # Test results in JSON format
```

## Test URLs

The test suite includes 30 carefully selected URLs across categories:

- **News** (CNN, BBC, Reuters, Al Jazeera, Asahi)
- **E-commerce** (Amazon, eBay)
- **Documentation** (MDN, GitHub, Rust Book, React)
- **Blogs** (Medium, Cloudflare)
- **Social Media** (Reddit, Twitter)
- **API Documentation** (Stripe, OpenAI)
- **Academic** (ArXiv)
- **Forums** (StackOverflow)
- **Government** (USA.gov)
- **Media** (YouTube)
- **Dynamic Content** (Weather.com)
- **Paywalls** (NYTimes)
- **SPAs** (React docs)
- **Heavy JS** (Figma)

## Quick Start

### Run All Tests

```bash
cd tests/webpage-extraction/scripts
./run-all-tests.sh run
```

This will:
- Test all 30 URLs with all extraction methods
- Retry failures up to 3 times
- Log all output to `logs/`
- Save structured results to `results/`

### Generate Report

```bash
./run-all-tests.sh report [session-file]
```

Generates a summary report with:
- Overall success rates
- Per-method statistics
- Failed test details
- Performance metrics

### Compare Methods

```bash
./compare-results.sh methods [session-file]
```

Shows:
- Success rates by method
- Average duration by method
- Average content length by method
- Best overall method recommendation

### Compare Sessions

```bash
./compare-results.sh sessions session-1.json session-2.json
```

Identifies:
- Improvements (tests that now pass)
- Regressions (tests that now fail)
- Performance changes (>20% difference)

## Usage Examples

### Basic Test Run

```bash
# Run all tests
./scripts/run-all-tests.sh run

# View latest results
./scripts/run-all-tests.sh report

# Compare methods in latest session
./scripts/compare-results.sh methods
```

### Advanced Usage

```bash
# Run specific methods only
METHODS=("jina" "playwright") ./scripts/run-all-tests.sh run

# Increase timeout for slow sites
TIMEOUT=60 ./scripts/run-all-tests.sh run

# Adjust retry attempts
RETRY_ATTEMPTS=5 ./scripts/run-all-tests.sh run

# Compare two specific sessions
./scripts/compare-results.sh sessions \
    session-1734567890.json \
    session-1734567999.json

# Quick diff of two most recent sessions
./scripts/compare-results.sh diff
```

## Rust API Usage

```rust
use webpage_extraction_tests::{TestHarness, ComparisonTool};
use std::path::PathBuf;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Create test harness
    let harness = TestHarness::new(
        PathBuf::from("./results"),
        PathBuf::from("../../target/release/eventmesh-cli")
    );

    // Load test URLs
    let urls = harness.load_test_urls(
        &PathBuf::from("./test-urls.json")
    ).await?;

    // Run tests
    let methods = vec!["jina".to_string(), "playwright".to_string()];
    let session = harness.run_test_suite(&urls, &methods).await?;

    println!("Tests complete: {}/{} passed",
        session.successful_tests,
        session.total_tests);

    // Compare results
    let tool = ComparisonTool::new(PathBuf::from("./results"));
    let report = tool.compare_methods(&session)?;
    tool.print_report(&report);

    Ok(())
}
```

## Test Result Format

Each test produces structured JSON:

```json
{
  "test_id": "news-cnn-article",
  "method": "jina",
  "url": "https://www.cnn.com/...",
  "success": true,
  "duration_ms": 1234,
  "content_length": 5678,
  "error": null,
  "warnings": [],
  "metadata": {
    "has_html_tags": true,
    "has_json_structure": false,
    "line_count": 123
  },
  "content_preview": "First 500 chars...",
  "timestamp": "2025-01-15T12:34:56Z"
}
```

## Session Format

Test sessions contain complete results:

```json
{
  "session_id": "test-session-1734567890",
  "start_time": "2025-01-15T12:00:00Z",
  "end_time": "2025-01-15T12:30:00Z",
  "total_tests": 180,
  "successful_tests": 162,
  "failed_tests": 18,
  "results": [/* ... */]
}
```

## Error Handling

The test suite gracefully handles:

- **Timeouts**: 30s default, configurable
- **Network failures**: Automatic retries (3 attempts default)
- **Malformed responses**: Captured in error field
- **404/500 errors**: Logged with status codes
- **Authentication requirements**: Detected and reported
- **Rate limiting**: Exponential backoff between retries

## Logging

Logs are organized by:

```
logs/
├── {test_id}_{method}_attempt1.log
├── {test_id}_{method}_attempt2.log
└── {test_id}_{method}_attempt3.log
```

Each log contains:
- Full stdout/stderr from extraction
- Timing information
- Error messages
- Retry attempts

## Comparison Reports

Comparison reports include:

```json
{
  "session_id": "comparison-1734567890",
  "timestamp": "2025-01-15T12:30:00Z",
  "comparisons": [
    {
      "test_id": "news-cnn-article",
      "url": "https://...",
      "methods": {
        "jina": {
          "success": true,
          "duration_ms": 1234,
          "content_length": 5678
        },
        "playwright": {
          "success": true,
          "duration_ms": 2345,
          "content_length": 6789
        }
      },
      "best_method": "jina",
      "differences": [
        "Performance variation: 1234ms - 2345ms"
      ]
    }
  ],
  "summary": {
    "total_urls": 30,
    "method_success_rates": {
      "jina": 0.95,
      "playwright": 0.98
    },
    "method_avg_duration": {
      "jina": 1500.0,
      "playwright": 2200.0
    },
    "best_overall_method": "jina",
    "recommendations": [
      "🏆 jina is the best overall method (success: 95.0%, avg: 1500ms)"
    ]
  }
}
```

## CI/CD Integration

```bash
# In your CI pipeline
cd tests/webpage-extraction
./scripts/run-all-tests.sh run

# Check for regressions
if [[ -f results/session-latest.json ]]; then
    ./scripts/compare-results.sh sessions \
        results/session-baseline.json \
        results/session-latest.json
fi

# Fail if success rate drops below threshold
SUCCESS_RATE=$(jq '.success_rate' results/session-latest.json)
if (( $(echo "$SUCCESS_RATE < 90.0" | bc -l) )); then
    echo "Test success rate below 90%: $SUCCESS_RATE"
    exit 1
fi
```

## Maintenance

### Adding New URLs

Edit `test-urls.json`:

```json
{
  "id": "unique-test-id",
  "url": "https://example.com/page",
  "category": "category-name",
  "expected": {
    "has_title": true,
    "min_content_length": 500
  },
  "notes": "Description of test case"
}
```

### Cleaning Up

```bash
# Remove old results and logs
./scripts/run-all-tests.sh clean

# Keep last N sessions
find results/ -name "session-*.json" | sort -r | tail -n +11 | xargs rm -f
```

## Performance Benchmarks

Typical execution times:
- 30 URLs × 6 methods = 180 tests
- Average: 2-3 seconds per test
- Total: 6-9 minutes for complete suite
- With retries: 10-15 minutes worst case

## Contributing

When adding new extraction methods:

1. Update `METHODS` array in scripts
2. Ensure binary supports `--method <name>`
3. Run full test suite
4. Compare against existing methods
5. Document performance characteristics
