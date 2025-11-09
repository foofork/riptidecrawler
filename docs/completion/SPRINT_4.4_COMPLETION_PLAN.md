# Sprint 4.4: Resource Manager Facade Integration - Completion Plan

## Current Status

### Files Created ✅
1. `crates/riptide-types/src/ports/rate_limit.rs` (137 LOC) - RateLimiter port trait
2. `crates/riptide-cache/src/adapters/redis_rate_limiter.rs` (291 LOC) - Redis adapter
3. `crates/riptide-facade/src/facades/resource.rs` (482 LOC) - ResourceFacade orchestration
4. `crates/riptide-facade/src/metrics/performance.rs` (372 LOC) - PerformanceMonitor

### Remaining Work

#### 1. AppState Integration (state.rs)
- [x] Add `resource_facade: Arc<ResourceFacade<()>>` field to AppState
- [ ] Wire dependencies in `new_base()`:
  - Create mock Pool<()> (placeholder until real WASM pool ready)
  - Create RedisRateLimiter from existing RedisManager
  - Create BusinessMetrics (already exists in AppState)
  - Initialize ResourceFacade with these dependencies

#### 2. Handler Migration
- [ ] **handlers/pdf.rs** (Line 162): Replace `resource_manager.acquire_pdf_resources()` with `resource_facade.acquire_wasm_slot()`
- [ ] **handlers/resources.rs** (Lines 82, 85, 152, 175, 187, 221, 233, 241): Replace ResourceManager calls
- [ ] **handlers/telemetry.rs** (Line 278): Replace ResourceManager calls

#### 3. Cleanup Old Files (Target: <500 LOC remaining)
- [ ] Delete `resource_manager/mod.rs` (653 LOC) - orchestration moved to facade
- [ ] Delete `resource_manager/rate_limiter.rs` (374 LOC) - moved to RedisRateLimiter
- [ ] Delete `resource_manager/performance.rs` (384 LOC) - moved to facade/metrics
- [ ] Keep:
  - `guards.rs` (API layer guards - OK)
  - `errors.rs` (API layer errors - OK)
  - `memory_manager.rs` (specific to API layer)
  - `metrics.rs` (API layer metrics)
  - `wasm_manager.rs` (WASM-specific logic)

#### 4. Testing & Validation
- [ ] Run: `cargo test -p riptide-facade --test resource`
- [ ] Run: `cargo clippy --all -- -D warnings`
- [ ] Verify handlers compile and function correctly

## Implementation Steps

### Step 1: Wire ResourceFacade in AppState
```rust
// In AppState::new_base() after resource_manager initialization:

// Create mock WASM pool (placeholder)
let wasm_pool = Arc::new(MockPool::new()) as Arc<dyn Pool<()>>;

// Create Redis rate limiter
let redis_rate_limiter = Arc::new(
    RedisRateLimiter::new(
        resource_manager.redis_manager.clone(),
        api_config.rate_limiting.requests_per_second_per_host as usize,
        Duration::from_secs(60),
    )
) as Arc<dyn RateLimiter>;

// Get business metrics
let business_metrics = Arc::new(BusinessMetrics::new()?) as Arc<dyn riptide_types::ports::BusinessMetrics>;

// Initialize ResourceFacade
let resource_facade = Arc::new(ResourceFacade::new(
    wasm_pool,
    redis_rate_limiter,
    business_metrics,
    ResourceConfig::default(),
));
```

### Step 2: Update Handler Pattern
```rust
// OLD:
match state.resource_manager.acquire_pdf_resources().await {
    Ok(ResourceResult::Success(guard)) => Ok(guard),
    // ...
}

// NEW:
match state.resource_facade.acquire_wasm_slot(tenant_id).await? {
    ResourceResult::Success(slot) => {
        // Use slot
    },
    ResourceResult::RateLimited { retry_after } => {
        Err(ApiError::rate_limited())
    },
    // ...
}
```

### Step 3: Delete Old Files
After handlers are migrated and tests pass:
```bash
rm crates/riptide-api/src/resource_manager/mod.rs
rm crates/riptide-api/src/resource_manager/rate_limiter.rs
rm crates/riptide-api/src/resource_manager/performance.rs
```

## Success Criteria
- [ ] AppState has working ResourceFacade with proper dependencies
- [ ] All handlers use ResourceFacade instead of ResourceManager
- [ ] Old orchestration files deleted
- [ ] Tests passing: `cargo test -p riptide-facade --test resource`
- [ ] Zero clippy warnings: `cargo clippy --all -- -D warnings`
- [ ] resource_manager directory <500 LOC (currently 3231 LOC)

## Notes
- ResourceManager is NOT deleted - it still manages browser pools, WASM instances, etc.
- Only orchestration logic (mod.rs), rate limiting, and performance monitoring are moved
- Guards and errors remain in API layer where they belong
