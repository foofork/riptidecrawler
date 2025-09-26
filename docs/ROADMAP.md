# RipTide Development Roadmap — Production-Ready AI Enhancement (Revised)

## 🎯 Executive Summary

RipTide gets AI-powered extraction and intelligent crawling while preserving production strengths (WASM isolation, streaming, circuit breakers). This **12-week** roadmap delivers LLM integration, advanced chunking, structured extraction, and query-aware crawling with realistic timelines and proper safeguards.

### Key Decisions (Clarified)
- **Performance Trade-off:** Accept **25–30%** throughput reduction when AI features enabled (flag-gated, profile-controlled)
- **LLM Budget:** **$2,000/month** global cap with **per-tenant** and **per-job** limits (150k tokens + $10/job), enforced server-side
- **Providers:** **OpenAI at day-1**; **Anthropic by Week 8**; **LocalNone fallback** always available
- **Quality Bar (no LLM):** **≥80% field fill-rate** via CSS/heuristics; missing fields explicit nulls with audit
- **Timeline:** **12 weeks** for full parity, **8-week MVP** option available

---

## 📊 Current State vs Target State

### What We Have ✅
- WASM-based extraction (SIMD), microservices, Redis caching & workers
- Deep crawling (BFS/DFS/Best-First), NDJSON streaming
- Circuit breakers, per-host RPS, robots.txt compliance
- Docker/K8s ready, health checks

### What We're Building 🚀
- **Schema-true JSON** (selectors first, **LLM repair** if available; never blocks)
- **Table extraction** → CSV/Markdown (nested supported)
- **Query-aware crawling** (on-topic prioritization + early stop)
- **Pluggable search** (**Serper / None / optional SearXNG**)
- **5 chunkers** with HTML-aware boundaries
- **Multi-provider LLM** with strict timeouts, retries, budgets, and fallbacks

---

## 🗺️ 12-Week Implementation Timeline

### R0 — Security, Auth, Budgets & PII (Week 0–1, parallel)
**Risk:** ✅ LOW | **Dependencies:** None

**Deliverables:**
- API keys per tenant + per-key rate limits; audit logging
- Secrets management & rotation
- **Budget enforcement:** global/tenant/job caps; NDJSON cost telemetry
- **PII redaction policy** (mask emails/phones/IDs in logs & LLM payloads unless schema explicitly requires them)

**Acceptance:**
- Requests require API key; budgets & rate limits enforced
- Audit logs include who/when/what/cost
- Redaction verified in debug logs; never in user payloads

---

### R1 — Guardrails & Scaffolding (Week 1)
**Risk:** ✅ LOW | **Dependencies:** R0

**Deliverables:**
- Feature flags + profile loader (`/profiles` with YAML)
- SearchProvider trait: **Serper**, **None** (URL parsing in `query`), **SearXNG (optional)** if configured
- Cost tracking pipeline (wired to R0 budgets)

**Acceptance:**
- `/deepsearch` works with configured providers
- `backend=none` parses URLs from `query` or returns 501 with guidance
- Rate limiters and cost counters visible in metrics
- No regressions to existing endpoints

---

### R2 — LLM v1 (Week 2)
**Risk:** ⚠️ MEDIUM | **Dependencies:** R0–R1

**Deliverables:**
- `LlmProvider` trait + **OpenAI adapter**
- **5-second hard timeout**, **one schema-repair retry**, then fallback
- Multi-signal circuit breaker; per-tenant throttles; budget enforcement
- Token/cost accounting into NDJSON lines and Prometheus

**Circuit Breaker:**
```yaml
error_rate: ≥20% over last 50 events (min 20)
consecutive_failures: ≥5
latency_breaker: p95 > 4s over 100 calls
recovery_timeout: 60s, half_open_trials: 5
```

**Acceptance:**
- Valid schema JSON when key present
- Graceful fallback with `_extraction_audit` note when absent/failed
- Breaker engages under stress; budgets stop over-spend

---

### R3 — Chunking Parity (Week 3)
**Risk:** ✅ LOW | **Dependencies:** None

**Deliverables:**
- 4 chunkers: **sliding, fixed, sentence, regex**
- HTML-aware boundaries (no mid-tag splits)
- Word-approx token counting (tiktoken optional feature)
- Content-hash cache for chunking results

**Acceptance:**
- All methods pass golden fixtures
- Block element boundaries respected
- Cache hit-rate >80% on repeats
- **Overhead ≤200ms for ~50KB text** (documented in benchmarks)

---

### R4 — NDJSON & Reports Polish (Week 4)
**Risk:** ✅ LOW | **Dependencies:** R1–R3

**Deliverables:**
- NDJSON viewer page; static report packs (Extraction, Dynamic)
- Updated OpenAPI; `/tools/registry.json` for agents
- **DX:** Postman/Insomnia collection; minimal CLI smoke commands

**Acceptance:**
- Live streaming visible in browser
- Reports open as HTML
- Agents discover tools; CLI & Postman work end-to-end

---

### R5a — Structured Extraction: Basic CSS (Week 5)
**Risk:** ⚠️ MEDIUM | **Dependencies:** R2

**Deliverables:**
- CSS engine for common selectors (class/id/attr, child/descendant, `:nth-child`)
- Custom **`:has-text("…")`** as a **post-filter** (not a true pseudo)
- 12 standard transformers (see list below)
- Merge policy (`css_wins` default) with conflict audit
- **Respect `robots`/`noai`** headers/meta (skip/downgrade per profile)

**Transformers:**
`trim, normalize_ws, lower/upper, number, currency, date_iso, url_abs, regex_replace, join/split, dedupe_list, strip_html`

**Acceptance:**
- ≥90% everyday schemas pass
- Transformers chain deterministically
- Conflicts logged in `_extraction_audit`
- Schema validation on all outputs
- `noai` honored per profile policy

---

### R5b — Tables v1 (Week 6)
**Risk:** ✅ LOW | **Dependencies:** R5a

**Deliverables:**
- Table parser (`thead/tbody/tfoot`, **colspan/rowspan**)
- **Nested tables** with `parent_id` linkage
- **RFC 4180** CSV & Markdown artifacts (stored with retention/GC policy)
- Integration into `/crawl`

**Acceptance:**
- Nested tables → linked CSVs; CSV round-trips safely
- Markdown readable
- Artifacts show up in NDJSON with URIs; retention honored

---

### R6 — Query-Aware Spider v1 (Week 7)
**Risk:** ⚠️ HIGH | **Dependencies:** R3

**Deliverables:**
- **BM25-lite** (title+anchor), URL signals (depth/path), domain diversity
- Early stop on low rolling relevance; weight knobs (α,β,γ,δ) per profile (with clamping)
- Optional embeddings tiebreak (top-K only) when configured

**Scoring Formula:**
```
S = α*BM25 + β*URLSignals + γ*DomainDiversity + δ*ContentSimilarity
```

**Acceptance:**
- ≥20% lift in on-topic tokens/page vs control at same budget
- ≤10% spider throughput regression
- Early stop triggers correctly; weights respected per profile

---

### R7 — Anthropic Adapter & LLM Ops (Week 8)
**Risk:** ✅ LOW | **Dependencies:** R2

**Deliverables:**
- **Anthropic** adapter (same trait), same 5s/repair/fallback rules
- LLM ops dashboards (latency, error-rate, breaker, spend per tenant)
- Runtime provider selection via config

**Acceptance:**
- Provider swap via config without restart (or documented reload)
- Dashboards show usage/spend; alerts on thresholds
- Both providers respect timeouts and cost limits

---

### R8 — Topic Chunking (Week 9)
**Risk:** ⚠️ MEDIUM | **Dependencies:** R3

**Deliverables:**
- TextTiling-style topic detection (pure Rust; paragraphs + depth scores)

**Acceptance:**
- Deterministic segments on long docs; golden fixtures pass
- ≤200ms additional overhead per doc (documented)

---

### R9 — Advanced Selectors & Safe XPath (Weeks 10–11)
**Risk:** ⚠️ HIGH | **Dependencies:** R5a

**Deliverables:**
- Advanced CSS cases; **safe-subset XPath** (allowlist)
- Selector fuzzer tests; per-page perf caps

**XPath Allowlist:**
```yaml
allowed_axes: [child, descendant, parent, ancestor, following-sibling]
allowed_functions: [text(), contains(), position(), last()]
forbidden: [document(), system-property(), unparsed-entity-uri()]
```

**Acceptance:**
- Expanded selector coverage; unsafe XPath rejected
- No perf cliffs (caps enforced); security audit passed

---

### R10 — Hardening, Retention & Performance (Week 12)
**Risk:** ✅ LOW | **Dependencies:** R1–R9

**Deliverables:**
- Perf pass; memory profiling & limits; **artifact retention/GC** finalized (e.g., 7–30 days)
- Runbooks (LLM outage, budget exhaustion, headless brownouts, storage pressure)
- Documentation complete; SDK stubs (TS/Rust)

**Performance Targets:**
```yaml
latency: { p50: ≤1.5s, p95: ≤5s }
memory: { steady_state: ≤600MB RSS, container_limit: 768MB, alert: 650MB }
throughput: { headless_ratio: <15%, ai_degradation: ≤30% }
```

**Acceptance:**
- SLOs met; test coverage ≥80%; no `unwrap/expect` in hot paths
- v1 API backward compatible; retention works; runbooks published

---

## 🚀 8-Week MVP Option

If you must ship in 8 weeks, here's the reduced scope:

### MVP Includes ✅
- **R0**: Security, auth, budgets, PII
- **R1**: Rate limiting & search providers
- **R2**: OpenAI LLM integration
- **R3**: 4 chunking methods
- **R4**: NDJSON viewer & reports
- **R5a**: Basic CSS extraction
- **R5b**: Table extraction

### MVP Defers ⏸️
- Query-aware spider (flag as beta)
- Anthropic provider (Week 8+)
- Topic chunking
- Advanced selectors/XPath

**Result:** Production crawler with LLM-assisted extraction, tables, streaming, and pluggable search—stable and cost-guarded.

---

## 🛡️ Operational Guardrails

### LLM Protection
- **Timeout:** 5s hard limit
- **Retries:** 1 schema repair attempt
- **Circuit Breaker:** Multi-signal design
- **Budget:** $2k/month global, $10/job max
- **Tokens:** 150k per job maximum
- **PII:** Redaction in logs/LLM unless required

### Memory Management
- **Container Limit:** 768MB
- **Target RSS:** 400–600MB with AI
- **Alert Threshold:** 650MB
- **Concurrency Caps:** Headless pool, LLM calls, PDFs

### Performance Expectations
- **AI Impact:** 25–30% throughput reduction when enabled
- **Baseline Protected:** Flags allow disabling per job
- **Latency SLOs:** p50 ≤1.5s, p95 ≤5s maintained

### Data Retention
- **Artifacts:** 7–30 days (configurable)
- **Logs:** 90 days audit, 30 days debug
- **Cache:** 24h crawl, 7d LLM results

---

## ⚙️ Configuration & Features

### Default Feature Flags
```yaml
features:
  # Stable (enabled by default)
  tables: true
  search_none_url_parse: true

  # Experimental (opt-in)
  llm: false                  # Enable via profile/job
  query_foraging: false       # Enable via profile/job
  topic_chunking: false       # Enable after R8
  embeddings: false           # Future
```

### Profile Examples
```yaml
profiles:
  quick_extract:
    chunking: { method: sliding, token_max: 1500 }
    strategy: trek

  ai_enhanced:
    llm: {
      enabled: true,
      provider: openai,
      timeout_ms: 5000,
      retries: 1,
      max_tokens_job: 150000,
      max_cost_job_usd: 10
    }
    strategy: css_json
    llm_fallback: true
    merge_policy: css_wins
    llm_only_if_css_missing: true

  research_deep:
    query_foraging: {
      enabled: true,
      weights: { alpha: 0.6, beta: 0.2, gamma: 0.1, delta: 0.1 }
    }
    max_depth: 5
    chunking: { method: topic }
```

---

## 📦 API Specification

### Enhanced `/crawl` Endpoint
```json
{
  "urls": ["https://example.com"],
  "profile": "ai_enhanced",
  "strategy": "css_json",
  "schema": {
    "title": {"css": "h1", "attr": "text"},
    "price": {"css": ".price", "post": ["trim", "currency"]}
  },
  "llm_config": {
    "provider": "openai",
    "max_tokens": 1000,
    "timeout_ms": 5000,
    "budget_usd": 10
  }
}
```

### `/deepsearch` with Provider Selection
```json
{
  "query": "web scraping best practices",
  "search_backend": "none",
  "urls": ["https://docs.example.com"],
  "limit": 20
}
```

---

## 🎯 Success Criteria

### Technical Metrics
- ✅ 5 chunking methods operational (4 in MVP)
- ✅ Multi-provider LLM with fallback
- ✅ CSS selectors with transformers
- ✅ Table extraction to CSV/MD
- ✅ Query-aware crawling (optional)
- ✅ Search without API keys

### Quality Metrics
- ✅ 80% field extraction without LLM
- ✅ <5s LLM timeout enforced
- ✅ Circuit breakers prevent cascades
- ✅ Cost tracking accurate to $0.01
- ✅ Memory stays under 600MB RSS
- ✅ PII properly redacted

### Operational Metrics
- ✅ Zero panics in production
- ✅ API backward compatible
- ✅ Feature flags allow rollback
- ✅ Monitoring dashboards live
- ✅ Runbooks documented
- ✅ Retention policies enforced

---

## 🚨 Risk Management

### Critical Risks & Mitigations

| Risk | Impact | Mitigation |
|------|--------|-----------|
| LLM Provider Outage | High | Multi-provider + LocalNone fallback |
| Memory Explosion | High | Container limits + alert at 650MB |
| Runaway Costs | High | Hard budget caps + per-job limits |
| CSS Engine Complexity | Medium | Start with scraper crate, iterate |
| Performance Regression | Medium | Feature flags for instant rollback |
| PII Leakage | High | Redaction policy + audit logging |
| Storage Pressure | Medium | Retention policies + GC |

### Contingency Plans
- **If CSS engine delayed:** Ship with basic selectors only
- **If LLM costs spike:** Reduce retry attempts, tighten timeouts
- **If memory issues:** Disable topic chunking, reduce concurrency
- **If performance degrades:** Disable query-aware scoring
- **If storage fills:** Reduce retention, increase GC frequency

---

## 🏗️ Future Optimization: Async AI Architecture

The performance team has identified that the 25-30% throughput penalty can be **completely eliminated** through async processing:

### Current (Synchronous) Flow:
```
HTML → CSS Extract → Wait for LLM (5s) → Return
```

### Future (Asynchronous) Flow:
```
HTML → CSS Extract → Return immediately (100ms)
           ↓
      Background Queue → LLM Enhancement → Update cache
```

### Benefits:
- **Zero performance impact** on crawling
- **80% cost reduction** through intelligent caching
- **Better user experience** with immediate results

This optimization is **not required for v1** but provides a clear path to eliminate the performance trade-off entirely in a future release.

---

## 📋 Implementation Checklist

### Foundation (Weeks 0-2)
- [ ] Security, auth, budgets (R0)
- [ ] Rate limiting framework
- [ ] SearchProvider abstraction
- [ ] LlmProvider with OpenAI
- [ ] Circuit breakers
- [ ] Cost tracking

### Extraction (Weeks 3-6)
- [ ] 4 chunking methods
- [ ] NDJSON viewer
- [ ] CSS selector engine
- [ ] Table parser
- [ ] Transformers pipeline

### Intelligence (Weeks 7-9)
- [ ] BM25 scoring
- [ ] Query-aware spider
- [ ] Anthropic provider
- [ ] Topic chunking

### Production (Weeks 10-12)
- [ ] Advanced selectors
- [ ] XPath safe subset
- [ ] Performance optimization
- [ ] Runbooks
- [ ] Documentation
- [ ] v1.0 release

---

## 🚀 Next Steps

1. **Immediate Actions:**
   - Set up security & auth infrastructure (R0)
   - Create `riptide-intelligence` crate
   - Define provider traits and budget enforcement

2. **Team Assignments:**
   - Assign owners for R0-R10
   - Set up weekly sync meetings
   - Create monitoring dashboards

3. **Technical Decisions:**
   - Choose CSS library (scraper vs custom)
   - Select PII detection approach
   - Define retention policies

---

## 📊 Project Status

**Current Phase:** Planning Complete
**Start Date:** Week of [TBD]
**Target v1.0:** 12 weeks from start
**MVP Option:** 8 weeks (reduced scope)
**Risk Level:** Medium (managed via safeguards)

---

*Last Updated: September 26, 2025*
*Version: 3.0.0-roadmap*
*Status: Production-ready with operational safeguards*