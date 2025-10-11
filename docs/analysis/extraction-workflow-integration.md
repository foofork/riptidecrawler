# Extraction Method Integration Analysis

## Overview
This document maps how different extraction methods work together in the Riptide content extraction pipeline.

## Extraction Methods Identified

### 1. **Trek Extraction** (WASM-based, Primary)
- **Location**: `riptide-html/src/extraction_strategies.rs`
- **Type**: Core extraction strategy using WASM component
- **Confidence Score**: 0.8-1.0 (highest baseline)
- **Features**:
  - WASM-based for performance and isolation
  - Default strategy in `StrategyConfig`
  - Has fallback to HTML parsing if WASM unavailable
  - Uses `CmExtractor` with resource limits

**Integration Points**:
- Called by `StrategyManager.extract_content()`
- Fallback chain: WASM → HTML parsing → basic extraction
- Connected to strategies pipeline via `StrategiesPipelineOrchestrator`

### 2. **CSS Selector Extraction** (Enhanced)
- **Location**: `riptide-html/src/css_extraction.rs`
- **Type**: Configurable CSS selector-based extraction
- **Confidence Score**: 0.7-0.95 (context-dependent)
- **Features**:
  - 12 content transformers (trim, currency, date, URL, etc.)
  - `:has-text()` post-filtering with regex support
  - CSS-wins merge policy for conflict resolution
  - Fallback selector chains
  - Pre-built selector sets (news, blog, product, default)

**Integration Points**:
- Alternative strategy to Trek
- Can merge results with other methods using `MergePolicy`
- Provides specialized extractors for content types
- Uses `CssSelectorConfig` for enhanced configuration

**Transformers Chain**:
```
HTML → CSS Selectors → Text Extraction → :has-text() Filter → Transformers → Merged Result
```

### 3. **Regex Pattern Extraction**
- **Location**: `riptide-html/src/regex_extraction.rs`
- **Type**: Pattern-based structured data extraction
- **Confidence Score**: 0.5-0.95 (pattern-dependent)
- **Features**:
  - Pre-defined patterns (email, phone, URL, date, price, SSN, credit card)
  - Domain-specific pattern sets (news, contact, financial, social media)
  - HTML tag stripping before pattern matching
  - Required vs optional pattern handling

**Integration Points**:
- Complementary to CSS extraction
- Used for structured data extraction (contacts, dates, prices)
- Can extract from stripped text content
- Independent confidence scoring

### 4. **WASM Component Extraction**
- **Location**: `riptide-html/src/wasm_extraction.rs`
- **Type**: WebAssembly component-based extraction with resource limits
- **Confidence Score**: 0.8 (constant for prototype)
- **Features**:
  - Resource limiting (memory pages, fuel)
  - Performance metrics tracking
  - Component health monitoring
  - Extraction modes (Article, Full, Metadata, Custom)
  - AOT compilation caching
  - SIMD optimizations

**Integration Points**:
- Used by Trek extractor as implementation
- Provides `ExtractedDoc` → `ExtractedContent` conversion
- Tracks extraction statistics
- Memory-safe execution environment

**Resource Flow**:
```
HTML → WASM Engine (with fuel limits) → Component Execution → ExtractedDoc → ExtractedContent
```

### 5. **Strategies Pipeline Orchestrator**
- **Location**: `riptide-api/src/strategies_pipeline.rs`
- **Type**: High-level orchestration layer
- **Features**:
  - Cache integration
  - Gate analysis (quality scoring)
  - Decision tree (Raw, ProbesFirst, Headless)
  - PDF processing integration
  - Auto-strategy detection
  - Performance metrics aggregation

**Integration Points**:
- Coordinates all extraction methods
- Manages cache lifecycle
- Routes to appropriate extractor based on gate decision
- Handles PDF content specially

## Data Flow Architecture

```
┌─────────────────────────────────────────────────────────────┐
│                  StrategiesPipelineOrchestrator             │
│                                                             │
│  ┌──────────┐    ┌──────────┐    ┌──────────────┐         │
│  │  Cache   │ ←→ │   Gate   │ →  │   Decision   │         │
│  │  Check   │    │ Analysis │    │   (R/P/H)    │         │
│  └──────────┘    └──────────┘    └──────┬───────┘         │
│                                          ↓                  │
│  ┌──────────────────────────────────────┴────────┐         │
│  │          StrategyManager                      │         │
│  │  ┌────────────────────────────────────────┐  │         │
│  │  │  Strategy Selection                    │  │         │
│  │  │  (Trek / CSS / Regex / Multi)         │  │         │
│  │  └────────┬──────────────────────────────┘  │         │
│  │           ↓                                  │         │
│  │  ┌────────────────┐  ┌────────────────┐    │         │
│  │  │ Trek Extractor │  │ CSS Extractor  │    │         │
│  │  │  (WASM-based)  │  │ (Selector)     │    │         │
│  │  └────────┬───────┘  └────────┬───────┘    │         │
│  │           │                    │             │         │
│  │           └────────┬───────────┘             │         │
│  │                    ↓                         │         │
│  │         ┌──────────────────────┐             │         │
│  │         │  ProcessedContent    │             │         │
│  │         │  (with metadata)     │             │         │
│  │         └──────────────────────┘             │         │
│  └──────────────────────────────────────────────┘         │
│                       ↓                                    │
│            ┌──────────────────┐                            │
│            │  Cache Storage   │                            │
│            └──────────────────┘                            │
└─────────────────────────────────────────────────────────────┘

Supporting Systems:
┌──────────────┐  ┌──────────────┐  ┌──────────────┐
│ Regex        │  │ PDF          │  │ Headless     │
│ Extraction   │  │ Processing   │  │ Browser      │
└──────────────┘  └──────────────┘  └──────────────┘
```

## Transformation Pipelines

### Pipeline 1: Trek Strategy (Default)
```
URL → Fetch → HTML → Gate Analysis → Trek Extractor
                                          ↓
                                    WASM Component
                                          ↓
                                    ExtractedDoc
                                          ↓
                                    Metadata Extraction
                                          ↓
                                    ProcessedContent → Cache
```

### Pipeline 2: CSS Strategy
```
URL → Fetch → HTML → Gate Analysis → CSS Extractor
                                          ↓
                                    Selector Matching
                                          ↓
                                    :has-text() Filtering
                                          ↓
                                    Transformers (12 types)
                                          ↓
                                    ExtractedContent
                                          ↓
                                    Merge with Trek (optional)
                                          ↓
                                    ProcessedContent → Cache
```

### Pipeline 3: Multi-Strategy (Cascading)
```
URL → Fetch → HTML → Gate Analysis → Multi-Strategy
                                          ↓
                            ┌─────────────┴─────────────┐
                            ↓                           ↓
                     Trek Extractor              CSS Extractor
                            ↓                           ↓
                       Confidence                  Confidence
                       Score: 0.8                  Score: 0.7
                            ↓                           ↓
                            └─────────────┬─────────────┘
                                          ↓
                                   Merge Results
                                   (CSS-wins policy)
                                          ↓
                                   ProcessedContent → Cache
```

### Pipeline 4: Headless Strategy
```
URL → Fetch → HTML → Gate Analysis → Headless Decision
                                          ↓
                              Headless Browser Service
                                          ↓
                                   Rendered HTML
                                          ↓
                               Strategy Processing (Trek)
                                          ↓
                               ProcessedContent → Cache
```

## Dependencies Between Methods

### Direct Dependencies
1. **Trek → WASM Extraction**
   - Trek uses `CmExtractor` from `wasm_extraction.rs`
   - Falls back to `fallback_extract()` if WASM unavailable

2. **StrategyManager → All Extractors**
   - Manages lifecycle of all extraction strategies
   - Routes to appropriate extractor based on config

3. **Pipeline → StrategyManager**
   - Pipeline orchestrates strategy selection
   - Handles caching and gate decisions

### Conditional Dependencies
1. **Headless Decision → Browser Service**
   - Only invoked if gate analysis yields `Decision::Headless`
   - Falls back to direct extraction on failure

2. **PDF Content → PDF Processor**
   - Separate path for PDF content type
   - Converts PDF to HTML-like structure for strategy processing

3. **Merge Policy → Multiple Extractors**
   - CSS extraction can merge with other results
   - Conflict resolution via `MergePolicy`

## Potential Conflicts & Redundancies

### Identified Conflicts
1. **Multiple Extraction Results**
   - Both Trek and CSS can extract same content
   - Resolution: `MergePolicy` (CssWins, OtherWins, Merge, FirstValid)
   - Current default: `CssWins`

2. **Confidence Score Calculation**
   - Each extractor calculates confidence differently
   - Trek: 0.8 baseline + content indicators
   - CSS: selector match ratio + quality score
   - Regex: required pattern matches
   - No unified confidence scoring

3. **Content Transformation Duplication**
   - CSS extractors have transformers
   - Trek/WASM may apply their own transformations
   - Potential for double-transformation

### Redundancies
1. **Title Extraction**
   - Every extractor has title extraction logic
   - Multiple fallback chains (title, h1, og:title)
   - Could be unified into shared utility

2. **HTML Cleaning**
   - Regex extractor strips HTML tags
   - CSS extractor processes raw HTML
   - Trek/WASM handles HTML parsing
   - Redundant parsing overhead

3. **Metadata Extraction**
   - Multiple paths extract metadata
   - Gate analysis extracts features
   - Strategy manager extracts metadata
   - WASM component tracks metadata

## Integration Gaps

### Missing Integration Points
1. **No Unified Error Handling**
   - Each extractor has its own error types
   - No common error recovery strategy
   - Fallback paths not consistently documented

2. **Limited Strategy Composition**
   - Can't easily chain multiple strategies
   - No pipeline for progressive enhancement
   - Missing strategy priority system

3. **No Quality Feedback Loop**
   - Extraction confidence not fed back to gate
   - No learning from successful extractions
   - Cache doesn't track quality metrics

4. **Incomplete WASM Integration**
   - WASM component binding not fully implemented (TODO)
   - Currently returns mock data
   - Resource tracking implemented but unused

### Coordination Issues
1. **Cache Key Inconsistency**
   - Different strategies may generate different cache keys
   - Strategy config not consistently hashed
   - Could lead to cache misses for same content

2. **Performance Metrics Fragmentation**
   - Each component tracks metrics separately
   - No unified performance dashboard
   - Difficult to compare strategy efficiency

3. **No Strategy Selection Intelligence**
   - Auto-detection is simplistic
   - Doesn't learn from past performance
   - No content-type-specific routing

## Recommendations for Integration

### High Priority
1. **Implement Unified Confidence Scoring**
   - Create `ConfidenceCalculator` utility
   - Normalize scores across all extractors
   - Feed scores into strategy selection

2. **Complete WASM Component Binding**
   - Implement WIT interface for Trek extraction
   - Enable actual WASM execution
   - Remove mock data fallback

3. **Standardize Error Handling**
   - Create `ExtractionError` enum for all strategies
   - Implement consistent fallback chains
   - Add error recovery strategies

### Medium Priority
4. **Create Strategy Composition Framework**
   - Allow chaining multiple strategies
   - Implement priority-based selection
   - Enable progressive enhancement

5. **Add Performance Monitoring Dashboard**
   - Unified metrics collection
   - Strategy comparison tools
   - Performance bottleneck detection

6. **Implement Smart Strategy Selection**
   - Content-type detection
   - Historical performance tracking
   - Adaptive strategy routing

### Low Priority
7. **Refactor Common Utilities**
   - Shared title extraction
   - Unified HTML parsing
   - Common metadata extraction

8. **Add Quality Feedback Loop**
   - Track extraction success rates
   - Update gate thresholds dynamically
   - Cache quality metrics

## Testing Implications

The integration complexity requires:
1. **Unit tests** for each extractor
2. **Integration tests** for strategy manager
3. **End-to-end tests** for full pipeline
4. **Comparison tests** between strategies
5. **Fallback tests** for error scenarios
6. **Performance tests** for bottleneck identification
