# Handler Refactoring Summary - Quick Reference

## 🎯 Goal: All Handlers <50 LOC

### Current Status

| Handler | Current LOC | Target | Gap | Status |
|---------|------------|--------|-----|--------|
| workers.rs | 292 | <50 | -242 | 🔴 Critical |
| profiles.rs | 230 | <50 | -180 | 🔴 Critical |
| sessions.rs | 212 | <50 | -162 | 🔴 Critical |
| pdf.rs | 147 (104 net) | <50 | -54 | 🟡 High |
| engine_selection.rs | 112 | <50 | -62 | 🟡 High |
| tables.rs | 97 | <50 | -47 | 🟡 Moderate |

---

## 📊 Quick Wins

### 1. Extract All DTOs to `api/dto/` Module
**Impact**: -430 LOC from handlers
**Effort**: Medium (2-3 days)

```bash
# Create DTO module structure
mkdir -p crates/riptide-api/src/dto
```

**Files to create**:
- `dto/mod.rs` - Module declaration
- `dto/workers.rs` - 180 LOC (11 DTOs from workers.rs)
- `dto/profiles.rs` - 130 LOC (9 DTOs from profiles.rs)
- `dto/sessions.rs` - 80 LOC (7 DTOs from sessions.rs)
- `dto/pdf.rs` - 30 LOC (3 DTOs from pdf.rs)
- `dto/engine_selection.rs` - 55 LOC (4 DTOs from engine_selection.rs)
- `dto/tables.rs` - 60 LOC (4 DTOs from tables.rs)

### 2. Add DTO Conversion Methods
**Impact**: -150 LOC from handlers
**Effort**: Low (1-2 days)

**Pattern**:
```rust
// In DTO module
impl AnalyzeRequest {
    pub fn to_criteria(&self) -> EngineSelectionCriteria {
        EngineSelectionCriteria {
            html: self.html.clone(),
            url: self.url.clone(),
            flags: self.flags.clone().into(),
        }
    }
}

// In handler (before)
let criteria = EngineSelectionCriteria {
    html: request.html,
    url: request.url,
    flags: EngineSelectionFlags {
        use_visible_text_density: request.flags.use_visible_text_density,
        detect_placeholders: request.flags.detect_placeholders,
        probe_first_spa: request.flags.probe_first_spa,
    },
};

// In handler (after)
let criteria = request.to_criteria();
```

### 3. Create Error Handling Helper
**Impact**: -50 LOC from sessions.rs
**Effort**: Very Low (1 hour)

```rust
// Add to sessions.rs
fn handle_session_error<T>(
    result: Result<T, SessionError>,
    state: &AppState,
) -> Result<T, ApiError> {
    result.map_err(|e| {
        state.metrics.record_error(ErrorType::Redis);
        ApiError::dependency("session_manager", e.to_string())
    })
}
```

---

## 🚀 Implementation Order

### Week 1: Foundation
**Days 1-2**: Create DTO module structure
- [ ] Create `dto/` directory and `mod.rs`
- [ ] Move workers.rs DTOs → `dto/workers.rs` (173 LOC)
- [ ] Move profiles.rs DTOs → `dto/profiles.rs` (108 LOC)
- [ ] Update imports in workers.rs and profiles.rs
- [ ] Run tests: `cargo test -p riptide-api`

**Days 3-4**: Continue DTO extraction
- [ ] Move sessions.rs DTOs → `dto/sessions.rs` (51 LOC)
- [ ] Move engine_selection.rs DTOs → `dto/engine_selection.rs` (32 LOC)
- [ ] Move tables.rs DTOs → `dto/tables.rs` (34 LOC)
- [ ] Move pdf.rs DTOs → `dto/pdf.rs` (17 LOC)
- [ ] Update all handler imports
- [ ] Run tests: `cargo test -p riptide-api`

**Day 5**: Verification
- [ ] Build with zero warnings: `RUSTFLAGS="-D warnings" cargo build`
- [ ] Run clippy: `cargo clippy --all -- -D warnings`
- [ ] Verify LOC reduction

**Expected Result**: All handlers reduced by 415 LOC (DTOs moved)

### Week 2: Conversions & Logic
**Days 1-2**: Add DTO conversion methods
- [ ] `workers.rs`: Add `From` implementations for all DTOs
- [ ] `profiles.rs`: Add `CacheStatusInfo::from_profile()`
- [ ] `sessions.rs`: Add `From<&Session>` and `From<Cookie>`
- [ ] `engine_selection.rs`: Add `to_criteria()` methods
- [ ] `tables.rs`: Add `to_facade_request()` method
- [ ] `pdf.rs`: Add `to_facade_options()` method

**Days 3-4**: Extract facade methods
- [ ] `ProfileFacade::clear_all_caches()` (from profiles.rs line 216-218)
- [ ] `ProfileFacade::get_caching_metrics()` (from profiles.rs line 225-229)
- [ ] `WorkersFacade::map_to_status_response()` (from workers.rs line 225)
- [ ] `WorkersFacade::format_job_type()` (from workers.rs line 290-291)

**Day 5**: Error handling
- [ ] Add `handle_session_error()` helper to sessions.rs
- [ ] Add `ResourceManager::acquire_pdf_or_error()` to pdf.rs
- [ ] Refactor all handlers to use helpers

**Expected Result**: Handlers reduced by additional 350 LOC

### Week 3: Final Polish
**Days 1-3**: Update handlers
- [ ] Refactor workers.rs handlers (target: 45 LOC)
- [ ] Refactor profiles.rs handlers (target: 45 LOC)
- [ ] Refactor sessions.rs handlers (target: 40 LOC)
- [ ] Refactor pdf.rs handlers (target: 35 LOC)
- [ ] Refactor engine_selection.rs handlers (target: 35 LOC)
- [ ] Refactor tables.rs handlers (target: 40 LOC)

**Days 4-5**: Testing & verification
- [ ] Integration tests: `cargo test -p riptide-api --test '*'`
- [ ] LOC verification script
- [ ] Update documentation
- [ ] Final review

**Expected Result**: All handlers <50 LOC ✅

---

## 📋 Concrete Refactoring Examples

### Example 1: workers.rs - DTO Extraction

**Before** (workers.rs, lines 22-95):
```rust
#[derive(Deserialize, Debug, Clone)]
pub struct SubmitJobRequest {
    pub job_type: JobTypeRequest,
    pub priority: Option<JobPriority>,
    // ... 8 more fields
}

#[derive(Deserialize, Debug, Clone)]
#[serde(tag = "type")]
pub enum JobTypeRequest {
    // ... enum variants
}

impl From<JobTypeRequest> for JobType {
    // ... conversion logic
}
```

**After** (workers.rs):
```rust
use crate::dto::workers::*; // 1 line replaces 173 lines
```

**New file** (`dto/workers.rs`):
```rust
// All DTOs moved here with improved organization
```

### Example 2: sessions.rs - Error Handling

**Before** (repeated 10 times):
```rust
pub async fn create_session(State(state): State<AppState>) -> Result<impl IntoResponse, ApiError> {
    let session = state.session_manager.create_session().await.map_err(|e| {
        state.metrics.record_error(crate::metrics::ErrorType::Redis);
        ApiError::dependency("session_manager", e.to_string())
    })?;
    // ... 8 more lines
}
```

**After**:
```rust
fn handle_session_error<T>(result: Result<T, SessionError>, state: &AppState) -> Result<T, ApiError> {
    result.map_err(|e| {
        state.metrics.record_error(ErrorType::Redis);
        ApiError::dependency("session_manager", e.to_string())
    })
}

pub async fn create_session(State(state): State<AppState>) -> Result<impl IntoResponse, ApiError> {
    let session = handle_session_error(
        state.session_manager.create_session().await,
        &state
    )?;
    Ok(Json(CreateSessionResponse::from(&session)))
}
```

### Example 3: engine_selection.rs - DTO Conversion

**Before** (lines 52-56):
```rust
pub async fn analyze_engine(
    State(_state): State<AppState>,
    Json(request): Json<AnalyzeRequest>,
) -> ApiResult<Json<EngineConfig>> {
    let criteria = EngineSelectionCriteria {
        html: request.html,
        url: request.url,
        flags: EngineSelectionFlags::default(),
    };
    Ok(Json(_state.engine_facade.select_engine(criteria).await?))
}
```

**After**:
```rust
pub async fn analyze_engine(
    State(_state): State<AppState>,
    Json(request): Json<AnalyzeRequest>,
) -> ApiResult<Json<EngineConfig>> {
    Ok(Json(_state.engine_facade.select_engine(request.to_criteria()).await?))
}
```

**New method** (`dto/engine_selection.rs`):
```rust
impl AnalyzeRequest {
    pub fn to_criteria(&self) -> EngineSelectionCriteria {
        EngineSelectionCriteria {
            html: self.html.clone(),
            url: self.url.clone(),
            flags: EngineSelectionFlags::default(),
        }
    }
}
```

---

## ✅ Verification Checklist

### After Each Change
- [ ] Run `cargo build --workspace`
- [ ] Run `cargo test -p riptide-api`
- [ ] Check no new clippy warnings
- [ ] Verify handler LOC reduced

### Final Verification
```bash
# Zero warnings build
RUSTFLAGS="-D warnings" cargo build --workspace

# All tests pass
cargo test -p riptide-api

# Clippy clean
cargo clippy --all -- -D warnings

# LOC verification
for file in crates/riptide-api/src/handlers/{workers,profiles,sessions,pdf,engine_selection,tables}.rs; do
    lines=$(grep -v '^\s*$' "$file" | grep -v '^\s*//' | grep -v '^#\[cfg(test)\]' | wc -l)
    echo "$file: $lines LOC"
done
```

### Expected Output
```
workers.rs: 45 LOC ✅
profiles.rs: 45 LOC ✅
sessions.rs: 40 LOC ✅
pdf.rs: 35 LOC ✅
engine_selection.rs: 35 LOC ✅
tables.rs: 40 LOC ✅
```

---

## 🎯 Success Metrics

| Metric | Target | How to Measure |
|--------|--------|----------------|
| All handlers <50 LOC | 100% | LOC verification script |
| Zero compiler warnings | ✅ | `RUSTFLAGS="-D warnings" cargo build` |
| Zero clippy warnings | ✅ | `cargo clippy --all -- -D warnings` |
| All tests passing | ✅ | `cargo test -p riptide-api` |
| Code coverage maintained | >85% | `cargo tarpaulin -p riptide-api` |

---

## 📚 Reference Documents

1. **Detailed Analysis**: `docs/analysis/handler_refactoring_analysis.md`
2. **DTO Patterns**: See existing facade DTOs in `riptide-facade/src/facades/`
3. **Error Handling**: See `crates/riptide-api/src/errors.rs`
4. **Testing Guide**: `docs/testing/integration_tests.md`

---

## 🚨 Common Pitfalls to Avoid

1. **Don't break existing API contracts**
   - Response formats must stay the same
   - Test all endpoints after changes

2. **Don't introduce performance regressions**
   - DTO conversions are minimal overhead
   - Keep conversions simple (no complex computation)

3. **Don't skip tests**
   - Run tests after every file change
   - Integration tests are critical

4. **Don't forget imports**
   - Update all `use` statements
   - Run clippy to catch unused imports

5. **Don't mix concerns**
   - DTOs in `dto/` module only
   - Business logic in facades only
   - Handlers are pure HTTP mapping

---

## 📞 Need Help?

- **Build errors**: Check import paths and DTO modules
- **Test failures**: Verify response format unchanged
- **LOC still high**: Look for inline response mapping to extract
- **Questions**: See detailed analysis document

---

**Status**: Ready to implement
**Priority**: High (Phase 3 Sprint 3.1)
**Estimated Effort**: 2-3 weeks
**Risk Level**: Low (refactoring only, no new functionality)
