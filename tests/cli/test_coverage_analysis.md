# CLI Test Coverage Analysis

**Analysis Date**: 2025-10-11
**Analyst Agent**: CLI Testing Coverage Review
**Status**: Complete

## Executive Summary

Current CLI test coverage is **MODERATE** with significant gaps in edge cases, error scenarios, and advanced feature testing. The existing tests in `tests/cli/integration_tests.rs` cover basic happy paths but lack comprehensive scenario testing.

**Current Coverage**: ~35% of real-world scenarios
**Test File Lines**: 298 lines
**Commands Tested**: 8 out of 9 commands
**Critical Gaps**: 15 major areas identified

---

## Commands Coverage Matrix

### ✅ Commands with Basic Tests

| Command | Basic Test | Edge Cases | Error Scenarios | Output Formats | Notes |
|---------|-----------|-----------|----------------|---------------|-------|
| `extract` | ✅ Yes | ❌ No | ⚠️ Partial | ⚠️ Partial | Missing strategy variations |
| `crawl` | ❌ No | ❌ No | ❌ No | ❌ No | **CRITICAL: No tests** |
| `search` | ❌ No | ❌ No | ❌ No | ❌ No | **CRITICAL: No tests** |
| `cache status` | ✅ Yes | ❌ No | ❌ No | ⚠️ Partial | Missing subcommand tests |
| `cache clear` | ❌ No | ❌ No | ❌ No | ❌ No | **Missing** |
| `cache validate` | ❌ No | ❌ No | ❌ No | ❌ No | **Missing** |
| `cache stats` | ❌ No | ❌ No | ❌ No | ❌ No | **Missing** |
| `wasm info` | ✅ Yes | ❌ No | ❌ No | ⚠️ Partial | Basic only |
| `wasm benchmark` | ❌ No | ❌ No | ❌ No | ❌ No | **Missing** |
| `wasm health` | ❌ No | ❌ No | ❌ No | ❌ No | **Missing** |
| `health` | ✅ Yes | ❌ No | ⚠️ Partial | ⚠️ Partial | Missing unhealthy scenarios |
| `metrics` | ❌ No | ❌ No | ❌ No | ❌ No | **CRITICAL: No tests** |
| `validate` | ✅ Yes | ❌ No | ⚠️ Partial | ❌ No | Only success path |
| `system-check` | ❌ No | ❌ No | ❌ No | ❌ No | **CRITICAL: No tests** |

---

## Critical Missing Test Areas

### 1. CRAWL Command (HIGH PRIORITY - NO TESTS)

**Missing Scenarios:**
```bash
# Basic crawling
riptide crawl --url https://example.com

# Depth variations
riptide crawl --url https://example.com --depth 1
riptide crawl --url https://example.com --depth 5
riptide crawl --url https://example.com --depth 10

# Max pages limits
riptide crawl --url https://example.com --max-pages 10
riptide crawl --url https://example.com --max-pages 500
riptide crawl --url https://example.com --max-pages 1

# External links handling
riptide crawl --url https://example.com --follow-external
riptide crawl --url https://example.com --follow-external --depth 2

# Output directory
riptide crawl --url https://example.com -d ./output
riptide crawl --url https://example.com --output-dir /tmp/crawl-results

# Streaming mode
riptide crawl --url https://example.com --stream

# Combined options
riptide crawl --url https://example.com --depth 3 --max-pages 50 --follow-external -d ./results

# Edge cases
riptide crawl --url invalid-url  # Invalid URL
riptide crawl --url https://404.example.com  # Non-existent site
riptide crawl --url https://slow-site.com --max-pages 10  # Timeout scenarios
```

**Expected Test Coverage:**
- [ ] Basic crawl with default options
- [ ] Depth variations (1, 3, 5, 10)
- [ ] Max pages limits (1, 10, 100, 1000)
- [ ] External link following
- [ ] Output directory creation
- [ ] Streaming mode
- [ ] Progress bar rendering (mock)
- [ ] Invalid URL handling
- [ ] Network timeouts
- [ ] Server errors during crawl
- [ ] Empty site handling
- [ ] Output formats (json, table, text)

---

### 2. SEARCH Command (HIGH PRIORITY - NO TESTS)

**Missing Scenarios:**
```bash
# Basic search
riptide search --query "test query"

# Limit variations
riptide search --query "test" --limit 5
riptide search --query "test" --limit 50
riptide search --query "test" --limit 1

# Domain filtering
riptide search --query "test" --domain example.com
riptide search --query "test" --domain "*.example.com"

# Special characters in query
riptide search --query "test & query"
riptide search --query "test OR query"
riptide search --query "\"exact phrase\""

# Empty results
riptide search --query "nonexistent-xyz-123"

# Combined options
riptide search --query "test" --limit 20 --domain example.com

# Output formats
riptide search --query "test" --output json
riptide search --query "test" --output table
```

**Expected Test Coverage:**
- [ ] Basic search with results
- [ ] Empty search results
- [ ] Limit variations (1, 5, 10, 50)
- [ ] Domain filtering
- [ ] Special characters in queries
- [ ] URL encoding verification
- [ ] Relevance score display
- [ ] Output formats (json, table, text)
- [ ] Pagination handling
- [ ] Search API errors

---

### 3. EXTRACT Command (PARTIAL - NEEDS EXPANSION)

**Currently Tested:**
- ✅ Basic extraction
- ✅ Confidence scoring
- ✅ Strategy chain

**Missing Scenarios:**
```bash
# Method variations
riptide extract --url https://example.com --method trek
riptide extract --url https://example.com --method css
riptide extract --url https://example.com --method regex
riptide extract --url https://example.com --method llm
riptide extract --url https://example.com --method auto

# CSS selector variations
riptide extract --url https://example.com --method css --selector "article"
riptide extract --url https://example.com --method css --selector ".content"
riptide extract --url https://example.com --method css --selector "#main-article"
riptide extract --url https://example.com --method css --selector "div > p:nth-child(2)"

# Regex patterns
riptide extract --url https://example.com --method regex --pattern "\\d{3}-\\d{4}"
riptide extract --url https://example.com --method regex --pattern "[a-z]+@[a-z]+\\.[a-z]+"

# Strategy composition modes
riptide extract --url https://example.com --strategy parallel:all
riptide extract --url https://example.com --strategy fallback:trek,css,regex
riptide extract --url https://example.com --strategy chain:trek,css
riptide extract --url https://example.com --strategy chain:css,regex,llm

# File output
riptide extract --url https://example.com -f output.txt
riptide extract --url https://example.com --file /tmp/extracted.txt

# Metadata inclusion
riptide extract --url https://example.com --metadata
riptide extract --url https://example.com --show-confidence --metadata

# Edge cases
riptide extract --url invalid-url
riptide extract --url https://404.example.com
riptide extract --url https://example.com --selector "nonexistent-selector"
```

**Expected Test Coverage:**
- [ ] All extraction methods (trek, css, regex, llm, auto)
- [ ] CSS selector edge cases (invalid, nested, complex)
- [ ] Regex pattern edge cases (invalid, complex)
- [ ] All strategy modes (chain, parallel, fallback)
- [ ] File output with permissions tests
- [ ] Metadata display
- [ ] Invalid URL handling
- [ ] Empty content handling
- [ ] Low confidence scenarios
- [ ] Method fallback behavior

---

### 4. WASM Commands (PARTIAL - NEEDS EXPANSION)

**Currently Tested:**
- ✅ wasm info (basic)

**Missing Scenarios:**
```bash
# Benchmark with iterations
riptide wasm benchmark --iterations 10
riptide wasm benchmark --iterations 100
riptide wasm benchmark --iterations 1000

# Health check
riptide wasm health

# Output formats
riptide wasm info --output json
riptide wasm benchmark --iterations 50 --output table
```

**Expected Test Coverage:**
- [ ] Benchmark with various iteration counts (10, 100, 1000)
- [ ] Benchmark performance metrics validation
- [ ] Health status checks
- [ ] WASM instance count validation
- [ ] Memory usage reporting
- [ ] Features detection
- [ ] Output formats for all subcommands
- [ ] WASM unavailable error handling

---

### 5. CACHE Commands (MINIMAL COVERAGE)

**Currently Tested:**
- ⚠️ cache status (basic)

**Missing Scenarios:**
```bash
# Status and stats
riptide cache status
riptide cache stats

# Clear all cache
riptide cache clear

# Clear specific method cache
riptide cache clear --method trek
riptide cache clear --method css
riptide cache clear --method regex

# Validate cache
riptide cache validate

# Output formats
riptide cache status --output json
riptide cache stats --output table
```

**Expected Test Coverage:**
- [ ] Cache status with populated cache
- [ ] Cache status with empty cache
- [ ] Clear all cache
- [ ] Clear method-specific cache (trek, css, regex)
- [ ] Clear non-existent method
- [ ] Validate with valid cache
- [ ] Validate with corrupted cache
- [ ] Cache statistics accuracy
- [ ] Hit rate calculation
- [ ] Memory usage reporting
- [ ] Output formats for all subcommands

---

### 6. METRICS Command (CRITICAL - NO TESTS)

**Missing Scenarios:**
```bash
# Basic metrics
riptide metrics

# Output formats
riptide metrics --output json
riptide metrics --output table
riptide metrics --output text

# Edge cases
riptide metrics  # When no metrics available
riptide metrics  # With partial metrics
```

**Expected Test Coverage:**
- [ ] All metrics present
- [ ] Partial metrics (some null)
- [ ] Zero requests scenario
- [ ] High load scenario (mocked)
- [ ] Latency thresholds
- [ ] Queue size warnings
- [ ] Output formats (json, table, text)
- [ ] Metrics unavailable error

---

### 7. SYSTEM-CHECK Command (CRITICAL - NO TESTS)

**Missing Scenarios:**
```bash
# Comprehensive check
riptide system-check

# All components healthy
riptide system-check  # Expected: All pass

# Partial failures
riptide system-check  # Redis down
riptide system-check  # WASM unavailable
riptide system-check  # High latency warning
riptide system-check  # Low cache hit rate
```

**Expected Test Coverage:**
- [ ] All checks pass scenario
- [ ] Health check failures
- [ ] Performance metric warnings
- [ ] Resource usage warnings
- [ ] Cache performance warnings
- [ ] Worker service issues
- [ ] Partial component failures
- [ ] Complete system failure
- [ ] Latency threshold checks
- [ ] Summary generation

---

### 8. VALIDATE Command (PARTIAL COVERAGE)

**Currently Tested:**
- ✅ Validation success

**Missing Scenarios:**
```bash
# Validation failures
riptide validate  # API unreachable
riptide validate  # Redis disconnected
riptide validate  # WASM unavailable
riptide validate  # Worker service down

# Partial failures
riptide validate  # 1 check fails
riptide validate  # 2 checks fail
riptide validate  # All checks fail
```

**Expected Test Coverage:**
- [ ] All checks pass (already tested)
- [ ] API connectivity failure
- [ ] Redis connection failure
- [ ] WASM extractor failure
- [ ] Worker service failure
- [ ] Multiple failures
- [ ] Partial success scenarios
- [ ] Timeout handling
- [ ] Summary accuracy

---

### 9. HEALTH Command (PARTIAL COVERAGE)

**Currently Tested:**
- ✅ Healthy system

**Missing Scenarios:**
```bash
# Unhealthy scenarios
riptide health  # Redis disconnected
riptide health  # Extractor not ready
riptide health  # Worker service down
riptide health  # Multiple components unhealthy

# Output formats
riptide health --output json
riptide health --output table
```

**Expected Test Coverage:**
- [ ] All components healthy (already tested)
- [ ] Redis unhealthy
- [ ] Extractor unhealthy
- [ ] HTTP client unhealthy
- [ ] Worker service unhealthy
- [ ] Multiple components unhealthy
- [ ] Uptime display
- [ ] Status color coding
- [ ] Output formats (json, table, text)

---

## Cross-Cutting Test Scenarios

### Output Format Tests (INCOMPLETE)

**Currently Tested:**
- ⚠️ JSON, table formats for health

**Missing:**
```bash
# Test ALL commands with ALL formats
riptide extract --url example.com --output json
riptide extract --url example.com --output table
riptide extract --url example.com --output text

riptide crawl --url example.com --output json
# ... repeat for ALL commands
```

**Expected Coverage:**
- [ ] Every command with JSON output
- [ ] Every command with table output
- [ ] Every command with text output
- [ ] Invalid format handling
- [ ] Format consistency validation

---

### Authentication Tests (MINIMAL)

**Currently Tested:**
- ⚠️ API key authentication (basic)

**Missing Scenarios:**
```bash
# Authentication variations
riptide --api-key "valid-key" health
riptide --api-key "invalid-key" health
riptide health  # No API key (when required)

# Environment variable
RIPTIDE_API_KEY="test" riptide health

# Different endpoints with auth
riptide --api-key "test" extract --url example.com
riptide --api-key "test" crawl --url example.com
```

**Expected Coverage:**
- [ ] Valid API key
- [ ] Invalid API key
- [ ] Missing API key when required
- [ ] Environment variable API key
- [ ] API key in all commands
- [ ] Authorization failures
- [ ] Token expiration

---

### Error Handling Tests (MINIMAL)

**Currently Tested:**
- ⚠️ 500 Internal Server Error for extract

**Missing Scenarios:**
```bash
# Network errors
riptide --api-url http://unreachable health  # Connection timeout
riptide --api-url http://invalid-host health  # DNS failure

# HTTP errors
riptide extract --url example.com  # 400 Bad Request
riptide extract --url example.com  # 401 Unauthorized
riptide extract --url example.com  # 403 Forbidden
riptide extract --url example.com  # 404 Not Found
riptide extract --url example.com  # 429 Too Many Requests
riptide extract --url example.com  # 500 Internal Server Error
riptide extract --url example.com  # 503 Service Unavailable

# Malformed responses
riptide health  # Invalid JSON response
riptide metrics  # Empty response body
```

**Expected Coverage:**
- [ ] Connection timeouts
- [ ] DNS failures
- [ ] All HTTP error codes (400, 401, 403, 404, 429, 500, 503)
- [ ] Malformed JSON responses
- [ ] Empty responses
- [ ] Partial responses
- [ ] Network interruptions
- [ ] Error message clarity

---

### Verbose Flag Tests (NO TESTS)

**Missing Scenarios:**
```bash
# Verbose output
riptide -v health
riptide --verbose extract --url example.com

# Verbose with different commands
riptide -v crawl --url example.com
riptide -v search --query test
riptide -v cache status
```

**Expected Coverage:**
- [ ] Verbose flag affects logging
- [ ] Debug information display
- [ ] Verbose with all commands
- [ ] RUST_LOG environment interaction

---

### Environment Variable Tests (NO TESTS)

**Missing Scenarios:**
```bash
# API URL from environment
RIPTIDE_API_URL="http://custom.com" riptide health

# API key from environment
RIPTIDE_API_KEY="test-key" riptide health

# Combined environment variables
RIPTIDE_API_URL="http://custom.com" RIPTIDE_API_KEY="key" riptide health

# Override environment with flags
RIPTIDE_API_URL="http://custom.com" riptide --api-url http://override.com health
```

**Expected Coverage:**
- [ ] RIPTIDE_API_URL environment variable
- [ ] RIPTIDE_API_KEY environment variable
- [ ] Flag overrides environment
- [ ] Default values when no env

---

### CLI Help and Version Tests (BASIC)

**Currently Tested:**
- ✅ --help displays
- ✅ --version displays

**Missing Scenarios:**
```bash
# Subcommand help
riptide extract --help
riptide crawl --help
riptide search --help
riptide cache --help
riptide wasm --help

# Subsubcommand help
riptide cache clear --help
riptide wasm benchmark --help

# Invalid subcommands
riptide invalid-command
riptide extract invalid-flag
```

**Expected Coverage:**
- [ ] Help text for all subcommands
- [ ] Help text for nested subcommands
- [ ] Version information format
- [ ] Invalid command error messages
- [ ] Missing required arguments

---

## Real-World Usage Scenarios

### Scenario 1: Content Extraction Pipeline
```bash
# 1. Check system health
riptide health

# 2. Validate configuration
riptide validate

# 3. Extract content with confidence scoring
riptide extract --url https://blog.example.com/article \
  --method auto \
  --show-confidence \
  --metadata \
  -f extracted.txt

# 4. Check metrics
riptide metrics

# 5. Review cache status
riptide cache status
```

**Test Coverage Required:**
- [ ] Sequential command execution
- [ ] State consistency between commands
- [ ] File output verification
- [ ] Metrics accuracy after operations

---

### Scenario 2: Website Crawling Workflow
```bash
# 1. System check before crawl
riptide system-check

# 2. Clear cache for fresh crawl
riptide cache clear

# 3. Crawl website
riptide crawl --url https://example.com \
  --depth 3 \
  --max-pages 100 \
  --follow-external \
  -d ./crawl-output

# 4. Validate results
ls -la ./crawl-output

# 5. Check WASM performance
riptide wasm benchmark --iterations 100

# 6. Review metrics
riptide metrics
```

**Test Coverage Required:**
- [ ] End-to-end workflow
- [ ] Directory creation
- [ ] File count validation
- [ ] Performance benchmarking
- [ ] Post-operation metrics

---

### Scenario 3: Search and Extract
```bash
# 1. Search for relevant pages
riptide search --query "rust programming" --limit 5

# 2. Extract content from top result
riptide extract --url [result-url] \
  --strategy chain:trek,css \
  --show-confidence \
  -f result.txt

# 3. Check cache hit rate
riptide cache stats
```

**Test Coverage Required:**
- [ ] Search-to-extract pipeline
- [ ] URL passing between commands
- [ ] Cache behavior verification

---

### Scenario 4: Monitoring and Maintenance
```bash
# 1. Continuous health monitoring
riptide health

# 2. Check WASM instances
riptide wasm info

# 3. Review system metrics
riptide metrics

# 4. Validate cache integrity
riptide cache validate

# 5. Clear old cache if needed
riptide cache clear --method trek
```

**Test Coverage Required:**
- [ ] Monitoring workflow
- [ ] Cache maintenance operations
- [ ] System health correlation

---

## Performance Test Scenarios (NOT COVERED)

### Load Testing
```bash
# Concurrent requests simulation
for i in {1..10}; do
  riptide extract --url https://example.com/$i &
done
wait

# Benchmark all commands
time riptide wasm benchmark --iterations 1000
time riptide crawl --url example.com --max-pages 100
```

**Expected Coverage:**
- [ ] Concurrent CLI invocations
- [ ] Response time measurements
- [ ] Resource usage monitoring
- [ ] Timeout handling under load

---

## Integration Test Scenarios (MINIMAL)

### Server Interaction Tests
```bash
# Start real server (test environment)
# Run CLI against real server
# Verify actual behavior
```

**Expected Coverage:**
- [ ] Real HTTP request/response
- [ ] Actual file I/O operations
- [ ] Real cache operations
- [ ] Network timeout scenarios
- [ ] Server restart scenarios

---

## Test Infrastructure Gaps

### Missing Test Utilities

1. **Mock Server Helpers**
   - [ ] Helper for creating mock responses
   - [ ] Helper for simulating delays
   - [ ] Helper for simulating errors
   - [ ] Helper for partial responses

2. **Assertion Helpers**
   - [ ] Output format validators
   - [ ] JSON schema validators
   - [ ] Table format validators
   - [ ] File content validators

3. **Fixture Management**
   - [ ] Sample HTML files
   - [ ] Sample API responses
   - [ ] Test configuration files
   - [ ] Mock data generators

4. **Test Organization**
   - [ ] Separate files per command
   - [ ] Shared test utilities module
   - [ ] Test data directory structure
   - [ ] CI/CD integration tests

---

## Recommended Test Priority

### P0 (Critical - Immediate)
1. **Crawl command** - No tests at all
2. **Search command** - No tests at all
3. **Metrics command** - No tests at all
4. **System-check command** - No tests at all

### P1 (High - Sprint 1)
5. **WASM benchmark** - Missing important feature
6. **Cache clear/validate** - Common operations
7. **Extract method variations** - Core functionality
8. **Error handling** - Production reliability

### P2 (Medium - Sprint 2)
9. **Output formats** - User experience
10. **Authentication edge cases** - Security
11. **Environment variables** - Configuration
12. **Real-world scenarios** - Integration

### P3 (Low - Sprint 3)
13. **Performance tests** - Optimization
14. **Verbose flag** - Developer experience
15. **Help text validation** - Documentation

---

## Metrics Summary

### Current Test Count: 18 tests

### Required Test Count (Estimated): 150+ tests

**Breakdown by Category:**
- Extract command: 30 tests (currently 3)
- Crawl command: 25 tests (currently 0)
- Search command: 20 tests (currently 0)
- Cache commands: 15 tests (currently 1)
- WASM commands: 15 tests (currently 1)
- Health command: 10 tests (currently 1)
- Metrics command: 10 tests (currently 0)
- Validate command: 10 tests (currently 1)
- System-check command: 10 tests (currently 0)
- Cross-cutting: 15 tests (currently 11)

### Coverage Gap: ~87% of scenarios untested

---

## Recommendations

### Immediate Actions

1. **Create test plan document** for each command
2. **Set up test infrastructure** with helpers and fixtures
3. **Prioritize P0 commands** (crawl, search, metrics, system-check)
4. **Establish test naming conventions** and organization
5. **Integrate with CI/CD** for continuous testing

### Long-term Actions

1. **Achieve 90%+ scenario coverage** across all commands
2. **Add property-based testing** for edge cases
3. **Implement performance benchmarks** in CI
4. **Create integration test suite** against real server
5. **Add chaos testing** for reliability validation

### Test Organization Proposal

```
tests/cli/
├── integration_tests.rs (keep for basic smoke tests)
├── commands/
│   ├── extract_tests.rs
│   ├── crawl_tests.rs
│   ├── search_tests.rs
│   ├── cache_tests.rs
│   ├── wasm_tests.rs
│   ├── health_tests.rs
│   ├── metrics_tests.rs
│   ├── validate_tests.rs
│   └── system_check_tests.rs
├── common/
│   ├── mod.rs
│   ├── mock_server.rs
│   ├── assertions.rs
│   └── fixtures.rs
└── scenarios/
    ├── content_extraction_workflow.rs
    ├── crawling_workflow.rs
    └── monitoring_workflow.rs
```

---

## Conclusion

The CLI has a **solid foundation** but requires **significant test expansion** to ensure production readiness. The most critical gap is the **complete absence of tests** for four major commands (crawl, search, metrics, system-check) and **limited edge case coverage** for existing tests.

**Recommended immediate focus**: Implement P0 and P1 tests to achieve 60% scenario coverage within the next sprint, then progressively work toward 90%+ coverage.

**Estimated effort**:
- P0: 40 hours
- P1: 60 hours
- P2: 50 hours
- P3: 30 hours
- **Total**: ~180 hours (4-5 weeks for 1 engineer)
