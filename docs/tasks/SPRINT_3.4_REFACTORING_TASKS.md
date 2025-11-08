# Sprint 3.4 - Route Refactoring Tasks

**Sprint Goal**: Eliminate all business logic from route files
**Estimated Total Effort**: 1 hour
**Priority**: MEDIUM (not blocking, but improves architecture compliance)

---

## Task 1: Extract PDF Health Check Handler ⚠️

**File**: `crates/riptide-api/src/routes/pdf.rs`
**Status**: 🔴 NOT STARTED
**Priority**: MEDIUM
**Effort**: 30 minutes
**Assignee**: TBD

### Problem
Route file contains inline handler with 28 LOC of business logic:
- PDF integration capability checking
- Data transformation (file size calculation)
- JSON response construction

### Current State (Lines 30-58)
```rust
// In routes/pdf.rs
async fn pdf_health_check() -> axum::response::Json<serde_json::Value> {
    use riptide_pdf::integration::create_pdf_integration_for_pipeline;

    let integration = create_pdf_integration_for_pipeline();
    let available = integration.is_available();
    let capabilities = integration.capabilities();

    axum::response::Json(serde_json::json!({
        "status": if available { "healthy" } else { "unavailable" },
        "pdf_processing_available": available,
        "capabilities": {
            "text_extraction": capabilities.text_extraction,
            // ... 15 more lines of response construction
        }
    }))
}
```

### Target State

**In `routes/pdf.rs` (reduce to 28 LOC)**:
```rust
pub fn pdf_routes() -> Router<AppState> {
    Router::new()
        .route("/process", post(pdf::process_pdf))
        .route("/upload", post(pdf::upload_pdf))
        .route("/process-stream", post(pdf::process_pdf_stream))
        .route("/healthz", get(pdf::pdf_health_check))  // ← Delegate to handler
}
```

**In `handlers/pdf.rs` (add handler)**:
```rust
/// PDF processing health check endpoint
///
/// Returns comprehensive PDF processing capabilities and status.
pub async fn pdf_health_check() -> Json<Value> {
    use riptide_pdf::integration::create_pdf_integration_for_pipeline;

    let integration = create_pdf_integration_for_pipeline();
    let available = integration.is_available();
    let capabilities = integration.capabilities();

    Json(serde_json::json!({
        "status": if available { "healthy" } else { "unavailable" },
        "pdf_processing_available": available,
        "capabilities": {
            "text_extraction": capabilities.text_extraction,
            "image_extraction": capabilities.image_extraction,
            "metadata_extraction": capabilities.metadata_extraction,
            "table_extraction": capabilities.table_extraction,
            "form_extraction": capabilities.form_extraction,
            "encrypted_pdfs": capabilities.encrypted_pdfs,
            "max_file_size_mb": capabilities.max_file_size / (1024 * 1024),
            "supported_versions": capabilities.supported_versions
        },
        "features": {
            "progress_streaming": true,
            "concurrent_processing": true,
            "memory_monitoring": true,
            "performance_metrics": true
        }
    }))
}
```

### Implementation Steps

1. ✅ **Open handler file**:
   ```bash
   # Check if handler exists
   ls -la crates/riptide-api/src/handlers/pdf.rs
   ```

2. ✅ **Add handler function**:
   - Copy function from routes file
   - Change signature to `pub async fn`
   - Update imports if needed
   - Add documentation

3. ✅ **Update route file**:
   - Remove inline function
   - Update route registration: `.route("/healthz", get(pdf::pdf_health_check))`

4. ✅ **Add tests**:
   ```rust
   #[cfg(test)]
   mod tests {
       use super::*;

       #[tokio::test]
       async fn test_pdf_health_check() {
           let response = pdf_health_check().await;
           // Verify response structure
       }
   }
   ```

5. ✅ **Verify**:
   ```bash
   cargo build -p riptide-api
   cargo test -p riptide-api handlers::pdf::tests
   cargo clippy -p riptide-api -- -D warnings
   ```

### Acceptance Criteria
- ✅ `routes/pdf.rs` ≤ 30 LOC
- ✅ No inline handlers in route file
- ✅ Handler in `handlers/pdf.rs` with full implementation
- ✅ All tests passing
- ✅ Zero clippy warnings

---

## Task 2: Extract Stealth Health Check Handler ⚠️

**File**: `crates/riptide-api/src/routes/stealth.rs`
**Status**: 🔴 NOT STARTED
**Priority**: MEDIUM
**Effort**: 30 minutes
**Assignee**: TBD

### Problem
Route file contains inline handler with 22 LOC of business logic:
- Stealth controller instantiation
- Hardcoded configuration data
- JSON response construction

### Current State (Lines 30-52)
```rust
// In routes/stealth.rs
async fn stealth_health_check() -> axum::response::Json<serde_json::Value> {
    use riptide_stealth::{StealthController, StealthPreset};

    let _controller = StealthController::from_preset(StealthPreset::Medium);
    axum::response::Json(serde_json::json!({
        "status": "healthy",
        "stealth_available": true,
        "features": { /* ... */ },
        "presets": ["None", "Low", "Medium", "High"],
        "rotation_strategies": ["Random", "Sequential", "Sticky", "DomainBased"],
        // ... more configuration
    }))
}
```

### Target State

**In `routes/stealth.rs` (reduce to 28 LOC)**:
```rust
pub fn stealth_routes() -> Router<AppState> {
    Router::new()
        .route("/configure", post(stealth::configure_stealth))
        .route("/test", post(stealth::test_stealth))
        .route("/capabilities", get(stealth::get_stealth_capabilities))
        .route("/healthz", get(stealth::stealth_health_check))  // ← Delegate
}
```

**In `handlers/stealth.rs` (add handler)**:
```rust
/// Stealth features health check endpoint
///
/// Returns comprehensive stealth feature availability and configuration options.
pub async fn stealth_health_check() -> Json<Value> {
    use riptide_stealth::{StealthController, StealthPreset};

    // Test basic stealth functionality to verify module is available
    let _controller = StealthController::from_preset(StealthPreset::Medium);

    Json(serde_json::json!({
        "status": "healthy",
        "stealth_available": true,
        "features": {
            "user_agent_rotation": true,
            "header_randomization": true,
            "timing_jitter": true,
            "fingerprinting_countermeasures": true,
            "proxy_support": true,
            "javascript_evasion": true
        },
        "presets": ["None", "Low", "Medium", "High"],
        "rotation_strategies": ["Random", "Sequential", "Sticky", "DomainBased"],
        "version": riptide_stealth::VERSION,
        "crate_name": riptide_stealth::CRATE_NAME
    }))
}
```

### Implementation Steps

1. ✅ **Open handler file**:
   ```bash
   # Check if handler exists
   ls -la crates/riptide-api/src/handlers/stealth.rs
   ```

2. ✅ **Add handler function**:
   - Copy function from routes file
   - Change signature to `pub async fn`
   - Update imports if needed
   - Add documentation

3. ✅ **Update route file**:
   - Remove inline function
   - Update route registration: `.route("/healthz", get(stealth::stealth_health_check))`

4. ✅ **Add tests**:
   ```rust
   #[cfg(test)]
   mod tests {
       use super::*;

       #[tokio::test]
       async fn test_stealth_health_check() {
           let response = stealth_health_check().await;
           // Verify response structure
       }
   }
   ```

5. ✅ **Verify**:
   ```bash
   cargo build -p riptide-api
   cargo test -p riptide-api handlers::stealth::tests
   cargo clippy -p riptide-api -- -D warnings
   ```

### Acceptance Criteria
- ✅ `routes/stealth.rs` ≤ 28 LOC
- ✅ No inline handlers in route file
- ✅ Handler in `handlers/stealth.rs` with full implementation
- ✅ All tests passing
- ✅ Zero clippy warnings

---

## Task 3: Verify Route Layer Compliance ✅

**Status**: 🟡 PENDING TASKS 1-2
**Priority**: HIGH
**Effort**: 15 minutes
**Assignee**: TBD

### Description
Final verification that all route files meet Sprint 3.4 requirements after refactoring.

### Verification Steps

1. ✅ **Check LOC counts**:
   ```bash
   wc -l crates/riptide-api/src/routes/*.rs
   ```

   **Expected**:
   - All files ≤ 30 LOC (excluding documentation-heavy files)
   - `pdf.rs`: ~28-30 LOC
   - `stealth.rs`: ~28 LOC
   - Others: Already compliant

2. ✅ **Scan for violations**:
   ```bash
   # Check for inline async functions
   rg "async fn" crates/riptide-api/src/routes/

   # Check for business logic patterns
   rg "(if |for |while |match |let mut )" crates/riptide-api/src/routes/

   # Check for ServiceBuilder usage
   rg "ServiceBuilder" crates/riptide-api/src/routes/
   ```

3. ✅ **Run quality gates**:
   ```bash
   cargo build --workspace
   cargo test -p riptide-api
   cargo clippy --all -- -D warnings
   ```

4. ✅ **Generate compliance report**:
   ```bash
   # Create final report
   echo "Sprint 3.4 Route Compliance Report" > docs/reports/route_compliance_final.md
   wc -l crates/riptide-api/src/routes/*.rs >> docs/reports/route_compliance_final.md
   ```

### Acceptance Criteria
- ✅ All route files ≤ 30 LOC (code only, excluding docs)
- ✅ Zero inline handlers
- ✅ Zero business logic in routes
- ✅ All tests passing
- ✅ Zero clippy warnings
- ✅ Compliance report generated

---

## Optional Task 4: Split profiles.rs (OPTIONAL) 🟢

**File**: `crates/riptide-api/src/routes/profiles.rs`
**Status**: 🟢 OPTIONAL (NOT REQUIRED)
**Priority**: LOW
**Effort**: 15 minutes
**Assignee**: TBD

### Problem
File is 124 LOC, mostly documentation and dual implementations (feature/no-feature).

### Why Optional?
- ✅ File is **already compliant** - no business logic violations
- ✅ High LOC is due to comprehensive documentation (54%)
- ✅ Dual feature implementations are appropriate
- ⚠️ Splitting would be for organizational clarity only

### If Pursued: Target State

**Split into 2 files**:

1. **`profiles.rs`** (Active routes):
   ```rust
   #[cfg(feature = "llm")]
   pub fn profile_routes() -> Router<AppState> {
       // Active implementation
   }

   #[cfg(not(feature = "llm"))]
   pub use crate::routes::profiles_stubs::profile_routes;
   ```

2. **`profiles_stubs.rs`** (Stub routes):
   ```rust
   pub fn profile_routes() -> Router<AppState> {
       // Stub implementation
   }
   ```

### Implementation Steps (if pursued)

1. Create `routes/profiles_stubs.rs`
2. Move `#[cfg(not(feature = "llm"))]` section
3. Update `profiles.rs` to re-export
4. Update `mod.rs`
5. Verify compilation with and without `llm` feature

### Acceptance Criteria
- ✅ Both feature configurations compile
- ✅ All tests passing
- ✅ No functional changes

---

## Testing Checklist

### Per-Task Testing
- [ ] Unit tests pass for extracted handlers
- [ ] Integration tests pass for routes
- [ ] Clippy reports zero warnings
- [ ] Feature gates work correctly

### Full Suite Testing
```bash
# Build all features
cargo build --workspace

# Test with all features
cargo test --workspace --all-features

# Test without optional features
cargo test --workspace --no-default-features

# Clippy with warnings as errors
RUSTFLAGS="-D warnings" cargo clippy --all -- -D warnings

# Check formatting
cargo fmt --all -- --check
```

---

## Dependencies

```
Task 1 (PDF) → Task 3 (Verification)
Task 2 (Stealth) → Task 3 (Verification)
Task 4 (Optional) → Independent
```

---

## Rollback Plan

If issues arise:

1. **Git revert** changes:
   ```bash
   git checkout crates/riptide-api/src/routes/pdf.rs
   git checkout crates/riptide-api/src/routes/stealth.rs
   ```

2. **Verify original state**:
   ```bash
   cargo test -p riptide-api
   ```

---

## Success Metrics

### Code Quality
- ✅ All route files ≤ 30 LOC (code only)
- ✅ Zero business logic violations
- ✅ 100% test coverage for new handlers

### Build Metrics
- ✅ Zero compilation warnings
- ✅ Zero clippy warnings
- ✅ All tests passing

### Architecture Compliance
- ✅ Clean separation: Routes → Handlers → Facades → Domain
- ✅ No inline implementations
- ✅ Hexagonal architecture maintained

---

## Timeline

| Task | Effort | Start | Complete |
|------|--------|-------|----------|
| Task 1 (PDF) | 30 min | TBD | TBD |
| Task 2 (Stealth) | 30 min | TBD | TBD |
| Task 3 (Verify) | 15 min | TBD | TBD |
| **Total** | **75 min** | | |

---

## Notes

1. **Non-Blocking**: These refactorings improve architecture compliance but don't block Sprint 3.4 completion
2. **Low Risk**: Changes are isolated and well-defined
3. **High Value**: Improves maintainability and architectural consistency
4. **Test Coverage**: Ensure health check endpoints remain functional

---

## Review Checklist

Before marking complete:
- [ ] All acceptance criteria met
- [ ] Code reviewed
- [ ] Tests passing
- [ ] Documentation updated
- [ ] Compliance report generated
- [ ] Sprint 3.4 marked complete
