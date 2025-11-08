# Consolidation Roadmap - Detailed Migration Plans
**RipTide EventMesh Project**

**Date**: 2025-11-07
**Scope**: 5-Phase Architecture Refactoring (4-5 weeks, 64 hours)
**Current Progress**: Phase 1.2 Complete (33%)

---

## Executive Summary

### Roadmap Overview

This roadmap provides **detailed, step-by-step migration plans** for resolving 7 architectural violations across the RipTide EventMesh workspace. The term "consolidation" refers to **architectural consolidation** (fixing violations), not crate consolidation (the 31-crate structure is optimal).

### Current Status

| Phase | Tasks | Status | Hours | Progress |
|-------|-------|--------|-------|----------|
| **Phase 1** | Foundation (6 tasks) | 🟡 33% | 16h | 2/6 complete |
| **Phase 2** | Infrastructure (5 tasks) | ⏳ Pending | 12h | 0/5 |
| **Phase 3** | Facade (6 tasks) | ⏳ Pending | 16h | 0/6 |
| **Phase 4** | Handlers (4 tasks) | ⏳ Pending | 12h | 0/4 |
| **Phase 5** | Validation (4 tasks) | ⏳ Pending | 8h | 0/4 |
| **TOTAL** | **25 tasks** | **8% overall** | **64h** | **2/25** |

### Key Metrics

| Metric | Current | After Phase 1 | Final Target | Progress |
|--------|---------|---------------|--------------|----------|
| Types LOC | 2,892 | ~2,200 | 2,000 | 71% → 90% → 100% |
| Domain LOC | 475 (43%) | 859 (100%) | 859 | 55% → 100% |
| Issues Resolved | 0/7 | 2/7 | 7/7 | 0% → 29% → 100% |

---

## Phase 1: Foundation (Week 1, 16 Hours)

### Overview

**Goal**: Establish clean architectural boundaries by extracting 859 lines of business logic from riptide-types to riptide-domain.

**Status**: 🟡 33% Complete (2/6 tasks, 372/859 lines migrated)

**Issues Addressed**: #1 (types purity), #5 (pipeline redis)

---

### Task 1.1: Create riptide-domain Crate Structure ✅ COMPLETE

**Status**: ✅ **DONE** (2025-11-07)
**Effort**: 2 hours (actual)
**Assignee**: Developer 1

#### What Was Done

```bash
# Created crate structure
crates/riptide-domain/
├── Cargo.toml
├── src/
│   ├── lib.rs (module declarations)
│   ├── reliability/ (circuit breaker, timeouts)
│   ├── http/ (caching, conditional requests)
│   ├── security/ (redaction patterns)
│   ├── resilience/ (error classification, retry)
│   └── processing/ (content operations)
```

#### Validation

```bash
cargo check -p riptide-domain  # ✅ Passed
cargo test -p riptide-domain --no-run  # ✅ Passed
```

#### Deliverable

✅ Empty riptide-domain crate with all module structure in place

---

### Task 1.2: Move Circuit Breaker Implementation ✅ COMPLETE

**Status**: ✅ **DONE** (2025-11-07)
**Effort**: 4 hours (actual)
**Assignee**: Developer 1
**LOC Migrated**: 372 lines

#### What Was Done

1. **Copied Implementation**:
   - Source: `riptide-types/src/reliability/circuit.rs` (373 lines)
   - Destination: `riptide-domain/src/reliability/circuit_breaker.rs` (372 lines)
   - Difference: 1 line optimization

2. **Updated riptide-types**:
   - Replaced 373 lines with 14 lines of re-exports
   - Maintains backward compatibility
   - All imports still work

3. **Updated Dependent Crates**:
   - riptide-fetch, riptide-spider, riptide-reliability
   - Changed to import from riptide-domain
   - Zero breaking changes

4. **Testing**:
   - All circuit breaker tests pass (4/4)
   - Full workspace builds cleanly
   - 237 tests across 3 crates pass

#### Code Changes

**Before** (riptide-types/src/reliability/circuit.rs):
```rust
// 373 lines of circuit breaker implementation
pub struct CircuitBreaker { ... }
impl CircuitBreaker { ... }
// Full state machine, metrics, tests
```

**After** (riptide-types/src/reliability/circuit.rs):
```rust
// 14 lines of re-exports
pub use riptide_domain::reliability::{
    CircuitBreaker, CircuitBreakerMetrics, State,
    CircuitBreakerError, Clock, SystemClock
};
```

**New** (riptide-domain/src/reliability/circuit_breaker.rs):
```rust
// 372 lines - full implementation moved
pub struct CircuitBreaker { ... }
impl CircuitBreaker { ... }
```

#### Validation

```bash
cargo test -p riptide-domain -- circuit  # ✅ 4/4 passed
cargo check --workspace  # ✅ Clean build
tokei crates/riptide-types/src/  # ✅ 2,892 lines (down from 3,250)
```

#### Deliverable

✅ Circuit breaker in riptide-domain with all tests passing and -11% types LOC

**Details**: See `/workspaces/eventmesh/reports/PHASE_1_2_COMPLETE.md`

---

### Task 1.3: Move HTTP Caching Logic ⏳ NEXT

**Status**: ⏳ **NEXT** (Week 1, Day 3)
**Effort**: 3 hours (estimated)
**Assignee**: Developer 2
**LOC to Migrate**: 180 lines
**Dependencies**: Task 1.1 ✅

#### Scope

**Files to Migrate**:
1. `riptide-types/src/http/conditional.rs:123-133` (11 lines)
   - ETag generation logic
   - Move to: `riptide-domain/src/http/caching.rs`

2. `riptide-types/src/http/conditional.rs:136-166` (31 lines)
   - HTTP date parsing functions
   - Move to: `riptide-domain/src/http/date_parsing.rs`

3. `riptide-types/src/http/conditional.rs:180-205` (26 lines)
   - Cache validation logic
   - Move to: `riptide-domain/src/http/caching.rs`

4. Additional conditional request logic (~112 lines)
   - Move to: `riptide-domain/src/http/conditional.rs`

**Total**: 180 lines across 3 new files in riptide-domain

#### Step-by-Step Plan

**Step 1: Create Target Files** (30 minutes)

```bash
# Create new files in riptide-domain
touch crates/riptide-domain/src/http/caching.rs
touch crates/riptide-domain/src/http/date_parsing.rs
touch crates/riptide-domain/src/http/conditional.rs

# Update module declarations
# File: crates/riptide-domain/src/http/mod.rs
cat >> crates/riptide-domain/src/http/mod.rs << 'EOF'
pub mod caching;
pub mod date_parsing;
pub mod conditional;

pub use caching::{generate_etag, validate_cache};
pub use date_parsing::{parse_http_date, format_http_date};
pub use conditional::ConditionalRequestLogic;
EOF
```

**Step 2: Move ETag Generation** (30 minutes)

```rust
// Source: riptide-types/src/http/conditional.rs:123-133
// Destination: riptide-domain/src/http/caching.rs

use sha2::{Digest, Sha256};
use chrono::{DateTime, Utc};

pub fn generate_etag(content: &[u8], last_modified: DateTime<Utc>) -> String {
    let mut hasher = Sha256::new();
    hasher.update(content);
    hasher.update(last_modified.to_rfc3339().as_bytes());
    let result = hasher.finalize();
    format!("\"{:x}\"", result)
}
```

**Step 3: Move HTTP Date Parsing** (30 minutes)

```rust
// Source: riptide-types/src/http/conditional.rs:136-166
// Destination: riptide-domain/src/http/date_parsing.rs

use chrono::{DateTime, Utc, TimeZone};

pub fn parse_http_date(date_str: &str) -> Result<DateTime<Utc>, chrono::ParseError> {
    // IMF-fixdate format (RFC 7231)
    if let Ok(dt) = Utc.datetime_from_str(date_str, "%a, %d %b %Y %H:%M:%S GMT") {
        return Ok(dt);
    }
    // RFC 850 format
    if let Ok(dt) = Utc.datetime_from_str(date_str, "%A, %d-%b-%y %H:%M:%S GMT") {
        return Ok(dt);
    }
    // ANSI C asctime() format
    Utc.datetime_from_str(date_str, "%a %b %e %H:%M:%S %Y")
}

pub fn format_http_date(dt: DateTime<Utc>) -> String {
    dt.format("%a, %d %b %Y %H:%M:%S GMT").to_string()
}
```

**Step 4: Move Cache Validation** (30 minutes)

```rust
// Source: riptide-types/src/http/conditional.rs:180-205
// Destination: riptide-domain/src/http/caching.rs

use chrono::{DateTime, Utc};

pub fn validate_cache(
    cached_etag: Option<&str>,
    cached_last_modified: Option<DateTime<Utc>>,
    request_if_none_match: Option<&str>,
    request_if_modified_since: Option<DateTime<Utc>>,
) -> bool {
    // ETag validation (strong validator)
    if let (Some(cached), Some(request)) = (cached_etag, request_if_none_match) {
        if cached == request {
            return true; // Not modified
        }
    }

    // Last-Modified validation (weak validator)
    if let (Some(cached), Some(request)) = (cached_last_modified, request_if_modified_since) {
        if cached <= request {
            return true; // Not modified
        }
    }

    false // Modified or no validators
}
```

**Step 5: Update riptide-types** (30 minutes)

```rust
// File: riptide-types/src/http/conditional.rs
// Replace implementation with re-exports

pub use riptide_domain::http::{
    generate_etag, validate_cache,
    parse_http_date, format_http_date,
    ConditionalRequestLogic,
};

// Keep only data structures
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConditionalRequest {
    pub if_none_match: Option<String>,
    pub if_modified_since: Option<DateTime<Utc>>,
    // ... other fields (data only)
}
```

**Step 6: Update Imports** (30 minutes)

```bash
# Find all usages
rg "generate_etag|parse_http_date|validate_cache" --type rust crates/

# Update imports in dependent crates
# Most should work via re-export
# Some may need explicit: use riptide_domain::http::generate_etag;
```

#### Validation

```bash
# Test domain crate
cargo test -p riptide-domain -- http
# Expected: All HTTP tests pass

# Verify no implementation in types
grep -r "generate_etag\|parse_http_date" crates/riptide-types/src/http/conditional.rs
# Expected: Only re-exports, no impl

# Check types LOC
tokei crates/riptide-types/src/
# Expected: ~2,200 lines (down from 2,892)

# Full workspace build
cargo check --workspace
# Expected: Clean build
```

#### Deliverable

✅ 180 lines of HTTP logic moved to riptide-domain, types crate reduced to ~2,200 lines

---

### Task 1.4: Move Error Classification & Retry Logic

**Status**: ⏳ **TODO** (Week 1, Day 4)
**Effort**: 3 hours (estimated)
**Assignee**: Developer 1
**LOC to Migrate**: 100+ lines
**Dependencies**: Task 1.2 ✅

#### Scope

**Files to Migrate**:
1. `riptide-types/src/error/riptide_error.rs:94-124` (31 lines)
   - `is_retryable()`, `is_transient()`, `classify()` methods
   - Move to: `riptide-domain/src/resilience/classification.rs`

2. `riptide-types/src/error/strategy_error.rs:73-123` (51 lines)
   - Retry logic, backoff calculation
   - Move to: `riptide-domain/src/resilience/retry.rs`

3. Additional error handling logic (~20 lines)

**Total**: 100+ lines across 2 new files

#### Step-by-Step Plan

**Step 1: Create Trait in riptide-types** (30 minutes)

```rust
// File: riptide-types/src/error/traits.rs (NEW)

use async_trait::async_trait;
use std::time::Duration;

#[async_trait]
pub trait ErrorClassifier: Send + Sync {
    fn is_retryable(&self, error: &dyn std::error::Error) -> bool;
    fn is_transient(&self, error: &dyn std::error::Error) -> bool;
    fn classify(&self, error: &dyn std::error::Error) -> ErrorClass;
    fn calculate_backoff(&self, attempt: u32, base_delay: Duration) -> Duration;
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ErrorClass {
    Transient,      // Retry immediately possible
    Retryable,      // Retry with backoff
    Permanent,      // Do not retry
    RateLimited,    // Retry after delay
}
```

**Step 2: Implement Trait in riptide-domain** (60 minutes)

```rust
// File: riptide-domain/src/resilience/classification.rs

use riptide_types::error::{ErrorClassifier, ErrorClass};
use std::time::Duration;

pub struct StandardErrorClassifier {
    max_attempts: u32,
    base_delay: Duration,
}

impl ErrorClassifier for StandardErrorClassifier {
    fn is_retryable(&self, error: &dyn std::error::Error) -> bool {
        // Move implementation from riptide-types
        // 31 lines of classification logic
    }

    fn is_transient(&self, error: &dyn std::error::Error) -> bool {
        // Check for network errors, timeouts, etc.
    }

    fn classify(&self, error: &dyn std::error::Error) -> ErrorClass {
        // Classify by error type
    }

    fn calculate_backoff(&self, attempt: u32, base_delay: Duration) -> Duration {
        // Exponential backoff with jitter
        // Move from strategy_error.rs
    }
}
```

**Step 3: Move Retry Logic** (60 minutes)

```rust
// File: riptide-domain/src/resilience/retry.rs

use riptide_types::error::ErrorClassifier;
use std::time::Duration;

pub struct RetryPolicy {
    max_attempts: u32,
    base_delay: Duration,
    max_delay: Duration,
    classifier: Box<dyn ErrorClassifier>,
}

impl RetryPolicy {
    pub async fn execute<F, T, E>(&self, mut operation: F) -> Result<T, E>
    where
        F: FnMut() -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<T, E>> + Send>>,
        E: std::error::Error,
    {
        // Move retry loop from strategy_error.rs
        // 51 lines of retry logic with backoff
    }
}
```

**Step 4: Update riptide-types** (30 minutes)

```rust
// File: riptide-types/src/error/riptide_error.rs
// Remove impl blocks, keep enum

impl RiptideError {
    // Remove these methods:
    // fn is_retryable(&self) -> bool { ... }
    // fn is_transient(&self) -> bool { ... }
    // fn classify(&self) -> ErrorClass { ... }
}

// Add re-export
pub use riptide_domain::resilience::{
    StandardErrorClassifier, RetryPolicy
};
```

#### Validation

```bash
cargo test -p riptide-domain -- error
cargo test -p riptide-types -- error
cargo clippy -p riptide-types -- -D warnings
tokei crates/riptide-types/src/
# Expected: ~2,100 lines
```

#### Deliverable

✅ Error handling logic separated from error types, trait-based architecture

---

### Task 1.5: Move Security & Processing Logic

**Status**: ⏳ **TODO** (Week 1, Day 4-5)
**Effort**: 2 hours (estimated)
**Assignee**: Developer 2
**LOC to Migrate**: 40+ lines
**Dependencies**: Task 1.1 ✅

#### Scope

**Files to Migrate**:
1. `riptide-types/src/security/secrets.rs:85-111` (27 lines)
   - Redaction patterns for secrets
   - Move to: `riptide-domain/src/security/redaction.rs`

2. `riptide-types/src/http/http_types.rs:248-263` (16 lines)
   - Content truncation logic
   - Move to: `riptide-domain/src/processing/truncation.rs`

3. `riptide-types/src/data/extracted.rs:60-68` (9 lines)
   - Quality scoring algorithm
   - Move to: `riptide-domain/src/processing/quality.rs`

4. `riptide-types/src/data/extracted.rs:71-85` (15 lines)
   - Data converters (CSV, JSON)
   - Move to: `riptide-domain/src/processing/converters.rs`

**Total**: 67 lines across 4 new files

#### Step-by-Step Plan

**Step 1: Secret Redaction** (30 minutes)

```rust
// Destination: riptide-domain/src/security/redaction.rs

use regex::Regex;

pub struct SecretRedactor {
    patterns: Vec<(Regex, &'static str)>,
}

impl SecretRedactor {
    pub fn new() -> Self {
        let patterns = vec![
            (Regex::new(r"(?i)(api[_-]?key\s*[:=]\s*)[\w-]+").unwrap(),
             "$1[REDACTED]"),
            (Regex::new(r"(?i)(password\s*[:=]\s*)[\w-]+").unwrap(),
             "$1[REDACTED]"),
            (Regex::new(r"(?i)(token\s*[:=]\s*)[\w-]+").unwrap(),
             "$1[REDACTED]"),
            // ... 8 more patterns (27 lines total)
        ];
        Self { patterns }
    }

    pub fn redact(&self, input: &str) -> String {
        // Apply all patterns
    }
}
```

**Step 2: Content Truncation** (15 minutes)

```rust
// Destination: riptide-domain/src/processing/truncation.rs

pub fn truncate_with_ellipsis(text: &str, max_length: usize) -> String {
    if text.len() <= max_length {
        return text.to_string();
    }

    let ellipsis = "...";
    let truncate_at = max_length.saturating_sub(ellipsis.len());

    // Try to break at word boundary
    if let Some(pos) = text[..truncate_at].rfind(char::is_whitespace) {
        format!("{}{}", &text[..pos], ellipsis)
    } else {
        format!("{}{}", &text[..truncate_at], ellipsis)
    }
}
```

**Step 3: Quality Scoring** (15 minutes)

```rust
// Destination: riptide-domain/src/processing/quality.rs

pub fn calculate_content_quality(
    text: &str,
    has_metadata: bool,
    extraction_confidence: f64,
) -> f64 {
    let mut score = 0.0;

    // Text length score (0-0.4)
    score += (text.len() as f64 / 1000.0).min(0.4);

    // Metadata bonus (0-0.2)
    if has_metadata {
        score += 0.2;
    }

    // Extraction confidence (0-0.4)
    score += extraction_confidence * 0.4;

    score.min(1.0)
}
```

**Step 4: Data Converters** (30 minutes)

```rust
// Destination: riptide-domain/src/processing/converters.rs

use serde_json::Value;

pub fn table_to_csv(rows: &[Vec<String>]) -> String {
    // Move CSV conversion logic
}

pub fn table_to_json(rows: &[Vec<String>], headers: &[String]) -> Value {
    // Move JSON conversion logic
}
```

**Step 5: Update riptide-types** (30 minutes)

```rust
// Re-export from riptide-domain
pub use riptide_domain::security::SecretRedactor;
pub use riptide_domain::processing::{
    truncate_with_ellipsis, calculate_content_quality,
    table_to_csv, table_to_json,
};
```

#### Validation

```bash
cargo test -p riptide-domain -- security
cargo test -p riptide-domain -- processing
tokei crates/riptide-types/src/
# Expected: ~2,000 lines
```

#### Deliverable

✅ All 67 lines migrated, types crate at target size (~2,000 lines)

---

### Task 1.6: Clean Up & Validate

**Status**: ⏳ **TODO** (Week 1, Day 5)
**Effort**: 2 hours (estimated)
**Assignee**: Both developers
**Dependencies**: Tasks 1.3, 1.4, 1.5 ✅

#### Step-by-Step Plan

**Step 1: Remove Moved Code** (30 minutes)

```bash
# Verify re-exports are in place
grep -r "pub use riptide_domain" crates/riptide-types/src/

# Remove original implementations
# Already done in previous tasks - verify nothing left
```

**Step 2: Update Dependencies** (30 minutes)

```bash
# Update all workspace Cargo.toml files
# Most crates should already have riptide-domain via riptide-types
# Check for any that need explicit dependency

for crate in crates/*/Cargo.toml; do
    if grep -q "riptide-types" "$crate"; then
        # Check if riptide-domain is needed
        echo "Checking $crate"
    fi
done
```

**Step 3: Fix Compilation Errors** (30 minutes)

```bash
# Build entire workspace
cargo check --workspace 2>&1 | tee build-errors.log

# Fix any import errors
# Most should work via re-exports from riptide-types
```

**Step 4: Run Full Test Suite** (30 minutes)

```bash
# Run all tests
cargo test --workspace --no-fail-fast

# Expected: All tests pass (including new domain tests)
```

**Step 5: Measure & Validate** (30 minutes)

```bash
# Check types crate size
tokei crates/riptide-types/src/
# Expected: ~2,000 lines (down from 3,250)

# Check domain crate size
tokei crates/riptide-domain/src/
# Expected: ~859 lines (100% populated)

# Run validation script
./scripts/validate_architecture.sh | grep "Issue #1"
# Expected: ✅ Issue #1: Types Purity - PASSED
```

#### Deliverable

✅ Clean build, all tests passing, riptide-types at 2,000 lines, Issue #1 resolved

---

### Task 1.7: Clean Pipeline Redis Dependency

**Status**: ⏳ **TODO** (Can be done anytime)
**Effort**: 5 minutes
**Assignee**: Any developer
**Dependencies**: None

#### Step-by-Step Plan

**Step 1: Open Cargo.toml** (1 minute)

```bash
vi crates/riptide-pipeline/Cargo.toml
```

**Step 2: Remove Line** (1 minute)

```toml
# DELETE THIS LINE:
redis = { workspace = true }
```

**Step 3: Verify** (3 minutes)

```bash
# Check no redis references
grep "redis" crates/riptide-pipeline/Cargo.toml
# Expected: 0 results

# Check no redis usage in code
grep -r "redis::" crates/riptide-pipeline/src/
# Expected: 0 results

# Build pipeline crate
cargo check -p riptide-pipeline
# Expected: Clean build

# Run validation
./scripts/validate_architecture.sh | grep "Issue #5"
# Expected: ✅ Issue #5: Pipeline Redis - PASSED
```

#### Deliverable

✅ Pipeline Cargo.toml clean, Issue #5 resolved (5 minutes total)

---

## Phase 1 Exit Criteria

### All Tasks Complete When:

- [x] Task 1.1: Domain crate structure created ✅
- [x] Task 1.2: Circuit breaker moved (372 lines) ✅
- [ ] Task 1.3: HTTP caching moved (180 lines)
- [ ] Task 1.4: Error handling moved (100+ lines)
- [ ] Task 1.5: Security/processing moved (40+ lines)
- [ ] Task 1.6: Validation passed
- [ ] Task 1.7: Pipeline Redis removed

### Success Metrics:

```bash
# Issue #1: Types Purity
./scripts/validate_architecture.sh | grep "Issue #1"
# Expected: ✅ Issue #1: Types Purity - PASSED

# LOC Targets
tokei crates/riptide-types/src/
# Expected: ~2,000 lines (from 3,250)

tokei crates/riptide-domain/src/
# Expected: ~859 lines (100% populated)

# Issue #5: Pipeline Redis
grep "redis" crates/riptide-pipeline/Cargo.toml
# Expected: 0 results
```

### Phase 1 Complete When:

✅ All 6 tasks done
✅ 859 lines migrated (100%)
✅ riptide-types reduced to 2,000 lines
✅ All tests passing
✅ Issues #1 and #5 resolved

---

## Phase 2-5 Overview

### Phase 2: Infrastructure Purity (Week 2, 12 hours)

**Tasks**:
1. Create riptide-cache-warming crate (2h)
2. Extract 1,172 lines cache warming code (4h)
3. Clean cache dependencies (1h)
4. Move env access to API (2h)
5. Create abstraction traits (3h)

**Issues Resolved**: #6 (cache domain deps), #7 (env access)

---

### Phase 3: Facade Detox (Week 3, 16 hours)

**Tasks**:
1. Create domain FetchMethod enum (1h)
2. Create typed domain models (4h)
3. Replace 42+ JSON blobs with typed models (4h)
4. Define 11 service traits in riptide-types (3h)
5. Update facade to trait-based dependencies (4h)
6. Wire implementations at AppState (2h)

**Issues Resolved**: #3 (HTTP leakage), #4 (facade dependencies)

---

### Phase 4: Handler Simplification (Week 4, 12 hours)

**Tasks**:
1. Create TableExtractionFacade (3h)
2. Create RenderFacade (5h)
3. Create ReportFacade (3h)
4. Handler cleanup & validation (1h)

**Issues Resolved**: #2 (handler complexity)

---

### Phase 5: Validation & Monitoring (Week 5, 8 hours)

**Tasks**:
1. Full validation suite (1h)
2. Update documentation (3h)
3. CI/CD integration (4h)
4. Performance benchmarking (2h)

**Deliverable**: ✅ All 7 issues resolved, automated governance in place

---

## Critical Path & Dependencies

```
Week 1: Foundation
├─ 1.1 Domain structure ✅ (no deps)
├─ 1.2 Circuit breaker ✅ (requires 1.1)
├─ 1.3 HTTP caching ⏳ (requires 1.1) ← NEXT
├─ 1.4 Error handling (requires 1.2)
├─ 1.5 Security/processing (requires 1.1)
├─ 1.6 Validation (requires 1.3-1.5)
└─ 1.7 Pipeline Redis (no deps, anytime)
     ↓
Week 2: Infrastructure
├─ 2.1-2.3 Cache warming (no deps)
├─ 2.4 Env access (requires Phase 1)
└─ 2.5 Abstraction traits (requires 2.4)
     ↓
Week 3: Facade
├─ 3.1-3.3 HTTP removal (requires Phase 1)
├─ 3.4 Service traits (requires Phase 1)
├─ 3.5 Facade refactor (requires 3.4)
└─ 3.6 Wire implementations (requires 3.5)
     ↓
Week 4: Handlers
├─ 4.1 TableExtractionFacade (requires Phase 3)
├─ 4.2 RenderFacade (requires Phase 3)
├─ 4.3 ReportFacade (requires Phase 3)
└─ 4.4 Validation (requires 4.1-4.3)
     ↓
Week 5: Validation
└─ 5.1-5.4 All tasks (requires Phase 4)
```

**Critical Path**: Phases 1 → 3 → 4 (16 + 16 + 12 = 44 hours minimum)
**Parallelizable**: Phase 2 can run alongside other work

---

## Progress Tracking

### Daily Checklist

**Day 1** (Week 1):
- [x] Task 1.1: Domain structure ✅
- [x] Task 1.2: Circuit breaker (start) ✅

**Day 2** (Week 1):
- [x] Task 1.2: Circuit breaker (complete) ✅

**Day 3** (Week 1):
- [ ] Task 1.3: HTTP caching ← **YOU ARE HERE**
- [ ] Task 1.7: Pipeline Redis (5 min quick win)

**Day 4** (Week 1):
- [ ] Task 1.4: Error handling

**Day 5** (Week 1):
- [ ] Task 1.5: Security/processing
- [ ] Task 1.6: Validation

**Week 1 Review**:
- [ ] All Phase 1 tasks complete
- [ ] 859 lines migrated
- [ ] Types crate at 2,000 lines
- [ ] Issues #1 and #5 resolved

---

## Risk Management

### High-Risk Tasks

1. **Task 1.3: HTTP caching migration** (3 hours)
   - Risk: Breaking HTTP request handling
   - Mitigation: Extensive testing of conditional requests
   - Rollback: Revert via git, re-exports still work

2. **Task 1.4: Error classification** (3 hours)
   - Risk: Changing error semantics
   - Mitigation: Trait abstraction maintains contract
   - Rollback: Restore impl blocks, remove trait

3. **Phase 3.5: Facade refactoring** (4 hours)
   - Risk: Widespread compilation errors
   - Mitigation: Incremental trait introduction
   - Rollback: Git revert, facades use concrete types

### Medium-Risk Tasks

- Task 1.5: Security/processing (isolated changes)
- Task 1.6: Validation (just testing)
- Phase 2: Infrastructure (clear boundaries)

### Low-Risk Tasks

- Task 1.7: Pipeline Redis (unused dependency)
- Task 1.1: Domain structure (empty scaffolding)
- Phase 5: Validation (no code changes)

---

## Success Criteria - Master Checklist

### Phase 1 Complete ✅ When:

```bash
# All tasks done
[x] 1.1 Domain structure
[x] 1.2 Circuit breaker
[ ] 1.3 HTTP caching
[ ] 1.4 Error handling
[ ] 1.5 Security/processing
[ ] 1.6 Validation
[ ] 1.7 Pipeline Redis

# Metrics hit
tokei crates/riptide-types/src/ | grep Total
# Expected: ~2,000 lines

tokei crates/riptide-domain/src/ | grep Total
# Expected: ~859 lines

# Validation passes
./scripts/validate_architecture.sh | grep "Issue #1\|Issue #5"
# Expected: Both PASSED

# Tests pass
cargo test --workspace
# Expected: All pass
```

### All Phases Complete ✅ When:

```bash
./scripts/validate_architecture.sh

# Expected output:
✅ ARCHITECTURE VALIDATION PASSED
Passed: 28
Warnings: 0
Failed: 0

✅ Issue #1: Types Purity - PASSED
✅ Issue #2: Handler Simplicity - PASSED
✅ Issue #3: Facade HTTP - PASSED
✅ Issue #4: Facade Dependencies - PASSED
✅ Issue #5: Pipeline Redis - PASSED
✅ Issue #6: Cache Domain Deps - PASSED
✅ Issue #7: Domain Env Reads - PASSED

🎉 CLEAN ARCHITECTURE MIGRATION COMPLETE! 🎉
```

---

## Next Actions

### Immediate (This Week)

1. ⏳ **Task 1.3: HTTP Caching Migration** (3 hours) ← **START HERE**
   - Create domain files: caching.rs, date_parsing.rs, conditional.rs
   - Move 180 lines from riptide-types
   - Update re-exports
   - Test thoroughly

2. ⏳ **Task 1.7: Pipeline Redis Cleanup** (5 minutes) ← **QUICK WIN**
   - Delete one line from Cargo.toml
   - Validate build passes

3. ⏳ **Tasks 1.4-1.6**: Continue Phase 1
   - Complete all remaining tasks
   - Target: End of Week 1

### Next Week (Week 2)

- Start Phase 2: Infrastructure Purity
- Parallel work: Cache warming + env access

### Weeks 3-5

- Phase 3: Facade Detox
- Phase 4: Handler Simplification
- Phase 5: Validation & Deployment

---

**Roadmap Status**: ✅ **READY FOR EXECUTION**
**Current Phase**: Phase 1.3 (HTTP Caching)
**Progress**: 33% (2/6 tasks complete, 372/859 lines migrated)
**Next Milestone**: Complete Phase 1 by end of Week 1

**🗺️ LET'S COMPLETE THE MIGRATION! 🗺️**
