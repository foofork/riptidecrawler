# Pipeline Extraction Architecture Plan

## Executive Summary

**Objective**: Extract 2,126 lines of pipeline orchestration code from `riptide-api` into a new `riptide-pipeline` crate to break the circular dependency between `riptide-api` and `riptide-facade`.

**Current Problem**:
- Circular dependency: `riptide-api` ↔ `riptide-facade`
- `riptide-api` depends on `riptide-facade` for facades (BrowserFacade, ScraperFacade, etc.)
- `riptide-facade` depends on `riptide-api` for pipeline orchestrators (PipelineOrchestrator, StrategiesPipelineOrchestrator)

## Current Dependency Analysis

### riptide-api Dependencies (from Cargo.toml)
**Internal Crates**:
- riptide-types (core types)
- riptide-fetch (HTTP/network layer)
- riptide-extraction (content extraction)
- riptide-cache (caching layer)
- riptide-events (event system)
- riptide-config (configuration)
- riptide-reliability (circuit breakers, retries)
- riptide-monitoring (telemetry)
- riptide-performance (metrics)
- riptide-persistence (storage)
- **riptide-facade** ← CIRCULAR DEPENDENCY

**Optional Dependencies**:
- riptide-spider (crawler)
- riptide-browser (browser automation)
- riptide-headless (headless browser)
- riptide-intelligence (AI/LLM features, feature-gated with "llm")
- riptide-workers (background workers)
- riptide-search (search functionality)

### riptide-facade Dependencies (from Cargo.toml)
**Internal Crates**:
- riptide-types
- **riptide-api** ← CIRCULAR DEPENDENCY
- riptide-fetch
- riptide-extraction
- riptide-pdf
- riptide-cache
- riptide-browser
- riptide-stealth
- riptide-spider
- riptide-search
- riptide-utils
- riptide-monitoring (optional)

### Current Circular Dependency Flow

```
riptide-api (state.rs)
    ↓ imports
riptide-facade (BrowserFacade, ScraperFacade, ExtractionFacade, SearchFacade, SpiderFacade)
    ↓ imports
riptide-api (pipeline::PipelineOrchestrator, strategies_pipeline::StrategiesPipelineOrchestrator)
    ↑ CIRCULAR!
```

## Modules to Extract (2,126 lines total)

### 1. pipeline.rs (1,117 lines)
**Location**: `/workspaces/eventmesh/crates/riptide-api/src/pipeline.rs`

**Key Components**:
- `PipelineResult` struct (document, cache metadata, gate decision)
- `PipelineStats` struct (statistics and monitoring)
- `PipelineOrchestrator` struct (main orchestrator)
- `execute_single()` - single URL processing
- `execute_batch()` - batch URL processing
- Content fetching logic
- Gate analysis integration (`riptide_reliability::gate`)
- Cache integration
- WASM extraction integration

**Dependencies Used**:
```rust
use crate::errors::{ApiError, ApiResult};
use crate::state::AppState;
use reqwest::Response;
use riptide_events::{BaseEvent, EventSeverity};
use riptide_fetch as fetch;
use riptide_intelligence::smart_retry::{RetryConfig, SmartRetry, SmartRetryStrategy}; // feature-gated
use riptide_pdf::{self as pdf, utils as pdf_utils};
use riptide_reliability::gate::{decide, score, Decision, GateFeatures};
use riptide_types::config::CrawlOptions;
use riptide_types::{ExtractedDoc, RenderMode};
```

### 2. strategies_pipeline.rs (count TBD, estimated ~400 lines)
**Location**: `/workspaces/eventmesh/crates/riptide-api/src/strategies_pipeline.rs`

**Key Components**:
- `StrategiesPipelineResult` struct (enhanced result with strategy metadata)
- `StrategiesPipelineOrchestrator` struct (enhanced orchestrator)
- Multiple extraction strategies support (trek, css_json, regex, llm)
- Configurable chunking modes (regex, sentence, topic, fixed, sliding)
- Performance tracking and metrics
- Strategy auto-detection

**Dependencies Used**:
```rust
use crate::errors::{ApiError, ApiResult};
use crate::state::AppState;
use reqwest::Response;
use riptide_extraction::strategies::{
    ExtractionStrategyType, PerformanceMetrics, ProcessedContent, StrategyConfig, StrategyManager,
};
use riptide_fetch as fetch;
use riptide_pdf::{self as pdf, utils as pdf_utils};
use riptide_reliability::gate::{decide, score, Decision, GateFeatures};
use riptide_types::config::CrawlOptions;
use riptide_types::RenderMode;
```

### 3. pipeline_enhanced.rs (580 lines)
**Location**: `/workspaces/eventmesh/crates/riptide-api/src/pipeline_enhanced.rs`

**Key Components**:
- `EnhancedPipelineOrchestrator` struct (wraps standard pipeline)
- `EnhancedPipelineResult` struct (with phase timing)
- `EnhancedBatchStats` struct (batch statistics)
- Phase timing (fetch, gate, wasm, render)
- Enhanced metrics collection
- Debug logging for each phase
- Configuration via environment variables

**Dependencies Used**:
```rust
use crate::errors::ApiResult;
use crate::metrics::{PhaseTimer, PhaseType, RipTideMetrics};
use crate::pipeline::{PipelineOrchestrator, PipelineResult, PipelineStats};
use crate::state::{AppState, EnhancedPipelineConfig};
use riptide_types::config::CrawlOptions;
use riptide_types::ExtractedDoc;
```

**Note**: This wraps the standard `PipelineOrchestrator` and adds metrics/timing.

### 4. pipeline_dual.rs (429 lines) - CURRENTLY DISABLED
**Location**: `/workspaces/eventmesh/crates/riptide-api/src/pipeline_dual.rs`

**Status**: Currently commented out in `lib.rs` due to missing AI processor features
**Feature Gate**: Requires `"llm"` feature

**Key Components**:
- `DualPathOrchestrator` struct (fast path + AI enhancement)
- `FastPathResult` struct (CSS extraction result)
- `EnhancementResult` struct (AI enhancement result)
- `DualPathResult` struct (merged result)
- `DualPathConfig` struct (configuration)
- Zero-Impact AI Roadmap Phase 1 implementation

**Dependencies Used**:
```rust
use crate::errors::{ApiError, ApiResult};
use crate::state::AppState;
use riptide_events::{CrawlEvent, CrawlOperation, EventBus, EventEmitter, ExtractionMode};
use riptide_types::{CrawlOptions, ExtractedDoc};
use riptide_intelligence::{AiProcessorConfig, AiTask, BackgroundAiProcessor, TaskPriority};
```

### 5. Related Handler Files (NOT moving, but need updates)
These files use the pipeline but will stay in riptide-api:
- `/workspaces/eventmesh/crates/riptide-api/src/handlers/pipeline_metrics.rs`
- `/workspaces/eventmesh/crates/riptide-api/src/handlers/pipeline_phases.rs`
- `/workspaces/eventmesh/crates/riptide-api/src/streaming/pipeline.rs`

## New Dependency Graph Design

### Target Architecture
```
riptide-types (foundation)
    ↓
riptide-fetch, riptide-extraction, riptide-reliability, etc. (specialized crates)
    ↓
riptide-pipeline (NEW - orchestration layer)
    ↓
├── riptide-api (HTTP API handlers)
└── riptide-facade (high-level facades)
```

### riptide-pipeline Dependencies
The new crate will depend on:
- **riptide-types** (core types, ExtractedDoc, CrawlOptions)
- **riptide-fetch** (HTTP fetching)
- **riptide-extraction** (content extraction, strategies)
- **riptide-reliability** (gate logic, circuit breakers, retries)
- **riptide-cache** (caching layer)
- **riptide-events** (event system)
- **riptide-pdf** (PDF processing)
- **riptide-intelligence** (optional, feature-gated "llm")
- **External crates**: reqwest, tokio, serde, tracing, etc.

### riptide-api After Extraction
Will depend on:
- **riptide-pipeline** (NEW - for orchestrators)
- **riptide-facade** (for facades)
- All other existing dependencies

### riptide-facade After Extraction
Will depend on:
- **riptide-pipeline** (NEW - for orchestrators)
- NO dependency on riptide-api
- All other existing dependencies

## Breaking the Circular Dependency

### Current State
```
riptide-api ←→ riptide-facade (CIRCULAR)
```

### After Extraction
```
riptide-pipeline
    ↓
├── riptide-api
└── riptide-facade

No circular dependency!
```

## Implementation Strategy

### Phase 1: Create riptide-pipeline Crate Structure
1. Create `/workspaces/eventmesh/crates/riptide-pipeline/` directory
2. Create `Cargo.toml` with correct dependencies
3. Create `src/lib.rs` with module declarations
4. Add to workspace members in root `Cargo.toml`

### Phase 2: Move Pipeline Modules
1. Move `pipeline.rs` → `riptide-pipeline/src/pipeline.rs`
2. Move `strategies_pipeline.rs` → `riptide-pipeline/src/strategies_pipeline.rs`
3. Move `pipeline_enhanced.rs` → `riptide-pipeline/src/pipeline_enhanced.rs`
4. Move `pipeline_dual.rs` → `riptide-pipeline/src/pipeline_dual.rs` (keep feature-gated)

### Phase 3: Update Internal Imports
Within moved files, change:
```rust
// OLD
use crate::errors::{ApiError, ApiResult};
use crate::state::AppState;
use crate::metrics::RipTideMetrics;

// NEW
use riptide_api::errors::{ApiError, ApiResult};
use riptide_api::state::AppState;
use riptide_api::metrics::RipTideMetrics;
```

### Phase 4: Update riptide-api
1. Remove pipeline modules from `src/lib.rs`
2. Add `riptide-pipeline` dependency to `Cargo.toml`
3. Update imports in handlers and streaming modules:
```rust
// OLD
use crate::pipeline::PipelineOrchestrator;

// NEW
use riptide_pipeline::pipeline::PipelineOrchestrator;
```

### Phase 5: Update riptide-facade
1. Remove `riptide-api` dependency from `Cargo.toml`
2. Add `riptide-pipeline` dependency to `Cargo.toml`
3. Update imports in `facades/crawl_facade.rs`:
```rust
// OLD
use riptide_api::pipeline::{PipelineOrchestrator, PipelineResult};
use riptide_api::strategies_pipeline::{StrategiesPipelineOrchestrator, StrategiesPipelineResult};

// NEW
use riptide_pipeline::pipeline::{PipelineOrchestrator, PipelineResult};
use riptide_pipeline::strategies_pipeline::{StrategiesPipelineOrchestrator, StrategiesPipelineResult};
```

4. Handle `AppState` dependency (see Cross-Cutting Concerns below)

### Phase 6: Update riptide-py
Since riptide-py depends on both riptide-api and riptide-facade, it needs:
1. Add `riptide-pipeline` dependency to `Cargo.toml`
2. Update any direct pipeline imports

## Cross-Cutting Concerns

### AppState Dependency Challenge
**Problem**: Pipeline orchestrators currently take `AppState` from riptide-api as a parameter. This creates a tight coupling.

**Current Usage**:
```rust
pub struct PipelineOrchestrator {
    state: AppState,  // From riptide-api
    options: CrawlOptions,
}
```

**Solution Options**:

#### Option A: Keep AppState in riptide-api (RECOMMENDED)
- Pipeline constructors take `AppState` from riptide-api
- Requires `riptide-pipeline` to depend on `riptide-api` for AppState only
- **Dependency graph**: riptide-types → riptide-api → riptide-pipeline → (back to riptide-api for handlers)
- **Issue**: This might still create a cycle if not careful

#### Option B: Extract AppState to riptide-types or new riptide-state crate
- Create `riptide-state` crate with AppState, AppConfig, etc.
- Both riptide-api and riptide-pipeline depend on riptide-state
- **Pros**: Clean separation, no circular dependency
- **Cons**: Larger refactoring scope

#### Option C: Use Dependency Injection Pattern
- Pipeline takes trait objects or configuration structs instead of AppState
- Extract only needed components from AppState
- **Pros**: Loosely coupled, testable
- **Cons**: Significant refactoring needed

**RECOMMENDED**: Start with Option A (keep AppState in riptide-api) and evaluate if circular dependency occurs. If it does, move to Option B.

### Error Types (ApiError, ApiResult)
**Current**: Defined in `riptide-api::errors`
**Issue**: Pipeline uses ApiError extensively

**Solution**:
1. Keep ApiError in riptide-api
2. Pipeline depends on riptide-api for error types
3. Alternative: Move common errors to riptide-types

### Metrics (RipTideMetrics)
**Current**: Defined in `riptide-api::metrics`
**Issue**: `pipeline_enhanced.rs` uses RipTideMetrics

**Solution**:
1. Keep RipTideMetrics in riptide-api
2. Pipeline depends on riptide-api for metrics
3. Alternative: Extract metrics to riptide-monitoring

### Feature Gates
Must preserve feature gates:
- `#[cfg(feature = "llm")]` for pipeline_dual.rs and AI-related code
- `#[cfg(feature = "extraction")]` for extraction features
- Default features must remain compatible

## Import Changes Required

### Files in riptide-api Needing Updates (15+ files)
1. `src/handlers/crawl.rs` - imports PipelineOrchestrator, EnhancedPipelineOrchestrator
2. `src/handlers/strategies.rs` - imports StrategiesPipelineOrchestrator
3. `src/handlers/deepsearch.rs` - imports PipelineOrchestrator
4. `src/handlers/mod.rs` - re-exports pipeline_phases
5. `src/streaming/pipeline.rs` - imports PipelineOrchestrator
6. `src/streaming/processor.rs` - imports PipelineOrchestrator, PipelineResult
7. `src/streaming/websocket.rs` - imports PipelineOrchestrator
8. `src/streaming/sse.rs` - imports PipelineOrchestrator
9. `src/streaming/ndjson/helpers.rs` - imports PipelineOrchestrator
10. `src/lib.rs` - remove pipeline module declarations

### Files in riptide-facade Needing Updates (1 file)
1. `src/facades/crawl_facade.rs` - imports PipelineOrchestrator, PipelineResult, StrategiesPipelineOrchestrator, AppState

## New Cargo.toml for riptide-pipeline

```toml
[package]
name = "riptide-pipeline"
version = "0.9.0"
edition.workspace = true
license.workspace = true
authors.workspace = true

[lib]
name = "riptide_pipeline"
path = "src/lib.rs"

[dependencies]
# Internal dependencies - Foundation
riptide-types = { path = "../riptide-types" }
riptide-fetch = { path = "../riptide-fetch" }
riptide-extraction = { path = "../riptide-extraction", default-features = false, features = ["css-extraction", "regex-extraction", "dom-utils", "chunking", "native-parser"] }
riptide-cache = { path = "../riptide-cache" }
riptide-events = { path = "../riptide-events" }
riptide-reliability = { path = "../riptide-reliability", features = ["reliability-patterns"] }
riptide-pdf = { path = "../riptide-pdf", features = ["pdf"] }

# Internal dependencies - API layer (for AppState, errors, metrics)
riptide-api = { path = "../riptide-api", default-features = false }

# Optional internal dependencies
riptide-intelligence = { path = "../riptide-intelligence", optional = true }

# External dependencies
anyhow = { workspace = true }
async-trait = { workspace = true }
reqwest = { workspace = true }
serde = { workspace = true }
serde_json = { workspace = true }
thiserror = { workspace = true }
tokio = { workspace = true }
tracing = { workspace = true }
url = { workspace = true }
uuid = { workspace = true }

[dev-dependencies]
tokio-test = "0.4"
mockall = "0.13"
wiremock = { workspace = true }

[features]
default = ["native-parser"]

# Extraction strategy features
native-parser = ["riptide-extraction/native-parser"]
wasm-extractor = ["riptide-extraction/wasm-extractor"]

# AI enhancement features
llm = ["dep:riptide-intelligence"]
```

### Note on riptide-api Dependency
The pipeline crate will depend on riptide-api for:
1. `AppState` struct (main application state)
2. `ApiError` and `ApiResult` types (error handling)
3. `RipTideMetrics` (for enhanced pipeline)

We use `default-features = false` to avoid pulling in unnecessary riptide-api features and potentially creating deep dependency chains.

## Testing Strategy

### Unit Tests
Each moved module should maintain its existing tests:
- `pipeline.rs` - test extraction and caching
- `strategies_pipeline.rs` - test strategy selection
- `pipeline_enhanced.rs` - test metrics and timing

### Integration Tests
Create new integration tests in `riptide-pipeline/tests/`:
1. Test basic pipeline execution
2. Test cache integration
3. Test error handling
4. Test feature gates (llm)

### Validation Tests
After extraction, run:
1. `cargo check --workspace` - verify no circular dependencies
2. `cargo build --workspace` - full workspace build
3. `cargo test --workspace` - all tests pass
4. `cargo clippy --workspace -- -D warnings` - no warnings
5. Feature gate tests:
   - `cargo build -p riptide-pipeline --no-default-features`
   - `cargo build -p riptide-pipeline --features llm`

## Risk Assessment

### High Risk
1. **AppState Coupling** - Pipeline tightly coupled to riptide-api's AppState
   - Mitigation: Keep AppState in riptide-api initially, extract later if needed

2. **Circular Dependency Risk** - If pipeline depends on riptide-api, and api depends on pipeline
   - Mitigation: Careful import management, use `default-features = false`

### Medium Risk
1. **Feature Gate Complexity** - Multiple optional features need correct propagation
   - Mitigation: Thorough testing of feature combinations

2. **Import Updates** - 15+ files need import updates
   - Mitigation: Systematic approach, use compiler errors as checklist

### Low Risk
1. **Testing** - Tests should move cleanly with code
2. **Documentation** - Update docs after extraction

## Success Criteria

1. ✅ No circular dependency between riptide-api and riptide-facade
2. ✅ All tests pass: `cargo test --workspace`
3. ✅ Clean build: `cargo build --workspace` with ZERO warnings
4. ✅ Clippy clean: `cargo clippy --all -- -D warnings`
5. ✅ Feature gates work correctly
6. ✅ Python bindings (riptide-py) still build
7. ✅ API functionality unchanged (backward compatible)

## Timeline Estimate

- Phase 1 (Crate Setup): 30 minutes
- Phase 2 (Move Modules): 1 hour
- Phase 3 (Internal Imports): 1-2 hours
- Phase 4 (Update riptide-api): 1 hour
- Phase 5 (Update riptide-facade): 30 minutes
- Phase 6 (Update riptide-py): 30 minutes
- Testing & Validation: 2 hours
- **Total**: 6-7 hours

## Next Steps for Coder Agent

1. Create `riptide-pipeline` crate structure
2. Write `Cargo.toml` with dependencies
3. Create `src/lib.rs` with module structure
4. Move pipeline files one by one
5. Update imports systematically
6. Run tests after each major change
7. Final validation and cleanup

## Architecture Decision Records (ADRs)

### ADR-001: Keep AppState in riptide-api
**Decision**: Pipeline will depend on riptide-api for AppState, ApiError, and RipTideMetrics.

**Rationale**:
- Minimizes initial refactoring scope
- AppState is tightly coupled to API layer concerns
- Can be extracted later if circular dependency becomes problematic

**Alternatives Considered**:
- Extract AppState to riptide-types or new riptide-state crate
- Use dependency injection pattern

**Risks**:
- Potential for circular dependency if not managed carefully
- riptide-api becomes a "god crate" with too many responsibilities

**Mitigation**:
- Use `default-features = false` for riptide-api dependency
- Monitor for circular dependency during implementation
- Plan for AppState extraction in Phase 3 if needed

### ADR-002: Move All Pipeline Modules Together
**Decision**: Move all 4 pipeline modules (pipeline.rs, strategies_pipeline.rs, pipeline_enhanced.rs, pipeline_dual.rs) in one extraction.

**Rationale**:
- They are tightly related and reference each other
- Enhanced pipeline wraps standard pipeline
- Cleaner dependency graph
- Single migration reduces churn

**Alternatives Considered**:
- Move basic pipeline first, then enhanced versions later
- Leave enhanced versions in riptide-api

**Risks**:
- Larger initial change
- More imports to update

**Mitigation**:
- Systematic testing after each phase
- Use compiler errors as checklist

### ADR-003: Preserve Feature Gates
**Decision**: Maintain all existing feature gates, especially for "llm" and "wasm-extractor".

**Rationale**:
- Feature gates control optional dependencies
- AI features are experimental and not always needed
- Maintains backward compatibility

**Implementation**:
- `pipeline_dual.rs` remains feature-gated with `#[cfg(feature = "llm")]`
- Cargo.toml includes optional dependencies
- Default features maintain current behavior

## Memory Storage

This plan has been stored in swarm memory at:
- **Key**: `swarm/architect/pipeline-extraction-plan`
- **File**: `/workspaces/eventmesh/docs/architecture/pipeline-extraction-plan.md`

Next agent (coder) should retrieve this with:
```bash
npx claude-flow@alpha hooks session-restore --session-id "swarm-pipeline-extraction"
```
