# Performance Baseline Metrics

## Overview
This document establishes baseline performance metrics for EventMesh/RipTide to track improvements and regressions over time.

**Last Updated:** 2025-10-17
**Version:** Phase 1 (Pre-optimization)

## Testing Methodology

### Environment
- **Hardware:** [To be recorded during actual test run]
- **OS:** Linux (Azure Cloud)
- **Rust Version:** Latest stable
- **Build:** Release mode with optimizations
- **Concurrency:** 10 concurrent requests

### Test Tools
- **drill:** HTTP load testing
- **cargo bench:** Rust benchmarks
- **hyperfine:** Command-line benchmarking

### Test Scenarios
1. **Health Check:** Simple endpoint validation
2. **Basic Render:** HTML rendering without screenshot
3. **Full Render:** HTML rendering with screenshot
4. **Extraction:** Content extraction with CSS selectors
5. **Stealth Render:** Maximum stealth protection enabled

## Baseline Metrics (To Be Measured)

### Response Time Targets

| Operation | Target (p50) | Target (p95) | Target (p99) |
|-----------|--------------|--------------|--------------|
| Health Check | <10ms | <20ms | <50ms |
| Simple Render | <500ms | <1000ms | <2000ms |
| Full Render | <1500ms | <3000ms | <5000ms |
| Extraction | <300ms | <800ms | <1500ms |
| Stealth Render | <2000ms | <4000ms | <6000ms |

### Throughput Targets

| Concurrency | Requests/sec | Success Rate | Error Rate |
|-------------|--------------|--------------|------------|
| 10 | TBD | >99% | <1% |
| 50 | TBD | >95% | <5% |
| 100 | TBD | >90% | <10% |

### Resource Usage Targets

| Metric | Target |
|--------|--------|
| Memory per request | <100MB |
| CPU per request | <50% single core |
| Browser instances | 10-20 concurrent |
| Connection pool | 100 connections |

## Current Performance (Pre-Phase 4)

### Known Bottlenecks
1. **Browser Concurrency:** Limited to 50-70 concurrent sessions with chromiumoxide
2. **Memory Usage:** 80-120MB per browser session
3. **Response Time:** 150-300ms overhead for basic operations
4. **Extraction:** Single-strategy fallback causing retries

### Technical Debt
- Synchronous operations in some code paths
- Inefficient error handling with retries
- No request caching or deduplication
- Limited connection pooling

## Performance Testing Commands

### Run Basic Load Test
```bash
./scripts/run_load_test.sh basic
```

### Run Stress Test
```bash
./scripts/run_load_test.sh stress
```

### Run All Tests
```bash
./scripts/run_load_test.sh all
```

### View Results
```bash
cat tests/load/results/report_*.md
```

## Benchmark Results

### To Be Recorded After First Run
This section will be populated with actual benchmark data from:
- drill load testing results
- cargo bench output
- System resource utilization

## Phase 4 Performance Goals

### After Optimization
Expected improvements from Phase 4 work:

| Metric | Current | Target | Improvement |
|--------|---------|--------|-------------|
| Concurrency | 50-70 | 200+ | +200% |
| Response Time | 150-300ms | <100ms | -50%+ |
| Memory/Session | 80-120MB | <50MB | -40%+ |
| Error Rate | 2-5% | <1% | -75%+ |

### Optimization Strategies
1. **Browser Pool:** Reuse browser instances
2. **Connection Pool:** Optimize HTTP connections
3. **Async Operations:** Convert sync operations to async
4. **Smart Caching:** Cache successful renders/extractions
5. **Request Deduplication:** Avoid duplicate concurrent requests
6. **Resource Management:** Better cleanup and recycling

## Monitoring and Alerts

### Key Metrics to Track
- **Response time percentiles** (p50, p95, p99)
- **Error rates** by endpoint
- **Memory usage** per request
- **CPU utilization** during load
- **Browser session count**
- **Queue depth** (if using job queue)

### Alert Thresholds
- Response time p99 > 5s
- Error rate > 5%
- Memory usage > 80% of available
- CPU usage > 80% sustained
- Browser crashes > 1% of sessions

## Test Data

### Sample URLs for Testing
```yaml
simple_html:
  - "https://example.com"
  - "https://httpbin.org/html"

dynamic_content:
  - "https://quotes.toscrape.com/js/"
  - "https://books.toscrape.com/"

complex_layouts:
  - "https://news.ycombinator.com"
  - "https://old.reddit.com"
```

### Test Scenarios
```rust
// Scenario 1: Basic rendering
{
    "url": "https://example.com",
    "wait_condition": "load",
    "screenshot_mode": "none"
}

// Scenario 2: Full-page screenshot
{
    "url": "https://example.com",
    "wait_condition": "networkidle",
    "screenshot_mode": "full",
    "stealth_level": "high"
}

// Scenario 3: Content extraction
{
    "url": "https://example.com",
    "mode": "css",
    "selectors": ["h1", "p", "a"]
}
```

## Regression Testing

### Continuous Performance Testing
- Run basic load test on every major commit
- Full performance suite weekly
- Stress testing before releases
- Automated alerts for regressions

### Performance Regression Criteria
- Response time increase >10%
- Error rate increase >2%
- Memory usage increase >15%
- Throughput decrease >10%

## Appendix

### How to Run Performance Tests

#### Prerequisites
```bash
# Install drill for load testing
cargo install drill

# Install hyperfine for benchmarking
cargo install hyperfine

# Build release version
cargo build --release
```

#### Basic Test
```bash
# Start API
cargo run --release -- serve &

# Wait for startup
sleep 5

# Run load test
drill --benchmark tests/load/basic_load_test.yml --stats

# Stop API
pkill riptide-cli
```

#### Benchmark Specific Operations
```bash
# Benchmark render operation
hyperfine --warmup 3 \
  'curl -X POST http://localhost:8080/api/v1/render \
   -H "Content-Type: application/json" \
   -d "{\"url\": \"https://example.com\"}"'
```

### Interpreting Results

#### Good Performance Indicators
- ✅ Response time p95 under target
- ✅ Error rate <1%
- ✅ Memory usage stable over time
- ✅ CPU usage scales linearly with load

#### Performance Issues
- ⚠️ Response time p95 2x target
- ⚠️ Error rate 1-5%
- ⚠️ Memory leaks (increasing over time)
- ⚠️ CPU usage non-linear with load

#### Critical Issues
- 🚨 Response time p95 >5x target
- 🚨 Error rate >10%
- 🚨 Out of memory errors
- 🚨 System crashes under load

---

**Note:** This document will be updated with actual performance data after the first comprehensive load test run. The current metrics represent targets and expectations based on architecture analysis.
