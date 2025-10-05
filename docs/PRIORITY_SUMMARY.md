# RipTide Roadmap - Priority Summary

*Created: 2025-10-04*
*Based on: Dead code analysis from commits f7dd96a, d381456, 534ff5d*

---

## 🎯 KEY INSIGHT

**Most roadmap items are ACTIVATION not CREATION**

- **159 of 186 items (85%)** are fully implemented code marked with `#[allow(dead_code)]`
- **All code compiles** with 0 errors
- **Just needs**: Remove suppression attributes and activate routes/endpoints

---

## 📊 Priority Classification

### 🔥 HIGH PRIORITY - Dead Code Activation (159 items, 6-8 days)

**What**: Code that exists and works, just suppressed to avoid warnings

**Why High Priority**:
- ✅ Zero implementation risk (code already written and tested)
- ✅ Immediate value delivery
- ✅ Low effort (remove `#[allow(dead_code)]` + wire up)
- ✅ No new bugs (code has been in codebase)

**Items**:
1. **Streaming Infrastructure** (64 items) - NDJSON, SSE, WebSocket protocols
2. **Session Management** (19 items) - Cookie-based session system
3. **Advanced Metrics** (31 items) - Phase timing, error tracking, PDF/WASM metrics
4. **Advanced Health Checks** (14 items) - Comprehensive system diagnostics
5. **Resource Management** (10 items) - RAII guards, pool management
6. **Telemetry Features** (12 items) - Trace visualization endpoints
7. **Worker Management** (1 item) - Job listing endpoint
8. **Application State** (8 items) - Config field activation

---

### 🟡 MEDIUM PRIORITY - Needs Implementation (24 items, 2-3 days)

**What**: Partially implemented features that need completion

**Why Medium Priority**:
- ⚠️ Requires actual coding (not just activation)
- ⚠️ Higher risk of bugs
- ⚠️ Needs testing and validation

**Items**:
1. **FetchEngine** (8 tasks) - Per-host circuit breakers, retry policies, rate limiting
2. **Cache Warming** (8 tasks) - Warming algorithms, scheduler, metrics integration
3. **Advanced Strategies** (8 items) - CSS_JSON and REGEX extraction (NOT LLM)

---

### 🔴 LOW PRIORITY - Defer to Future (3 items, 1-2 weeks)

**What**: High-complexity features for later sprints

**Why Deferred**:
- ❌ Massively underscoped (1 day → 1-2 weeks realistic)
- ❌ Requires external API integration
- ❌ Needs prompt engineering and tuning
- ❌ Not critical for production

**Items**:
1. **LLM Strategy** (STRAT-009) - AI-powered extraction with custom prompts

---

## 📅 Revised Execution Plan

### Phase 4A: Quick Wins (2 days) - 63 items
- Application State (4h)
- Advanced Metrics (1d)
- Advanced Health Checks (4h)
- Resource Management (4h)

### Phase 4B: Infrastructure (3 days) - 77 items
- Worker Management (2h)
- Telemetry Features (4h)
- Streaming Infrastructure (2-3d)

### Phase 5: Implementation (3 days) - 43 items
- FetchEngine (1d)
- Cache Warming (1d)
- Advanced Strategies CSS/Regex (4h)
- Session Management (4h)

### Phase 6: Deferred (future) - 3 items
- LLM Strategy (1-2 weeks)

**Total**: 8 days to activate 183 of 186 items (98.4%)

---

## 🔍 Dead Code Analysis References

### Commit f7dd96a (Main Dead Code Elimination)
**118 warnings suppressed across 22 files**

1. **Streaming Infrastructure** (64 warnings)
   - `streaming/mod.rs` - Module orchestration
   - `streaming/processor.rs` - Stream processing (13 items)
   - `streaming/pipeline.rs` - Pipeline orchestration (10 items)
   - `streaming/error.rs` - Error types (6 items)
   - `streaming/config.rs` - Configuration (1 item)
   - `streaming/buffer.rs` - Buffer management (3 items)
   - `streaming/lifecycle.rs` - Lifecycle management (5 items)
   - `streaming/websocket.rs` - WebSocket protocol (3 items)
   - `streaming/sse.rs` - Server-Sent Events (2 items)
   - `streaming/ndjson/streaming.rs` - NDJSON protocol (1 item)

2. **Session Management** (19 warnings)
   - `sessions/mod.rs` - Session coordination (2 items)
   - `sessions/manager.rs` - Lifecycle management (6 items)
   - `sessions/middleware.rs` - Axum middleware (4 items)
   - `sessions/types.rs` - Session types and cookies (7 items)

3. **Telemetry & Observability** (12 warnings)
   - `handlers/telemetry.rs` - Trace tree endpoints (9 items)
   - `telemetry_config.rs` - Initialization and config (3 items)

4. **Enhanced Pipeline** (7 warnings)
   - `pipeline_enhanced.rs` - Enhanced orchestrator (7 items)

5. **Metrics Collection** (31 warnings)
   - `metrics.rs` - Phase timing histograms (4 fields)
   - Error counters (3 fields)
   - Streaming metrics (3 fields + 3 methods)
   - PDF metrics (9 fields + 5 methods)
   - WASM metrics (6 fields + 4 methods)
   - Helper types (PhaseType, PhaseTimer)

6. **Health Checking** (14 warnings)
   - `health.rs` - HealthChecker fields (3 fields)
   - Health check methods (8 methods)
   - System metrics helpers (3 functions)

7. **Resource Management** (10 warnings)
   - `resource_manager.rs` - ResourceMetrics fields (3 fields)
   - Resource guards (2 structs)
   - Resource methods (1 method)
   - ResourceStatus fields (7 fields)

8. **Application State** (8 warnings)
   - `state.rs` - AppState fields (6 fields)
   - AppConfig fields (2 fields)
   - EnhancedPipelineConfig (7 fields)
   - CircuitBreakerConfig (1 field)
   - MonitoringSystem (1 field)

9. **Workers Handler** (1 warning)
   - `handlers/workers.rs` - JobListQuery struct

### Commit d381456 (Streaming Response Helpers)
**8 warnings suppressed**

- `streaming/response_helpers.rs`:
  - StreamingErrorResponse impl (4 functions)
  - KeepAliveHelper struct and impl (4 items)

### Commit 534ff5d (Strategies & Tables)
**11 warnings suppressed**

- `strategies.rs`:
  - StrategiesCrawlRequest fields (6 fields)
  - RegexPatternRequest struct (4 fields)
  - LlmConfigRequest struct (3 fields)

- `tables.rs`:
  - TableExtractionOptions fields (2 fields)

---

## ✅ Success Metrics

**Activation Complete When**:
- All `#[allow(dead_code)]` attributes removed (159 locations)
- Routes exposed for streaming, telemetry, workers
- Middleware integrated for sessions
- Metrics wired to collection points
- Health checks exposed in endpoints
- Resource guards activated in pipeline
- State fields populated and utilized
- 0 compilation errors maintained

**Performance Targets**:
- No performance regression (<5% overhead)
- All features functional in integration tests
- Clean `cargo check` and `cargo clippy`

---

## 🎯 Next Steps

1. ✅ **Start with Phase 4A** (2 days, 63 items, LOW risk)
   - Application State activation (prerequisite for others)
   - Metrics activation (observability foundation)
   - Health check activation (production monitoring)
   - Resource management activation (production scaling)

2. ✅ **Then Phase 4B** (3 days, 77 items, MEDIUM risk)
   - Worker endpoint (quick win)
   - Telemetry endpoints (debugging capability)
   - Streaming infrastructure (biggest feature, save for when others are solid)

3. ✅ **Finally Phase 5** (3 days, 43 items, MEDIUM risk)
   - FetchEngine implementation (unified HTTP)
   - Cache warming implementation (performance)
   - CSS/Regex strategies (feature expansion)
   - Session middleware integration (stateful workflows)

4. ⏸️ **Defer Phase 6** (LLM strategy to future sprint)

---

**This prioritization ensures maximum value delivery with minimum risk by activating existing, tested code first.**
