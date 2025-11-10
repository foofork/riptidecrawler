# Riptide Roadmap Quality Gates

**Purpose**: Prevent "marked complete but not working" syndrome
**Philosophy**: "A sprint is only complete when the system builds, runs, passes tests, and can roll back safely."

---

## Overview

Quality gates are **mandatory checkpoints** that must pass before a sprint is considered complete. This document provides:

1. The 6 mandatory gates every sprint must pass
2. Detailed procedures for each gate
3. Per-sprint checklists
4. Common failure modes and remediation
5. Runtime validation quick reference

---

## The 6 Mandatory Quality Gates

### Gate 1: Builds in Both Modes
**Purpose**: Verify feature flags work correctly
**Pass Criteria**: Both `legacy-appstate` and `new-context` modes compile without errors

### Gate 2: Top Routes Run
**Purpose**: Ensure core functionality operational
**Pass Criteria**: 3 core endpoints return expected results

### Gate 3: All Ports Wired
**Purpose**: Validate dependency injection completeness
**Pass Criteria**: `ApplicationContext::validate()` passes

### Gate 4: Tests Pass
**Purpose**: Maintain code quality and coverage
**Pass Criteria**: Unit, integration, smoke tests all green; coverage not reduced

### Gate 5: Rollback Works
**Purpose**: Prove feature flags enable safe rollback
**Pass Criteria**: Feature flag flip restores legacy path in <5 minutes

### Gate 6: Docs Updated
**Purpose**: Keep documentation current
**Pass Criteria**: Dependency matrix and ADRs updated for sprint changes

---

## Gate 1: Builds in Both Modes

### Procedure

```bash
# Clean build environment
cargo clean

# Build legacy mode
cargo build --features legacy-appstate --release 2>&1 | tee build-legacy.log

# Verify success
if [ $? -eq 0 ]; then
    echo "✅ Legacy mode build: PASS"
else
    echo "❌ Legacy mode build: FAIL"
    exit 1
fi

# Build new-context mode
cargo build --features new-context --release 2>&1 | tee build-new-context.log

# Verify success
if [ $? -eq 0 ]; then
    echo "✅ New-context mode build: PASS"
else
    echo "❌ New-context mode build: FAIL"
    exit 1
fi

# Build with default features
cargo build --release 2>&1 | tee build-default.log

# Verify default is new-context (after Sprint 11)
cargo tree -e features | grep "new-context"
```

### Pass Criteria
- [ ] `legacy-appstate` build exits with code 0
- [ ] `new-context` build exits with code 0
- [ ] No warnings with `-D warnings`
- [ ] Build artifacts created

### Common Failures

**Failure**: "feature not found: legacy-appstate"
- **Cause**: Feature flag not defined in `Cargo.toml`
- **Fix**: Add `legacy-appstate = []` to `[features]`

**Failure**: "cannot find type `AppState` in this scope"
- **Cause**: Conditional compilation missing
- **Fix**: Wrap AppState with `#[cfg(feature = "legacy-appstate")]`

**Failure**: "conflicting implementations"
- **Cause**: Both feature flags enabled simultaneously
- **Fix**: Ensure features are mutually exclusive

### Checklist
- [ ] Builds pass in both modes
- [ ] No clippy warnings (`cargo clippy -- -D warnings`)
- [ ] No unused dependencies (`cargo machete`)
- [ ] Build logs reviewed for warnings

---

## Gate 2: Top Routes Run

### Procedure

```bash
# Start server in background
cargo run --release --features new-context &
SERVER_PID=$!

# Wait for server to be ready
sleep 5

# Test Route 1: /api/v1/crawl (POST)
CRAWL_RESPONSE=$(curl -s -w "\n%{http_code}" -X POST \
    http://localhost:3000/api/v1/crawl \
    -H "Content-Type: application/json" \
    -d '{"url":"https://example.com"}')

CRAWL_STATUS=$(echo "$CRAWL_RESPONSE" | tail -n1)
CRAWL_BODY=$(echo "$CRAWL_RESPONSE" | head -n-1)

if [ "$CRAWL_STATUS" = "200" ]; then
    echo "✅ /api/v1/crawl: PASS"
else
    echo "❌ /api/v1/crawl: FAIL (status: $CRAWL_STATUS)"
    echo "Response body: $CRAWL_BODY"
fi

# Test Route 2: /api/v1/extract (POST)
EXTRACT_RESPONSE=$(curl -s -w "\n%{http_code}" -X POST \
    http://localhost:3000/api/v1/extract \
    -H "Content-Type: application/json" \
    -d '{"url":"https://example.com"}')

EXTRACT_STATUS=$(echo "$EXTRACT_RESPONSE" | tail -n1)
EXTRACT_BODY=$(echo "$EXTRACT_RESPONSE" | head -n-1)

if [ "$EXTRACT_STATUS" = "200" ]; then
    echo "✅ /api/v1/extract: PASS"
else
    echo "❌ /api/v1/extract: FAIL (status: $EXTRACT_STATUS)"
    echo "Response body: $EXTRACT_BODY"
fi

# Test Route 3: /health (GET)
HEALTH_RESPONSE=$(curl -s -w "\n%{http_code}" http://localhost:3000/health)

HEALTH_STATUS=$(echo "$HEALTH_RESPONSE" | tail -n1)
HEALTH_BODY=$(echo "$HEALTH_RESPONSE" | head -n-1)

if [ "$HEALTH_STATUS" = "200" ]; then
    echo "✅ /health: PASS"
else
    echo "❌ /health: FAIL (status: $HEALTH_STATUS)"
    echo "Response body: $HEALTH_BODY"
fi

# Stop server
kill $SERVER_PID

# Verify all passed
if [ "$CRAWL_STATUS" = "200" ] && [ "$EXTRACT_STATUS" = "200" ] && [ "$HEALTH_STATUS" = "200" ]; then
    echo "✅ Gate 2: PASS (all routes operational)"
    exit 0
else
    echo "❌ Gate 2: FAIL (one or more routes failed)"
    exit 1
fi
```

### Pass Criteria
- [ ] `/api/v1/crawl` returns 200 OK
- [ ] `/api/v1/extract` returns 200 OK
- [ ] `/health` returns 200 OK
- [ ] Response payloads match schema

### Common Failures

**Failure**: "Connection refused"
- **Cause**: Server didn't start or crashed
- **Fix**: Check server logs for panic or errors

**Failure**: "500 Internal Server Error"
- **Cause**: Unhandled error in route handler
- **Fix**: Check server logs, add error handling

**Failure**: "Response schema mismatch"
- **Cause**: API contract changed without updating tests
- **Fix**: Update response types or fix handler

### Checklist
- [ ] All 3 routes return 200 OK
- [ ] Response bodies valid JSON
- [ ] Server logs show no errors
- [ ] Latency acceptable (<500ms)

---

## Gate 3: All Ports Wired

### Procedure

```rust
// Add to crates/riptide-api/src/composition/mod.rs

impl ApplicationContext {
    /// Validate that all required ports are wired and functional
    pub async fn validate(&self) -> Result<(), ContextError> {
        println!("🔍 Validating ApplicationContext...");

        // 1. Verify all ports exist (not None)
        self.verify_ports_exist()?;

        // 2. Health check each port
        self.health_check_all_ports().await?;

        // 3. Verify no circular dependencies
        self.verify_acyclic_dependencies()?;

        // 4. Check port configuration
        self.verify_port_configs()?;

        println!("✅ ApplicationContext validation: PASS");
        Ok(())
    }

    fn verify_ports_exist(&self) -> Result<(), ContextError> {
        // Check each port is initialized (not None)
        if self.browser_driver.is_none() {
            return Err(ContextError::MissingPort("browser_driver"));
        }
        if self.http_client.is_none() {
            return Err(ContextError::MissingPort("http_client"));
        }
        // ... check all 17 ports

        Ok(())
    }

    async fn health_check_all_ports(&self) -> Result<(), ContextError> {
        // Health check each port
        self.browser_driver.health_check().await
            .map_err(|e| ContextError::PortUnhealthy("browser_driver", e))?;

        self.http_client.health_check().await
            .map_err(|e| ContextError::PortUnhealthy("http_client", e))?;

        // ... health check all ports

        Ok(())
    }

    fn verify_acyclic_dependencies(&self) -> Result<(), ContextError> {
        // Use cargo tree to verify no cycles
        // This should be done in CI, not runtime
        Ok(())
    }

    fn verify_port_configs(&self) -> Result<(), ContextError> {
        // Verify port configurations are sane
        // e.g., rate limiter has non-zero quota
        // circuit breaker has reasonable threshold
        Ok(())
    }
}
```

Run validation:
```bash
# Integration test
cargo test --test context_validation

# Or in main.rs startup
./target/release/riptide-api --validate-context
```

### Pass Criteria
- [ ] `ApplicationContext::validate()` returns Ok(())
- [ ] All 17 ports initialized
- [ ] All port health checks pass
- [ ] No circular dependencies (cargo tree)
- [ ] Port configurations validated

### Common Failures

**Failure**: "Missing port: browser_driver"
- **Cause**: Builder didn't wire browser_driver
- **Fix**: Add `.with_browser_driver(Arc::new(adapter))`

**Failure**: "Port unhealthy: redis_cache"
- **Cause**: Redis not running or unreachable
- **Fix**: Start Redis or use in-memory adapter for dev

**Failure**: "Circular dependency detected"
- **Cause**: Port A depends on Port B, which depends on Port A
- **Fix**: Refactor to break cycle (use events, callbacks, etc.)

### Checklist
- [ ] Validation passes
- [ ] All ports healthy
- [ ] Dependency graph acyclic
- [ ] Port configs reasonable

---

## Gate 4: Tests Pass

### Procedure

```bash
# Run all tests
cargo test --workspace --all-features 2>&1 | tee test-results.log

# Parse results
TOTAL_TESTS=$(grep "test result:" test-results.log | awk '{print $3}')
PASSED=$(grep "test result:" test-results.log | awk '{print $4}')
FAILED=$(grep "test result:" test-results.log | awk '{print $6}')
IGNORED=$(grep "test result:" test-results.log | awk '{print $10}')

echo "Total: $TOTAL_TESTS"
echo "Passed: $PASSED"
echo "Failed: $FAILED"
echo "Ignored: $IGNORED"

# Check coverage (requires tarpaulin)
cargo tarpaulin --workspace --out Xml --output-dir coverage/

# Parse coverage
COVERAGE=$(grep -oP 'line-rate="\K[^"]+' coverage/cobertura.xml | head -1)
COVERAGE_PERCENT=$(echo "$COVERAGE * 100" | bc)

echo "Coverage: $COVERAGE_PERCENT%"

# Verify pass criteria
if [ "$FAILED" -eq 0 ] && [ "$IGNORED" -eq 0 ] && (( $(echo "$COVERAGE_PERCENT >= 90" | bc -l) )); then
    echo "✅ Gate 4: PASS"
    exit 0
else
    echo "❌ Gate 4: FAIL"
    echo "  Failed tests: $FAILED (must be 0)"
    echo "  Ignored tests: $IGNORED (must be 0)"
    echo "  Coverage: $COVERAGE_PERCENT% (must be ≥90%)"
    exit 1
fi
```

### Pass Criteria
- [ ] All tests pass (0 failures)
- [ ] No ignored tests (0 ignored)
- [ ] Coverage ≥90% (not reduced from baseline)
- [ ] No flaky tests (run 10x)

### Common Failures

**Failure**: "test panicked at 'assertion failed'"
- **Cause**: Logic error or broken test
- **Fix**: Debug test, fix logic, update assertions

**Failure**: "test ignored due to missing infrastructure"
- **Cause**: Test requires external service (Redis, Postgres)
- **Fix**: Use mocks instead of real infrastructure

**Failure**: "Coverage dropped from 85% to 78%"
- **Cause**: New code added without tests
- **Fix**: Write tests for new code

**Failure**: "Test flaky (passes sometimes, fails sometimes)"
- **Cause**: Race condition, timing issue
- **Fix**: Use deterministic timing (tokio::time::pause)

### Checklist
- [ ] All tests pass
- [ ] No ignored tests
- [ ] Coverage ≥90%
- [ ] Tests run in <5 minutes
- [ ] No flaky tests

---

## Gate 5: Rollback Works

### Procedure

```bash
# 1. Verify current mode (should be new-context after Sprint 11)
echo "Current mode:"
cargo tree -e features | grep -E "(legacy-appstate|new-context)"

# 2. Start server in new-context mode
cargo run --release --features new-context &
SERVER_PID=$!
sleep 5

# 3. Verify health in new-context mode
curl http://localhost:3000/health
# Expected: 200 OK

# 4. Perform rollback (flip feature flag)
kill $SERVER_PID

START_TIME=$(date +%s)

# Rebuild with legacy mode
cargo build --release --features legacy-appstate

# Restart server
cargo run --release --features legacy-appstate &
SERVER_PID=$!
sleep 5

END_TIME=$(date +%s)
ROLLBACK_TIME=$((END_TIME - START_TIME))

echo "Rollback time: ${ROLLBACK_TIME}s"

# 5. Verify health in legacy mode
HEALTH_STATUS=$(curl -s -w "%{http_code}" http://localhost:3000/health)

if [ "$HEALTH_STATUS" = "200" ]; then
    echo "✅ Rollback successful"
else
    echo "❌ Rollback failed (health check: $HEALTH_STATUS)"
fi

# 6. Verify rollback time <5 minutes
if [ "$ROLLBACK_TIME" -lt 300 ]; then
    echo "✅ Rollback time: ${ROLLBACK_TIME}s (<5 min)"
else
    echo "❌ Rollback time: ${ROLLBACK_TIME}s (>5 min)"
fi

# 7. Test critical routes in legacy mode
curl -X POST http://localhost:3000/api/v1/crawl \
    -H "Content-Type: application/json" \
    -d '{"url":"https://example.com"}'

# Expected: 200 OK (legacy mode works)

# Cleanup
kill $SERVER_PID

# Final verdict
if [ "$HEALTH_STATUS" = "200" ] && [ "$ROLLBACK_TIME" -lt 300 ]; then
    echo "✅ Gate 5: PASS (rollback works in <5 min)"
    exit 0
else
    echo "❌ Gate 5: FAIL"
    exit 1
fi
```

### Pass Criteria
- [ ] Feature flag flip completes in <5 minutes
- [ ] Server restarts successfully
- [ ] Health check passes in legacy mode
- [ ] Critical routes work in legacy mode
- [ ] No data loss during rollback
- [ ] No manual intervention required

### Common Failures

**Failure**: "Rollback takes >5 minutes"
- **Cause**: Rebuild too slow, or manual steps required
- **Fix**: Pre-build both modes, use feature flags not rebuilds

**Failure**: "Server won't start in legacy mode"
- **Cause**: Breaking change in data format
- **Fix**: Maintain backward compatibility, use migration

**Failure**: "Data lost during rollback"
- **Cause**: Incompatible database schema changes
- **Fix**: Use dual-write pattern during migration

### Checklist
- [ ] Rollback completes in <5 minutes
- [ ] Legacy mode functional
- [ ] No data loss
- [ ] No errors in logs
- [ ] Can roll forward again

---

## Gate 6: Docs Updated

### Procedure

```bash
# 1. Verify dependency matrix updated
git diff docs/architecture/DEPENDENCY-MATRIX.md

# Expected: Shows new ports/facades added this sprint

# 2. Verify ADRs written for new ports
ls docs/architecture/ADR-*.md | tail -3

# Expected: ADRs for sprint decisions (e.g., ADR-003-facade-context-dependency.md)

# 3. Verify migration guide current
git diff docs/guides/MIGRATION-GUIDE.md

# Expected: Updated with sprint changes

# 4. Check for stale TODOs
grep -r "TODO" docs/ --include="*.md"

# Expected: No stale TODOs

# 5. Verify examples compile
cd docs/examples/
cargo build --release

# Expected: All examples build successfully

# 6. Check broken links
markdown-link-check docs/**/*.md

# Expected: No broken links
```

### Pass Criteria
- [ ] Dependency matrix shows sprint changes
- [ ] ADRs written for new ports/facades
- [ ] Migration guide updated
- [ ] No stale TODOs in docs
- [ ] Code examples compile
- [ ] No broken links

### Common Failures

**Failure**: "Dependency matrix not updated"
- **Cause**: Forgot to update after adding port
- **Fix**: Update matrix with new port wiring

**Failure**: "No ADR for Sprint 5 decision"
- **Cause**: Didn't document architecture decision
- **Fix**: Write ADR (ADR-00X-{decision-name}.md)

**Failure**: "Example code doesn't compile"
- **Cause**: API changed but example not updated
- **Fix**: Update example to match current API

### Checklist
- [ ] Dependency matrix current
- [ ] ADRs for all sprint decisions
- [ ] Migration guide updated
- [ ] Examples compile
- [ ] README accurate
- [ ] No broken links

---

## Per-Sprint Quality Gate Checklist

Copy this checklist for each sprint:

```markdown
# Sprint [N] Quality Gate Checklist

**Sprint Goal**: [Description]
**Date**: [Date]
**Owner**: [Tech Lead]

## Pre-Gate Preparation
- [ ] Code freeze (no new features)
- [ ] All PRs merged and reviewed
- [ ] Changelog updated with sprint changes
- [ ] Team notified of gate execution

## Gate Execution

### Gate 1: Builds in Both Modes
- [ ] `cargo build --features legacy-appstate` ✅/❌
- [ ] `cargo build --features new-context` ✅/❌
- [ ] No warnings with `-D warnings` ✅/❌
- [ ] Build time: _____ seconds

### Gate 2: Top Routes Run
- [ ] `/api/v1/crawl` returns 200 ✅/❌
- [ ] `/api/v1/extract` returns 200 ✅/❌
- [ ] `/health` returns 200 ✅/❌
- [ ] Response payloads valid ✅/❌

### Gate 3: All Ports Wired
- [ ] `ApplicationContext::validate()` passes ✅/❌
- [ ] All [N] ports initialized ✅/❌
- [ ] Health checks pass ✅/❌
- [ ] No circular dependencies ✅/❌

### Gate 4: Tests Pass
- [ ] Total tests: _____
- [ ] Passed: _____
- [ ] Failed: _____ (must be 0)
- [ ] Ignored: _____ (must be 0)
- [ ] Coverage: _____% (must be ≥90%)

### Gate 5: Rollback Works
- [ ] Rollback time: _____ seconds (must be <300)
- [ ] Health check in legacy mode ✅/❌
- [ ] Routes work in legacy mode ✅/❌
- [ ] No data loss ✅/❌

### Gate 6: Docs Updated
- [ ] Dependency matrix updated ✅/❌
- [ ] ADRs written ✅/❌
- [ ] Migration guide current ✅/❌
- [ ] Examples compile ✅/❌

## Gate Results

**Overall Result**: ✅ PASS / ❌ FAIL

**Gates Passed**: ___ / 6

**Blockers** (if any):
- [List any gate failures]

**Remediation Plan** (if failed):
- [Steps to fix failures]

## Sign-off

- [ ] Tech Lead: _________________ Date: _______
- [ ] QA Lead: _________________ Date: _______
- [ ] Product Owner: _________________ Date: _______

## Next Steps

- [ ] Sprint retrospective scheduled
- [ ] Metrics dashboard updated
- [ ] Next sprint planning completed
- [ ] Team celebration (if PASS) 🎉
```

---

## Runtime Validation Quick Reference

### Quick Health Check (30 seconds)
```bash
# Start server
cargo run --release &

# Wait for startup
sleep 5

# Check health
curl http://localhost:3000/health

# Check metrics
curl http://localhost:3000/metrics | grep riptide_

# Check top routes
curl -X POST http://localhost:3000/api/v1/crawl \
    -d '{"url":"https://example.com"}'
```

### Quick Test Run (5 minutes)
```bash
# Run critical tests only
cargo test --test integration_tests --test facade_tests

# Run smoke tests
./scripts/smoke-tests.sh
```

### Quick Validation (1 minute)
```bash
# Build check
cargo check --workspace --all-features

# Port validation
cargo run -- --validate-context

# Dependency check
cargo tree -p riptide-api --depth 3 | grep circular
# Expected: 0 results
```

---

## Common Gate Failure Remediation

### If Gate 1 Fails (Builds)
1. Check compilation errors in logs
2. Verify feature flag syntax in Cargo.toml
3. Ensure conditional compilation correct (`#[cfg(feature = "...")]`)
4. Fix errors and re-run

### If Gate 2 Fails (Routes)
1. Check server logs for errors
2. Verify database/Redis are running
3. Check route handler logic
4. Verify request/response types match
5. Fix issues and restart server

### If Gate 3 Fails (Ports)
1. Check which port is missing
2. Verify builder wired port in main.rs
3. Check port adapter implementation
4. Verify health check logic
5. Wire missing ports and re-validate

### If Gate 4 Fails (Tests)
1. Identify failing tests
2. Debug test logic
3. Fix broken code or update test
4. Verify mocks are correct
5. Re-run tests until all pass

### If Gate 5 Fails (Rollback)
1. Check rollback time (optimize if >5 min)
2. Verify legacy mode compiles
3. Check for breaking changes
4. Ensure backward compatibility
5. Test rollback procedure again

### If Gate 6 Fails (Docs)
1. Update dependency matrix
2. Write missing ADRs
3. Update migration guide
4. Fix code examples
5. Check all links

---

## Automation Scripts

### Automated Gate Runner
```bash
#!/bin/bash
# scripts/run-quality-gates.sh

set -e  # Exit on first failure

echo "🚀 Running Quality Gates for Sprint $1"

# Gate 1
echo "📋 Gate 1: Builds in Both Modes"
./scripts/gate-1-builds.sh

# Gate 2
echo "📋 Gate 2: Top Routes Run"
./scripts/gate-2-routes.sh

# Gate 3
echo "📋 Gate 3: All Ports Wired"
./scripts/gate-3-ports.sh

# Gate 4
echo "📋 Gate 4: Tests Pass"
./scripts/gate-4-tests.sh

# Gate 5
echo "📋 Gate 5: Rollback Works"
./scripts/gate-5-rollback.sh

# Gate 6
echo "📋 Gate 6: Docs Updated"
./scripts/gate-6-docs.sh

echo "✅ All Quality Gates PASSED for Sprint $1"
```

### CI Integration
```yaml
# .github/workflows/quality-gates.yml
name: Quality Gates

on:
  pull_request:
    types: [labeled]

jobs:
  quality-gates:
    if: github.event.label.name == 'ready-for-gate-check'
    runs-on: ubuntu-latest

    steps:
      - uses: actions/checkout@v3

      - name: Setup Rust
        uses: actions-rs/toolchain@v1
        with:
          toolchain: stable

      - name: Run Quality Gates
        run: ./scripts/run-quality-gates.sh ${{ github.event.pull_request.number }}

      - name: Comment Results
        uses: actions/github-script@v6
        with:
          script: |
            github.rest.issues.createComment({
              issue_number: context.issue.number,
              owner: context.repo.owner,
              repo: context.repo.repo,
              body: '✅ All quality gates passed! Sprint ready for completion.'
            })
```

---

## Summary

**Remember**:
- All 6 gates MUST pass for sprint completion
- No exceptions, no shortcuts
- If a gate fails, remediate before proceeding
- Quality gates protect production from regressions
- Sprint is NOT done until all gates pass

**One-Line Philosophy**:
> "A sprint is only complete when the system builds, runs, passes tests, and can roll back safely."

---

**Version**: 2.0
**Last Updated**: 2025-11-10
**Status**: Active for all sprints (Week 0 - Sprint 16)
