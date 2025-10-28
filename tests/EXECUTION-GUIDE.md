# Production Verification Execution Guide

## Quick Start

### One-Command Verification

```bash
/workspaces/eventmesh/tests/run-verification.sh
```

This wrapper script will:
1. ✅ Detect Python availability
2. ✅ Check if server is running
3. ✅ Start server if needed (via Docker Compose)
4. ✅ Execute comprehensive test suite
5. ✅ Generate detailed report

---

## Pre-Verification Checklist

### 1. Build Server

```bash
cd /workspaces/eventmesh

# Option A: Docker Compose (recommended)
docker-compose -f docker-compose.lite.yml build riptide-api

# Option B: Cargo (development)
cd crates/riptide-api
cargo build --release
```

### 2. Start Server

```bash
# Option A: Docker Compose (recommended)
docker-compose -f docker-compose.lite.yml up -d riptide-api

# Wait for readiness
sleep 15

# Option B: Cargo (development)
cd crates/riptide-api
cargo run --release &
sleep 10
```

### 3. Verify Server Health

```bash
# Test health endpoint
curl http://localhost:3000/health

# Expected output: 200 OK with health status

# Test metrics endpoint
curl http://localhost:3000/metrics | head -20

# Expected output: Prometheus-formatted metrics
```

---

## Execution Methods

### Method 1: Automated Wrapper (Recommended)

```bash
# Handles everything automatically
/workspaces/eventmesh/tests/run-verification.sh
```

**Features**:
- Auto-detects Python vs Bash
- Starts server if not running
- Shows real-time progress
- Generates comprehensive report

**Output Location**:
- Report: `/workspaces/eventmesh/tests/FINAL-PRODUCTION-VERIFICATION.md`
- Results: `/workspaces/eventmesh/tests/results/`

### Method 2: Python Test Suite

```bash
# Direct Python execution
python3 /workspaces/eventmesh/tests/production_verification.py
```

**Advantages**:
- Rich formatted output
- Detailed JSON results
- Statistical analysis
- Concurrent testing
- Better error reporting

**Requirements**:
- Python 3.7+
- `requests` library (usually available)

### Method 3: Bash Test Suite

```bash
# Direct Bash execution
/workspaces/eventmesh/tests/production-verification-suite.sh
```

**Advantages**:
- No Python dependencies
- Portable shell script
- CI/CD compatible
- Fast execution

**Requirements**:
- Bash 4.0+
- Standard Unix tools (curl, jq)

---

## Reading Results

### Quick Status Check

```bash
# View final score and recommendation
grep -A 5 "Overall Assessment" /workspaces/eventmesh/tests/FINAL-PRODUCTION-VERIFICATION.md

# View category scores
grep -E "^###.*\([0-9]+.*points\)" /workspaces/eventmesh/tests/FINAL-PRODUCTION-VERIFICATION.md
```

### Detailed Report

```bash
# View full report
cat /workspaces/eventmesh/tests/FINAL-PRODUCTION-VERIFICATION.md

# View in formatted markdown (if available)
mdcat /workspaces/eventmesh/tests/FINAL-PRODUCTION-VERIFICATION.md

# Open in VS Code
code /workspaces/eventmesh/tests/FINAL-PRODUCTION-VERIFICATION.md
```

### Individual Test Results

```bash
# List all results
ls /workspaces/eventmesh/tests/results/

# View extraction results
cat /workspaces/eventmesh/tests/results/extraction_static_simple.json | jq '.'

# View metrics snapshot
cat /workspaces/eventmesh/tests/results/metrics.txt | head -50

# View execution log
tail -100 /workspaces/eventmesh/tests/results/verification_*.log
```

---

## Interpreting Scores

### Score Ranges

| Score | Status | Meaning | Action |
|-------|--------|---------|--------|
| 90-100 | ✅ GO | Production ready | Deploy with confidence |
| 80-89 | ⚠️ CONDITIONAL GO | Minor issues | Review and address warnings |
| 70-79 | ⚠️ NO-GO | Significant issues | Fix critical failures |
| <70 | ❌ NO-GO | Critical issues | Major refactoring needed |

### Category Breakdown

1. **Full Extraction Workflow** (10 pts)
   - Tests: 10 diverse URLs
   - Pass criteria: 8+ successful extractions

2. **Observability Validation** (15 pts)
   - Tests: Logging, correlation IDs, metrics
   - Pass criteria: All observability features working

3. **Metrics Validation** (15 pts)
   - Tests: 5 metric families + labels
   - Pass criteria: All metrics present and accurate

4. **Response Metadata** (10 pts)
   - Tests: 4 required fields
   - Pass criteria: All fields present in responses

5. **Performance** (15 pts)
   - Tests: Response time, concurrency, memory
   - Pass criteria: Within targets (<5s, stable)

6. **Error Handling** (15 pts)
   - Tests: 4 error scenarios
   - Pass criteria: Graceful handling of all errors

7. **Production Readiness** (20 pts)
   - Tests: Health, metrics, docs, config
   - Pass criteria: All production requirements met

---

## Troubleshooting

### Server Won't Start

```bash
# Check if port is in use
lsof -i :3000

# Kill existing process
kill -9 $(lsof -t -i:3000)

# Check Docker logs
docker-compose -f docker-compose.lite.yml logs riptide-api

# Rebuild if needed
docker-compose -f docker-compose.lite.yml down
docker-compose -f docker-compose.lite.yml build --no-cache riptide-api
docker-compose -f docker-compose.lite.yml up -d riptide-api
```

### Tests Failing

```bash
# Check server health
curl -v http://localhost:3000/health

# Test single endpoint manually
curl -X POST http://localhost:3000/api/v1/scrape \
  -H "Content-Type: application/json" \
  -d '{"url": "http://example.com"}' | jq '.'

# Check server logs for errors
docker-compose -f docker-compose.lite.yml logs --tail=100 riptide-api

# Review detailed test logs
cat /workspaces/eventmesh/tests/results/verification_*.log
```

### Python Issues

```bash
# Check Python availability
python3 --version

# Install requests if needed
pip3 install requests

# Use Bash version instead
/workspaces/eventmesh/tests/production-verification-suite.sh
```

### Permission Issues

```bash
# Make scripts executable
chmod +x /workspaces/eventmesh/tests/*.sh
chmod +x /workspaces/eventmesh/tests/*.py

# Check file permissions
ls -la /workspaces/eventmesh/tests/
```

---

## CI/CD Integration

### GitHub Actions

```yaml
# .github/workflows/verification.yml
name: Production Verification

on:
  push:
    branches: [main, develop]
  pull_request:
  schedule:
    - cron: '0 0 * * *'  # Daily verification

jobs:
  verify:
    runs-on: ubuntu-latest
    timeout-minutes: 15

    steps:
      - name: Checkout
        uses: actions/checkout@v3

      - name: Build Server
        run: |
          docker-compose -f docker-compose.lite.yml build riptide-api

      - name: Start Server
        run: |
          docker-compose -f docker-compose.lite.yml up -d riptide-api
          sleep 15

      - name: Health Check
        run: |
          curl -f http://localhost:3000/health || exit 1

      - name: Run Verification
        run: |
          ./tests/run-verification.sh

      - name: Upload Report
        uses: actions/upload-artifact@v3
        if: always()
        with:
          name: verification-report-${{ github.sha }}
          path: |
            tests/FINAL-PRODUCTION-VERIFICATION.md
            tests/results/
          retention-days: 30

      - name: Comment on PR
        if: github.event_name == 'pull_request'
        uses: actions/github-script@v6
        with:
          script: |
            const fs = require('fs');
            const report = fs.readFileSync('tests/FINAL-PRODUCTION-VERIFICATION.md', 'utf8');
            const summary = report.split('---')[1]; // Extract executive summary
            github.rest.issues.createComment({
              issue_number: context.issue.number,
              owner: context.repo.owner,
              repo: context.repo.repo,
              body: `## Production Verification Results\n\n${summary}`
            });

      - name: Check Score
        run: |
          score=$(grep "Final Score" tests/FINAL-PRODUCTION-VERIFICATION.md | grep -oP '\d+' | head -1)
          echo "Score: $score/100"
          if [ "$score" -lt 80 ]; then
            echo "❌ Score below threshold"
            exit 1
          fi

      - name: Cleanup
        if: always()
        run: |
          docker-compose -f docker-compose.lite.yml down -v
```

### GitLab CI

```yaml
# .gitlab-ci.yml
stages:
  - build
  - test
  - report

build:
  stage: build
  script:
    - docker-compose -f docker-compose.lite.yml build riptide-api
  artifacts:
    paths:
      - docker-compose.yml

verify:
  stage: test
  script:
    - docker-compose -f docker-compose.lite.yml up -d riptide-api
    - sleep 15
    - ./tests/run-verification.sh
  artifacts:
    when: always
    paths:
      - tests/FINAL-PRODUCTION-VERIFICATION.md
      - tests/results/
    expire_in: 30 days
  retry:
    max: 2
    when: runner_system_failure

report:
  stage: report
  dependencies:
    - verify
  script:
    - echo "Generating verification badge..."
    - score=$(grep "Final Score" tests/FINAL-PRODUCTION-VERIFICATION.md | grep -oP '\d+' | head -1)
    - echo "VERIFICATION_SCORE=$score" >> build.env
  artifacts:
    reports:
      dotenv: build.env
```

---

## Production Deployment Workflow

### 1. Pre-Deployment Verification

```bash
# Run full verification
./tests/run-verification.sh

# Check score
score=$(grep "Final Score" tests/FINAL-PRODUCTION-VERIFICATION.md | grep -oP '\d+' | head -1)
echo "Score: $score/100"

# Manual review
if [ "$score" -ge 90 ]; then
  echo "✅ Ready for production"
else
  echo "⚠️ Review required"
fi
```

### 2. Staging Deployment

```bash
# Deploy to staging
docker-compose -f docker-compose.staging.yml up -d

# Wait for warmup
sleep 30

# Run verification against staging
API_BASE=https://staging.example.com python3 tests/production_verification.py

# Smoke test critical paths
curl https://staging.example.com/health
curl https://staging.example.com/metrics
```

### 3. Production Deployment

```bash
# Tag release
git tag -a v0.9.0 -m "Production release v0.9.0"
git push origin v0.9.0

# Deploy to production
docker-compose -f docker-compose.prod.yml up -d

# Monitor deployment
watch -n 5 'curl -s https://api.example.com/health | jq .'

# Run post-deployment verification
API_BASE=https://api.example.com python3 tests/production_verification.py
```

### 4. Post-Deployment Monitoring

```bash
# Check metrics
curl https://api.example.com/metrics | grep -E 'riptide_scrape_requests_total|riptide_scrape_duration'

# Monitor logs
docker-compose -f docker-compose.prod.yml logs -f --tail=100 riptide-api

# Set up alerts
# Configure Prometheus alerts for:
# - Error rate > 1%
# - Response time p95 > 5s
# - Memory usage > 80%
```

---

## Continuous Verification

### Daily Health Checks

```bash
# Add to crontab
0 0 * * * cd /workspaces/eventmesh && ./tests/run-verification.sh > /var/log/verification-$(date +\%Y\%m\%d).log 2>&1
```

### Performance Trending

```bash
# Track scores over time
echo "$(date +%Y-%m-%d),$score" >> tests/results/score-history.csv

# Generate trend report
awk -F, '{sum+=$2; count++} END {print "Average:", sum/count}' tests/results/score-history.csv
```

### Regression Detection

```bash
# Compare with baseline
baseline_score=95
current_score=$(grep "Final Score" tests/FINAL-PRODUCTION-VERIFICATION.md | grep -oP '\d+' | head -1)

if [ "$current_score" -lt "$baseline_score" ]; then
  echo "⚠️ Regression detected: $current_score < $baseline_score"
  # Trigger alert
fi
```

---

## Support & Feedback

### Documentation
- Verification Guide: `/workspaces/eventmesh/tests/README-VERIFICATION.md`
- Execution Guide: `/workspaces/eventmesh/tests/EXECUTION-GUIDE.md` (this file)
- Final Report: `/workspaces/eventmesh/tests/FINAL-PRODUCTION-VERIFICATION.md`

### Issues
- GitHub: Create issue with verification report attached
- Include: Score, failed tests, logs, environment details

### Improvements
- Suggest new test cases
- Report false positives/negatives
- Contribute test improvements via PR

---

**Quick Reference Commands**

```bash
# Run verification
./tests/run-verification.sh

# View report
cat tests/FINAL-PRODUCTION-VERIFICATION.md

# Check score
grep "Final Score" tests/FINAL-PRODUCTION-VERIFICATION.md

# View results
ls tests/results/

# Start server
docker-compose -f docker-compose.lite.yml up -d riptide-api

# Check health
curl http://localhost:3000/health

# View logs
docker-compose logs -f riptide-api
```

---

**Version**: 1.0.0
**Last Updated**: 2025-10-28
**EventMesh Version**: 0.9.0
