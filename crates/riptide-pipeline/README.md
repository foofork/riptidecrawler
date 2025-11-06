# RipTide Pipeline

**Purpose**: Break circular dependency between `riptide-api` and `riptide-facade` by extracting shared pipeline types.

## Problem Solved

**Before (Week 9):**
```
riptide-api ←→ riptide-facade (CIRCULAR!)
```

**After (Week 9 Fix):**
```
                riptide-pipeline (types)
                    ↗        ↖
riptide-facade           riptide-api (implementation)
```

## Architecture

This crate contains **only the public types** for pipeline orchestration:

### Core Types
- `PipelineResult` - Standard pipeline execution result
- `PipelineStats` - Batch processing statistics
- `PipelineRetryConfig` - Retry configuration
- `GateDecisionStats` - Gate decision breakdown

### Strategies Types
- `StrategiesPipelineResult` - Enhanced pipeline with extraction strategies

### Dual-Path Types (LLM Feature)
- `FastPathResult` - Fast CSS extraction path
- `EnhancementResult` - AI enhancement path
- `DualPathResult` - Merged result

## Usage

### For Consumers (riptide-facade)
```rust
use riptide_pipeline::{PipelineResult, PipelineStats};
use riptide_api::pipeline::PipelineOrchestrator;

// Import TYPES from riptide-pipeline
// Import IMPLEMENTATION from riptide-api
```

### For Implementers (riptide-api)
```rust
// Re-export types from riptide-pipeline
pub use riptide_pipeline::{
    PipelineResult, PipelineStats, PipelineRetryConfig
};

// Implementation stays in riptide-api
pub struct PipelineOrchestrator {
    // ... 1,071 lines of implementation
}
```

## Note on Circular Dependency

While riptide-api still depends on riptide-facade (for AppState facades), the **type-level** circular dependency is broken. The pipeline types are now in a separate crate that both can depend on.

**Remaining work** (for Week 10+): Remove facade fields from AppState to fully break the implementation-level circular dependency.

## Files Modified

| File | Change |
|------|--------|
| `crates/riptide-pipeline/` | **NEW** - Created with public types |
| `crates/riptide-api/src/pipeline.rs` | Re-exports types from riptide-pipeline |
| `crates/riptide-api/src/strategies_pipeline.rs` | Conversion to riptide-pipeline types |
| `crates/riptide-facade/src/facades/crawl_facade.rs` | Imports types from riptide-pipeline |

## Week 9 Deliverable

✅ **Week 9 Goal Achieved**: Pipeline orchestrator extracted to break circular dependency
- Pipeline types extracted to separate crate
- Type-level circular dependency eliminated
- Implementation preserved (no rewrite, only refactoring)
- All 1,596 lines of production code wrapped, not rebuilt
