# EventMesh Operations Runbook

**Version:** 1.0.0
**Last Updated:** 2025-10-17
**Owner:** DevOps Engineer

## Overview

This runbook provides operational procedures for deploying, monitoring, and troubleshooting EventMesh services.

## Table of Contents

1. [Quick Reference](#quick-reference)
2. [Deployment Procedures](#deployment-procedures)
3. [Monitoring](#monitoring)
4. [Health Checks](#health-checks)
5. [Troubleshooting](#troubleshooting)
6. [Rollback Procedures](#rollback-procedures)
7. [Performance Optimization](#performance-optimization)
8. [Security](#security)
9. [Disaster Recovery](#disaster-recovery)

---

## Quick Reference

### Critical Commands

```bash
# Deploy to staging
./scripts/deploy.sh staging

# Monitor health (local)
./scripts/monitor_health.sh local 30

# Monitor resources
./scripts/monitor_resources.sh 300 5

# Generate metrics report
./scripts/generate_metrics_report.sh 10

# Check CI/CD status
gh workflow list
gh run list --workflow=ci.yml --limit 5
```

### Service URLs by Environment

| Environment | Health Check | API | Metrics |
|-------------|--------------|-----|---------|
| **Local** | http://localhost:8080/healthz | http://localhost:8080/api/v1 | http://localhost:8080/metrics |
| **Dev** | http://dev.eventmesh.internal:8080/healthz | http://dev.eventmesh.internal:8080/api/v1 | http://dev.eventmesh.internal:8080/metrics |
| **Staging** | http://staging.eventmesh.internal:8080/healthz | http://staging.eventmesh.internal:8080/api/v1 | http://staging.eventmesh.internal:8080/metrics |
| **Production** | https://api.eventmesh.com/healthz | https://api.eventmesh.com/api/v1 | https://api.eventmesh.com/metrics |

---

## Deployment Procedures

### Pre-Deployment Checklist

- [ ] All tests passing on CI/CD
- [ ] Code review approved
- [ ] Security scan passed
- [ ] Performance benchmarks acceptable
- [ ] Database migrations tested
- [ ] Rollback plan prepared
- [ ] Stakeholders notified

### Standard Deployment

#### 1. Development Environment

```bash
# Automated deployment
./scripts/deploy.sh dev

# Or manual steps
cargo build --release
cargo test --release
scp target/release/riptide-* user@dev.eventmesh.internal:/opt/eventmesh/dev/bin/
ssh user@dev.eventmesh.internal "sudo systemctl restart eventmesh-api"
```

#### 2. Staging Environment

```bash
# Run deployment script
./scripts/deploy.sh staging

# Verify deployment
./scripts/monitor_health.sh staging 2

# Run smoke tests
cargo test --release --test integration_tests
```

#### 3. Production Environment

```bash
# Production deployment requires approval
./scripts/deploy.sh production

# Monitor closely
./scripts/monitor_health.sh production 5

# Verify all services healthy
curl https://api.eventmesh.com/healthz
curl https://api.eventmesh.com/metrics
```

### Deployment Options

```bash
# Dry run (test without deploying)
DRY_RUN=true ./scripts/deploy.sh staging

# Skip tests (emergency deployment)
SKIP_TESTS=true ./scripts/deploy.sh staging

# Disable backup
BACKUP_ENABLED=false ./scripts/deploy.sh dev
```

---

## Monitoring

### Continuous Monitoring

#### Health Monitoring

```bash
# Start continuous health monitoring
./scripts/monitor_health.sh local 30
./scripts/monitor_health.sh staging 30
./scripts/monitor_health.sh production 15
```

**Monitoring includes:**
- HTTP status codes
- Response times
- Consecutive failure tracking
- Metrics endpoint availability

#### Resource Monitoring

```bash
# Monitor resources during operations
./scripts/monitor_resources.sh 300 5  # 5 min, 5 sec intervals

# Long-term monitoring (1 hour)
./scripts/monitor_resources.sh 3600 10
```

**Tracks:**
- CPU utilization
- Memory usage
- Disk I/O
- Top processes

### CI/CD Metrics

```bash
# Generate metrics report
./scripts/generate_metrics_report.sh 10  # Last 10 builds

# View metrics artifacts
ls -lah metrics/

# Check specific metrics
cat metrics/build_metrics.csv
cat metrics/test_metrics.csv
cat metrics/cache_hits.csv
```

### Key Metrics to Monitor

| Metric | Normal | Warning | Critical |
|--------|--------|---------|----------|
| Health check response | <500ms | 500ms-2s | >2s or failures |
| API response time | <200ms | 200ms-1s | >1s |
| Memory usage | <70% | 70-85% | >85% |
| CPU usage | <60% | 60-80% | >80% |
| Error rate | <1% | 1-5% | >5% |
| Build time | <30min | 30-40min | >40min |

---

## Health Checks

### Manual Health Verification

```bash
# Basic health check
curl -i http://localhost:8080/healthz

# Expected response:
# HTTP/1.1 200 OK
# {"status": "healthy", "version": "0.1.0"}

# Detailed status
curl -i http://localhost:8080/api/v1/status

# Metrics
curl http://localhost:8080/metrics
```

### Service Health Indicators

#### Healthy Service
- HTTP 200 response
- Response time <500ms
- All dependencies available
- No error logs
- CPU <60%, Memory <70%

#### Degraded Service
- HTTP 200 but slow (>1s)
- High resource usage
- Some non-critical errors
- Requires monitoring

#### Unhealthy Service
- HTTP 5xx responses
- Timeouts or no response
- Critical errors in logs
- Resource exhaustion

---

## Troubleshooting

### Common Issues

#### 1. Build Failures

**Symptoms:** CI/CD pipeline fails during build

```bash
# Check compilation errors
cargo build --all --release 2>&1 | tee build_error.log
grep "error:" build_error.log

# Common fixes:
cargo clean                    # Clear build cache
cargo update                   # Update dependencies
cargo check --all-targets     # Check all targets
```

**Current Known Issues:**
- `riptide-cli`: Missing struct field initialization
- `riptide-api`: Compilation errors
- Action: See error logs for specific fixes needed

#### 2. Test Failures

**Symptoms:** Tests fail during CI/CD

```bash
# Run tests with verbose output
cargo test --verbose 2>&1 | tee test_error.log

# Run specific failing test
cargo test --test <test_name> -- --nocapture

# Check environment
echo $REDIS_URL
echo $RUST_LOG
env | grep -E "(REDIS|API|RUST)"
```

#### 3. Deployment Failures

**Symptoms:** Deployment script exits with error

```bash
# Check deployment logs
tail -f logs/deploy_*.log

# Verify package creation
ls -lh eventmesh-*.tar.gz

# Test package contents
tar -tzf eventmesh-*.tar.gz

# Common issues:
- Missing binaries → Check build succeeded
- Permission denied → Check SSH keys and permissions
- Health check fails → Check service logs
```

#### 4. Health Check Failures

**Symptoms:** Service doesn't respond to health endpoint

```bash
# Check if service is running
ps aux | grep riptide

# Check port binding
netstat -tlnp | grep 8080
# or
lsof -i :8080

# Check logs
tail -f /var/log/eventmesh/api.log

# Test local connection
curl -v http://localhost:8080/healthz

# Common issues:
- Port already in use → Kill conflicting process
- Service crashed → Check logs for panic/error
- Network issues → Check firewall rules
```

#### 5. Performance Issues

**Symptoms:** Slow response times, high resource usage

```bash
# Monitor resources in real-time
./scripts/monitor_resources.sh 60 1

# Check for resource bottlenecks
top -H    # CPU per thread
free -h   # Memory usage
iostat -x # Disk I/O

# Profile the application
cargo build --release
perf record -F 99 -g ./target/release/riptide-api
perf report

# Common causes:
- Memory leak → Check memory growth over time
- CPU bottleneck → Profile hot paths
- I/O bottleneck → Check disk performance
- Connection exhaustion → Check connection pool
```

### Error Log Analysis

```bash
# Search for errors
grep -i "error\|panic\|fatal" /var/log/eventmesh/*.log

# Count error types
grep -i "error" /var/log/eventmesh/api.log | awk '{print $5}' | sort | uniq -c

# Check for patterns
tail -f /var/log/eventmesh/api.log | grep -E "(ERROR|WARN)"
```

### Debug Mode

```bash
# Run with debug logging
RUST_LOG=debug cargo run --release --bin riptide-api

# Specific module debugging
RUST_LOG=riptide_api=trace,riptide_core=debug cargo run

# Enable backtrace
RUST_BACKTRACE=full cargo run
```

---

## Rollback Procedures

### Quick Rollback

```bash
# Stop current version
ssh user@$DEPLOY_HOST "sudo systemctl stop eventmesh-api"

# Restore from backup
ssh user@$DEPLOY_HOST "cd /opt/eventmesh/backups && \
  tar -xzf eventmesh-backup-<timestamp>.tar.gz -C /opt/eventmesh/staging"

# Restart service
ssh user@$DEPLOY_HOST "sudo systemctl start eventmesh-api"

# Verify health
./scripts/monitor_health.sh staging 2
```

### Rollback Checklist

1. [ ] Stop affected services
2. [ ] Restore previous version from backup
3. [ ] Verify configuration compatibility
4. [ ] Restart services
5. [ ] Run health checks
6. [ ] Monitor for 10-15 minutes
7. [ ] Notify stakeholders
8. [ ] Document incident

---

## Performance Optimization

### Build Performance

```bash
# Clear build cache
cargo clean

# Build with optimized settings
CARGO_BUILD_JOBS=8 cargo build --release

# Use sccache (if available)
export RUSTC_WRAPPER=sccache
cargo build --release
```

### Test Performance

```bash
# Increase test parallelism
cargo test -- --test-threads=8

# Run specific test suites
cargo test --test integration_tests
cargo test --lib  # Only unit tests

# Skip slow tests
cargo test -- --skip slow_test
```

### Runtime Performance

```bash
# CPU profiling
perf record -F 99 -g ./target/release/riptide-api
perf report

# Memory profiling
valgrind --tool=massif ./target/release/riptide-api

# Benchmarking
cargo bench --package riptide-api
```

---

## Security

### Security Checklist

- [ ] No secrets in code or logs
- [ ] All dependencies up to date
- [ ] Security audit passed (cargo-audit)
- [ ] No unsafe code without SAFETY comments
- [ ] Input validation on all endpoints
- [ ] Rate limiting configured
- [ ] TLS/HTTPS enabled (production)
- [ ] Regular security scans scheduled

### Security Scans

```bash
# Dependency audit
cargo audit

# Check for vulnerabilities
cargo deny check advisories

# Unsafe code audit
rg "unsafe" --type rust crates/ wasm/ | grep -v "SAFETY:"

# OWASP ZAP scan (API)
# See .github/workflows/api-validation.yml
```

### Secret Management

```bash
# Never commit secrets
git secrets --scan

# Use environment variables
export DATABASE_URL="postgresql://..."
export REDIS_URL="redis://..."
export API_KEY="..."

# Rotate secrets regularly (every 90 days recommended)
```

---

## Disaster Recovery

### Backup Strategy

**Automated Backups:**
- Code: Git repository (GitHub)
- Build artifacts: CI/CD artifacts (30 days retention)
- Configuration: Version controlled
- Logs: Retained for 90 days

**Manual Backups:**
```bash
# Backup current deployment
ssh user@$DEPLOY_HOST "cd /opt/eventmesh/production && \
  tar -czf /opt/eventmesh/backups/manual-backup-$(date +%Y%m%d).tar.gz ."
```

### Recovery Procedures

#### Complete System Failure

1. Identify failure scope and cause
2. Check backup availability
3. Provision new infrastructure (if needed)
4. Restore from most recent backup
5. Verify system health
6. Resume normal operations
7. Post-mortem analysis

#### Data Corruption

1. Stop affected services immediately
2. Assess corruption extent
3. Restore from backup
4. Verify data integrity
5. Replay transactions if applicable
6. Resume operations
7. Implement prevention measures

### Contact Information

**Emergency Contacts:**
- DevOps Team: devops@company.com
- On-call Engineer: +1-XXX-XXX-XXXX
- Security Team: security@company.com
- Management: ops-manager@company.com

**Escalation Path:**
1. On-call DevOps Engineer
2. DevOps Team Lead
3. Engineering Manager
4. CTO

---

## Appendix

### Useful Commands

```bash
# Cargo commands
cargo build --release              # Build release
cargo test --workspace             # Run all tests
cargo clippy --all-targets         # Linting
cargo audit                        # Security audit
cargo bloat --release --crates     # Binary size analysis

# Git commands
git log --oneline -10              # Recent commits
git diff main..HEAD                # Changes vs main
git describe --tags --always       # Current version

# System commands
df -h                              # Disk usage
free -h                            # Memory usage
ps aux | grep riptide              # Running processes
netstat -tlnp                      # Network connections
```

### Metrics Files

```bash
metrics/
├── build_metrics.csv         # Build timing data
├── test_metrics.csv          # Test execution data
├── cache_hits.csv            # Cache performance
└── resource_usage.log        # Resource consumption
```

### Log Files

```bash
logs/
├── health_monitor_*.log      # Health check history
├── health_alerts_*.log       # Alert history
├── memory_usage_*.log        # Memory monitoring
├── cpu_usage_*.log           # CPU monitoring
├── disk_io_*.log             # Disk I/O monitoring
└── resource_summary_*.log    # Resource summaries
```

---

**Document Version:** 1.0.0
**Last Review:** 2025-10-17
**Next Review:** 2025-11-17 (Monthly)
**Owner:** DevOps Engineer, Phase 1 & 2 Execution Team
