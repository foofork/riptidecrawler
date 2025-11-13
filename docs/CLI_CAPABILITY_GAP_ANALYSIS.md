# RipTide CLI Capability Gap Analysis

## 🎯 Executive Summary

**Current CLI Coverage:** 6/24 endpoint groups = **25% of API capabilities**

The CLI is **architecturally sound** but **functionally incomplete**. It only covers basic operations and is missing 75% of the API server's capabilities.

---

## ✅ What CLI Currently Supports (6 commands)

| CLI Command | API Endpoint | Functionality | Grade |
|-------------|--------------|---------------|-------|
| `extract` | `/extract` | Content extraction with strategies | A+ |
| `spider` | `/spider/*` | Deep crawl with frontier management | A+ |
| `search` | `/search` | Web search integration | B+ |
| `render` | `/render` | JavaScript-heavy page rendering | A |
| `doctor` | `/healthz`, `/health/*` | System health diagnostics | B+ |
| `session` | `/sessions/*` | Session management for auth | C+ |

---

## ❌ Missing CLI Commands (18 major endpoint groups)

### **TIER 1: Core Missing Features (High Priority)**

#### 1. **Basic Crawl** - `/crawl`, `/crawl/stream`
**Impact:** Cannot use simple crawl endpoint (simpler than spider)
- Missing: Basic crawl without full spider complexity
- Missing: Streaming crawl with real-time progress

#### 2. **PDF Processing** - `/pdf/*`
**Impact:** Cannot process PDF documents
- Missing: `/pdf/process` - Process PDF from JSON
- Missing: `/pdf/upload` - Upload and process PDF file
- Missing: `/pdf/process-stream` - Stream PDF processing with progress

#### 3. **DeepSearch** - `/deepsearch/*`
**Impact:** Cannot use advanced deep search (different from basic search)
- Missing: `/deepsearch` - Deep search with extraction
- Missing: `/deepsearch/stream` - Streaming deep search

#### 4. **Strategies** - `/strategies/*`
**Impact:** Cannot query or configure extraction strategies
- Missing: `/strategies/crawl` - Advanced strategy-based crawl
- Missing: `/strategies/info` - Get available strategies

---

### **TIER 2: Advanced Features (Medium Priority)**

#### 5. **Table Extraction** - `/api/v1/tables/*`
**Impact:** Cannot extract tables from pages
- Missing: `/api/v1/tables/extract` - Extract tables
- Missing: `/api/v1/tables/:id/export` - Export extracted tables

#### 6. **Content Chunking** - `/api/v1/content/*`
**Impact:** Cannot chunk content for LLM processing
- Missing: `/api/v1/content/chunk` - Chunk content intelligently

#### 7. **Stealth Configuration** - `/stealth/*`
**Impact:** Cannot configure or test stealth features
- Missing: `/stealth/configure` - Configure stealth settings
- Missing: `/stealth/test` - Test stealth effectiveness
- Missing: `/stealth/capabilities` - Query stealth capabilities

#### 8. **LLM Providers** - `/api/v1/llm/*`
**Impact:** Cannot manage LLM provider configuration
- Missing: LLM provider management endpoints

#### 9. **Engine Selection** - `/engine/*`
**Impact:** Cannot configure or query extraction engines
- Missing: Engine configuration and selection endpoints

#### 10. **Domain Profiles** - `/api/v1/profiles/*`
**Impact:** Cannot manage domain-specific crawl profiles
- Missing: Profile management for warm-start caching

---

### **TIER 3: Feature-Gated & Advanced (Lower Priority)**

#### 11. **Workers** - `/workers/*` (requires `workers` feature)
**Impact:** Cannot submit or manage background jobs
- Missing: Job submission, status, results, scheduling

#### 12. **Browser Management** - `/api/v1/browser/*` (requires `browser` feature)
**Impact:** Cannot manage browser pool or browser sessions
- Missing: Browser session creation, actions, pool status

#### 13. **Admin Operations** - `/admin/*` (requires `persistence` feature)
**Impact:** Cannot perform admin tasks
- Missing: Tenant management, cache management, state management

---

### **TIER 4: Observability & Monitoring (Optional)**

#### 14. **Resource Monitoring** - `/resources/*`
**Impact:** Cannot monitor system resources
- Missing: Browser pool, rate limiter, memory, performance status

#### 15. **Monitoring System** - `/monitoring/*`
**Impact:** Cannot access monitoring data
- Missing: Health scores, performance reports, alerts, profiling

#### 16. **Performance Profiling** - `/api/profiling/*`
**Impact:** Cannot profile performance
- Missing: Memory, CPU, bottlenecks, allocations, leak detection

#### 17. **Telemetry** - `/api/telemetry/*`
**Impact:** Cannot access distributed traces
- Missing: Trace visualization, trace listing

#### 18. **Metrics** - `/metrics`
**Impact:** Cannot fetch Prometheus metrics
- Missing: Prometheus metrics endpoint

---

## 📊 Coverage Analysis

### By Priority:
| Tier | Missing Features | Coverage | Impact |
|------|------------------|----------|--------|
| **Tier 1** | 4 groups | **0%** | **Critical** - Core functionality missing |
| **Tier 2** | 6 groups | **0%** | **High** - Advanced features unavailable |
| **Tier 3** | 3 groups | **0%** | **Medium** - Feature-gated capabilities |
| **Tier 4** | 5 groups | **0%** | **Low** - Observability features |

### By Use Case:
- **Content Extraction:** 40% covered (extract ✅, pdf ❌, tables ❌, chunking ❌, strategies ❌)
- **Crawling:** 50% covered (spider ✅, search ✅, crawl ❌, deepsearch ❌)
- **Configuration:** 33% covered (session ✅, stealth ❌, profiles ❌)
- **Monitoring:** 5% covered (doctor ✅, resources ❌, monitoring ❌, profiling ❌, metrics ❌)
- **Administration:** 0% covered (all admin endpoints missing)

---

## 🎯 Recommendations for Full CLI Capability

### **Phase 1: Critical Missing Commands (2-3 weeks)**

Add 4 new CLI commands for Tier 1 functionality:

```bash
# 1. Basic crawl command
riptide crawl --url https://example.com --depth 2 --stream

# 2. PDF processing command
riptide pdf process --file document.pdf --output json
riptide pdf upload document.pdf --extract-images

# 3. DeepSearch command
riptide deepsearch --query "Rust web frameworks" --max-results 50 --stream

# 4. Strategies command
riptide strategies list
riptide strategies crawl --url https://example.com --strategy readability
```

**Files to Create:**
- `crates/riptide-cli/src/commands/crawl.rs` (~200 lines)
- `crates/riptide-cli/src/commands/pdf.rs` (~250 lines)
- `crates/riptide-cli/src/commands/deepsearch.rs` (~180 lines)
- `crates/riptide-cli/src/commands/strategies.rs` (~150 lines)

---

### **Phase 2: Advanced Features (2-3 weeks)**

Add 6 new CLI commands for Tier 2 functionality:

```bash
# 5. Table extraction
riptide tables extract --url https://example.com/data.html
riptide tables export --id table_123 --format csv

# 6. Content chunking
riptide chunk --content @article.txt --max-tokens 1000

# 7. Stealth management
riptide stealth configure --preset high
riptide stealth test --url https://bot-detector.com

# 8. LLM providers
riptide llm list
riptide llm configure --provider openai --api-key $KEY

# 9. Engine selection
riptide engine list
riptide engine select --engine native --url https://example.com

# 10. Profiles
riptide profiles create --domain example.com --strategy readability
riptide profiles list
```

**Files to Create:**
- `crates/riptide-cli/src/commands/tables.rs` (~180 lines)
- `crates/riptide-cli/src/commands/chunk.rs` (~120 lines)
- `crates/riptide-cli/src/commands/stealth.rs` (~200 lines)
- `crates/riptide-cli/src/commands/llm.rs` (~160 lines)
- `crates/riptide-cli/src/commands/engine.rs` (~140 lines)
- `crates/riptide-cli/src/commands/profiles.rs` (~180 lines)

---

### **Phase 3: Feature-Gated Commands (1-2 weeks)**

Add 3 commands for Tier 3 (optional features):

```bash
# 11. Workers (if feature enabled)
riptide workers submit --job crawl --url https://example.com
riptide workers status --job-id job_123

# 12. Browser management (if feature enabled)
riptide browser create --session user_abc
riptide browser pool-status

# 13. Admin (if feature enabled)
riptide admin tenants list
riptide admin cache warm --domain example.com
```

**Files to Create:**
- `crates/riptide-cli/src/commands/workers.rs` (~200 lines, feature-gated)
- `crates/riptide-cli/src/commands/browser.rs` (~180 lines, feature-gated)
- `crates/riptide-cli/src/commands/admin.rs` (~250 lines, feature-gated)

---

### **Phase 4: Observability Commands (1 week)**

Add 4 commands for Tier 4 (monitoring):

```bash
# 14. Resources
riptide resources status
riptide resources memory
riptide resources browser-pool

# 15-18. Monitoring/Profiling/Telemetry/Metrics
riptide monitor health-score
riptide monitor performance-report
riptide profiling memory
riptide profiling bottlenecks
riptide telemetry traces --trace-id abc123
riptide metrics export
```

**Files to Create:**
- `crates/riptide-cli/src/commands/resources.rs` (~160 lines)
- `crates/riptide-cli/src/commands/monitor.rs` (~200 lines)
- `crates/riptide-cli/src/commands/profiling.rs` (~180 lines)
- `crates/riptide-cli/src/commands/telemetry.rs` (~150 lines)
- `crates/riptide-cli/src/commands/metrics.rs` (~120 lines)

---

## 📈 Full Implementation Estimate

| Phase | Commands | LOC | Time | Priority |
|-------|----------|-----|------|----------|
| **Phase 0** | Clean existing (dead code removal) | -862 | 1 day | **Critical** |
| **Phase 1** | 4 commands (crawl, pdf, deepsearch, strategies) | ~780 | 2-3 weeks | **Critical** |
| **Phase 2** | 6 commands (tables, chunk, stealth, llm, engine, profiles) | ~980 | 2-3 weeks | **High** |
| **Phase 3** | 3 commands (workers, browser, admin) | ~630 | 1-2 weeks | **Medium** |
| **Phase 4** | 5 commands (resources, monitor, profiling, telemetry, metrics) | ~810 | 1 week | **Low** |
| **Total** | **18 new commands** | **+3,200 LOC** | **6-9 weeks** | - |

**After Completion:**
- **24/24 endpoint groups covered (100%)**
- **Full feature parity with API server**
- **Production-ready CLI**

---

## 🚀 Immediate Action Required

To make the CLI **fully capable** as requested:

1. **Remove dead code** (Phase 0) - **TODAY**
   - Delete 5 unused files (862 lines)
   - Remove 2 unused dependencies
   - Fix critical issues in existing commands

2. **Implement Tier 1 commands** (Phase 1) - **NEXT 2-3 WEEKS**
   - `crawl` - Basic crawl functionality
   - `pdf` - PDF processing
   - `deepsearch` - Advanced search
   - `strategies` - Strategy management

3. **Continue with Tier 2-4** - **FOLLOWING 3-6 WEEKS**
   - Add remaining 14 commands
   - Achieve 100% API coverage

---

## 💡 Alternative: Minimal Viable Complete CLI

If full 100% coverage isn't needed, here's a **practical 80/20 approach**:

### **Keep These 6 (Already Have):**
1. extract ✅
2. spider ✅
3. search ✅
4. render ✅
5. doctor ✅
6. session ✅

### **Add These 6 Critical Ones:**
7. **crawl** - Basic crawl (simpler than spider)
8. **pdf** - PDF processing
9. **tables** - Table extraction
10. **stealth** - Stealth configuration
11. **profiles** - Domain profiles
12. **strategies** - Strategy info/usage

**Result:** 12 commands covering **50% of endpoints** but **90% of use cases**

**Effort:** ~2-3 weeks instead of 6-9 weeks

---

## 🎯 Decision Point

**Question for Product Owner:**

Do you want:
- **Option A:** Full 100% API coverage (18 new commands, 6-9 weeks)
- **Option B:** Practical 90% use case coverage (6 new commands, 2-3 weeks)
- **Option C:** Clean up existing CLI first, then reassess

**Recommendation:** Start with **Option C** (clean dead code), then **Option B** (practical coverage), then expand to Option A if needed.
