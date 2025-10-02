# Feature Wiring Gaps Analysis Report

## Executive Summary

This report analyzes the integration status of advanced features from Weeks 5-9 in the EventMesh codebase. The analysis examines whether these features are properly wired to API endpoints and accessible to users.

**Overall Status**: Mixed implementation - Core features are well-developed but API integration varies significantly.

## Features Analysis

### ✅ FULLY WIRED FEATURES

#### 1. Query-Aware Spider (Week 7) - BM25 Scoring
**Status: FULLY IMPLEMENTED AND WIRED**

- **Implementation**: Complete BM25 scoring system in `/workspaces/eventmesh/crates/riptide-core/src/spider/query_aware.rs`
- **API Integration**: Fully integrated via spider endpoints
- **API Endpoints**:
  - `POST /spider/crawl` - Main spider crawling with BM25 scoring
  - `POST /spider/status` - Spider status with scoring metrics
  - `POST /spider/control` - Spider control operations
- **Configuration**: Configurable via `QueryAwareConfig` with BM25 parameters (k1, b, weights)
- **Feature Flags**: Controlled by `spider: false` in `configs/features.yml` (disabled by default)

**Evidence**:
- Handler at `/workspaces/eventmesh/crates/riptide-api/src/handlers/spider.rs` lines 87, 96
- Uses `ScoringConfig::default()` in spider configuration
- BM25 scorer supports corpus updates and relevance scoring

#### 2. CSS Advanced Selectors (Week 5) - 12 Transformers
**Status: FULLY IMPLEMENTED AND WIRED**

- **Implementation**: Complete transformer system in `/workspaces/eventmesh/crates/riptide-html/src/css_extraction.rs`
- **API Integration**: Accessible via strategies endpoints
- **API Endpoints**:
  - `POST /strategies/crawl` - Supports CSS selector strategies with transformers
  - `GET /strategies/info` - Lists available strategies including CSS_JSON
- **12 Transformers Available**:
  1. `trim` - Whitespace trimming
  2. `normalize_ws` - Whitespace normalization
  3. `number` - Numeric extraction
  4. `currency` - Currency parsing
  5. `date_iso` - ISO date conversion
  6. `url_abs` - Absolute URL conversion
  7. `lowercase` - Text lowercasing
  8. `uppercase` - Text uppercasing
  9. `split` - Text splitting
  10. `join` - Array joining
  11. `regex_extract` - Regex extraction
  12. `regex_replace` - Regex replacement
  13. `json_parse` - JSON parsing
  14. `html_decode` - HTML entity decoding

**Evidence**: Lines 96-114 in css_extraction.rs register all transformers

#### 3. Table Extraction (Week 6) - CSV/Markdown Export
**Status: IMPLEMENTATION COMPLETE, LIMITED API INTEGRATION**

- **Implementation**: Complete table extraction system in `/workspaces/eventmesh/crates/riptide-html/src/table_extraction.rs`
- **Features Available**:
  - RFC 4180 compliant CSV export (`to_csv()` method)
  - Markdown export with metadata (`to_markdown()` method)
  - NDJSON artifacts (`to_ndjson_artifacts()` method)
  - Complex table support (colspan/rowspan, nested tables)
- **API Integration**: LIMITED
- **Missing**: Direct API endpoints for table extraction
- **Available**: Only through PDF pipeline capabilities check

**Evidence**:
- Full implementation in table_extraction.rs (lines 686-920)
- Only API reference in `/workspaces/eventmesh/crates/riptide-api/src/routes/pdf.rs:42`

### ⚠️ PARTIALLY WIRED FEATURES

#### 4. Multi-Provider LLM (Week 8) - Provider Switching
**Status: IMPLEMENTATION COMPLETE, NO API INTEGRATION**

- **Implementation**: Complete LLM abstraction layer in `/workspaces/eventmesh/crates/riptide-intelligence/`
- **Features Available**:
  - 7 LLM providers (OpenAI, Anthropic, Ollama, LocalAI, Azure, Bedrock, Vertex)
  - Runtime provider switching via `RuntimeSwitchManager`
  - Circuit breakers, timeouts, fallback chains
  - Cost tracking and metrics
- **API Integration**: NONE
- **Missing**: No API endpoints to switch providers or configure LLM settings
- **Available**: Only through strategies endpoint with limited LLM config

**Evidence**:
- Complete implementation in riptide-intelligence crate
- Only strategy reference in strategies.rs line 218: "LLM-based extraction (hook-based, disabled by default)"

#### 5. Topic Chunking (Week 9) - TextTiling
**Status: IMPLEMENTATION COMPLETE, NO DIRECT API ACCESS**

- **Implementation**: Complete TextTiling algorithm in `/workspaces/eventmesh/crates/riptide-html/src/chunking/topic.rs`
- **Features Available**:
  - TextTiling algorithm with lexical cohesion analysis
  - Performance optimized (<200ms overhead)
  - Fallback to sliding window chunking
  - Multiple similarity measures (cosine, Jaccard, TF distribution)
- **API Integration**: MENTIONED BUT NOT ACCESSIBLE
- **Missing**: No way to specify topic chunking via API
- **Available**: Listed in strategies info as available chunking mode

**Evidence**:
- Full implementation in topic.rs (895 lines)
- Listed in strategies.rs line 248: "topic chunking" but no actual configuration path

### ❌ MAJOR WIRING GAPS

#### 1. Table Extraction API Endpoints
**CRITICAL GAP**: No direct API endpoints for table extraction

**Missing Endpoints**:
- `POST /tables/extract` - Extract tables from HTML
- `GET /tables/{id}/csv` - Export table as CSV
- `GET /tables/{id}/markdown` - Export table as Markdown
- `POST /tables/batch` - Batch table extraction

#### 2. LLM Provider Management API
**CRITICAL GAP**: No API endpoints for LLM provider management

**Missing Endpoints**:
- `GET /llm/providers` - List available providers
- `POST /llm/providers/switch` - Switch active provider
- `GET /llm/providers/status` - Provider health status
- `POST /llm/providers/config` - Configure provider settings

#### 3. Advanced Chunking Configuration
**MODERATE GAP**: Chunking strategies mentioned but not configurable

**Missing**:
- API parameter to specify chunking strategy in crawl requests
- Configuration options for TextTiling parameters
- Chunking preview/test endpoints

## Configuration Exposure Analysis

### ✅ WELL EXPOSED
- **Feature Flags**: Comprehensive system in `configs/features.yml` and runtime flags
- **Spider Configuration**: Full exposure via API requests
- **CSS Selector Configuration**: Available via strategies endpoint

### ⚠️ PARTIALLY EXPOSED
- **Performance Limits**: Defined in configs but not runtime adjustable
- **Circuit Breaker Settings**: Configured but not exposed via API

### ❌ NOT EXPOSED
- **LLM Provider Settings**: No API access to provider configuration
- **Table Extraction Settings**: No configuration options exposed
- **Advanced Chunking Parameters**: Not accessible via API

## Feature Flags Analysis

### Runtime Flags (Changeable)
- Located: `/workspaces/eventmesh/config/feature-flags/runtime.json`
- **Status**: Comprehensive coverage for core features
- **Gaps**: No flags for advanced Week 5-9 features

### Compile-time Flags
- Located: `/workspaces/eventmesh/config/feature-flags/compile-time.toml`
- **Status**: Good coverage for system-level features
- **Gaps**: Advanced extraction features not flagged

### Phase 3 Feature Flags
- Located: `/workspaces/eventmesh/configs/features.yml`
- **Status**: Good coverage for major features
- **Notable**: Spider disabled by default (`spider: false`)

## Handler Invocation Analysis

### ✅ PROPERLY INVOKED
1. **Spider Handlers**: All spider functionality properly wired through handlers
2. **Strategy Handlers**: CSS and basic strategies work correctly
3. **Feature Flag Loading**: Proper configuration loading

### ⚠️ PARTIALLY INVOKED
1. **Table Extraction**: Implementation exists but not called from API handlers
2. **Advanced Chunking**: Listed in info but not actually used

### ❌ NOT INVOKED
1. **LLM Provider Management**: No handlers for provider operations
2. **Advanced Table Export**: Export methods not called from any handlers

## Recommendations

### High Priority (Immediate Action Required)

1. **Add Table Extraction Endpoints**
   ```rust
   // Add to main.rs router
   .route("/tables/extract", post(handlers::tables::extract_tables))
   .route("/tables/{id}/csv", get(handlers::tables::export_csv))
   .route("/tables/{id}/markdown", get(handlers::tables::export_markdown))
   ```

2. **Add LLM Provider Management**
   ```rust
   // Add LLM management endpoints
   .route("/llm/providers", get(handlers::llm::list_providers))
   .route("/llm/providers/switch", post(handlers::llm::switch_provider))
   ```

3. **Wire Advanced Chunking**
   - Add chunking strategy parameter to crawl requests
   - Implement chunking mode selection in handlers

### Medium Priority

1. **Expose Configuration APIs**
   - Runtime feature flag toggle endpoints
   - Configuration update endpoints
   - Performance limit adjustment APIs

2. **Add Feature Discovery**
   - Endpoint to list available features and their status
   - Feature capability detection APIs

### Low Priority

1. **Enhanced Monitoring**
   - Feature usage metrics
   - Performance impact tracking
   - A/B testing infrastructure

## Conclusion

The EventMesh codebase has excellent implementations of advanced features from Weeks 5-9, but significant gaps exist in API integration. The core functionality is solid, but users cannot access many advanced features due to missing API endpoints.

**Critical Actions Needed**:
1. Wire table extraction to API endpoints
2. Add LLM provider management APIs
3. Enable advanced chunking configuration via API
4. Expose runtime configuration management

**Overall Assessment**: The foundation is excellent, but API completeness needs immediate attention to make advanced features accessible to users.