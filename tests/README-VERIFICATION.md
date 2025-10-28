# EventMesh Production Verification Suite

Comprehensive end-to-end testing suite for validating all EventMesh improvements and production readiness.

## Overview

This verification suite provides automated testing across 7 critical categories with a 100-point scoring system to determine production readiness.

### Test Categories

1. **Full Extraction Workflow** (10 points)
   - Tests 10 diverse URLs across different site types
   - Validates extraction quality and format conversion
   - Covers static sites, SPAs, news sites, developer sites, international content

2. **Observability Validation** (15 points)
   - Structured logging verification
   - Request correlation ID tracking
   - Parser selection decision logging
   - Confidence score telemetry
   - Fallback event tracking

3. **Metrics Validation** (15 points)
   - Prometheus metrics endpoint validation
   - 5 metric families: requests, duration, parser selections, confidence scores, fallbacks
   - Label verification (strategy, path, outcome)
   - Counter and histogram validation

4. **Response Metadata Validation** (10 points)
   - `parser_used` field population
   - `confidence_score` presence
   - `fallback_occurred` tracking
   - `parse_time_ms` accuracy

5. **Performance Validation** (15 points)
   - Response time targets (<5s for simple pages)
   - Concurrent request handling (10 parallel requests)
   - Memory usage monitoring
   - Resource efficiency verification

6. **Error Handling Tests** (15 points)
   - Invalid URL handling
   - Missing parameter validation
   - Network timeout management
   - Unicode edge case handling

7. **Production Readiness Checks** (20 points)
   - Health endpoint validation
   - Metrics endpoint verification
   - Documentation completeness
   - Configuration templates
   - Docker setup validation
   - Log cleanliness (no critical warnings)

## Quick Start

### Prerequisites

```bash
# Ensure server is running (automatically started by scripts if needed)
cd /workspaces/eventmesh
docker-compose -f docker-compose.lite.yml up -d riptide-api

# Or using cargo
cd crates/riptide-api
cargo run --release
```

### Run Verification (Recommended: Python)

```bash
# Python version - Rich output and detailed reporting
python3 /workspaces/eventmesh/tests/production_verification.py

# View report
cat /workspaces/eventmesh/tests/FINAL-PRODUCTION-VERIFICATION.md
```

### Run Verification (Bash Alternative)

```bash
# Bash version - Works without Python dependencies
/workspaces/eventmesh/tests/production-verification-suite.sh

# View report
cat /workspaces/eventmesh/tests/FINAL-PRODUCTION-VERIFICATION.md
```

## Scoring System

- **90-100 points**: ✅ GO - Ready for production
- **80-89 points**: ⚠️  CONDITIONAL GO - Minor issues to address
- **70-79 points**: ⚠️  NO-GO - Significant issues
- **<70 points**: ❌ NO-GO - Critical issues

## Test Results

All test results are saved in `/workspaces/eventmesh/tests/results/`:

```
results/
├── extraction_*.json        # Individual URL extraction results
├── metadata_test.json       # Response metadata validation
├── metrics.txt              # Prometheus metrics snapshot
└── verification_*.log       # Detailed execution logs
```

## Output Files

### Primary Report

**Location**: `/workspaces/eventmesh/tests/FINAL-PRODUCTION-VERIFICATION.md`

**Contents**:
- Executive summary with final score
- Detailed test results by category
- Performance benchmarks
- Known issues
- Go/No-Go recommendation
- Production deployment checklist

### Detailed Logs

**Location**: `/workspaces/eventmesh/tests/results/verification_TIMESTAMP.log`

**Contents**:
- Full test execution trace
- Request/response details
- Timing information
- Error details

## Integration with CI/CD

### GitHub Actions Example

```yaml
name: Production Verification

on:
  push:
    branches: [main]
  pull_request:

jobs:
  verify:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3

      - name: Start EventMesh
        run: |
          docker-compose -f docker-compose.lite.yml up -d riptide-api
          sleep 10

      - name: Run Verification
        run: |
          python3 tests/production_verification.py

      - name: Upload Report
        uses: actions/upload-artifact@v3
        with:
          name: verification-report
          path: tests/FINAL-PRODUCTION-VERIFICATION.md

      - name: Check Score
        run: |
          if ! python3 tests/production_verification.py; then
            echo "❌ Verification failed"
            exit 1
          fi
```

## Manual Testing Guide

### 1. Start Fresh

```bash
# Clean previous results
rm -rf /workspaces/eventmesh/tests/results/*

# Ensure clean Docker state
docker-compose -f docker-compose.lite.yml down
docker-compose -f docker-compose.lite.yml up -d riptide-api
sleep 15
```

### 2. Verify Server Health

```bash
curl http://localhost:3000/health
# Expected: 200 OK

curl http://localhost:3000/metrics
# Expected: Prometheus metrics
```

### 3. Run Single Test Category

```python
# Edit production_verification.py to run single category
verifier = ProductionVerifier()
verifier.check_server()
verifier.test_extraction_workflow()  # Or any other test method
verifier.generate_report()
```

### 4. Custom URL Testing

```bash
# Test specific URL
curl -X POST http://localhost:3000/api/v1/scrape \
  -H "Content-Type: application/json" \
  -d '{
    "url": "https://your-test-url.com",
    "scrape_options": {
      "return_format": "markdown"
    }
  }' | jq '.'
```

## Troubleshooting

### Server Not Starting

```bash
# Check Docker logs
docker-compose -f docker-compose.lite.yml logs riptide-api

# Check port availability
lsof -i :3000

# Rebuild if needed
docker-compose -f docker-compose.lite.yml build --no-cache riptide-api
docker-compose -f docker-compose.lite.yml up -d riptide-api
```

### Python Dependencies

```bash
# Install required packages
pip3 install requests

# If not available, use bash version
/workspaces/eventmesh/tests/production-verification-suite.sh
```

### Failed Tests

1. **Check server logs**: `docker-compose logs riptide-api`
2. **Review detailed logs**: `cat tests/results/verification_*.log`
3. **Test individual endpoints**: Use curl commands from manual testing guide
4. **Verify configuration**: Check `.env` and `docker-compose.yml`

### Metrics Not Available

```bash
# Verify Prometheus endpoint
curl http://localhost:3000/metrics

# Check if metrics feature is enabled
grep -r "prometheus" crates/riptide-api/Cargo.toml
```

## Expected Test Duration

- Full suite: ~2-5 minutes
- Per category: ~15-60 seconds
- Depends on network conditions for URL fetching

## Success Criteria

For production deployment:

✅ **Required**:
- Score ≥ 90/100
- All 7 categories passing
- No failed tests
- Health and metrics endpoints active
- No critical warnings in logs

⚠️  **Acceptable with Review**:
- Score 80-89/100
- Maximum 2 minor test failures
- All critical paths working
- Known issues documented

❌ **Not Ready**:
- Score < 80/100
- Multiple critical test failures
- Health checks failing
- Excessive warnings in logs

## Continuous Improvement

### Adding New Tests

1. Add test method to `ProductionVerifier` class
2. Update `test_categories` list
3. Adjust scoring weights if needed
4. Update this README

### Metrics to Track Over Time

- Average response times
- Test pass rates
- Score trends
- Failed test patterns
- Performance regressions

## Support

- **Issues**: Create GitHub issue with verification report attached
- **Documentation**: See `/workspaces/eventmesh/docs/`
- **RipTide Team**: Contact for production deployment assistance

---

## Hooks Integration

This verification suite integrates with Claude Flow hooks:

```bash
# Pre-task hook
npx claude-flow@alpha hooks pre-task --description "Final production verification"

# Post-task hook
npx claude-flow@alpha hooks post-task --task-id "final-verification"

# Memory coordination
npx claude-flow@alpha hooks post-edit --file "tests/FINAL-PRODUCTION-VERIFICATION.md" \
  --memory-key "swarm/tester/verification-complete"
```

## Architecture

```
Production Verification Suite
│
├── production_verification.py     # Python implementation (recommended)
│   ├── Rich output formatting
│   ├── Detailed JSON results
│   ├── Concurrent testing
│   └── Statistical analysis
│
├── production-verification-suite.sh  # Bash implementation (portable)
│   ├── No dependencies
│   ├── Shell-based testing
│   └── Compatible with CI/CD
│
├── results/                       # Test artifacts
│   ├── extraction_*.json
│   ├── metadata_test.json
│   ├── metrics.txt
│   └── verification_*.log
│
└── FINAL-PRODUCTION-VERIFICATION.md  # Generated report
```

---

**Last Updated**: 2025-10-28
**Version**: 1.0.0
**EventMesh Version**: 0.9.0
