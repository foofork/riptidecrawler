# Handler Refactoring Analysis - Phase 3 Sprint 3.1

**Goal**: Reduce all handlers to <50 LOC by extracting business logic to facades.

**Analysis Date**: 2025-11-08
**Target**: 6 handlers exceeding 50 LOC target

---

## Executive Summary

| Handler | Current LOC | Target LOC | Reduction Needed | Primary Issues |
|---------|------------|-----------|------------------|----------------|
| workers.rs | 292 | <50 | -242 (-82.9%) | DTOs, conversions, inline response mapping |
| profiles.rs | 230 | <50 | -180 (-78.3%) | DTO conversions, response mapping, cache logic |
| sessions.rs | 212 | <50 | -162 (-76.4%) | Repetitive error handling, response mapping |
| pdf.rs | 147 | <50 | -97 (-66.0%) | Resource acquisition helper, test code |
| engine_selection.rs | 112 | <50 | -62 (-55.4%) | DTO mapping, flag conversion |
| tables.rs | 97 | <50 | -47 (-48.5%) | Validation helpers, default functions |

---

## 1. workers.rs Analysis (292 LOC → <50 LOC)

### Current Structure Breakdown
```
Lines 1-21:   Documentation & imports (21 LOC)
Lines 22-194: DTO definitions (173 LOC) ⚠️ MOVE TO SEPARATE MODULE
Lines 196-292: Handler functions (97 LOC)
```

### Issues Identified

1. **DTO Bloat (173 LOC)**
   - 11 request/response DTOs defined inline
   - `From` trait implementations (57-86, 77-87)
   - Should be in separate `api/dto/workers.rs` module

2. **Inline Response Mapping (Multiple handlers)**
   ```rust
   // Line 215: Inline JSON construction
   Ok(Json(SubmitJobResponse {
       job_id, status: "submitted".to_string(),
       submitted_at: Utc::now(),
       message: "Job submitted successfully".to_string()
   }))

   // Line 225: Complex inline transformation
   let processing_time_ms = if let (Some(s), Some(c)) = (job.started_at, job.completed_at) {
       Some((c - s).num_milliseconds() as u64)
   } else {
       job.started_at.map(|s| (Utc::now() - s).num_milliseconds() as u64)
   };
   ```

3. **Job Type String Mapping (Lines 290-291)**
   - Complex pattern matching inline in handler
   - Should be in facade or DTO module

### Refactoring Plan

**Phase 1: Extract DTOs (Saves 173 LOC)**
```rust
// NEW: crates/riptide-api/src/dto/workers.rs
// Move all request/response types here
pub struct SubmitJobRequest { ... }
pub struct JobStatusResponse { ... }
impl From<Job> for JobStatusResponse { ... }
```

**Phase 2: Extract Response Mapping (Saves ~40 LOC)**
```rust
// ADD TO: WorkersFacade
impl WorkersFacade {
    pub fn map_to_status_response(job: Job) -> JobStatusResponse {
        let processing_time_ms = calculate_processing_time(&job);
        JobStatusResponse { /* ... */ }
    }

    pub fn calculate_processing_time(job: &Job) -> Option<u64> {
        // Extract complex time calculation
    }

    pub fn format_job_type(job_type: &JobType) -> String {
        // Extract job type string formatting
    }
}
```

**Phase 3: Consolidate Handlers**
```rust
// RESULT: workers.rs will be ~50 LOC

use crate::dto::workers::*;

pub async fn submit_job(
    State(state): State<AppState>,
    Json(request): Json<SubmitJobRequest>,
) -> Result<Json<SubmitJobResponse>, StatusCode> {
    let job = request.into_job()?; // Conversion in DTO
    let job_id = state.worker_service.submit_job(job).await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    state.metrics.record_worker_job_submission();
    Ok(Json(SubmitJobResponse::new(job_id))) // Factory in DTO
}
```

**Target Structure:**
- `workers.rs`: 45 LOC (handlers only)
- `dto/workers.rs`: 180 LOC (all DTOs)
- `facades/workers.rs`: +30 LOC (response mapping)

---

## 2. profiles.rs Analysis (230 LOC → <50 LOC)

### Current Structure Breakdown
```
Lines 1-22:   Documentation & imports (22 LOC)
Lines 23-130: DTO definitions (108 LOC) ⚠️ MOVE TO SEPARATE MODULE
Lines 131-138: Helper converters (8 LOC) ⚠️ MOVE TO DTO MODULE
Lines 140-230: Handler functions (91 LOC)
```

### Issues Identified

1. **DTO Bloat (108 LOC)**
   - 9 request/response DTOs (lines 26-130)
   - Inline helper converters (lines 132-138)
   - Should be in `api/dto/profiles.rs`

2. **Complex Cache Status Mapping (Lines 185-186)**
   ```rust
   let cache_status = if let Some((engine, confidence, expires_at)) = p.get_cached_engine_info() {
       CacheStatusInfo {
           has_cached_engine: true,
           is_valid: p.is_cache_valid(),
           engine: Some(format!("{:?}", engine)),
           confidence: Some(confidence),
           expires_at: Some(expires_at.to_rfc3339())
       }
   } else {
       CacheStatusInfo { /* defaults */ }
   };
   ```

3. **Inline Cache Clearing Logic (Lines 216-218)**
   ```rust
   for mut p in profiles {
       if p.preferred_engine.is_some() {
           p.invalidate_cache();
           match ProfileManager::save(&p, None) { /* ... */ }
       }
   }
   ```

4. **Metrics Aggregation (Lines 225-229)**
   - Complex inline statistics calculation
   - Should be in facade

### Refactoring Plan

**Phase 1: Extract DTOs (Saves 108 LOC)**
```rust
// NEW: crates/riptide-api/src/dto/profiles.rs
pub struct CreateProfileRequest { ... }
pub struct ProfileStatsResponse { ... }
pub struct CacheStatusInfo { ... }

impl From<&DomainProfile> for ProfileStatsResponse {
    fn from(profile: &DomainProfile) -> Self { /* ... */ }
}

impl CacheStatusInfo {
    pub fn from_profile(profile: &DomainProfile) -> Self {
        // Extract cache status mapping logic
    }
}
```

**Phase 2: Move Cache Operations to Facade (Saves ~30 LOC)**
```rust
// ADD TO: ProfileFacade
impl ProfileFacade {
    pub fn clear_all_caches(&self) -> Result<(usize, usize)> {
        // Move inline cache clearing logic here
        let profiles = ProfileManager::list_all()?;
        let (mut cleared, mut failed) = (0, 0);
        for mut p in profiles {
            if p.preferred_engine.is_some() {
                p.invalidate_cache();
                match ProfileManager::save(&p, None) {
                    Ok(_) => cleared += 1,
                    Err(_) => failed += 1,
                }
            }
        }
        Ok((cleared, failed))
    }

    pub fn get_caching_metrics(&self) -> Result<CachingMetrics> {
        // Move metrics calculation here
    }
}
```

**Phase 3: Consolidate Handlers**
```rust
// RESULT: profiles.rs will be ~45 LOC

use crate::dto::profiles::*;

pub async fn get_profile_stats(
    State(_state): State<AppState>,
    Path(domain): Path<String>,
) -> Result<impl IntoResponse, ApiError> {
    let profile = ProfileManager::load(&domain)
        .map_err(|_| ApiError::NotFound { resource: format!("Profile: {}", domain) })?;
    Ok(Json(ProfileStatsResponse::from(&profile))) // Conversion in DTO
}

pub async fn clear_all_caches(
    State(_state): State<AppState>,
) -> Result<impl IntoResponse, ApiError> {
    let facade = ProfileFacade::new();
    let (cleared, failed) = facade.clear_all_caches()
        .map_err(|e| ApiError::InternalError { message: e.to_string() })?;
    Ok(Json(serde_json::json!({
        "success": true, "cleared": cleared, "failed": failed
    })))
}
```

**Target Structure:**
- `profiles.rs`: 45 LOC (handlers only)
- `dto/profiles.rs`: 130 LOC (all DTOs + conversions)
- `facades/profile.rs`: +50 LOC (cache ops, metrics)

---

## 3. sessions.rs Analysis (212 LOC → <50 LOC)

### Current Structure Breakdown
```
Lines 1-16:   Documentation & imports (16 LOC)
Lines 17-67:  DTO definitions (51 LOC) ⚠️ MOVE TO SEPARATE MODULE
Lines 69-212: Handler functions (144 LOC) ⚠️ REPETITIVE ERROR HANDLING
```

### Issues Identified

1. **DTO Bloat (51 LOC)**
   - 7 request/response DTOs
   - Should be in `api/dto/sessions.rs`

2. **Repetitive Error Handling Pattern (10 handlers)**
   ```rust
   // REPEATED 10+ TIMES:
   .map_err(|e| {
       state.metrics.record_error(crate::metrics::ErrorType::Redis);
       ApiError::dependency("session_manager", e.to_string())
   })?
   ```

3. **Inline Response Mapping (Multiple handlers)**
   ```rust
   // Lines 74-79: Complex Unix timestamp conversion
   CreateSessionResponse {
       session_id: session.session_id.clone(),
       user_data_dir: session.user_data_dir.to_string_lossy().to_string(),
       created_at: session.created_at.duration_since(UNIX_EPOCH)
           .unwrap_or_default().as_secs().to_string(),
       expires_at: session.expires_at.duration_since(UNIX_EPOCH)
           .unwrap_or_default().as_secs().to_string(),
   }

   // Lines 134-142: Cookie response conversion
   CookieResponse {
       name: cookie.name,
       value: cookie.value,
       domain: cookie.domain,
       path: cookie.path,
       expires: cookie.expires.map(|exp| /* ... */),
       secure: cookie.secure,
       http_only: cookie.http_only,
   }
   ```

### Refactoring Plan

**Phase 1: Extract DTOs (Saves 51 LOC)**
```rust
// NEW: crates/riptide-api/src/dto/sessions.rs
pub struct CreateSessionResponse { ... }
pub struct SessionInfoResponse { ... }
pub struct CookieResponse { ... }

impl From<&Session> for CreateSessionResponse {
    fn from(session: &Session) -> Self {
        // Move timestamp conversion here
    }
}

impl From<Cookie> for CookieResponse {
    fn from(cookie: Cookie) -> Self {
        // Move cookie mapping here
    }
}
```

**Phase 2: Create Error Handling Helper (Saves ~50 LOC)**
```rust
// ADD TO: handlers/sessions.rs or utils
fn handle_session_error<T>(
    result: Result<T, E>,
    state: &AppState,
) -> Result<T, ApiError> {
    result.map_err(|e| {
        state.metrics.record_error(ErrorType::Redis);
        ApiError::dependency("session_manager", e.to_string())
    })
}

// OR: Add to SessionManager facade
impl SessionManagerFacade {
    async fn get_session_safe(&self, id: &str) -> Result<Option<Session>, ApiError> {
        self.inner.get_session(id).await
            .map_err(|e| {
                self.metrics.record_error(ErrorType::Redis);
                ApiError::dependency("session_manager", e.to_string())
            })
    }
}
```

**Phase 3: Consolidate Handlers**
```rust
// RESULT: sessions.rs will be ~40 LOC

use crate::dto::sessions::*;

pub async fn create_session(
    State(state): State<AppState>,
) -> Result<impl IntoResponse, ApiError> {
    let session = handle_session_error(
        state.session_manager.create_session().await,
        &state
    )?;
    Ok(Json(CreateSessionResponse::from(&session))) // Conversion in DTO
}

pub async fn get_cookie(
    State(state): State<AppState>,
    Path((session_id, domain, name)): Path<(String, String, String)>,
) -> Result<impl IntoResponse, ApiError> {
    let cookie = handle_session_error(
        state.session_manager.get_cookie(&session_id, &domain, &name).await,
        &state
    )?.ok_or_else(|| ApiError::not_found("Cookie not found"))?;
    Ok(Json(CookieResponse::from(cookie))) // Conversion in DTO
}
```

**Target Structure:**
- `sessions.rs`: 40 LOC (handlers + error helper)
- `dto/sessions.rs`: 80 LOC (all DTOs + conversions)

---

## 4. pdf.rs Analysis (147 LOC → <50 LOC)

### Current Structure Breakdown
```
Lines 1-11:   Documentation & imports (11 LOC)
Lines 12-28:  DTO definitions (17 LOC)
Lines 33-103: Handler functions (71 LOC)
Lines 105-147: Test code (43 LOC) ⚠️ ACCEPTABLE (tests don't count)
```

### Issues Identified

1. **Resource Acquisition Helper (Lines 94-103)**
   - 11 LOC helper function
   - Repetitive pattern matching
   - Could be simplified or moved

2. **Inline Facade Options Creation (Lines 41-44, 61-64)**
   ```rust
   let options = riptide_facade::facades::PdfProcessOptions {
       extract_text: true,
       extract_metadata: true,
       extract_images: false,
       include_page_numbers: true,
       filename: req.filename,
       url: req.url,
       timeout: req.timeout,
   };
   ```

3. **Test Code (43 LOC)**
   - Tests don't count toward handler LOC
   - Keep as-is

### Refactoring Plan

**Phase 1: Simplify Resource Acquisition (Saves ~5 LOC)**
```rust
// ADD TO: ResourceManager or make it a method
impl ResourceManager {
    pub async fn acquire_pdf_or_error(
        &self,
        metrics: &Metrics,
    ) -> Result<PdfResourceGuard, ApiError> {
        self.acquire_pdf_resources().await
            .and_then(|result| match result {
                ResourceResult::Success(guard) => Ok(guard),
                ResourceResult::Timeout => Err(ApiError::timeout("Resource acquisition", "...")),
                ResourceResult::ResourceExhausted => Err(ApiError::internal("...")),
                // ... other cases
            })
            .map_err(|e| {
                metrics.record_error(ErrorType::Http);
                e
            })
    }
}
```

**Phase 2: Extract Options Builder (Saves ~6 LOC)**
```rust
// ADD TO: dto/pdf.rs or PdfProcessRequest
impl PdfProcessRequest {
    pub fn to_facade_options(&self) -> PdfProcessOptions {
        PdfProcessOptions {
            extract_text: true,
            extract_metadata: true,
            extract_images: false,
            include_page_numbers: true,
            filename: self.filename.clone(),
            url: self.url.clone(),
            timeout: self.timeout,
        }
    }
}
```

**Phase 3: Consolidate Handlers**
```rust
// RESULT: pdf.rs will be ~35 LOC (excluding tests)

pub async fn process_pdf(
    State(state): State<AppState>,
    Json(req): Json<PdfProcessRequest>,
) -> Result<Json<PdfProcessResponse>, ApiError> {
    let pdf_data = req.pdf_data
        .ok_or_else(|| ApiError::validation("PDF data required"))?;
    let _guard = state.resource_manager
        .acquire_pdf_or_error(&state.metrics).await?;

    let facade = PdfFacade::new();
    let result = facade
        .process_pdf(PdfInput::Base64(pdf_data), req.to_facade_options())
        .await
        .map_err(|e| {
            state.metrics.record_error(ErrorType::Http);
            ApiError::from(e)
        })?;

    Ok(Json(PdfProcessResponse {
        success: true,
        document: Some(result.document),
        error: None,
        stats: result.stats,
    }))
}
```

**Target Structure:**
- `pdf.rs`: 35 LOC handlers + 43 LOC tests = 78 LOC total (acceptable)
- `dto/pdf.rs`: +15 LOC (options builder)
- `resource_manager.rs`: +10 LOC (acquire helper)

---

## 5. engine_selection.rs Analysis (112 LOC → <50 LOC)

### Current Structure Breakdown
```
Lines 1-12:   Documentation & imports (12 LOC)
Lines 13-44:  DTO definitions (32 LOC) ⚠️ MOVE TO SEPARATE MODULE
Lines 46-112: Handler functions (67 LOC)
```

### Issues Identified

1. **DTO Bloat (32 LOC)**
   - 4 request/response DTOs
   - Should be in `api/dto/engine_selection.rs`

2. **Repetitive Flag Conversion (Lines 52-56, 66-73)**
   ```rust
   // REPEATED TWICE:
   flags: EngineSelectionFlags {
       use_visible_text_density: request.flags.use_visible_text_density,
       detect_placeholders: request.flags.detect_placeholders,
       probe_first_spa: request.flags.probe_first_spa,
   }
   ```

3. **Inline Criteria Building (Lines 52-56, 66-73)**
   ```rust
   let criteria = EngineSelectionCriteria {
       html: request.html,
       url: request.url,
       flags: /* ... */,
   };
   ```

### Refactoring Plan

**Phase 1: Extract DTOs (Saves 32 LOC)**
```rust
// NEW: crates/riptide-api/src/dto/engine_selection.rs
pub struct AnalyzeRequest { ... }
pub struct DecideRequest { ... }
pub struct EngineSelectionFlagsRequest { ... }

impl From<EngineSelectionFlagsRequest> for EngineSelectionFlags {
    fn from(req: EngineSelectionFlagsRequest) -> Self {
        Self {
            use_visible_text_density: req.use_visible_text_density,
            detect_placeholders: req.detect_placeholders,
            probe_first_spa: req.probe_first_spa,
        }
    }
}

impl AnalyzeRequest {
    pub fn to_criteria(&self) -> EngineSelectionCriteria {
        EngineSelectionCriteria {
            html: self.html.clone(),
            url: self.url.clone(),
            flags: EngineSelectionFlags::default(),
        }
    }
}

impl DecideRequest {
    pub fn to_criteria(&self) -> EngineSelectionCriteria {
        EngineSelectionCriteria {
            html: self.html.clone(),
            url: self.url.clone(),
            flags: self.flags.clone().into(),
        }
    }
}
```

**Phase 2: Consolidate Handlers**
```rust
// RESULT: engine_selection.rs will be ~35 LOC

use crate::dto::engine_selection::*;

pub async fn analyze_engine(
    State(_state): State<AppState>,
    Json(request): Json<AnalyzeRequest>,
) -> ApiResult<Json<EngineConfig>> {
    let config = _state.engine_facade
        .select_engine(request.to_criteria()) // Conversion in DTO
        .await?;
    Ok(Json(config))
}

pub async fn decide_engine(
    State(_state): State<AppState>,
    Json(request): Json<DecideRequest>,
) -> ApiResult<Json<EngineConfig>> {
    let config = _state.engine_facade
        .select_engine(request.to_criteria()) // Conversion in DTO
        .await?;
    Ok(Json(config))
}
```

**Target Structure:**
- `engine_selection.rs`: 35 LOC (handlers only)
- `dto/engine_selection.rs`: 55 LOC (DTOs + conversions)

---

## 6. tables.rs Analysis (97 LOC → <50 LOC)

### Current Structure Breakdown
```
Lines 1-12:   Documentation & imports (12 LOC)
Lines 13-46:  DTO definitions (34 LOC) ⚠️ MOVE TO SEPARATE MODULE
Lines 47-49:  Helper functions (3 LOC) ⚠️ MOVE TO DTO MODULE
Lines 48-49:  Facade singleton (2 LOC)
Lines 51-97:  Handler functions (47 LOC) ✅ ALREADY GOOD
```

### Issues Identified

1. **DTO Bloat (34 LOC)**
   - 4 request/response DTOs
   - Should be in `api/dto/tables.rs`

2. **Default Helper Functions (Lines 45-46)**
   ```rust
   fn default_true() -> bool { true }
   fn default_max_nesting() -> usize { 3 }
   ```
   - Move to DTO module

3. **Facade Singleton Pattern (Lines 48-49)**
   - Keep or move to module level?

### Refactoring Plan

**Phase 1: Extract DTOs (Saves 34 LOC)**
```rust
// NEW: crates/riptide-api/src/dto/tables.rs
pub struct ApiTableRequest { ... }
pub struct TableResponse { ... }
pub struct ExportQuery { ... }
pub struct TableOptions { ... }

fn default_true() -> bool { true }
fn default_max_nesting() -> usize { 3 }

impl ApiTableRequest {
    pub fn to_facade_request(&self) -> TableExtractionRequest {
        let opts = self.extract_options.as_ref()
            .cloned()
            .unwrap_or_default();
        TableExtractionRequest {
            html_content: self.html_content.clone(),
            include_nested: opts.include_nested,
            preserve_html: opts.preserve_formatting,
            max_nesting_depth: opts.max_nesting_depth,
            min_size: opts.min_size,
            headers_only: opts.headers_only,
            include_headers: opts.include_headers,
            detect_data_types: opts.detect_data_types,
        }
    }
}
```

**Phase 2: Consolidate Handlers**
```rust
// RESULT: tables.rs will be ~40 LOC

use crate::dto::tables::*;

static FACADE: OnceLock<TableFacade> = OnceLock::new();
fn facade() -> &'static TableFacade {
    FACADE.get_or_init(TableFacade::new)
}

pub async fn extract_tables(
    State(state): State<AppState>,
    Json(req): Json<ApiTableRequest>,
) -> Result<impl IntoResponse, ApiError> {
    let start = Instant::now();
    let tables = facade()
        .extract_tables_full(req.to_facade_request()) // Conversion in DTO
        .await
        .map_err(|e| {
            state.metrics.record_error(ErrorType::Wasm);
            ApiError::from(e)
        })?;

    state.metrics.record_http_request(
        "POST", "/api/v1/tables/extract", 200,
        start.elapsed().as_secs_f64()
    );
    Ok((StatusCode::OK, Json(TableResponse {
        total_tables: tables.len(),
        extraction_time_ms: start.elapsed().as_millis() as u64,
        tables,
    })))
}
```

**Target Structure:**
- `tables.rs`: 40 LOC (handlers only)
- `dto/tables.rs`: 60 LOC (DTOs + conversions + helpers)

---

## Implementation Strategy

### Phase 1: Create DTO Module Structure (Week 1)
```bash
mkdir -p crates/riptide-api/src/dto
touch crates/riptide-api/src/dto/mod.rs
touch crates/riptide-api/src/dto/{workers,profiles,sessions,pdf,engine_selection,tables}.rs
```

### Phase 2: Extract DTOs (Week 1-2)
1. Move all request/response types to respective DTO modules
2. Add `From`/`Into` trait implementations for conversions
3. Add builder methods on request types
4. Update handler imports

### Phase 3: Extract Business Logic to Facades (Week 2)
1. Add response mapping methods to facades
2. Add helper methods for complex calculations
3. Move validation logic to facades

### Phase 4: Consolidate Handlers (Week 2-3)
1. Remove inline DTOs
2. Remove inline conversions
3. Use DTO conversion methods
4. Verify all handlers are <50 LOC

### Phase 5: Verification (Week 3)
```bash
# Run this to verify all handlers are <50 LOC
for file in crates/riptide-api/src/handlers/*.rs; do
    lines=$(grep -v '^\s*$' "$file" | grep -v '^\s*//' | wc -l)
    echo "$file: $lines LOC"
done
```

---

## Expected Results

### Before
```
workers.rs:         292 LOC
profiles.rs:        230 LOC
sessions.rs:        212 LOC
pdf.rs:             147 LOC (104 without tests)
engine_selection:   112 LOC
tables.rs:           97 LOC
---------------------------------
TOTAL:            1,090 LOC (1,047 without tests)
```

### After
```
workers.rs:          45 LOC (-247 LOC, -84.6%)
profiles.rs:         45 LOC (-185 LOC, -80.4%)
sessions.rs:         40 LOC (-172 LOC, -81.1%)
pdf.rs:              35 LOC (-69 LOC, -66.3% handlers only)
engine_selection:    35 LOC (-77 LOC, -68.8%)
tables.rs:           40 LOC (-57 LOC, -58.8%)
---------------------------------
TOTAL:              240 LOC (-807 LOC, -74.0%)

NEW DTO MODULE:     ~520 LOC
FACADE ADDITIONS:   ~90 LOC
```

### Quality Metrics
- **Handler LOC Reduction**: 74.0%
- **All handlers <50 LOC**: ✅ 100% compliant
- **Code organization**: ✅ DTOs separated
- **Reusability**: ✅ DTO conversions reusable
- **Testability**: ✅ Facades independently testable

---

## Risk Assessment

| Risk | Probability | Impact | Mitigation |
|------|-------------|--------|------------|
| Breaking changes in DTO conversions | Low | High | Comprehensive test coverage |
| Import path changes | High | Low | Use find/replace, run clippy |
| Performance regression | Very Low | Medium | Inline conversions are minimal overhead |
| Merge conflicts | Medium | Low | Small, focused PRs |

---

## Testing Strategy

### Unit Tests
- Test all DTO conversions
- Test facade response mappers
- Test error handling helpers

### Integration Tests
- Test all handler endpoints
- Verify response format unchanged
- Test error scenarios

### Verification Commands
```bash
# Build and verify no warnings
RUSTFLAGS="-D warnings" cargo build --workspace

# Run handler-specific tests
cargo test -p riptide-api --lib handlers::

# Run integration tests
cargo test -p riptide-api --test integration_tests

# Verify LOC targets
./scripts/verify_handler_loc.sh
```

---

## Conclusion

All 6 handlers can be reduced to <50 LOC by:
1. **Extracting DTOs** to separate module (saves 430 LOC)
2. **Moving conversions** to DTO `From`/`Into` implementations (saves 150 LOC)
3. **Extracting helpers** to facades or utilities (saves 100 LOC)
4. **Consolidating error handling** (saves 127 LOC)

**Total reduction**: 807 LOC (74.0%)
**New organized code**: 610 LOC in proper locations

This achieves the <50 LOC target while improving code organization, testability, and maintainability.
