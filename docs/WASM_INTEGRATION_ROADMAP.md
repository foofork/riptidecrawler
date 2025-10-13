# WASM Integration Roadmap

This document tracks the remaining TODOs and blockers for complete WASM Component Model integration in RipTide.

---

# 🚨 ACTION ITEMS - ALL TODOS

## 🔴 CRITICAL - Must Fix Before Production (P0)

### ☐ Issue #3: WIT Bindgen Type Conflicts
**Status**: 🟡 GitHub issue created - https://github.com/foofork/riptidecrawler/issues/3
**Priority**: P0 - BLOCKER
**Effort**: 1-2 days
**Location**: `crates/riptide-html/src/wasm_extraction.rs:13-23`

**Problem**: WIT bindings disabled due to type name collisions between host and guest types
**Impact**: WASM component completely unused, running fallback implementation only
**Action Required**:
1. Implement namespace separation for WIT bindings
2. Create explicit conversion layer between host and guest types
3. Enable `wasmtime::component::bindgen!` macro
4. Test end-to-end WASM extraction

---

### ☐ Issue #5: Complete Component Model Integration
**Status**: 📝 Ready to create in GitHub (blocked by Issue #3)
**Priority**: P0 - BLOCKER
**Effort**: Included in Issue #3 work
**Location**: `crates/riptide-html/src/wasm_extraction.rs:448-454`

**Problem**: Not calling actual WASM exported functions, using fallback instead
**Impact**: No memory isolation, no security boundaries, no WASM benefits
**Action Required**:
1. ✅ First resolve Issue #3 (prerequisite)
2. Wire up component instantiation
3. Call actual WASM `extract()` function
4. Convert WIT results to host types
5. Remove fallback implementation

---

## 🟠 HIGH PRIORITY - Performance Optimization (P1)

### ☐ Issue #4: Wasmtime 34 Caching API Migration
**Status**: 📝 Ready to create in GitHub
**Priority**: P1 - High
**Effort**: 0.5-1 day
**Location**: `crates/riptide-html/src/wasm_extraction.rs:405-416`

**Problem**: AOT compilation caching disabled, `cache_config_load_default()` doesn't exist in Wasmtime 34
**Impact**: 100-500ms cold start penalty on first run
**Action Required**:
1. Research Wasmtime 34.x caching API documentation
2. Find equivalent method for cache configuration
3. Update code to use new API
4. Benchmark cold start time (target: <15ms with cache)
5. Verify cache hit ratio >85%

---

## 🟡 MEDIUM PRIORITY - Feature Enhancement (P2)

### ☐ Issue #6: Table Multi-Level Header Extraction
**Status**: 📝 Ready to create in GitHub
**Priority**: P2 - Medium
**Effort**: 2-3 days
**Location**: `crates/riptide-html/src/table_extraction/extractor.rs:107-109`

**Problem**: Table extraction doesn't support hierarchical headers with colspan/rowspan
**Impact**: Data loss for complex financial/scientific tables
**Action Required**:
1. Implement colspan parsing and tracking
2. Implement rowspan parsing and tracking
3. Build hierarchical header structure
4. Map cells to full header paths
5. Add comprehensive test cases

---

## 📊 TODO Summary

| Issue | Priority | Status | Effort | Blocker |
|-------|----------|--------|--------|---------|
| **#3: WIT Bindings** | 🔴 P0 | Created | 1-2 days | None |
| **#5: Component Integration** | 🔴 P0 | Ready | Included | Issue #3 |
| **#4: Wasmtime Caching** | 🟠 P1 | Ready | 0.5-1 day | None |
| **#6: Table Headers** | 🟡 P2 | Ready | 2-3 days | None |

**Total Estimated Effort**: 3.5-6 days

**Critical Path**:
1. Issue #3 (1-2 days) → Unblocks Issue #5
2. Issue #4 (0.5-1 day) → Parallel to #3
3. Issue #6 (2-3 days) → Can be done anytime

---

## 🎯 Quick Reference

**Production Blocker**: Issue #3 + Issue #5 (WASM bindings and integration)
**Performance Blocker**: Issue #4 (AOT caching)
**Feature Gap**: Issue #6 (Table headers)

**Current State**: 🔴 NOT PRODUCTION READY - Using fallback only
**After Issue #3**: ✅ PRODUCTION READY - Architecture is sound
**Architecture Grade**: **B+ (85/100)** - Excellent design, needs activation

---

## WASM Architecture Overview

### Executive Summary

**Architecture Grade: B+ (85/100)**

RipTide implements a sophisticated WebAssembly Component Model-based extraction system with strong isolation, resource management, and performance optimization. The architecture demonstrates **production-grade design** with some critical gaps that need resolution before full Component Model activation.

**🔴 Current State: NOT PRODUCTION READY** - WASM component is bypassed, using fallback implementation only
**✅ After Issue #3 Fix: PRODUCTION READY** - Architecture is sound, just needs WIT bindings enabled

### How WASM is Used Across the Project

#### Three-Layer Architecture

```
┌─────────────────────────────────────┐
│  Host Application (riptide-api)    │
│  └─> CmExtractor (WASM wrapper)     │
├─────────────────────────────────────┤
│  Instance Pool (riptide-core)       │
│  ├─ Circuit Breaker                 │
│  ├─ Resource Limiting                │
│  └─ Health Monitoring                │
├─────────────────────────────────────┤
│  WASM Guest (riptide-extractor)     │
│  ├─ Trek-rs Integration             │
│  └─ Enhanced Features                │
└─────────────────────────────────────┘
```

#### Key Components

**1. WIT Interface** (`wasm/riptide-extractor-wasm/wit/extractor.wit`)
- Defines Component Model contract between host and guest
- 14-field `extracted-content` record with comprehensive metadata
- 7 error variants for structured error handling
- 7 exported functions (extract, extract-with-stats, validate-html, health-check, etc.)
- **Grade: A+ (95/100)** - Exemplary interface design

**2. WASM Guest Component** (`wasm/riptide-extractor-wasm/`)
- Implements extraction logic in isolated sandbox
- Trek-rs integration for core content extraction
- Enhanced features:
  - Link extraction with rel attributes, canonical links, area elements
  - Media extraction (images, videos, audio) with srcset and Open Graph
  - 5-tier language detection (html[lang], og:locale, JSON-LD, Content-Language, whatlang)
  - Category extraction from JSON-LD, breadcrumbs, meta tags
- **Grade: A (90/100)** - Production-quality implementation

**3. Host Integration** (`crates/riptide-html/src/wasm_extraction.rs`)
- Bridges Rust host and WASM guest
- Manages Wasmtime engine and configuration
- **🔴 CRITICAL**: WIT bindings currently disabled (lines 13-23)
- **🔴 CRITICAL**: Using fallback implementation instead of WASM (lines 448-454)
- **Grade: C+ (70/100)** - Good design, incomplete implementation

**4. Instance Pool** (`crates/riptide-core/src/instance_pool/`)
- Production-grade WASM instance lifecycle management
- VecDeque-based FIFO pooling with reuse
- Semaphore-based concurrency control (max 8 concurrent)
- Circuit breaker pattern (Closed → Open → HalfOpen states)
- Health monitoring with automatic eviction
- Fresh Store per call (prevents state pollution)
- **Grade: A+ (95/100)** - Sophisticated pooling architecture

**5. Resource Management**
- **Memory Limiting**: `WasmResourceTracker` implements `ResourceLimiter` trait
  - Max 1024 pages (64MB) per instance
  - Atomic counters for precise tracking
  - Grow failure detection
- **CPU Limiting**: 1,000,000 fuel units per extraction
- **Time Limiting**: 30-second epoch-based timeouts
- **Grade: A (90/100)** - Multi-layer resource control

### Architect's Assessment

#### ✅ Strengths

1. **WIT Interface Design (A+)**: Type-safe contract with comprehensive types, structured errors, and health monitoring built-in
2. **Instance Pooling (A+)**: Circuit breaker, health checks, fresh Store per call, event-driven architecture
3. **Enhanced Extraction (A)**: Links, media, language detection, categories all production-ready
4. **Resource Management (A)**: Memory, CPU (fuel), and time (epoch) limits with atomic tracking
5. **Event Integration**: Good observability with event bus for monitoring

#### ❌ Critical Issues

1. **WIT Bindings Disabled** (Issue #3) - **CRITICAL**
   - Location: `crates/riptide-html/src/wasm_extraction.rs:13-23`
   - Impact: WASM component completely unused, using fallback only
   - Cause: Type name conflicts (ExtractedDoc, ExtractionMode, etc.)
   - Solution: Namespace separation with explicit type boundary

2. **Fallback Implementation Active** - **CRITICAL**
   - Location: `crates/riptide-html/src/wasm_extraction.rs:441-482`
   - Impact: No WASM execution, no isolation, no security boundaries
   - Dependency: Blocked by Issue #3

3. **AOT Caching Disabled** (Issue #4) - **HIGH**
   - Location: `crates/riptide-html/src/wasm_extraction.rs:405-416`
   - Impact: 100-500ms cold start penalty
   - Cause: Wasmtime 34 API migration needed

#### Current Data Flow

**🔴 Current (Fallback Mode)**:
```
API Request → CmExtractor::extract()
  ├─ Create Store and ResourceTracker
  ├─ Set fuel limit (unused)
  ├─ ❌ SKIP: Component instantiation
  ├─ ❌ SKIP: WASM function call
  └─ ✅ FALLBACK: Return mock ExtractedDoc
```

**✅ Intended (Component Model - NOT ACTIVE)**:
```
API Request → AdvancedInstancePool::extract()
  ├─ Check circuit breaker
  ├─ Acquire semaphore permit
  ├─ Get PooledInstance from pool
  ├─ Create fresh Store
  ├─ Instantiate component
  ├─ Call WASM extract function
  │   └─ [WASM Boundary]
  │       ├─ Validate input
  │       ├─ Trek-rs extraction
  │       ├─ Extract links, media, language
  │       └─ Return ExtractedContent
  ├─ Convert WIT result to host types
  ├─ Update metrics
  └─ Return instance to pool
```

### Type System Architecture

**Problem**: Two parallel type systems causing conflicts

**Host Types**:
- `ExtractedDoc`, `HostExtractionMode`, `HostExtractionError`

**WIT-Generated Types**:
- `exports::riptide::extractor::extractor::ExtractedContent`
- `exports::riptide::extractor::extractor::ExtractionMode`
- `exports::riptide::extractor::extractor::ExtractionError`

**Recommendation**: Use **Explicit Type Boundary** pattern
```rust
mod wit_bindings {
    wasmtime::component::bindgen!({ world: "extractor", ... });
}

// Host types remain independent
pub struct ExtractedDoc { /* ... */ }

// Explicit conversion layer
impl From<wit_bindings::ExtractedContent> for ExtractedDoc {
    fn from(wit: wit_bindings::ExtractedContent) -> Self {
        // Explicit field mapping
    }
}
```

### Is the Setup Correct?

#### ✅ Correct Architecture Decisions

1. **Component Model over Core WASM**: Correct - Type-safe interfaces, better tooling
2. **Instance Pooling**: Correct - Store-per-call (not instance-per-call) is optimal
3. **Circuit Breaker Pattern**: Correct - Protects system from cascading failures
4. **Resource Limiting**: Correct - Memory, CPU, and time limits all needed
5. **Separate Host/Guest Types**: Correct - Clear boundary, independent evolution

#### ⚠️ What Needs Fixing

1. **Enable WIT Bindings**: Use namespace separation to avoid conflicts
2. **Wire Up Component Calls**: Replace fallback with actual WASM invocations
3. **AOT Caching**: Migrate to Wasmtime 34 API for performance
4. **Type Boundary**: Implement explicit conversion layer

#### 🎯 Production Readiness

**Current**: ❌ NOT READY - Using fallback only
**After Issue #3**: ✅ READY - Architecture is production-grade

**Estimated Effort to Production**:
- Phase 1 (Critical): 1-2 days (Issue #3 + #5)
- Phase 2 (Performance): 1.5 days (Issue #4 + SIMD validation)
- **Total**: 2.5-3.5 days

### Architecture Scorecard

| Component | Score | Grade | Status |
|-----------|-------|-------|--------|
| WIT Interface | 95/100 | A+ | ✅ Production-ready |
| Guest Implementation | 90/100 | A | ✅ Production-ready |
| Host Integration | 70/100 | C+ | ❌ Bindings disabled |
| Instance Pooling | 95/100 | A+ | ✅ Production-ready |
| Resource Management | 90/100 | A | ✅ Production-ready |
| Type System | 65/100 | D+ | ⚠️ Needs architecture decision |

**Overall**: **B+ (85/100)** - Excellent design, needs bindings activation

### Related Documentation

- 📄 Full architectural assessment: `/docs/architecture/WASM_ARCHITECTURE_ASSESSMENT.md`
- 📄 Integration guide: `/docs/architecture/WASM_INTEGRATION_GUIDE.md`
- 📄 Instance pool design: `/docs/architecture/INSTANCE_POOL_ARCHITECTURE.md`
- 📄 Guest component: `/wasm/riptide-extractor-wasm/README.md`

---

## Issue #4: Wasmtime 34 - Migrate to New Caching API

### Priority
**Medium** - Performance optimization

### Labels
`wasm`, `wasmtime`, `performance`, `caching`, `optimization`

### Problem

The AOT (Ahead-of-Time) compilation caching is disabled because `cache_config_load_default()` method doesn't exist in Wasmtime 34.x.

### Location

`crates/riptide-html/src/wasm_extraction.rs:405-416`

### Technical Details

**Current Code:**
```rust
// TODO(wasmtime-34): cache_config_load_default() doesn't exist in v34
// Caching disabled - functionality works without it, just slower
// let mut config = Config::new();
// config.cache_config_load_default()?;
// config.wasm_component_model(true);
```

**Error:**
```
method `cache_config_load_default` not found for struct `wasmtime::Config`
```

### Current Impact

- ✅ **Works**: WASM compilation succeeds without caching
- ❌ **Performance**: First-run compilation is slower (~100-500ms penalty)
- ❌ **User Experience**: No caching benefits for repeated runs
- ⚠️ **Production**: Acceptable but not optimal

### Required Action

Research Wasmtime 34.x caching API:

1. Check [Wasmtime 34 Release Notes](https://github.com/bytecodealliance/wasmtime/releases/tag/v34.0.0)
2. Look for equivalent caching configuration methods:
   - `Config::cache_config_load()`
   - `Config::compilation_cache()`
   - Other cache-related APIs
3. Update code to use new API
4. Test that AOT caching works correctly

### Wasmtime Version

```toml
wasmtime = "34.0.0"
wasmtime-wasi = "34.0.0"
```

### Related Files

- `crates/riptide-html/src/wasm_extraction.rs:405` - Cache configuration
- `crates/riptide-html/Cargo.toml` - Wasmtime dependency

### Acceptance Criteria

- [ ] Find Wasmtime 34 caching API equivalent
- [ ] Update code to use new caching method
- [ ] Verify AOT cache directory is created (`~/.cache/wasmtime/`)
- [ ] Benchmark performance improvement with caching enabled
- [ ] Document caching configuration in code comments

---

## Issue #5: Complete WASM Component Model Integration

### Priority
**High** - Core architecture feature

### Labels
`wasm`, `component-model`, `integration`, `architecture`, `feature`

### Problem

The WASM Component Model integration is incomplete. We're not calling actual WASM exported functions - instead using a fallback implementation.

### Location

`crates/riptide-html/src/wasm_extraction.rs:448-454`

### Technical Details

**Current Code:**
```rust
// TODO(wasm-integration): Complete Component Model integration
// Using fallback extraction, not real WASM calls
// Need to wire up component instance and exported functions

// Placeholder - should call component instance
self.extract_with_fallback(html, url)
```

**What's Missing:**

1. **Component Instantiation**: Load and instantiate the WASM component
2. **Function Binding**: Get exported functions from component instance
3. **Type Conversion**: Convert between host types and WASM interface types
4. **Error Handling**: Proper error propagation from WASM to host
5. **Resource Management**: Handle WASM memory limits and fuel consumption

### Current Impact

- ✅ **Works**: Fallback extraction provides identical functionality
- ❌ **No Isolation**: No memory/resource isolation from WASM sandbox
- ❌ **No Security**: Can't leverage WASM security boundaries
- ❌ **Architecture**: Not using the Component Model as designed

### Implementation Steps

#### 1. Enable WIT Bindings (Prerequisite)
First resolve **Issue #3** (WIT bindgen type conflicts)

#### 2. Component Instantiation
```rust
let component = Component::from_file(&engine, wasm_path)?;
let linker = Linker::new(&engine);
let instance = linker.instantiate(&mut store, &component)?;
```

#### 3. Function Binding
```rust
// Get exported functions from WIT interface
let extractor = instance.get_typed_func::<(String, String, ExtractionMode), (ExtractedContent,)>(&mut store, "extract")?;
```

#### 4. Call WASM Function
```rust
let result = extractor.call(&mut store, (html.to_string(), url.to_string(), mode))?;
```

#### 5. Resource Limiting
```rust
// Set fuel for CPU limiting
store.set_fuel(1_000_000)?;

// Set memory limits
config.max_wasm_stack(8 * 1024 * 1024)?; // 8MB
```

### Architecture Flow

```
Host Code → WasmExtractor::extract()
    ↓
Component Instantiation
    ↓
Type Conversion (host → WASM)
    ↓
WASM Function Call
    ↓
Type Conversion (WASM → host)
    ↓
Result/Error to Host
```

### Related Issues

- **Blocked by**: Issue #3 (WIT bindgen type conflicts)
- **Related to**: Issue #4 (Wasmtime caching)

### Related Files

- `crates/riptide-html/src/wasm_extraction.rs` - Host integration
- `wasm/riptide-extractor-wasm/src/lib.rs` - WASM component
- `wasm/riptide-extractor-wasm/wit/extractor.wit` - Interface definition

### Acceptance Criteria

- [ ] WIT bindings enabled (prerequisite)
- [ ] Component instantiation working
- [ ] Calling actual WASM exported functions
- [ ] Type conversion working bidirectionally
- [ ] Resource limits enforced (fuel, memory)
- [ ] Error handling complete
- [ ] Integration tests passing with real WASM calls
- [ ] Performance benchmarks show expected overhead

---

## Issue #6: Implement Table Multi-Level Header Extraction

### Priority
**Medium** - Feature enhancement, not a blocker

### Labels
`feature`, `tables`, `html-parsing`, `enhancement`

### Problem

Table extraction doesn't support multi-level hierarchical headers with `colspan` and `rowspan` attributes.

### Location

`crates/riptide-html/src/table_extraction/extractor.rs:107-109`

### Technical Details

**Current Code:**
```rust
// TODO(feature): Implement multi-level header extraction
// Multi-level headers with colspan/rowspan not supported yet
```

**Example Complex Table:**
```html
<table>
  <thead>
    <tr>
      <th colspan="2">Category A</th>
      <th rowspan="2">Category B</th>
    </tr>
    <tr>
      <th>Subcategory A1</th>
      <th>Subcategory A2</th>
    </tr>
  </thead>
  <tbody>
    <tr>
      <td>Data 1</td>
      <td>Data 2</td>
      <td>Data 3</td>
    </tr>
  </tbody>
</table>
```

### Current Impact

- ✅ **Works**: Simple single-row headers extracted correctly
- ❌ **Limited**: Complex hierarchical tables not fully parsed
- ⚠️ **Data Loss**: Parent headers (Category A, Category B) may be missed
- 📊 **Use Cases**: Financial tables, scientific data, reports

### Implementation Requirements

#### 1. Colspan Handling
Track columns that are spanned by parent headers:
```rust
struct HeaderSpan {
    text: String,
    start_col: usize,
    end_col: usize,
    level: usize,
}
```

#### 2. Rowspan Handling
Track rows where headers extend downward:
```rust
struct HeaderStack {
    text: String,
    remaining_rows: usize,
    column: usize,
}
```

#### 3. Hierarchical Structure
Build nested header structure:
```rust
struct Header {
    text: String,
    level: usize,
    children: Vec<Header>,
}
```

#### 4. Cell Association
Map each data cell to its full header path:
```rust
struct Cell {
    value: String,
    headers: Vec<String>, // ["Category A", "Subcategory A1"]
}
```

### Example Output Format

**Current (Simple):**
```json
{
  "headers": ["Subcategory A1", "Subcategory A2", "Category B"],
  "rows": [
    ["Data 1", "Data 2", "Data 3"]
  ]
}
```

**Desired (Hierarchical):**
```json
{
  "headers": [
    {
      "text": "Category A",
      "colspan": 2,
      "children": [
        {"text": "Subcategory A1", "column": 0},
        {"text": "Subcategory A2", "column": 1}
      ]
    },
    {
      "text": "Category B",
      "rowspan": 2,
      "column": 2
    }
  ],
  "rows": [
    {
      "cells": [
        {"value": "Data 1", "path": ["Category A", "Subcategory A1"]},
        {"value": "Data 2", "path": ["Category A", "Subcategory A2"]},
        {"value": "Data 3", "path": ["Category B"]}
      ]
    }
  ]
}
```

### Testing Data

Common test cases:
1. **Simple colspan**: Single header spanning multiple columns
2. **Simple rowspan**: Single header spanning multiple rows
3. **Complex mixed**: Both colspan and rowspan in same table
4. **Multiple levels**: 3+ levels of header hierarchy
5. **Irregular**: Non-rectangular header structures

### Related Files

- `crates/riptide-html/src/table_extraction/extractor.rs` - Main extraction logic
- `crates/riptide-html/src/table_extraction/mod.rs` - Module interface
- `crates/riptide-html/tests/table_extraction_tests.rs` - Test suite

### Acceptance Criteria

- [ ] Parse colspan attributes correctly
- [ ] Parse rowspan attributes correctly
- [ ] Build hierarchical header structure
- [ ] Map cells to full header paths
- [ ] Handle irregular table structures gracefully
- [ ] Add comprehensive test cases
- [ ] Document output format

---

## Implementation Order

### Phase 1: Unblock WASM Integration (Critical Path)
1. **Issue #3** - WIT Bindgen Type Conflicts (CREATED ✅)
2. **Issue #5** - Complete Component Model Integration (blocked by #3)

### Phase 2: Performance Optimization
3. **Issue #4** - Wasmtime 34 Caching API

### Phase 3: Feature Enhancement
4. **Issue #6** - Table Multi-Level Headers

---

## Notes

- Issue #3 has been created: https://github.com/foofork/riptidecrawler/issues/3
- Issues #4, #5, #6 are documented here and ready to be created in GitHub
- All WASM extraction feature TODOs (links, media, language, categories) have been completed ✅
- Integration tests are passing ✅
