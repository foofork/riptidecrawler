# Week 1 Action Plan - Get It Working

**Goal**: Test with real URLs, wire everything up, launch-ready

---

## Day 1-2: Identify What's Actually Broken

### Task 1: Run Full Test Suite and Analyze
```bash
# Run all tests and capture results
cargo test --workspace 2>&1 | tee test-results.txt

# Analyze failures
grep "FAILED" test-results.txt
grep "test result:" test-results.txt
```

**Questions to answer**:
- Which 20 tests are failing?
- Are they critical bugs or environment issues?
- Can we skip/fix non-critical ones?

### Task 2: Real-World URL Testing
Create a test list of 20 diverse URLs:

```bash
# Create test-urls.txt
cat > test-urls.txt <<EOF
# News sites
https://www.bbc.com/news/technology
https://techcrunch.com/latest

# Blogs
https://martinfowler.com/articles/

# Documentation
https://docs.rust-lang.org/book/

# E-commerce
https://www.amazon.com/dp/B08N5WRWNW

# Social
https://dev.to/

# Complex SPAs
https://github.com/trending

# Simple static
https://example.com
EOF
```

**Test each URL**:
```bash
# Test extraction for each URL
while read url; do
  echo "Testing: $url"
  cargo run --bin riptide-cli extract "$url" --output json > "results/$(echo $url | md5sum | cut -d' ' -f1).json"
  echo "---"
done < test-urls.txt
```

**Look for**:
- Crashes or errors
- Empty extractions
- Timeouts
- Quality issues

---

## Day 3-4: Fix Critical Issues Only

### Priority 1: Crashes and Errors
Fix anything that causes:
- Panics
- HTTP 500 errors
- Timeouts on normal sites
- Empty results when content exists

### Priority 2: Wire Up Unused Metrics
Fix the compiler warnings:
```rust
// In pipeline.rs - add timing calls
let start = Instant::now();
// ... gate analysis ...
metrics.record_pipeline_phase_ms("gate_analysis", start.elapsed().as_millis() as u64);

let start = Instant::now();
// ... extraction ...
metrics.record_pipeline_phase_ms("extraction", start.elapsed().as_millis() as u64);
```

### Skip for Now:
- P1/P2 TODOs
- Code refactoring
- Test improvements for non-critical tests

---

## Day 5-6: Minimal Monitoring

### Docker Compose Stack
Create `deployment/docker-compose.monitoring.yml`:

```yaml
version: '3.8'

services:
  prometheus:
    image: prom/prometheus:latest
    ports:
      - "9090:9090"
    volumes:
      - ./prometheus.yml:/etc/prometheus/prometheus.yml
      - prometheus-data:/prometheus
    command:
      - '--config.file=/etc/prometheus/prometheus.yml'
      - '--storage.tsdb.retention.time=30d'

  grafana:
    image: grafana/grafana:latest
    ports:
      - "3000:3000"
    environment:
      - GF_SECURITY_ADMIN_PASSWORD=admin
    volumes:
      - grafana-data:/var/lib/grafana
      - ./grafana/provisioning:/etc/grafana/provisioning
    depends_on:
      - prometheus

volumes:
  prometheus-data:
  grafana-data:
```

### Prometheus Config
Create `deployment/prometheus.yml`:

```yaml
global:
  scrape_interval: 15s

scrape_configs:
  - job_name: 'riptide'
    static_configs:
      - targets: ['host.docker.internal:8080']
```

### One Simple Dashboard
Create `deployment/grafana/dashboards/riptide-overview.json`:

**Panels**:
1. Request rate (requests/sec)
2. Error rate (%)
3. Response latency (p50, p95, p99)
4. Extraction success rate by mode
5. Memory usage
6. Active requests

**One Alert**: Error rate >5% for 5 minutes

---

## Day 7: Smoke Test Everything

### Smoke Test Script
Create `scripts/smoke-test.sh`:

```bash
#!/bin/bash
set -e

echo "Starting RipTide API..."
cargo run --release --bin riptide-api &
API_PID=$!
sleep 5

echo "Running smoke tests..."

# Test 1: Health check
curl http://localhost:8080/health

# Test 2: Simple extraction
curl -X POST http://localhost:8080/extract \
  -H "Content-Type: application/json" \
  -d '{"url": "https://example.com"}'

# Test 3-22: Real URLs from test-urls.txt
while read url; do
  echo "Smoke testing: $url"
  curl -X POST http://localhost:8080/extract \
    -H "Content-Type: application/json" \
    -d "{\"url\": \"$url\"}" | jq '.success'
done < test-urls.txt

echo "All smoke tests passed!"
kill $API_PID
```

### Checklist
- [ ] API starts without errors
- [ ] Health endpoint responds
- [ ] Can extract from example.com
- [ ] Can extract from 20 diverse real URLs
- [ ] No crashes or panics
- [ ] Prometheus collecting metrics
- [ ] Grafana dashboard shows data

---

## End of Week 1 Checklist

### Must Have (Blocking Launch)
- [ ] API runs and responds to requests
- [ ] Extraction works on 90% of test URLs
- [ ] No critical bugs (crashes, data loss, security issues)
- [ ] Basic monitoring deployed (Prometheus + Grafana + 1 dashboard)
- [ ] Error rate <5% on test URLs

### Nice to Have (Not Blocking)
- [ ] 100% test pass rate (95% is fine)
- [ ] All compiler warnings fixed
- [ ] All TODO items resolved
- [ ] Perfect extraction quality

### Explicitly NOT Needed
- ❌ Refactored code
- ❌ Advanced dashboards
- ❌ A/B testing
- ❌ Threshold tuning
- ❌ CSS enhancements
- ❌ Complete documentation
- ❌ CI/CD pipeline

---

## Success Criteria

**Week 1 is successful if**:
1. You can extract content from real websites
2. The API doesn't crash
3. You have basic visibility (monitoring dashboard)
4. You know what the critical bugs are (if any)

**You can move to Week 2 even if**:
- Some tests fail (as long as they're not critical bugs)
- Code isn't perfectly organized
- Some features aren't polished
- Documentation is incomplete

---

## Quick Commands Reference

```bash
# Run API server
cargo run --release --bin riptide-api

# Test extraction
curl -X POST http://localhost:8080/extract \
  -H "Content-Type: application/json" \
  -d '{"url": "https://example.com"}'

# Run tests
cargo test --workspace

# Start monitoring stack
cd deployment
docker-compose -f docker-compose.monitoring.yml up -d

# View Grafana
open http://localhost:3000  # Login: admin/admin

# View Prometheus
open http://localhost:9090

# Check metrics
curl http://localhost:8080/metrics
```

---

## Daily Goals

**Monday**: Analyze test failures, identify critical bugs
**Tuesday**: Test 20 real URLs, document any issues
**Wednesday**: Fix critical bugs only
**Thursday**: Wire up unused metrics, fix warnings
**Friday**: Deploy monitoring stack
**Saturday**: Create simple dashboard and alert
**Sunday**: Smoke test everything, review for Week 2

---

## When to Ask for Help

**Critical Issues** (need immediate help):
- API crashes on startup
- Can't extract from ANY URLs
- Security vulnerabilities found
- Data corruption issues

**Non-Critical Issues** (can defer):
- Some tests failing
- Extraction quality could be better
- Code organization messy
- Missing features

---

## Remember

> "Perfect is the enemy of done. Ship a working product, iterate based on real usage."

You don't need:
- Perfect code
- Complete features
- 100% test coverage
- Comprehensive docs

You need:
- Working extraction
- Basic monitoring
- No critical bugs
- Ability to deploy

**Focus on getting to launch, not perfection.**
