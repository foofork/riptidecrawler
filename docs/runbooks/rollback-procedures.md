# RipTide EventMesh Rollback Procedures

## Overview

This runbook provides step-by-step procedures for safely rolling back changes when golden tests detect regressions or system violations.

## Quick Reference

| Severity | Response Time | Auto-Rollback | Manual Steps |
|----------|---------------|---------------|--------------|
| Critical | Immediate | Yes | Verify + Monitor |
| High | < 5 minutes | Yes | Investigate + Fix |
| Medium | < 30 minutes | Optional | Plan Fix |
| Low | < 2 hours | No | Schedule Fix |

## Pre-Rollback Checklist

- [ ] Confirm golden test failures
- [ ] Check performance baseline violations
- [ ] Verify memory limit breaches
- [ ] Review error logs and patterns
- [ ] Identify affected components
- [ ] Estimate rollback impact

## Automated Rollback Triggers

### Critical Triggers (Immediate Auto-Rollback)

1. **Memory Limit Exceeded**: RSS > 600MB
2. **Performance Regression**: >5% degradation in P50/P95
3. **System Instability**: >5 failures in 10-minute window
4. **Core Functionality Broken**: Golden test failures

### Configuration

```json
{
  "rollback": {
    "auto_rollback_enabled": true,
    "rollback_threshold_failures": 5,
    "rollback_window_minutes": 10,
    "rollback_strategy": "gradual"
  }
}
```

## Manual Rollback Procedures

### 1. Immediate Rollback (Critical Issues)

```bash
# Step 1: Stop current deployment
git log --oneline -10  # Identify last good commit

# Step 2: Revert to last known good state
git checkout <last-good-commit>

# Step 3: Run golden tests to verify
cargo test --test golden_tests -- --nocapture

# Step 4: Deploy rollback
cargo build --release
systemctl restart riptide-eventmesh

# Step 5: Verify rollback success
curl -f http://localhost:8080/health
tail -f /var/log/riptide/application.log
```

### 2. Gradual Rollback (High/Medium Issues)

```bash
# Step 1: Enable safe mode
echo '{"safety": {"safe_mode": true}}' > config/feature-flags/runtime.json

# Step 2: Reduce traffic
# Update load balancer to route 10% traffic

# Step 3: Monitor performance
watch -n 5 'cargo run --bin golden-test-cli -- verify-against-baselines'

# Step 4: Full rollback if needed
if [[ $? -ne 0 ]]; then
  git checkout <last-good-commit>
  cargo build --release
  systemctl restart riptide-eventmesh
fi
```

### 3. Feature Flag Rollback

```bash
# Disable problematic features without code changes
vim config/feature-flags/runtime.json

# Example: Disable search provider integration
{
  "features": {
    "search_provider": {
      "enabled": false
    }
  }
}

# Restart service to pick up changes
systemctl restart riptide-eventmesh
```

## Rollback Verification

### Performance Verification

```bash
# Run complete golden test suite
cargo test --test golden_tests -- --test-threads=1

# Check specific performance metrics
grep -A 5 "Performance Summary" /var/log/golden-tests.log

# Verify memory usage
ps aux | grep riptide-eventmesh
cat /proc/$(pgrep riptide-eventmesh)/status | grep VmRSS
```

### Functional Verification

```bash
# Health check
curl -f http://localhost:8080/health

# API endpoints
curl -f http://localhost:8080/api/v1/status

# Search functionality
curl -X POST -H "Content-Type: application/json" \
  -d '{"query": "test"}' \
  http://localhost:8080/api/v1/search

# Event system
curl -X POST -H "Content-Type: application/json" \
  -d '{"event": "test_event"}' \
  http://localhost:8080/api/v1/events
```

## Post-Rollback Actions

### Immediate (0-30 minutes)

1. **Verify System Stability**
   ```bash
   # Monitor for 30 minutes
   watch -n 60 'curl -s http://localhost:8080/health'
   ```

2. **Check Performance Metrics**
   ```bash
   # Golden test verification
   cargo run --bin golden-test-cli -- verify-against-baselines
   ```

3. **Alert Stakeholders**
   - Send rollback notification
   - Update incident tracking
   - Schedule post-mortem

### Short-term (30 minutes - 2 hours)

1. **Root Cause Analysis**
   - Review failed golden tests
   - Analyze performance regression data
   - Check memory usage patterns
   - Review error logs

2. **Create Fix Plan**
   - Identify specific issues
   - Estimate fix complexity
   - Plan testing strategy
   - Schedule fix deployment

### Long-term (2+ hours)

1. **Implement Fix**
   - Code changes
   - Additional tests
   - Performance improvements

2. **Enhanced Testing**
   - Update golden test baselines if needed
   - Add regression tests
   - Improve monitoring

## Rollback Strategies

### 1. Blue-Green Deployment Rollback

```bash
# Switch traffic back to blue environment
# Update load balancer configuration
echo "upstream riptide {
  server blue-server:8080;
}" > /etc/nginx/conf.d/riptide.conf

nginx -s reload
```

### 2. Canary Rollback

```bash
# Reduce canary traffic to 0%
# Route all traffic to stable version
echo "weight 0" > /etc/consul/services/riptide-canary.json
consul reload
```

### 3. Database Migration Rollback

```bash
# If database changes are involved
# Use migration scripts with caution
cargo install diesel_cli
diesel migration revert
```

## Monitoring During Rollback

### Key Metrics to Watch

1. **Performance Metrics**
   - P50, P95, P99 latency
   - Throughput (requests/second)
   - Error rates

2. **Resource Metrics**
   - Memory usage (RSS)
   - CPU utilization
   - Disk I/O

3. **Business Metrics**
   - Page extraction success rate
   - Search functionality
   - Event processing

### Monitoring Commands

```bash
# Real-time performance monitoring
watch -n 5 'curl -s http://localhost:8080/metrics'

# Memory monitoring
watch -n 10 'ps aux | grep riptide-eventmesh'

# Log monitoring
tail -f /var/log/riptide/application.log | grep -E "ERROR|WARN"

# Golden test continuous monitoring
while true; do
  cargo run --bin golden-test-cli -- verify-against-baselines
  sleep 300  # Check every 5 minutes
done
```

## Emergency Contacts

| Role | Contact | Escalation Time |
|------|---------|----------------|
| Primary Engineer | On-call rotation | Immediate |
| System Admin | Infrastructure team | 15 minutes |
| Product Owner | Business stakeholder | 30 minutes |
| Architecture Team | Technical leadership | 1 hour |

## Troubleshooting Common Issues

### Issue: Golden Tests Still Failing After Rollback

```bash
# Check if baseline files are corrupted
file tests/benchmarks/baselines.json
jq . tests/benchmarks/baselines.json

# Regenerate baselines if needed
cargo run --bin golden-test-cli -- capture-baselines
```

### Issue: Performance Still Degraded

```bash
# Check system resources
top
free -h
df -h

# Check for resource leaks
lsof | grep riptide-eventmesh
netstat -tlnp | grep 8080
```

### Issue: Memory Usage Still High

```bash
# Force garbage collection
kill -USR1 $(pgrep riptide-eventmesh)

# Check for memory leaks
valgrind --tool=memcheck ./target/release/riptide-eventmesh
```

## Rollback Success Criteria

- [ ] All golden tests pass
- [ ] Performance within baseline thresholds
- [ ] Memory usage < 600MB
- [ ] No error spikes in logs
- [ ] API endpoints responding correctly
- [ ] Core functionality working
- [ ] Monitoring shows stable metrics

## Documentation Updates

After successful rollback:

1. Update incident log
2. Document lessons learned
3. Update rollback procedures if needed
4. Schedule team retrospective
5. Plan preventive measures

---

**Last Updated**: 2025-09-27
**Next Review**: 2025-10-27
**Owner**: RipTide Architecture Team