# riptide-workers Fix Guide
**Priority**: P0 (CRITICAL BLOCKER)
**Date**: 2025-10-19
**For**: Coder Agent

## Executive Summary

The `riptide-workers` crate has 26 compilation errors preventing all workspace tests from running. All errors stem from unresolved `riptide_core` dependencies that need migration to the new architecture.

## Error Breakdown

### Import Errors (5 errors)

#### 1. CrawlOptions Import Path
**File**: `src/processors.rs:5`
```rust
// ❌ CURRENT (ERROR)
use riptide_types::{CrawlOptions, ExtractedDoc};

// ✅ FIX
use riptide_types::{config::CrawlOptions, ExtractedDoc};
```

#### 2-4. CacheManager Import
**File**: `src/service.rs:11`
```rust
// ❌ CURRENT (ERROR)
use riptide_core::cache::CacheManager;

// ✅ FIX (Option 1: Move to riptide-cache)
use riptide_cache::CacheManager;

// ✅ FIX (Option 2: Keep in riptide-types if already moved)
use riptide_types::cache::CacheManager;
```

**Files Affected**:
- `src/service.rs:11`
- `src/processors.rs:17, 29, 276, 283`

#### 5. ExtractorConfig Import
**File**: `src/service.rs:313`
```rust
// ❌ CURRENT (ERROR)
use riptide_types::component::ExtractorConfig;

// ✅ FIX - Check if ExtractorConfig exists in component module
// If not, it may need to be moved from riptide-core
use riptide_extraction::ExtractorConfig; // Or wherever it was migrated
```

### Type Resolution Errors (21 errors)

#### Category A: WasmExtractor (7 errors)
**Files**: `src/processors.rs` (lines 15, 28, 274, 282) and `src/service.rs` (line 316)

```rust
// ❌ CURRENT (ERROR)
extractor: Arc<dyn riptide_core::extract::WasmExtractor>

// ✅ FIX
use riptide_extraction::wasm::WasmExtractor; // Verify exact path
extractor: Arc<dyn riptide_extraction::wasm::WasmExtractor>
```

**Compiler suggestion**: The compiler hints to use `riptide_types::extract::WasmExtractor`, but this should be in `riptide-extraction` crate.

#### Category B: CrawlOptions in Function Signatures (2 errors)
**Files**: `src/job.rs` (lines 68, 73)

```rust
// ❌ CURRENT (ERROR)
options: Option<riptide_core::types::CrawlOptions>

// ✅ FIX
use riptide_types::config::CrawlOptions;
options: Option<CrawlOptions>
```

#### Category C: PDF Pipeline (12 errors)
**Files**: `src/processors.rs` (lines 501, 503, 512, 521, 530, 532, 544, 587, 593, 612, 620)

```rust
// ❌ CURRENT (ERROR)
use riptide_core::pdf::{PdfPipelineIntegration, PdfConfig};
use riptide_core::convert_pdf_extracted_doc;
use riptide_core::pdf::types::create_progress_channel;

// ✅ FIX
use riptide_pdf::{PdfPipelineIntegration, PdfConfig};
use riptide_pdf::convert_pdf_extracted_doc;
use riptide_pdf::types::create_progress_channel;
```

**Specific Locations**:
- Line 501: `pdf_pipeline: Arc<riptide_core::pdf::PdfPipelineIntegration>`
- Line 503: `default_config: riptide_core::pdf::PdfConfig`
- Line 512: `let default_config = riptide_core::pdf::PdfConfig {`
- Line 521, 532: `Arc::new(riptide_core::pdf::PdfPipelineIntegration::with_config(...)`
- Line 530: Function parameter `config: riptide_core::pdf::PdfConfig`
- Line 544: Return type `-> riptide_core::pdf::PdfConfig`
- Line 587: Parameter `_config: &riptide_core::pdf::PdfConfig`
- Line 593: `riptide_core::pdf::types::create_progress_channel()`
- Line 612, 620: `riptide_core::convert_pdf_extracted_doc(result)`

## Migration Plan

### Step 1: Update Cargo.toml Dependencies
**File**: `/workspaces/eventmesh/crates/riptide-workers/Cargo.toml`

Add missing dependencies:
```toml
[dependencies]
# Existing dependencies...
riptide-cache = { path = "../riptide-cache" }
riptide-extraction = { path = "../riptide-extraction" }
riptide-pdf = { path = "../riptide-pdf" }
```

### Step 2: Update Import Statements
**File**: `src/processors.rs`

```rust
// Add at top of file
use riptide_types::config::CrawlOptions;
use riptide_cache::CacheManager; // Or riptide_types::cache::CacheManager
use riptide_extraction::wasm::WasmExtractor; // Verify path
use riptide_pdf::{
    PdfPipelineIntegration,
    PdfConfig,
    convert_pdf_extracted_doc,
    types::create_progress_channel,
};
```

**File**: `src/service.rs`

```rust
// Add at top of file
use riptide_cache::CacheManager; // Or riptide_types::cache::CacheManager
use riptide_extraction::{
    wasm::WasmExtractor,
    ExtractorConfig, // Verify this module exists
};
```

**File**: `src/job.rs`

```rust
// Add at top of file
use riptide_types::config::CrawlOptions;
```

### Step 3: Search and Replace in Code

Run these replacements across affected files:

```bash
# In processors.rs, service.rs, job.rs
sed -i 's/riptide_core::cache::CacheManager/CacheManager/g' src/*.rs
sed -i 's/riptide_core::extract::WasmExtractor/WasmExtractor/g' src/*.rs
sed -i 's/riptide_core::types::CrawlOptions/CrawlOptions/g' src/*.rs
sed -i 's/riptide_core::pdf::PdfPipelineIntegration/PdfPipelineIntegration/g' src/*.rs
sed -i 's/riptide_core::pdf::PdfConfig/PdfConfig/g' src/*.rs
sed -i 's/riptide_core::pdf::types::create_progress_channel/create_progress_channel/g' src/*.rs
sed -i 's/riptide_core::convert_pdf_extracted_doc/convert_pdf_extracted_doc/g' src/*.rs
```

**⚠️ WARNING**: Review changes manually after sed operations to ensure correctness.

### Step 4: Verify Module Paths

Some items may need path verification:

1. **WasmExtractor**: Check if it's in `riptide_extraction::wasm` or `riptide_extraction::extract`
2. **ExtractorConfig**: Verify this exists and its module path
3. **CacheManager**: Confirm if it's in `riptide-cache` crate or `riptide-types::cache`

Run this to find the actual locations:
```bash
grep -rn "pub struct WasmExtractor" crates/riptide-extraction/
grep -rn "pub struct ExtractorConfig" crates/riptide-extraction/
grep -rn "pub struct CacheManager" crates/riptide-cache/ crates/riptide-types/
```

## Verification Steps

After applying fixes:

### 1. Compile Check
```bash
cargo check -p riptide-workers
```

### 2. Build Check
```bash
cargo build -p riptide-workers
```

### 3. Test Check
```bash
cargo test -p riptide-workers
```

### 4. Full Workspace Test
```bash
cargo test --workspace --no-fail-fast
```

## Expected Outcome

✅ All 26 errors resolved
✅ riptide-workers compiles successfully
✅ Workspace tests can run
✅ Baseline test suite execution proceeds

## Files to Modify

1. `/workspaces/eventmesh/crates/riptide-workers/Cargo.toml`
2. `/workspaces/eventmesh/crates/riptide-workers/src/processors.rs`
3. `/workspaces/eventmesh/crates/riptide-workers/src/service.rs`
4. `/workspaces/eventmesh/crates/riptide-workers/src/job.rs`

## Coordination

After fix completion:
```bash
npx claude-flow@alpha hooks notify --message "riptide-workers compilation errors fixed - ready for testing"
npx claude-flow@alpha hooks post-edit --file "crates/riptide-workers/src/*.rs" --memory-key "swarm/coder/workers-fix"
```

---

**Status**: AWAITING CODER AGENT ACTION
**Blocker Impact**: Prevents all 7 validation phases from executing
**Estimated Fix Time**: 30-60 minutes
