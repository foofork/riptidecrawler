# Dependency Maintenance Guide

## Current Advisory Ignores

This document tracks why certain security advisories are ignored and provides a roadmap for addressing them.

### Unmaintained Dependencies

#### RUSTSEC-2025-0052: async-std

**Status**: Ignored (Informational)
**Severity**: Low
**Type**: Unmaintained crate

**Why we ignore it:**
- `async-std` is a transitive dependency pulled in by:
  - `chromiumoxide` (headless browser automation)
  - `httpmock` (dev-dependency for testing)
- The project uses `tokio` as the primary async runtime
- `async-std` is only used by external dependencies, not our code

**Replacement Strategy:**
1. **Short-term**: Monitor for chromiumoxide updates that remove async-std
2. **Medium-term**: Consider alternatives:
   - `chromiumoxide-cdp` (newer fork)
   - Direct CDP protocol implementation with tokio
3. **For httpmock**: Consider alternatives like `wiremock` or `mockito`

**Alternatives suggested by advisory:**
- `smol` - lightweight async runtime
- Note: Can't directly replace as it's a transitive dependency

---

#### RUSTSEC-2024-0436: paste

**Status**: Ignored (Informational)
**Severity**: Low
**Type**: Unmaintained proc-macro

**Why we ignore it:**
- `paste` is a deeply nested transitive dependency:
  - `jemalloc-ctl` → performance monitoring
  - `rav1e` → image encoding (via pdfium-render)
- It's a proc-macro crate (compile-time only, no runtime impact)
- No known security vulnerabilities, just lack of maintenance

**Replacement Strategy:**
1. **Low priority**: This is a proc-macro with no runtime impact
2. Monitor upstream dependencies for migration to `pastey`
3. Consider if `jemalloc` allocation tracking is essential
4. PDF rendering may not need advanced image encoding

**Alternatives:**
- `pastey` - fork with additional features
- Note: Requires upstream dependencies to migrate

---

## License Compliance

All dependencies must use licenses compatible with Apache-2.0:

### Allowed Licenses
- MIT
- Apache-2.0
- BSD-2-Clause / BSD-3-Clause
- ISC
- CC0-1.0
- Zlib
- MPL-2.0
- Unicode-3.0 / Unicode-DFS-2016
- 0BSD

### License Checking

Run license checks with:
```bash
cargo deny check licenses
```

---

## Build Issues: Prometheus API Changes

### Issue: protobuf 3.x API Compatibility

**Error seen in CI:**
```
error[E0603]: module `proto_ext` is private
error[E0599]: no method named `get_value` found
```

**Status**: ✅ **FIXED**

**Fix Applied:**
The code in `crates/riptide-api/src/health.rs` now uses the correct protobuf 3.x API:

```rust
// ✅ Correct protobuf 3.x API
for metric in family.get_metric() {
    if let Some(counter) = metric.counter.as_ref() {
        if let Some(value) = counter.value {
            total_requests += value as u64;
        }
    }
}
```

**What changed:**
- Removed: `use prometheus::proto_ext::MessageFieldExt;`
- Changed: `counter.get_value()` → `counter.value` with `as_ref()`
- API: MessageField now uses direct field access instead of trait methods

---

## Monitoring & Updates

### Regular Maintenance Tasks

1. **Weekly**: Check for security advisories
   ```bash
   cargo audit
   ```

2. **Monthly**: Update dependencies
   ```bash
   cargo update
   cargo test --workspace
   ```

3. **Quarterly**: Review ignored advisories
   - Check if upstream deps have been updated
   - Re-evaluate replacement strategies
   - Update this document

### Automated CI Checks

The following checks run in CI:
- `cargo deny check advisories` - Security advisories
- `cargo deny check licenses` - License compliance
- `cargo deny check bans` - Duplicate dependencies
- `cargo audit` - Known vulnerabilities

### Fixing Advisory Failures in CI

If you see advisory failures in GitHub Actions:

1. **Check if it's an ignored advisory**: See `deny.toml` ignore list
2. **For new advisories**:
   - Assess severity (critical > high > medium > low > informational)
   - Check if direct or transitive dependency
   - For critical/high: Immediate fix required
   - For medium/low: Plan replacement strategy
   - For informational: Consider ignoring with justification

3. **For unmaintained crates**:
   - Check cargo tree: `cargo tree -i <crate-name>`
   - Document in this file
   - Add to ignore list with detailed comment
   - Create ticket for long-term replacement

---

## Dependency Update Strategy

### Critical Updates (Security)
- **Response time**: Within 24 hours
- **Process**:
  1. Update dependency in Cargo.toml
  2. Run full test suite
  3. Deploy via hotfix branch
  4. Update this document

### Non-Critical Updates
- **Frequency**: Monthly
- **Process**:
  1. Run `cargo update`
  2. Review changelog for breaking changes
  3. Update tests if needed
  4. Run full CI/CD pipeline

### Major Version Updates
- **Planning**: Quarterly review
- **Process**:
  1. Review breaking changes
  2. Create feature branch
  3. Update code for new API
  4. Comprehensive testing
  5. Stage rollout

---

## Current Dependency Health

Last updated: 2025-10-08

### Core Runtime
- ✅ `tokio` - Well maintained, stable (primary async runtime)
- ✅ `axum` - Well maintained, stable (web framework)
- ✅ `chromiumoxide` - **ACCEPTABLE WITH ISOLATION**
  - **Status**: Uses async-std (RUSTSEC-2025-0052) but isolated to browser pool
  - **Justification**: Provides Chrome DevTools Protocol (CDP) for headless browsing
  - **Isolation**: Confined to `resource_manager.rs` browser pool management
  - **Runtime**: Main app remains pure Tokio; async-std tasks in separate browser processes
  - **Alternatives considered**: thirtyfour (less CDP control), sidecar (over-engineering)
  - **Mitigation**: Can feature-gate if needed; monitoring for Tokio migration upstream
  - **Impact**: No runtime conflicts; chromiumoxide manages its own task executor

### Monitoring & Performance
- ✅ `prometheus` - Well maintained
- ✅ `jemalloc-ctl` - **ACCEPTABLE WITH FEATURE GATE**
  - **Status**: Uses paste (RUSTSEC-2024-0436) proc-macro
  - **Justification**: Compile-time only (proc-macro), no runtime impact
  - **Isolation**: Behind optional `jemalloc` feature in `riptide-performance`
  - **Impact**: Zero security risk; transitive dependency for perf monitoring

### Image Processing
- ✅ `pdfium-render` - Well maintained
- ✅ `rav1e` - **ACCEPTABLE AS TRANSITIVE**
  - **Status**: Uses paste (RUSTSEC-2024-0436) via deep dependency chain
  - **Justification**: Proc-macro only, no runtime code execution
  - **Isolation**: Optional feature behind `pdf` flag in `riptide-pdf`
  - **Impact**: Zero security risk; compile-time only

### Testing
- ✅ `wiremock` - Well maintained, Tokio-native (HTTP mocking)
- ~~`httpmock`~~ - **REMOVED** (replaced by wiremock, async-std eliminated)

---

## Contact

For questions about dependency management, contact the platform team.

For security concerns, see `SECURITY.md`.
