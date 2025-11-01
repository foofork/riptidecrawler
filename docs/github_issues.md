# GitHub Issue Templates - Post-Audit P1 Items

Generated from Development Roadmap - 2025-11-01

---

## Issue 1: [CRITICAL] Fix WASM Configuration Test Failures

**Labels:** `P1`, `critical`, `technical-debt`, `breaking-change`, `api-layer`, `testing`

**Title:** CRITICAL: Fix WASM configuration test failures in config_env_tests.rs

### Description
WASM configuration tests are failing with compilation errors due to missing `wasm` field in `ApiConfig` struct. This is blocking the build and preventing test execution.

### Error Details
```
error[E0609]: no field `wasm` on type `riptide_api::config::ApiConfig`
   --> crates/riptide-api/tests/config_env_tests.rs:205:31
```

**8 compilation errors total** in `config_env_tests.rs` (lines 205-211, 346)

### Root Cause
The `wasm` field was removed from `ApiConfig` but tests still reference it. Either:
1. Configuration structure changed without test updates
2. WASM configuration was intentionally removed
3. WASM config was moved to a different structure

### Required Actions

**Option A: Restore Configuration**
- [ ] Restore `wasm` field to `ApiConfig`
- [ ] Add back WASM configuration structure
- [ ] Document why it was removed and why we're restoring

**Option B: Refactor Tests** (Recommended)
- [ ] Identify new location of WASM configuration
- [ ] Update test assertions to use new structure
- [ ] Update test environment variable names if changed
- [ ] Add migration notes to CHANGELOG.md

### Acceptance Criteria
- [ ] All tests in `config_env_tests.rs` compile
- [ ] `cargo test --package riptide-api --test config_env_tests` passes
- [ ] No regression in other configuration tests
- [ ] CI/CD pipeline green

### Affected Files
- `crates/riptide-api/tests/config_env_tests.rs`
- `crates/riptide-api/src/config.rs` (likely)

### Estimated Effort
4-6 hours

### Dependencies
None - This is a blocker for other work

---

## Issue 2: [P1] Implement Authentication Middleware

**Labels:** `P1`, `feature:incomplete`, `wire-up`, `security`, `api-layer`

**Title:** Implement authentication middleware for API security

### Description
Authentication middleware is marked as TODO but not implemented. This is a critical security feature needed before production deployment.

### Current State
```rust
// File: crates/riptide-api/src/errors.rs:31
// TODO(P1): Implement authentication middleware
```

### Requirements

**Phase 1: Basic Auth**
- [ ] Design authentication strategy (JWT, API keys, OAuth2)
- [ ] Implement middleware layer in Axum
- [ ] Add authentication error types to `errors.rs`
- [ ] Create auth configuration structure

**Phase 2: Integration**
- [ ] Wire middleware into API routes
- [ ] Add authentication to protected endpoints
- [ ] Implement role-based access control (if needed)
- [ ] Add authentication metrics/logging

**Phase 3: Testing**
- [ ] Unit tests for auth logic
- [ ] Integration tests for protected routes
- [ ] Test auth failure scenarios
- [ ] Test token refresh/expiration

### Design Considerations
1. **Token Storage:** Where to store auth tokens? (env vars, config, database)
2. **Scope:** Which endpoints need authentication?
3. **Integration:** Does this integrate with external identity providers?
4. **Rate Limiting:** Should auth failures trigger rate limiting?

### Acceptance Criteria
- [ ] Authentication middleware implemented and tested
- [ ] Protected routes require valid authentication
- [ ] Clear error messages for auth failures
- [ ] Documentation for API consumers
- [ ] No performance regression (< 5ms latency added)

### Affected Files
- `crates/riptide-api/src/errors.rs`
- `crates/riptide-api/src/middleware/` (new)
- `crates/riptide-api/src/routes/*.rs` (integration points)

### Estimated Effort
2-3 days

### Dependencies
- Configuration system must support auth settings
- Error handling infrastructure (exists)

### Security Considerations
⚠️ **Security Review Required** before merging
- Input validation for tokens
- Secure token storage
- HTTPS enforcement
- Rate limiting integration

---

## Issue 3: [P1] Wire Trace Backend Integration (Telemetry)

**Labels:** `P1`, `wire-up`, `observability`, `api-layer`

**Title:** Connect telemetry handlers to actual trace backend (Jaeger/Zipkin/OTLP)

### Description
Telemetry infrastructure is prepared but not connected to a real trace backend. Handlers return mock/placeholder data instead of actual traces.

### Current State
```rust
// File: crates/riptide-api/src/handlers/telemetry.rs:166
// TODO(P1): Wire up to actual trace backend (Jaeger/Zipkin/OTLP)

// File: crates/riptide-api/src/handlers/telemetry.rs:225
// TODO(P1): Wire up to actual trace backend for trace tree retrieval
```

### Requirements

**Phase 1: Backend Selection**
- [ ] Choose trace backend (Jaeger vs Zipkin vs OTLP)
- [ ] Document decision rationale
- [ ] Add backend dependency to `Cargo.toml`

**Phase 2: Integration**
- [ ] Configure trace exporter
- [ ] Wire `/traces` endpoint to query backend
- [ ] Wire `/traces/{id}/tree` to retrieve trace trees
- [ ] Add backend health check

**Phase 3: Configuration**
- [ ] Add trace backend config to `ApiConfig`
- [ ] Support environment-based configuration
- [ ] Add connection retry logic
- [ ] Implement graceful degradation if backend unavailable

**Phase 4: Testing**
- [ ] Integration tests with trace backend
- [ ] Test trace export and retrieval
- [ ] Test backend failure scenarios
- [ ] Performance testing (trace overhead)

### Recommended Backend: OpenTelemetry (OTLP)
**Rationale:**
- Vendor-neutral standard
- Supports multiple backends
- Future-proof for observability stack changes
- Good Rust ecosystem support

### Acceptance Criteria
- [ ] Traces successfully exported to backend
- [ ] `/traces` endpoint returns actual trace data
- [ ] `/traces/{id}/tree` returns trace hierarchy
- [ ] Configuration documented
- [ ] Backend health visible in `/health` endpoint
- [ ] < 1% performance overhead from tracing

### Affected Files
- `crates/riptide-api/src/handlers/telemetry.rs`
- `crates/riptide-api/src/config.rs` (add trace config)
- `crates/riptide-api/Cargo.toml` (add dependencies)

### Estimated Effort
1-2 days

### Dependencies
- Decision on trace backend
- Network access to trace backend
- OpenTelemetry SDK integration

---

## Issue 4: [P1] Complete chromiumoxide Migration

**Labels:** `P1`, `technical-debt`, `migration`, `browser`, `cli-layer`

**Title:** Complete migration to chromiumoxide for browser automation

### Description
Browser automation is partially migrated to chromiumoxide but several modules are disabled or incomplete. This blocks headless rendering functionality.

### Current State
Multiple TODOs marked `TODO(chromiumoxide-migration)`:
- `crates/riptide-cli/src/main.rs:18` - Module disabled
- `crates/riptide-cli/src/main.rs:69` - Code path disabled
- `crates/riptide-cli/src/main.rs:171` - Integration incomplete
- `crates/riptide-cli/src/commands/render.rs:688` - Type access needed
- `crates/riptide-cli/src/commands/render.rs:776` - Type access needed

### Migration Blockers
```rust
// TODO: Re-implement with proper chromiumoxide type access
```

**Root Issue:** chromiumoxide types not properly exposed or imported

### Requirements

**Phase 1: Type Access**
- [ ] Audit chromiumoxide API surface
- [ ] Identify required types and traits
- [ ] Create proper type imports/re-exports
- [ ] Document type mappings from old implementation

**Phase 2: Re-enable Modules**
- [ ] Re-enable Phase 4 modules in `commands/mod.rs`
- [ ] Implement missing `global()` methods
- [ ] Update render command implementation
- [ ] Fix type access in render.rs:688, 776

**Phase 3: Integration**
- [ ] Re-enable browser pool integration in main.rs
- [ ] Test headless rendering end-to-end
- [ ] Verify stealth mode compatibility
- [ ] Test error handling paths

**Phase 4: Cleanup**
- [ ] Remove old browser implementation code
- [ ] Update documentation
- [ ] Remove migration TODOs
- [ ] Add migration notes to CHANGELOG

### Acceptance Criteria
- [ ] All chromiumoxide TODOs resolved
- [ ] `cargo build --package riptide-cli` succeeds
- [ ] Headless rendering tests pass
- [ ] Browser pool functional
- [ ] No regression in rendering quality
- [ ] Documentation updated

### Affected Files
- `crates/riptide-cli/src/main.rs`
- `crates/riptide-cli/src/commands/mod.rs`
- `crates/riptide-cli/src/commands/render.rs`
- `crates/riptide-cli/src/commands/optimized_executor.rs`

### Estimated Effort
3-5 days

### Dependencies
- chromiumoxide crate version compatibility
- Browser pool implementation (may need updates)

### Testing Checklist
- [ ] Unit tests for new chromiumoxide integration
- [ ] Integration tests for full rendering pipeline
- [ ] Performance comparison with old implementation
- [ ] Memory leak testing (browser instances)

---

## Issue 5: [P1] Implement Session Persistence for Stateful Rendering

**Labels:** `P1`, `feature:incomplete`, `browser`, `api-layer`

**Title:** Add session context to RPC client for browser state persistence

### Description
Browser rendering sessions are not persisted, preventing stateful interactions and multi-step workflows.

### Current State
```rust
// File: crates/riptide-api/src/rpc_client.rs:56
// TODO(P1): Implement session persistence for stateful rendering

// File: crates/riptide-api/src/handlers/render/processors.rs:111
// TODO(P1): Pass session context to RPC client for browser state persistence
```

### Use Cases
1. **Multi-page workflows:** Login → Navigate → Extract data
2. **Authenticated scraping:** Maintain cookies/auth state
3. **Progressive rendering:** Resume paused rendering jobs
4. **Session sharing:** Multiple requests using same browser context

### Requirements

**Phase 1: Session Model**
- [ ] Design session data structure (cookies, storage, auth)
- [ ] Implement session ID generation
- [ ] Create session store (in-memory + persistent)
- [ ] Define session lifecycle (TTL, eviction)

**Phase 2: RPC Integration**
- [ ] Add session context to RPC client
- [ ] Wire session ID through render pipeline
- [ ] Implement session restoration in browser pool
- [ ] Add session cleanup on timeout/completion

**Phase 3: API Interface**
- [ ] Add session endpoints (create/retrieve/delete)
- [ ] Accept session ID in render requests
- [ ] Return session ID in responses
- [ ] Document session API

**Phase 4: Persistence**
- [ ] Store sessions in persistence layer
- [ ] Implement session serialization/deserialization
- [ ] Add session recovery after restarts
- [ ] Handle session migration

### Acceptance Criteria
- [ ] Sessions persist across render requests
- [ ] Cookies and local storage maintained
- [ ] Session TTL and eviction working
- [ ] API consumers can create/manage sessions
- [ ] Sessions cleaned up properly
- [ ] Documentation with examples

### Affected Files
- `crates/riptide-api/src/rpc_client.rs`
- `crates/riptide-api/src/handlers/render/processors.rs`
- `crates/riptide-api/src/session.rs` (new)
- `crates/riptide-persistence/` (session storage)

### Estimated Effort
2-3 days

### Dependencies
- Browser pool session support
- Persistence layer integration
- Session store design decision

### Security Considerations
⚠️ **Security Review Required**
- Secure session ID generation
- Session hijacking prevention
- Cookie security flags
- GDPR compliance (session data retention)

---

## Issue 6: [P1] Fix Extractor Module Type Conflicts

**Labels:** `P1`, `wire-up`, `technical-debt`, `extraction`

**Title:** Resolve type mismatches between extraction strategies and composition

### Description
Extractor modules are disabled due to type conflicts preventing compilation. This blocks extraction functionality.

### Current State
```rust
// File: crates/riptide-extraction/src/lib.rs:37
// TODO: Re-enable after resolving type mismatches between strategies and composition

// File: crates/riptide-extraction/src/lib.rs:40
// TODO: Re-enable after fixing ExtractedContent type conflicts

// File: crates/riptide-extraction/src/lib.rs:119
// TODO: Re-enable these after resolving type conflicts
```

### Root Cause Analysis Needed
- [ ] Audit `strategies` module types
- [ ] Audit `composition` module types
- [ ] Identify type mismatches
- [ ] Document expected vs actual types
- [ ] Determine if architecture refactor needed

### Requirements

**Phase 1: Investigation**
- [ ] Enable compilation with verbose errors
- [ ] Document all type conflicts
- [ ] Identify common patterns in mismatches
- [ ] Check if recent changes introduced conflicts

**Phase 2: Resolution**
- [ ] Align `ExtractedContent` type across modules
- [ ] Fix strategy trait implementations
- [ ] Update composition layer to match
- [ ] Add type aliases if needed for compatibility

**Phase 3: Re-enable**
- [ ] Uncomment disabled modules
- [ ] Verify compilation
- [ ] Run extraction tests
- [ ] Check integration with API layer

**Phase 4: Prevention**
- [ ] Add type safety tests
- [ ] Document type architecture
- [ ] Consider using trait objects or generics
- [ ] Add CI check for module compilation

### Acceptance Criteria
- [ ] All extraction modules enabled
- [ ] No type conflicts in compilation
- [ ] Extraction tests pass
- [ ] API integration works
- [ ] Performance not degraded
- [ ] Architecture documented

### Affected Files
- `crates/riptide-extraction/src/lib.rs`
- `crates/riptide-extraction/src/strategies/`
- `crates/riptide-extraction/src/composition/`

### Estimated Effort
1-2 days

### Dependencies
None

### Risk Assessment
**Medium:** Type refactoring could have ripple effects across API layer

---

## Issue 7: [P1] Add Data Validation Tests (CSV & Markdown)

**Labels:** `P1`, `test-coverage`, `data-quality`, `api-layer`

**Title:** Implement CSV and Markdown structure validation in integration tests

### Description
Integration tests verify response status but don't validate data structure/format. This could allow malformed data to pass tests.

### Current State
```rust
// File: crates/riptide-api/tests/integration_tests.rs:363
// TODO(P1): Validate CSV content structure

// File: crates/riptide-api/tests/integration_tests.rs:401
// TODO(P1): Validate Markdown table format
```

### Requirements

**CSV Validation**
- [ ] Verify column headers present and correct
- [ ] Validate row count and data types
- [ ] Check for proper escaping/quoting
- [ ] Test edge cases (empty fields, special chars)
- [ ] Verify consistent column count across rows

**Markdown Validation**
- [ ] Verify table header format
- [ ] Check separator line syntax
- [ ] Validate column alignment
- [ ] Test cell content escaping
- [ ] Verify table rendering in parsers

### Acceptance Criteria
- [ ] CSV structure validation implemented
- [ ] Markdown table validation implemented
- [ ] Tests catch malformed output
- [ ] Edge cases covered
- [ ] Documentation with examples

### Affected Files
- `crates/riptide-api/tests/integration_tests.rs`
- `crates/riptide-test-utils/` (validation helpers)

### Estimated Effort
0.5-1 day

### Dependencies
None

---

## Issue 8: [P1] Implement Failover Behavior Tests

**Labels:** `P1`, `test-coverage`, `reliability`, `api-layer`

**Title:** Test actual failover behavior in API integration tests

### Description
Failover infrastructure exists but isn't tested end-to-end. Need to verify failover works correctly under failure conditions.

### Current State
```rust
// File: crates/riptide-api/tests/integration_tests.rs:869
// TODO(P1): Test actual failover behavior
```

### Test Scenarios
- [ ] Primary extractor fails → fallback to secondary
- [ ] All extractors fail → return error
- [ ] Partial failure → retry logic
- [ ] Timeout → fallback chain
- [ ] Browser crash → recovery
- [ ] Network failure → retry with backoff

### Requirements

**Phase 1: Test Infrastructure**
- [ ] Create failure injection mechanisms
- [ ] Mock extractor failures
- [ ] Simulate network timeouts
- [ ] Add controllable failure scenarios

**Phase 2: Test Cases**
- [ ] Test each failover path
- [ ] Verify error propagation
- [ ] Test retry exhaustion
- [ ] Validate fallback ordering

**Phase 3: Validation**
- [ ] Verify correct extractor selected
- [ ] Check metrics/logging for failover events
- [ ] Test performance under failures
- [ ] Ensure no data corruption

### Acceptance Criteria
- [ ] All failover paths tested
- [ ] Failures properly handled
- [ ] Metrics capture failover events
- [ ] No infinite retry loops
- [ ] Documentation with examples

### Affected Files
- `crates/riptide-api/tests/integration_tests.rs`
- `crates/riptide-test-utils/` (failure injection)

### Estimated Effort
1 day

### Dependencies
- Failure injection infrastructure
- Mock extractors

---

## Issue 9: [P1] Integrate Background Processor with LLM Client Pool

**Labels:** `P1`, `feature:incomplete`, `wire-up`, `intelligence`

**Title:** Wire background processor to LLM client pool for intelligent processing

### Description
Background processor has placeholder for LLM integration but isn't connected to actual LLM client pool.

### Current State
```rust
// File: crates/riptide-intelligence/src/background_processor.rs:412
/// TODO: Integrate with LLM client pool
```

### Requirements

**Phase 1: LLM Client Pool Design**
- [ ] Define LLM client pool interface
- [ ] Choose LLM provider(s) (OpenAI, Anthropic, local)
- [ ] Implement connection pooling
- [ ] Add rate limiting and quotas

**Phase 2: Integration**
- [ ] Wire LLM pool into background processor
- [ ] Implement prompt templates
- [ ] Add result parsing/validation
- [ ] Handle LLM errors and retries

**Phase 3: Use Cases**
- [ ] Content classification
- [ ] Extraction pattern learning
- [ ] Quality assessment
- [ ] Intelligent fallback decisions

**Phase 4: Testing**
- [ ] Mock LLM responses for tests
- [ ] Test error handling
- [ ] Validate rate limiting
- [ ] Performance testing

### Acceptance Criteria
- [ ] LLM client pool implemented
- [ ] Background processor uses LLM
- [ ] Rate limiting prevents abuse
- [ ] Error handling robust
- [ ] Costs tracked/limited
- [ ] Documentation with examples

### Affected Files
- `crates/riptide-intelligence/src/background_processor.rs`
- `crates/riptide-intelligence/src/llm_pool.rs` (new)
- `crates/riptide-api/src/config.rs` (LLM config)

### Estimated Effort
1-2 days

### Dependencies
- LLM provider selection
- API keys/credentials
- Rate limiting strategy

### Cost Considerations
⚠️ LLM API calls have cost implications - implement quotas and monitoring

---

## Summary Table: P1 Issues

| # | Title | Component | Effort | Blocking? |
|---|-------|-----------|--------|-----------|
| 1 | Fix WASM config tests | Testing | 4-6h | ✅ Yes |
| 2 | Auth middleware | API/Security | 2-3d | ⚠️ Yes (prod) |
| 3 | Trace backend | Observability | 1-2d | No |
| 4 | chromiumoxide migration | CLI/Browser | 3-5d | ⚠️ Yes (render) |
| 5 | Session persistence | API/Browser | 2-3d | No |
| 6 | Extractor type conflicts | Extraction | 1-2d | ⚠️ Yes (extract) |
| 7 | Data validation tests | Testing | 0.5-1d | No |
| 8 | Failover tests | Testing/Reliability | 1d | No |
| 9 | LLM integration | Intelligence | 1-2d | No |

**Total Estimated Effort:** ~15-21 days (3-4 weeks for 1 developer)

**Critical Path:**
1. Issue #1 (WASM tests) - MUST complete first
2. Issue #4 (chromiumoxide) - Blocks rendering
3. Issue #6 (extractor) - Blocks extraction
4. Issue #2 (auth) - Blocks production deployment

---

**Instructions for creating issues:**
1. Copy each issue template to GitHub
2. Assign to appropriate team member
3. Add to project board in "Sprint 1" column
4. Link related issues for dependency tracking
5. Set milestone: "Production Readiness"
