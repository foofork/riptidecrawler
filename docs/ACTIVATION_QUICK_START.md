# Dead Code Activation - Quick Start Guide

## Overview

This guide provides the fastest path to activate all 159 HIGH priority dead code items. For detailed implementation steps, see [ACTIVATION_IMPLEMENTATION_PLAN.md](./ACTIVATION_IMPLEMENTATION_PLAN.md).

## TL;DR - Execute This Plan

### Week 1 (Days 1-3): Foundation
```bash
# Day 1: Application State + Metrics
cd /workspaces/eventmesh
git checkout -b feature/activate-dead-code-phase-4a

# Activate Application State (4 hours)
# Remove #[allow(dead_code)] from state.rs lines 64, 75, 83, 97, 105, 110
# Wire up health_checker, telemetry, pdf_metrics, performance_metrics

# Activate Advanced Metrics (4 hours)
# Remove #[allow(dead_code)] from metrics.rs
# Add phase timing, error tracking, connection tracking

# Test
cargo build --release
cargo test --package riptide-api

# Day 2: Finish Metrics + Health Checks
# Complete metrics integration (4 hours)
# Activate Advanced Health Checks (4 hours)
# Create health endpoints

# Day 3: Resources + Testing
# Activate Resource Management (4 hours)
# Create resource status endpoints
# Comprehensive testing (4 hours)

git add -A
git commit -m "Phase 4A: Activate application state, metrics, health, resources"
```

### Week 2 (Days 4-7): Advanced Features
```bash
# Day 4: Workers + Telemetry
# Activate Worker Management (2 hours)
# Activate Telemetry Features (4 hours)

# Days 5-7: Streaming Infrastructure
# Activate Response Helpers (Day 5)
# Activate NDJSON/SSE (Day 6)
# Activate Lifecycle Management (Day 7)

git add -A
git commit -m "Phase 4B: Activate workers, telemetry, streaming"
git push origin feature/activate-dead-code-phase-4a
```

## Feature Priority Order

### Must Do First (Phase 4A - Days 1-3)
1. **Application State Fields** - Foundation for everything else
2. **Advanced Metrics** - Observability critical
3. **Advanced Health Checks** - Production monitoring
4. **Resource Management** - Performance controls

### Can Do Second (Phase 4B - Days 4-7)
5. **Worker Management** - Background processing
6. **Telemetry Features** - Enhanced observability
7. **Streaming Infrastructure** - Real-time features

### Deferred (Requires Analysis)
8. **Session Management** - Complex state management

## Quick File Checklist

### Phase 4A Files to Modify
- [ ] `/workspaces/eventmesh/crates/riptide-api/src/state.rs`
- [ ] `/workspaces/eventmesh/crates/riptide-api/src/metrics.rs`
- [ ] `/workspaces/eventmesh/crates/riptide-api/src/health.rs`
- [ ] `/workspaces/eventmesh/crates/riptide-api/src/resource_manager.rs`

### Phase 4A Files to Create
- [ ] `/workspaces/eventmesh/crates/riptide-api/src/handlers/health.rs`
- [ ] `/workspaces/eventmesh/crates/riptide-api/src/handlers/resources.rs`
- [ ] `/workspaces/eventmesh/crates/riptide-api/src/middleware/metrics.rs`

## Validation Commands

```bash
# 1. Zero dead code warnings
cargo build 2>&1 | grep -i "dead_code" | wc -l  # Should output: 0

# 2. All tests pass
cargo test --all

# 3. Health check works
curl http://localhost:8080/health/detailed | jq

# 4. Metrics exposed
curl http://localhost:8080/metrics | grep riptide_ | wc -l  # Should be >50

# 5. Resource status works
curl http://localhost:8080/resources/status | jq
```

## Success Metrics

### Phase 4A Success (End of Day 3)
- ✅ 0 dead code warnings for features 1-4
- ✅ 69 items activated (8 + 31 + 14 + 10 + 6 from config structs)
- ✅ Health endpoints operational
- ✅ Metrics collecting
- ✅ Resource limits enforced

### Phase 4B Success (End of Day 7)
- ✅ 0 dead code warnings for features 5-7
- ✅ 77 additional items activated (1 + 12 + 64)
- ✅ Workers operational
- ✅ Telemetry collecting
- ✅ Streaming functional

### Overall Success
- ✅ **159 total items activated**
- ✅ **0 dead code warnings** in entire codebase
- ✅ All tests passing
- ✅ Production-ready

## Risk Mitigation

### If Something Breaks
1. **Revert the specific feature**:
   ```bash
   git revert HEAD~1
   ```

2. **Re-add suppression temporarily**:
   ```rust
   #[allow(dead_code)]  // TODO: Fix integration issue #XXX
   pub field_name: Type,
   ```

3. **Deploy previous version**:
   ```bash
   git checkout main
   cargo build --release
   ```

### Monitoring During Activation
```bash
# Watch for errors
tail -f /var/log/riptide/error.log

# Monitor metrics
watch -n 1 'curl -s http://localhost:8080/metrics | grep riptide_errors_total'

# Check resource usage
watch -n 1 'curl -s http://localhost:8080/resources/status | jq .memory'
```

## Common Issues & Solutions

### Issue: Build fails after removing dead_code
**Solution**: The code is being used somewhere. Check compiler error for exact location.

### Issue: Tests fail after activation
**Solution**: Update test expectations. New metrics/endpoints may change responses.

### Issue: Performance degradation
**Solution**: Check metrics collection overhead. May need to adjust sampling rates.

### Issue: Memory usage increases
**Solution**: Verify cleanup tasks are running. Check `/resources/status` endpoint.

## Next Steps After Activation

1. **Update documentation**
   - API docs with new endpoints
   - Metrics catalog
   - Deployment guides

2. **Configure monitoring**
   - Grafana dashboards
   - Alert rules
   - SLO definitions

3. **Performance validation**
   - Load testing
   - Benchmark baselines
   - Optimization if needed

4. **Team training**
   - Feature walkthrough
   - On-call runbook
   - Knowledge transfer

## Timeline

```
Week 1: Foundation (Phase 4A)
├─ Day 1: State + Metrics Start
├─ Day 2: Metrics Finish + Health
└─ Day 3: Resources + Testing

Week 2: Advanced (Phase 4B)
├─ Day 4: Workers + Telemetry
├─ Day 5: Streaming Response Helpers
├─ Day 6: Streaming Protocols
└─ Day 7: Streaming Lifecycle + Final Testing
```

## Resources

- **Detailed Plan**: [ACTIVATION_IMPLEMENTATION_PLAN.md](./ACTIVATION_IMPLEMENTATION_PLAN.md)
- **Architecture**: [ARCHITECTURE_QUICK_REFERENCE.md](./ARCHITECTURE_QUICK_REFERENCE.md)
- **Roadmap**: [ROADMAP.md](./ROADMAP.md)
- **Dead Code Analysis**: [dead-code-analysis-report.md](./dead-code-analysis-report.md)

## Support

For questions or issues during activation:
1. Check the detailed implementation plan
2. Review architecture documentation
3. Consult dead code analysis report
4. Create issue with `activation` label

---

**Last Updated**: 2025-10-04
**Status**: Ready for Execution
**Estimated Completion**: 7-10 days
