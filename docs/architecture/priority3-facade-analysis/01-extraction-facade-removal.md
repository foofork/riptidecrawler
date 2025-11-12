# ExtractionFacade Removal Plan

**Facade**: ExtractionFacade
**Action**: Complete Removal
**Reason**: Duplicates existing `ContentExtractor` trait
**Risk Level**: Low
**Estimated Time**: 2-3 hours

---

## Analysis Summary

The `ExtractionFacade` is a wrapper around `Arc<dyn ContentExtractor>` that adds:
- Backpressure management
- Quality gate enforcement
- HTTP fetching logic

**Problem**: All of this logic can be moved to:
1. A separate coordinator/orchestrator (if needed)
2. The existing `ReliableContentExtractor` trait (for retry logic)
3. Handler-level orchestration

**The facade provides NO unique domain abstraction** - it's purely orchestration.

---

## Current State

### Field in ApplicationContext

**File**: `/workspaces/riptidecrawler/crates/riptide-api/src/context.rs`
**Line**: 181

```rust
pub struct ApplicationContext {
    // ... other fields ...

    /// ExtractionFacade for URL-based extraction with quality gates
    pub extraction_facade: Arc<riptide_facade::facades::ExtractionFacade>,

    // ... other fields ...
}
```

### Initialization Code

**Lines**: 1278-1285, 1416-1423, 1872-1876

```rust
// In new() method:
let extraction_facade = Arc::new(
    riptide_facade::facades::ExtractionFacade::new(
        http_client.clone(),
        extractor.clone(),
        config.clone(),
    )
    .await
    .expect("Failed to create extraction facade"),
);

// Later assigned:
self.extraction_facade = Arc::new(...);
```

### Call Sites

**Searched with**:
```bash
grep -rn "\.extraction_facade" /workspaces/riptidecrawler/crates/riptide-api --include="*.rs"
```

**Results**: Only initialization code - **NO HANDLER USAGE FOUND**

This means the facade is initialized but not actually used anywhere! Safe to remove immediately.

---

## Removal Steps

### Step 1: Remove Field Declaration

**File**: `/workspaces/riptidecrawler/crates/riptide-api/src/context.rs`
**Line**: 181-182

```diff
pub struct ApplicationContext {
    // ... other fields ...
-
-   /// ExtractionFacade for URL-based extraction with quality gates
-   pub extraction_facade: Arc<riptide_facade::facades::ExtractionFacade>,

    // ... other fields ...
}
```

### Step 2: Remove Initialization Code

**Lines to delete**:
1. Lines 1278-1285 (initial creation)
2. Lines 1416-1423 (refresh)
3. Lines 1872-1876 (test setup)
4. Line 1378/1983 (field assignment in struct literal)

**File**: `/workspaces/riptidecrawler/crates/riptide-api/src/context.rs`

```diff
// Remove initialization:
-        let extraction_facade = Arc::new(
-            riptide_facade::facades::ExtractionFacade::new(
-                http_client.clone(),
-                extractor.clone(),
-                config.clone(),
-            )
-            .await
-            .expect("Failed to create extraction facade"),
-        );

// Remove from struct literal:
Self {
    http_client,
    cache,
-   extraction_facade,
    // ... other fields ...
}
```

### Step 3: Remove Imports

**File**: `/workspaces/riptidecrawler/crates/riptide-api/src/context.rs`

```diff
-use riptide_facade::facades::ExtractionFacade;
```

(Or remove from glob import if using `use riptide_facade::facades::*;`)

### Step 4: Verify Compilation

```bash
# Check only the affected crate
cargo check -p riptide-api

# Run tests to ensure nothing breaks
cargo test -p riptide-api

# Full clippy check
cargo clippy -p riptide-api -- -D warnings
```

Expected outcome: **Zero errors** since the field is unused.

---

## Alternative: Use Existing Extractor Field

If any code **does** use `extraction_facade`, migrate to existing fields:

### Option 1: Use `extractor` field directly

ApplicationContext already has:
```rust
#[cfg(feature = "extraction")]
pub extractor: Arc<dyn ContentExtractor>,
```

**Migration**:
```rust
// Before:
let result = ctx.extraction_facade.extract_from_url(url, options).await?;

// After:
let html = ctx.http_client.get(url).send().await?.text().await?;
let result = ctx.extractor.extract(&html, url).await?;
```

### Option 2: Use `reliable_extractor` field

ApplicationContext already has:
```rust
pub reliable_extractor: Arc<dyn ReliableContentExtractor>,
```

**Migration**:
```rust
// Before:
let result = ctx.extraction_facade.extract_from_url(url, options).await?;

// After:
let html = ctx.http_client.get(url).send().await?.text().await?;
let result = ctx.reliable_extractor.extract_with_retry(&html, url).await?;
```

The `reliable_extractor` already handles:
- ✅ Retry logic
- ✅ Circuit breaker
- ✅ Quality fallback

---

## Business Logic Migration

If the facade's quality gates and backpressure are needed, create a **coordinator** function:

### Create extraction_coordinator.rs

**File**: `/workspaces/riptidecrawler/crates/riptide-api/src/coordinators/extraction_coordinator.rs`

```rust
//! Extraction coordinator for URL-based extraction with quality gates
//!
//! This coordinator orchestrates:
//! - Backpressure management
//! - HTTP fetching
//! - Content extraction
//! - Quality gate enforcement

use crate::context::ApplicationContext;
use riptide_types::ports::ContentExtractor;
use std::sync::Arc;
use tokio_util::sync::CancellationToken;

pub struct ExtractionCoordinator {
    context: Arc<ApplicationContext>,
    backpressure: BackpressureManager,
}

impl ExtractionCoordinator {
    pub fn new(context: Arc<ApplicationContext>) -> Self {
        Self {
            context,
            backpressure: BackpressureManager::new(50),
        }
    }

    pub async fn extract_from_url(&self, url: &str) -> Result<ExtractedDoc> {
        // 1. Acquire backpressure permit
        let cancel = CancellationToken::new();
        let _guard = self.backpressure.acquire(&cancel).await?;

        // 2. Fetch HTML
        let html = self.context.http_client
            .get(url)
            .send()
            .await?
            .text()
            .await?;

        // 3. Extract with retry
        let result = self.context.reliable_extractor
            .extract_with_retry(&html, url)
            .await?;

        // 4. Apply quality gates
        if result.quality_score < 0.3 {
            return Err(RiptideError::extraction("Quality threshold not met"));
        }

        Ok(result)
    }
}
```

**Benefit**: Keeps orchestration logic separate from domain traits.

---

## Testing Strategy

### Unit Tests

Since the facade is unused, no tests should break.

**Verify with**:
```bash
cargo test -p riptide-api
```

### Integration Tests

If any integration tests reference `extraction_facade`, update them:

```rust
// Before:
let result = context.extraction_facade.extract_from_url("https://example.com", opts).await?;

// After:
let html = context.http_client.get("https://example.com").send().await?.text().await?;
let result = context.extractor.extract(&html, "https://example.com").await?;
```

---

## Rollback Plan

If issues arise, rollback is trivial:

```bash
git checkout HEAD -- crates/riptide-api/src/context.rs
cargo check -p riptide-api
```

The facade can be re-added without side effects since it's a self-contained module.

---

## Benefits of Removal

1. ✅ **Reduces coupling** - One less concrete dependency in ApplicationContext
2. ✅ **Improves testability** - Use existing trait mocks instead of facade mocks
3. ✅ **Simplifies architecture** - Direct trait usage, no wrapper confusion
4. ✅ **Zero breaking changes** - Field is unused, so removal is transparent
5. ✅ **Clearer intent** - `extractor` and `reliable_extractor` are self-documenting

---

## Success Criteria

- ✅ `extraction_facade` field removed from ApplicationContext
- ✅ All initialization code removed
- ✅ `cargo check -p riptide-api` passes
- ✅ `cargo test -p riptide-api` passes
- ✅ `cargo clippy -p riptide-api -- -D warnings` passes
- ✅ Zero references to `extraction_facade` in codebase

---

## Post-Removal Validation

```bash
# 1. Ensure field is gone
grep -n "extraction_facade" /workspaces/riptidecrawler/crates/riptide-api/src/context.rs
# Expected: No results

# 2. Ensure compilation works
cargo check -p riptide-api

# 3. Run tests
cargo test -p riptide-api

# 4. Check for any lingering references
rg "extraction_facade" /workspaces/riptidecrawler/crates/riptide-api --type rust
# Expected: No results
```

---

**Status**: ✅ Ready for Immediate Removal
**Complexity**: Low
**Impact**: Zero (field unused)
**Time Estimate**: 15-30 minutes
