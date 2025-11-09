# Next Agent Instructions - API Fix Assignment

**Priority:** 🔴 **CRITICAL**
**Estimated Time:** 4-6 hours
**Agent Type:** Coder Agent (API Specialist)
**Prerequisites:** Read PHASE_5_RIPTIDE_API_ERROR_REPORT.md

---

## Mission

Fix **42 compilation errors** in `riptide-api` crate to unblock full workspace validation and browser testing readiness.

---

## Quick Start

### 1. Verify Current State
```bash
# Check disk space (should have >10GB)
df -h / | head -2

# Confirm error count
cargo check -p riptide-api 2>&1 | grep "^error\[" | wc -l
# Expected: 42 errors

# Get detailed error list
cargo check -p riptide-api 2>&1 | grep -E "^error\[" | sort -u
```

### 2. Read Documentation
```bash
# Detailed error analysis
cat docs/completion/PHASE_5_RIPTIDE_API_ERROR_REPORT.md

# Overall progress
cat docs/completion/PHASE_5_FINAL_SUMMARY.md

# Validation status
cat docs/completion/PHASE_5_VALIDATION_PROGRESS.md
```

---

## Phased Fix Approach

### Phase 1: Foundation Fixes (30-45 minutes) 🏗️

**Objective:** Fix critical infrastructure issues that block multiple handlers

#### 1.1 Add Missing ApiError Variant
```bash
# Find ApiError enum definition
rg "pub enum ApiError" crates/riptide-api/src/

# Add variant:
#[error("Rate limit exceeded for tenant: {tenant_id}")]
RateLimitExceeded { tenant_id: String },
```

**Test:**
```bash
cargo check -p riptide-api 2>&1 | grep "RateLimitExceeded"
# Should show 0 results after fix
```

#### 1.2 Fix CacheStorage Future Resolution
```bash
# Find the issue
rg "impl Future<Output = Result<RedisStorage" crates/riptide-api/

# Add .await to resolve Future before using as CacheStorage
# Example fix:
# Before: let cache = create_redis_storage();
# After:  let cache = create_redis_storage().await?;
```

**Test:**
```bash
cargo check -p riptide-api 2>&1 | grep "CacheStorage"
# Should reduce from 2 errors to 0
```

#### 1.3 Add Serialize Derive
```bash
# Find FacadeTableSummary
rg "struct FacadeTableSummary" crates/riptide-api/src/

# Add derive:
#[derive(Debug, Clone, Serialize)]
pub struct FacadeTableSummary { ... }
```

**Test:**
```bash
cargo check -p riptide-api 2>&1 | grep "FacadeTableSummary"
# Should show 0 results
```

#### 1.4 Fix BusinessMetrics Trait Implementation
```bash
# Find BusinessMetrics usage
rg "BusinessMetrics" crates/riptide-api/src/

# Either:
# Option A: Implement the port trait
# Option B: Convert facade metrics to port metrics
# Option C: Use facade metrics directly if port is not needed
```

**Test:**
```bash
cargo check -p riptide-api 2>&1 | grep "BusinessMetrics"
# Should reduce errors
```

**Phase 1 Target:** Reduce to ~30 errors

---

### Phase 2: Facade Method Additions (1-2 hours) 🔌

**Objective:** Add or update facade method calls to match current API

#### 2.1 ProfileFacade Methods
```bash
# Find ProfileFacade usage
rg "ProfileFacade" crates/riptide-api/src/handlers/

# Check what methods are available
rg "impl ProfileFacade" crates/riptide-facade/src/

# For each missing method:
# Option A: Add method to facade
# Option B: Update handler to use existing method
# Option C: Remove handler if feature removed
```

Missing methods to address:
- `create_profile` (E0599)
- `batch_create_profiles` (E0599)
- `get_caching_metrics` (E0599)
- `clear_all_caches` (E0599)

**Test each fix:**
```bash
cargo check -p riptide-api 2>&1 | grep "create_profile"
# Repeat for each method
```

#### 2.2 ProfileManager Methods
```bash
# Find ProfileManager
rg "ProfileManager::" crates/riptide-api/src/handlers/

# Check facade implementation
cat crates/riptide-facade/src/facades/mod.rs

# Add or update:
# - search() method
# - list_by_tag() method
```

#### 2.3 StreamingModule Initialization
```bash
# Find streaming initialization
rg "with_lifecycle_manager" crates/riptide-api/src/

# Check current API
rg "impl StreamingModule" crates/riptide-streaming/src/

# Update initialization call to match current API
```

#### 2.4 TableFacade Stats
```bash
# Find table stats usage
rg "get_extraction_stats" crates/riptide-api/src/

# Either add method or use alternative stats API
```

**Phase 2 Target:** Reduce to ~15-20 errors

---

### Phase 3: Type System Fixes (1-2 hours) 🔧

**Objective:** Fix field access and type conversion issues

#### 3.1 DomainProfile Fields
```bash
# Find DomainProfile definition
rg "struct DomainProfile" crates/

# Compare with API usage
rg "avg_response_time_ms\|last_accessed\|success_rate\|total_requests" crates/riptide-api/src/

# Option A: Add fields to DomainProfile
# Option B: Create DTO with needed fields
# Option C: Update serialization to map from different field names
```

Missing fields:
- `avg_response_time_ms`
- `last_accessed`
- `success_rate`
- `total_requests`

#### 3.2 ResourceStatus Fields
```bash
# Find ResourceStatus
rg "struct ResourceStatus" crates/

# Add missing fields or use alternative API:
# - headless_pool_capacity
# - headless_pool_in_use
```

#### 3.3 Cookie Type Conversions
```bash
# Find cookie handling
rg "CookieResponse" crates/riptide-api/src/

# Implement From trait or use manual conversion:
impl From<&Cookie> for CookieResponse {
    fn from(cookie: &Cookie) -> Self {
        CookieResponse {
            name: cookie.name.clone(),
            value: cookie.value.clone(),
            // ... map fields
        }
    }
}
```

#### 3.4 String Response Construction
```bash
# Find String response errors
cargo check -p riptide-api 2>&1 | grep "IntoResponseParts" -A 5

# Wrap String in proper response type:
# Before: (String, StatusCode)
# After:  (Json(string), StatusCode)
# Or:     Response::builder().body(string).unwrap()
```

**Phase 3 Target:** Reduce to ~5-10 errors

---

### Phase 4: Logic Fixes (1 hour) 🐛

**Objective:** Fix remaining logic and syntax issues

#### 4.1 ResourceResult Pattern Matching
```bash
# Find pattern match
cargo check -p riptide-api 2>&1 | grep "ResourceResult" -A 10

# Add missing arms:
Ok(ResourceResult::RateLimited { .. }) => { ... },
Ok(ResourceResult::Error(e)) => { ... },
```

#### 4.2 Function Argument Fixes
```bash
# Find argument mismatches
cargo check -p riptide-api 2>&1 | grep "E0061" -A 5

# Update function calls to match signatures
```

#### 4.3 Request Ownership Fix
```bash
# Find borrow issue
cargo check -p riptide-api 2>&1 | grep "E0382" -A 10

# Either clone before partial move or restructure
```

#### 4.4 UnboundedReceiver Iterator
```bash
# Find iterator usage
rg "UnboundedReceiver.*iter" crates/riptide-api/src/

# Use proper async receiver:
# Before: for update in receiver { ... }
# After:  while let Some(update) = receiver.recv().await { ... }
```

#### 4.5 f64 Multiplication Fix
```bash
# Find numeric operation
cargo check -p riptide-api 2>&1 | grep "multiply.*f64" -A 5

# Cast integer to f64:
# Before: value * 100
# After:  value * 100.0  or  value * 100 as f64
```

**Phase 4 Target:** 0 errors ✅

---

## Validation Commands

### After Each Phase
```bash
# Quick error count
cargo check -p riptide-api 2>&1 | grep "^error\[" | wc -l

# Detailed errors remaining
cargo check -p riptide-api 2>&1 | grep -E "^error\[" | sort -u

# Check specific error type
cargo check -p riptide-api 2>&1 | grep "E0599"  # Missing methods
cargo check -p riptide-api 2>&1 | grep "E0277"  # Trait bounds
cargo check -p riptide-api 2>&1 | grep "E0308"  # Type mismatches
```

### Final Validation (After All Fixes)
```bash
# 1. Zero errors
cargo check -p riptide-api
# Expected: "Finished" with no errors

# 2. Full workspace
cargo check --workspace
# Expected: All 23 crates compile

# 3. Zero warnings (CLAUDE.md requirement)
RUSTFLAGS="-D warnings" cargo check -p riptide-api
cargo clippy -p riptide-api -- -D warnings

# 4. Run tests
cargo test -p riptide-api --lib
```

---

## Success Criteria

- [ ] `cargo check -p riptide-api` - 0 errors
- [ ] `cargo check --workspace` - All crates compile
- [ ] `cargo clippy -p riptide-api -- -D warnings` - 0 warnings
- [ ] `cargo test -p riptide-api --lib` - Tests run (may fail, but compile)
- [ ] Document changes made

---

## Tips for Efficiency

### 1. Use Incremental Compilation
```bash
# Check only riptide-api during fixes
cargo check -p riptide-api

# Only run full workspace check when close to complete
cargo check --workspace
```

### 2. Fix by Error Type
Focus on one error type at a time:
```bash
# All missing method errors
cargo check -p riptide-api 2>&1 | grep "E0599"

# All trait bound errors
cargo check -p riptide-api 2>&1 | grep "E0277"
```

### 3. Use Rust Analyzer
```bash
# Get detailed error with suggestions
cargo check -p riptide-api 2>&1 | grep "error\[E" -A 20
```

### 4. Test Incrementally
After fixing a group of related errors:
```bash
cargo check -p riptide-api 2>&1 | tail -20
```

---

## Common Patterns

### Pattern 1: Missing Method
```rust
// Handler calls:
facade.missing_method()

// Fix Option A: Add to facade
impl ProfileFacade {
    pub fn missing_method(&self) -> Result<...> {
        // Implementation
    }
}

// Fix Option B: Use existing method
facade.existing_alternative_method()
```

### Pattern 2: Missing Field
```rust
// Handler accesses:
profile.missing_field

// Fix Option A: Add field to struct
pub struct DomainProfile {
    pub missing_field: Type,
    // ...
}

// Fix Option B: Create DTO
#[derive(Serialize)]
pub struct DomainProfileDto {
    pub missing_field: Type,
    // ...
}

impl From<DomainProfile> for DomainProfileDto {
    fn from(profile: DomainProfile) -> Self {
        // Map fields
    }
}
```

### Pattern 3: Trait Not Satisfied
```rust
// Error: Type X doesn't implement trait Y

// Fix Option A: Implement trait
impl TraitY for TypeX {
    // Implementation
}

// Fix Option B: Convert to type that implements trait
let converted = TypeX::into_trait_impl();

// Fix Option C: Use different type altogether
```

### Pattern 4: Future Not Awaited
```rust
// Error: Result<X> is not a Future

// Fix: Remove .await from non-async call
// Before: let x = sync_function().await;
// After:  let x = sync_function();
```

---

## Documentation Requirements

### Update as You Fix
Create: `docs/completion/API_FIXES_APPLIED.md`

Document:
1. Each error fixed
2. Approach chosen (Option A, B, or C)
3. Files modified
4. Breaking changes (if any)
5. Migration notes for API users

### Template
```markdown
## Fix: Missing ProfileFacade.create_profile

**Error:** E0599 - Method not found
**Files:**
- crates/riptide-facade/src/facades/mod.rs
- crates/riptide-api/src/handlers/profiles.rs

**Approach:** Added method to ProfileFacade

**Changes:**
- Added `create_profile()` method with X, Y, Z parameters
- Updated handler to use new method
- Added test coverage

**Breaking Changes:** None (new method)
```

---

## Coordination

### Before Starting
```bash
npx claude-flow@alpha hooks pre-task --description "api-fix-phase-1"
```

### During Work
```bash
# After each phase
npx claude-flow@alpha hooks post-edit --file "crates/riptide-api/src/handlers/profiles.rs" --memory-key "swarm/coder/phase-1-complete"
```

### After Completion
```bash
npx claude-flow@alpha hooks post-task --task-id "api-fix-complete"
```

---

## Emergency Fallback

### If Stuck on Specific Error
1. Document the blocker
2. Skip to next error category
3. Return to blocker after other fixes
4. Ask for help in docs/completion/BLOCKERS.md

### If Time Exceeds Estimate
1. Document progress
2. Update error count
3. Create continuation plan
4. Hand off to next shift

---

## Final Deliverables

1. ✅ Zero compilation errors in riptide-api
2. ✅ Full workspace compiles
3. ✅ Documentation of changes
4. ✅ Ready for test suite execution
5. ✅ Ready for clippy validation

---

**Good Luck! You've got this! 🚀**

*Remember: Incremental progress. Test after each fix. Document your changes.*

---

*Generated by QA Validation Agent*
*Date: 2025-11-09*
*For: API Fix Agent (Next in sequence)*
