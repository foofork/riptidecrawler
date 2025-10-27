# Schemathesis Failures - Quick Reference Guide

**Full Analysis:** [schemathesis-failure-analysis.md](./schemathesis-failure-analysis.md)

## 🔥 Top Priority Fixes (Week 1)

### 1. Fix WebSocket Method Validation Bug (Day 1)
**File:** `/workspaces/eventmesh/crates/riptide-api/src/middleware/request_validation.rs:338`
**Tests Failing:** 2 (test_get_allowed_methods_websocket)
**Fix:** Ensure WebSocket endpoints return `["GET", "OPTIONS"]` in allowed methods

### 2. Add Missing Response Components (Day 1-2)
**File:** `/workspaces/eventmesh/docs/02-api-reference/openapi.yaml`
**Action:** Add to `components/responses`:
- `BadGateway` (502)
- `ServiceUnavailable` (503)

### 3. Bulk Add 503 to All Endpoints (Day 2-3)
**Script:** Use `scripts/add_missing_status_codes.py` (needs creation)
**Affected:** 96 endpoints missing 503
**Effort:** Low (automated)

### 4. Align Validation Constraints (Day 3-5)
**Files:** 
- `/workspaces/eventmesh/crates/riptide-api/src/validation.rs`
- `/workspaces/eventmesh/docs/02-api-reference/openapi.yaml`

**Action:** Extract all validation rules and add to OpenAPI:
- `minLength`, `maxLength` on strings
- `pattern` for URL validation
- Document case normalization
- Add business rule descriptions

### 5. Fix Dependency Error Handling (Day 5)
**Files:**
- `/workspaces/eventmesh/crates/riptide-api/src/handlers/*.rs`
- `/workspaces/eventmesh/crates/riptide-core/src/error.rs`

**Action:** 
- Graceful degradation for optional features
- Clear 503 errors with setup instructions
- Update health check

## 📊 Failure Breakdown

| Category | Count | Severity | Week |
|----------|-------|----------|------|
| Schema-compliant rejected | 98 | 🔴 CRITICAL | 1 |
| Undocumented status codes | 59 | 🟠 HIGH | 1-2 |
| Server errors | 19 | 🔴 CRITICAL | 1 |
| Response schema violations | 7 | 🔴 CRITICAL | 1 |
| Schema-violating accepted | 7 | 🟠 HIGH | 2 |
| Unsupported method response | 15 | 🟡 MEDIUM | 3 |
| Undocumented Content-Type | 6 | 🟡 MEDIUM | 3 |

## 🎯 Success Metrics

**Current:** 211 failures
**Target:** < 10 failures (95% reduction)

**Pass Rate:**
- Current: ~71% (149/211 pass)
- Target: >95%

## 📁 Key Files to Modify

1. `/workspaces/eventmesh/docs/02-api-reference/openapi.yaml` - Add status codes, constraints
2. `/workspaces/eventmesh/crates/riptide-api/src/middleware/request_validation.rs` - Fix WebSocket, add Allow header
3. `/workspaces/eventmesh/crates/riptide-api/src/validation.rs` - Document all rules
4. `/workspaces/eventmesh/crates/riptide-core/src/error.rs` - Map errors to status codes

## 🛠️ Scripts Needed

1. `scripts/extract_validators.py` - Extract Rust validation rules
2. `scripts/add_missing_status_codes.py` - Bulk add 502/503 responses
3. `scripts/compare_validations.py` - Compare Rust vs OpenAPI
4. `scripts/validate_schema_sync.sh` - CI check for validation alignment

## 📞 Coordination Memory Keys

Stored in `.swarm/memory.db`:
- `swarm/researcher/schemathesis-complete` - Analysis complete flag
- `swarm/shared/priority-fixes` - Priority matrix (see JSON below)
- `swarm/shared/affected-endpoints` - Endpoint groupings
- `swarm/coder/openapi-changes` - Required OpenAPI mods

## 🚀 Quick Commands

```bash
# Re-run Schemathesis
schemathesis run docs/02-api-reference/openapi.yaml \
  --base-url http://localhost:8080 \
  --checks all --max-examples=100

# Validate OpenAPI
npx @apidevtools/swagger-cli validate docs/02-api-reference/openapi.yaml

# Check validation sync (after creating script)
./scripts/validate_schema_sync.sh

# Run API tests
cargo test -p riptide-api --lib
```

## 📋 Next Actions

- [ ] Review full analysis: `docs/analysis/schemathesis-failure-analysis.md`
- [ ] Assign coder agent to OpenAPI updates
- [ ] Assign coder agent to WebSocket validation fix
- [ ] Create automation scripts
- [ ] Set up CI validation checks
