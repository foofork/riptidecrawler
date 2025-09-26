# RipTide Development Roadmap - AI-First Evolution

---

## 🏗️ Project Structure Evolution

### Current Structure Assessment
The current structure is production-solid with clean workspace separation. The following changes will accommodate AI features while maintaining architectural integrity.

### Recommended Structure Changes

#### 1. New Intelligence Crate (Phase 1)
```
crates/
├── riptide-intelligence/    # NEW: All AI/ML features (optional)
│   ├── src/
│   │   ├── llm/            # LLM providers (OpenAI, Anthropic)
│   │   ├── search/         # Search providers (Serper, None, SearXNG)
│   │   ├── embeddings/     # Future: embedding providers
│   │   ├── schemas/        # JSON schema definitions
│   │   └── lib.rs         # Feature-flagged exports
│   └── Cargo.toml         # Optional dependencies
```

#### 2. Extraction Strategy Reorganization (Phase 2)
```
crates/riptide-core/src/strategies/
├── extraction/
│   ├── structured/         # NEW: CSS/XPath extraction
│   │   ├── css.rs
│   │   ├── xpath.rs
│   │   ├── schema.rs
│   │   └── post_processors.rs
│   └── tables/            # NEW: Table extraction
│       ├── parser.rs
│       ├── csv.rs
│       └── markdown.rs
└── chunking/
    ├── sentence.rs         # NEW implementations
    ├── topic.rs
    └── regex.rs
```

#### 3. Configuration & Profiles (Phase 2)
```
configs/
├── riptide.yml            # Main config
├── profiles/              # NEW: Preset profiles
│   ├── default.yml
│   ├── research.yml
│   ├── structured.yml
│   └── performance.yml
└── schemas/               # NEW: Extraction schemas
    ├── ecommerce.json
    ├── article.json
    └── documentation.json
```

#### 4. Test Structure Enhancement (Phase 3)
```
tests/
├── golden/                # NEW: Golden test files
│   ├── chunking/
│   ├── extraction/
│   ├── tables/
│   └── crawling/
└── benchmarks/           # NEW: Performance benchmarks
    ├── llm_extraction.rs
    └── query_crawling.rs
```

#### 5. Examples Directory (Phase 3)
```
examples/                  # NEW: Usage examples
├── basic/
│   ├── simple_crawl.rs
│   └── search.rs
├── advanced/
│   ├── llm_extraction.rs
│   ├── structured_data.rs
│   └── query_crawling.rs
└── profiles/
    └── custom_profile.rs
```

### Dependency Management Strategy

#### Feature Flags in Workspace Cargo.toml:
```toml
[workspace.features]
default = ["core", "wasm"]
core = []
wasm = ["wasmtime"]
intelligence = ["riptide-intelligence", "openai", "anthropic"]  # Optional
embeddings = ["candle"]  # Future
search-providers = ["searxng-client"]  # Optional
full = ["core", "wasm", "intelligence", "search-providers"]

# Binary size optimization
[profile.release]
opt-level = 3
lto = true
codegen-units = 1
strip = true
```

### Migration Path for Structure Changes

**Phase 1: Foundation (Weeks 1-2)**
1. Create `riptide-intelligence` crate
2. Move search provider abstractions there
3. Add LLM provider implementations
4. Feature-flag all AI dependencies

**Phase 2: Implementation (Weeks 3-6)**
1. Reorganize extraction strategies
2. Add profiles directory structure
3. Create golden test fixtures
4. Add schema examples

**Phase 3: Polish (Weeks 7-8)**
1. Add examples directory
2. Create benchmarks
3. Document module boundaries
4. Migration guide for v1.0 users

### What NOT to Change
❌ **Don't touch:**
- Current crate separation (working well)
- WASM module location (security boundary)
- Docker/infra structure (production-ready)
- Core streaming pipeline (proven architecture)
- API endpoint paths (backward compatibility)

### Future Consideration
**Project Rename:** Consider renaming `/workspaces/eventmesh/` → `/workspaces/riptide/` for consistency.

---

## 🎯 Reality-Based Gap Analysis

RipTide is production-ready but missing AI enhancement. Key findings:

### What We Have ✅
- Production microservices architecture
- WASM-based extraction with SIMD optimization
- Deep crawling with BFS/DFS/Best-First strategies
- Adaptive stopping based on content gain
- Redis caching and queueing
- Docker/K8s deployment ready

### What We're Missing ❌
- **Working LLM integration** (placeholder code exists)
- **Advanced chunking** (only sliding window)
- **Intelligent extraction** (no LLM-powered structured data)
- **Query-aware adaptive crawling** (has stopping but not foraging)
- **Flexible API-less operation** (Serper required for search)

---

## 🛠 Implementation Roadmap (Priority Order)

### 1) Search Provider Abstraction
**Goal:** Remove Serper hard requirement for `/deepsearch`
**Risk Level:** ✅ LOW - Clean abstraction, no breaking changes

```rust
pub trait SearchProvider {
    async fn search(&self, q: &str, limit: u32, country: &str, locale: &str)
        -> anyhow::Result<Vec<SearchHit>>;
}
pub enum SearchBackend { Serper, None, SearXNG } // Optional SearXNG for self-hosted
```

**Implementation Details:**
- **Providers:** Serper + None built-in, optional SearXNG if time allows
- **None provider:** Detect URLs in query, parse comma/space/newline-separated
- **Fallback:** Return 501 with "Paste URLs or configure a search backend"

**Architectural Notes:**
- **Integration Point:** New `search` module in `riptide-core/src/search/`
- **Circuit Breaker:** Required for external API calls (50% failure threshold)
- **Config Location:** Extend `ApiConfig` in `riptide-api/src/config.rs`
- **Memory Impact:** Minimal (~5MB for provider abstractions)
- **Latency Impact:** None for existing Serper path

**Tasks:**
- [ ] Create SearchProvider trait in `riptide-core`
- [ ] Implement SerperProvider (existing code)
- [ ] Add NoneProvider with URL detection from query
- [ ] Add CircuitBreaker wrapper for providers
- [ ] Optional: SearXNG provider for self-hosted option
- [ ] Config: `search.backend = "serper" | "none" | "searxng"`
- [ ] Update handlers to use abstraction

**Acceptance:**
- `/deepsearch` with `backend=none` and URLs in query returns those URLs
- Without URLs → 501 with helpful message
- `/crawl` continues to work with direct URLs
- Circuit breaker trips after 3 consecutive failures

---

### 2) Minimal Working LLM Integration
**Goal:** Make `strategy = "llm"` functional without external hooks
**Risk Level:** ⚠️ MEDIUM - External dependencies, cost implications, performance impact

```rust
pub trait LlmProvider {
    async fn extract(&self, prompt: &str, schema: &serde_json::Value, text: &str)
        -> anyhow::Result<serde_json::Value>;
}
pub enum LlmBackend { OpenAI, Anthropic, LocalNone }
```

**Implementation Details:**
- **Priority:** OpenAI first, Anthropic second
- **Schema failures:** One retry with repair prompt, then fallback to non-LLM
- **Local models:** Defer Ollama to later release
- **Fallback:** Emit warning in NDJSON, continue with non-LLM extraction

**Architectural Notes:**
- **Integration Point:** Replace `riptide-core/src/strategies/extraction/llm.rs`
- **Production Safeguards Required:**
  - Circuit breaker: 50% failure rate threshold
  - Timeouts: 30s for LLM calls, 5s for schema validation
  - Memory limit: 32K tokens max (prompt + content)
  - Rate limiting: Track token usage per provider
  - Cost tracking: Monitor API usage, implement spending limits
- **Memory Impact:** +50-100MB for provider libraries
- **Latency Impact:** +500ms-3s per extraction (with timeouts)
- **Fallback Chain:** LLM → Trek extraction → Basic extraction

**Tasks:**
- [ ] Create LlmProvider trait with timeout support
- [ ] Implement OpenAI provider first (best tooling/SDKs)
- [ ] Add circuit breaker wrapper for LLM calls
- [ ] Implement token counting and limits
- [ ] Add cost tracking metrics
- [ ] Implement Anthropic provider second
- [ ] Schema validation with `schemars` + one retry on failure
- [ ] Graceful fallback with "LLM_UNAVAILABLE" warning
- [ ] Environment: `LLM_BACKEND`, `OPENAI_API_KEY`, `ANTHROPIC_API_KEY`

**Acceptance:**
- With API key: `strategy=llm` + schema → validated JSON
- Schema invalid: 1 retry → fallback with warning
- Without keys: Downgrade to non-LLM with structured note
- Circuit breaker prevents cascade failures
- Token limits enforced (32K max)

---

### 3) Complete Chunking Feature Set
**Goal:** Feature parity with 5 chunking methods
**Risk Level:** ✅ LOW - Extends existing system, backward compatible

```rust
pub enum ChunkMethod {
    Sliding {token_max, overlap},
    Fixed {token_max},
    Sentence,
    Regex {pattern},
    Topic {window, smoothing}
}
```

**Implementation Details:**
- **TextTiling:** Pure Rust lightweight (paragraph blocks + depth scores)
- **HTML-aware:** Never split mid-tag, prefer block boundaries (`p`, `h1-h6`, etc.)
- **Token counting:** Word-based by default, optional `tiktoken-rs` behind flag

**Architectural Notes:**
- **Integration Point:** Extend `riptide-core/src/strategies/chunking/`
- **HTML Boundary Detection:** Parse DOM, identify block-level elements
- **Memory Impact:** +10-20MB for NLP models (sentence/topic)
- **Latency Impact:** +50-200ms per document
- **Performance Optimization:** Cache chunking results by content hash
- **Block Elements to Respect:** `<p>`, `<div>`, `<h1-6>`, `<article>`, `<section>`, `<pre>`, `<blockquote>`, `<li>`

**Tasks:**
- [ ] Implement sentence splitter (rule-based, HTML-aware)
- [ ] Add abbreviation dictionary for sentence boundaries
- [ ] Implement regex splitter with boundary respect
- [ ] Implement topic chunking (pure Rust TextTiling)
- [ ] Add HTML boundary detection (block-level elements)
- [ ] Word-based token counter + optional tiktoken feature
- [ ] Add chunking result caching
- [ ] Add chunking config to CrawlOptions
- [ ] Golden tests for each method

**Acceptance:**
- All 5 methods work on fixtures
- HTML boundaries respected (no mid-tag splits)
- Optional exact token counting available
- Chunking cache hit rate >80% on repeated content

---

### 4) Structured Extraction (CSS/XPath + LLM)
**Goal:** Schema-guided extraction with selector fallback
**Risk Level:** ⚠️ MEDIUM - Complex selector engine, dependency on LLM reliability

```json
{
  "strategy": "css_json",
  "schema": {
    "title": {"css": "h1", "attr":"text"},
    "price": {"css": ".price", "attr":"text", "post":"currency"},
    "images": {"css": "img", "attr":"src", "multi": true}
  },
  "llm_fallback": true,
  "merge_policy": "css_wins"
}
```

**Implementation Details:**
- **Selectors:** CSS with `:nth-child`, custom `:has-text()` pseudo
- **XPath:** Safe subset only (no side effects)
- **Post-processors:** `trim`, `normalize_ws`, `lower/upper`, `number`, `currency`, `date_iso`, `url_abs`, `regex_replace`, `join/split`, `dedupe_list`, `strip_html`
- **Conflict resolution:** CSS wins by default, configurable merge policy

**Architectural Notes:**
- **Integration Point:** New strategy in `riptide-core/src/strategies/extraction/structured.rs`
- **Dependencies:** Consider `scraper` crate for CSS, custom XPath parser
- **XPath Safety:** Block `//` (descendant-or-self), function calls, variables
- **Memory Impact:** +15-25MB for selector parsing engines
- **Latency Impact:** +100-300ms for complex selectors
- **Conflict Audit:** Log differences in NDJSON `_extraction_audit` field
- **Post-processor Pipeline:** Chain of responsibility pattern

**Tasks:**
- [ ] Integrate `scraper` crate for CSS selector engine
- [ ] Implement custom `:has-text()` pseudo-selector
- [ ] Build XPath safe subset parser (no functions/variables)
- [ ] Implement 12 standard post-processors
- [ ] Add post-processor chaining logic
- [ ] Schema parser and validator with `schemars`
- [ ] LLM fallback for missing fields only (by default)
- [ ] Conflict resolution with audit logging
- [ ] JSON schema output validation
- [ ] Integration with extraction pipeline

**Acceptance:**
- Selectors support 90% of common patterns
- XPath rejects unsafe operations
- Post-processors applied in order
- Conflicts resolved per policy with audit field
- Schema validation passes 100% of valid inputs

---

### 5) Table Extraction
**Goal:** Extract HTML tables with optional LLM enhancement
**Risk Level:** ✅ LOW-MEDIUM - Well-defined scope, integrates with existing streaming

**Implementation Details:**
- **Nested tables:** Yes, recurse with parent/child linkage
- **CSV format:** RFC 4180 compliant (proper escaping)
- **Integration:** Built into `/crawl`, emit both Markdown and CSV artifacts

**Architectural Notes:**
- **Integration Point:** Artifact generation in `riptide-api/src/streaming/ndjson.rs`
- **Table Parser:** Handle `colspan`, `rowspan`, `thead`, `tbody`, `tfoot`
- **Memory Impact:** Minimal, streaming table processing
- **Latency Impact:** +50-100ms for table-heavy documents
- **Artifact Schema:** `{type: "table", table_id, parent_table_id, format, content}`
- **CSV Escaping:** Quote fields with: commas, newlines, quotes (double quotes to escape)
- **Markdown Format:** GitHub-flavored markdown table syntax

**Tasks:**
- [ ] HTML table parser with colspan/rowspan handling
- [ ] Handle malformed tables (missing closing tags)
- [ ] Nested table recursion with parent_id tracking
- [ ] RFC 4180 CSV formatter (quote fields with special chars)
- [ ] Markdown table formatter for human readability
- [ ] Table caption and summary extraction
- [ ] LLM table reconstruction for broken markup
- [ ] Integration as artifacts in NDJSON output
- [ ] Test fixtures with real and nested tables

**Acceptance:**
- Nested tables → multiple CSVs with parent/child metadata
- CSVs round-trip safely (RFC 4180)
- Markdown contains readable tables
- Available in `/crawl` endpoint
- Handles 99% of real-world table structures

---

### 6) Query-Aware Deep Crawling
**Goal:** Information foraging with query-driven prioritization
**Risk Level:** ⚠️ MEDIUM-HIGH - Complex scoring, potential performance regression

```rust
// Frontier scoring formula
S = α * BM25(title+anchor, query) +
    β * URLSignals(depth/path/dup) +
    γ * IntraDomainDiversity +
    δ * TextSim(prev_chunks, new_chunks)
```

**Implementation Details:**
- **BM25:** Tiny pure Rust implementation (title+anchor only)
- **Weights:** Profile-defaulted, per-request override allowed
- **Embeddings:** Optional tie-breaker for top-K when configured

**Architectural Notes:**
- **Integration Point:** Enhance `riptide-core/src/spider/frontier.rs`
- **BM25 Parameters:** k1=1.2, b=0.75 (standard defaults)
- **Memory Impact:** +20-30MB for scoring structures
- **Latency Impact:** +200-500ms for frontier reordering
- **Performance Risk:** Frontier reordering could impact spider throughput
- **Scoring Weights Default:** α=0.6, β=0.2, γ=0.1, δ=0.1
- **Early Stopping:** Trigger when avg relevance <0.3 for 5 consecutive pages
- **Embeddings:** Only compute for top-10 candidates to limit cost

**Tasks:**
- [ ] BM25-lite implementation in pure Rust
- [ ] Add inverted index for title/anchor text
- [ ] URL depth and path diversity scoring
- [ ] Intra-domain diversity tracker
- [ ] Content similarity with MinHash or SimHash
- [ ] Configurable frontier weights (α, β, γ, δ) with sane defaults
- [ ] Query-aware frontier reordering
- [ ] Optional embeddings for top-K tie-breaking
- [ ] Early stopping based on query relevance
- [ ] Performance benchmarks to prevent regression
- [ ] Integration with existing Spider module

**Acceptance:**
- Frontier reorders toward on-topic pages
- Early-stop triggers sooner on off-topic branches
- Embeddings (when enabled) only affect top-K candidates
- No >10% performance regression in spider throughput
- >20% improvement in relevance scoring

---

## 🛡️ Risk Assessment & Production Safeguards

### Risk Matrix
| Component | Risk Level | Primary Concerns | Mitigation Strategy |
|-----------|------------|------------------|---------------------|
| Search Provider | ✅ LOW | API availability | Circuit breaker, graceful degradation |
| LLM Integration | ⚠️ MEDIUM | Cost, latency, failures | Timeouts, fallbacks, token limits |
| Chunking | ✅ LOW | Performance | Caching, HTML-aware boundaries |
| Structured Extraction | ⚠️ MEDIUM | Selector complexity | Safe subset, audit logging |
| Table Extraction | ✅ LOW-MEDIUM | Memory with large tables | Streaming processing |
| Query-Aware Crawling | ⚠️ MEDIUM-HIGH | Performance regression | Benchmarks, feature flag |

### Critical Safeguards

**LLM Provider Protection:**
- **Circuit Breaker:** 50% failure threshold, 1-minute cooldown
- **Timeouts:** 30s hard limit for LLM calls
- **Token Limits:** 32K max (prompt + content)
- **Cost Controls:** Daily spending limits, usage tracking
- **Fallback Chain:** LLM → Trek → Basic extraction

**Performance Protection:**
- **Memory Monitoring:** Alert on >200MB RSS spikes
- **Latency Tracking:** p95 must stay <5s
- **Feature Flags:** Disable expensive features on degradation
- **Resource Limits:** Per-request memory caps

**Error Recovery:**
```rust
pub enum RecoveryStrategy {
    Retry { max_attempts: u32, backoff: Duration },
    Fallback { to: ExtractorType },
    Degrade { disable_features: Vec<Feature> },
    CircuitBreak { cooldown: Duration },
}
```

### Monitoring Requirements

**Key Metrics to Track:**
- LLM API response times and error rates
- Token usage and costs per provider
- Circuit breaker trips per component
- Feature flag override frequency
- Memory pressure events
- Extraction strategy fallback rates
- Query-aware crawling relevance scores

**Alerting Thresholds:**
- LLM error rate >10% → Warning
- Circuit breaker open >5 minutes → Critical
- Memory usage >300MB → Warning
- p95 latency >5s sustained → Critical
- Token costs >$100/day → Warning

---

## 📦 Interface Updates

### `/crawl` Endpoint Additions
```json
{
  "urls": ["https://..."],
  "profile": "quick_read_and_cite",
  "strategy": "trek | css_json | regex | llm",
  "schema": { "field": "selector" },
  "chunking": {
    "method": "sliding|fixed|sentence|regex|topic",
    "token_max": 1200,
    "overlap": 120
  },
  "llm_fallback": true
}
```

### `/deepsearch` Provider Selection
```json
{
  "query": "...",
  "limit": 12,
  "search_backend": "default"  // resolves from config
}
```

---

## 🔒 Production Strengths to Maintain

**Keep These Working:**
- WASM isolation + pooling + SIMD
- Headless pool cap & 3s hard-cap
- Per-host RPS + robots.txt compliance
- Workers for long jobs
- NDJSON streaming everywhere
- Circuit breakers for reliability
- Redis caching (24h TTL)
- Docker/K8s deployment

---

## 📏 Success Metrics

### Performance Targets (Maintain)
- Fast-path: p50 ≤ 1.5s, p95 ≤ 5s
- Streaming TTFB < 500ms
- Memory: No >200MB RSS spikes
- Headless ratio < 15%

### Feature Completeness (New)
- [ ] Search works without API keys
- [ ] LLM extraction produces valid JSON
- [ ] All 5 chunking methods implemented
- [ ] CSS/XPath extraction functional
- [ ] Tables extract to CSV/MD
- [ ] Query-aware crawling improves relevance

### Quality Metrics
- [ ] Test coverage ≥ 80%
- [ ] Zero panics in production code
- [ ] API compatibility maintained
- [ ] Documentation updated

---

## ⚙️ Configuration Architecture

### Complete Configuration Schema
```yaml
search:
  backend: serper            # serper | searxng | none
  searxng_base_url: ""       # if self-hosted
  none_parse_urls: true      # detect URLs in query

llm:
  enabled: true
  backend: openai            # openai | anthropic | local_none
  retry_on_schema_fail: 1
  fallback_on_fail: true

chunking:
  method: sliding           # sliding | fixed | sentence | regex | topic
  token_counter: words      # words | tiktoken
  html_aware_boundaries: true
  enable_topic: true

structured:
  merge_policy: css_wins    # css_wins | llm_wins | prefer_confidence
  llm_only_if_css_missing: true
  transformers: [trim, normalize_ws, currency, date_iso, url_abs]

tables:
  enabled: true
  nested: true
  artifacts: { markdown: true, csv: true }

foraging:
  enabled: true
  weights: { alpha: 0.6, beta: 0.2, gamma: 0.1, delta: 0.1 }
  allow_request_override: true
  embeddings_tiebreak: false

features:
  llm: false                     # Feature flag for LLM
  tables: false                  # Feature flag for tables
  topic_chunking: false          # Feature flag for topic chunking
  query_foraging: false          # Feature flag for query-aware
  search_none_url_parse: true    # Feature flag for URL parsing
  embeddings: false              # Feature flag for embeddings
```

### Feature Flags Strategy
- **Default OFF:** New features ship disabled for safe rollout
- **Per-request override:** Allow feature enabling via request headers
- **Profile-based:** Profiles can enable feature sets
- **A/B testing ready:** Feature flags enable gradual rollout

### API Versioning Strategy
- **Stay v1:** Additive parameters only, no breaking changes
- **Future v2:** Only if response shapes break, maintain v1 alongside
- **Profile system:** Ship YAML presets with binary, allow user overrides

---

## 🚀 Implementation Phases

### Phase 1: API Flexibility (Week 1)
- Search provider abstraction
- Config-driven provider selection
- Graceful degradation

### Phase 2: LLM Integration (Week 2-3)
- LLM provider trait
- OpenAI/Anthropic implementations
- Schema validation
- Fallback handling

### Phase 3: Extraction Enhancement (Week 3-4)
- Complete chunking methods
- CSS/XPath extraction
- Table handling
- Structured output

### Phase 4: Intelligence Layer (Week 5-6)
- Query-aware crawling
- Information foraging
- Frontier scoring
- Adaptive stopping improvements

### Phase 5: Polish & Testing (Week 7)
- Integration tests
- Performance validation
- Documentation
- Example profiles

---

## 🎯 Definition of Done

**Each feature must:**
1. Pass all existing tests (no regression)
2. Include new tests with >80% coverage
3. Maintain performance targets
4. Update API documentation
5. Include usage examples
6. Handle errors gracefully
7. Expose metrics

**Project complete when:**
- All 6 feature gaps closed
- Performance maintained or improved
- Documentation comprehensive
- Migration guide written
- Deployed to staging environment

---

## 💡 Future Enhancements (Post-Parity)

### Nice-to-Have
- Additional search providers (Bing, DuckDuckGo)
- Local LLM support (Ollama, llama.cpp)
- Visual extraction (screenshot + OCR)
- Workflow automation
- MCP server mode

### Innovation Opportunities
- Embedding-based similarity
- Multi-modal extraction
- Distributed crawling
- Real-time collaboration
- Agent frameworks integration

---

## 🎯 Profile System

### Built-in Profiles (Ship with Binary)
```yaml
profiles:
  quick_read_and_cite:
    chunking: { method: sliding, token_max: 1500 }
    strategy: trek
    include_links: true

  structured_data:
    strategy: css_json
    llm_fallback: true
    merge_policy: css_wins

  research_deep:
    foraging: { enabled: true }
    max_depth: 5
    strategy: llm
    chunking: { method: topic }

  tables_and_data:
    tables: { enabled: true }
    strategy: css_json
    artifacts: true
```

### Profile Endpoints
- `GET /profiles` - List available profiles
- `GET /profiles/{name}` - Get profile configuration
- `POST /profiles/validate` - Validate custom profile

---

## 🔄 Deployment & Rollout

### Rollout Strategy
1. **Feature flags OFF by default** - Safe initial deployment
2. **Test with profiles** - Enable features via specific profiles
3. **Gradual enablement** - Turn on features one by one
4. **Monitor metrics** - Watch performance and error rates
5. **Full enablement** - Default ON once stable

### Backward Compatibility
- All v1 endpoints preserved
- New parameters are optional
- Response shapes unchanged (additive only)
- Legacy `strategy: "trek"` continues working

---

## 📚 References

- **Completed Work:** [`COMPLETED.md`](./COMPLETED.md)
- **Architecture:** [`docs/architecture/`](./architecture/)
- **API Documentation:** [`docs/api/`](./api/)
- **Performance Reports:** [`docs/benchmarks/`](./benchmarks/)

---

## ✅ Implementation Checklist

### Search Provider Abstraction
- [ ] SearchProvider trait definition
- [ ] SerperProvider implementation
- [ ] NoneProvider with URL detection
- [ ] Configuration integration
- [ ] Handler updates
- [ ] Tests with/without API keys

### LLM Integration
- [ ] LlmProvider trait definition
- [ ] OpenAI provider implementation
- [ ] Anthropic provider implementation
- [ ] Schema validation with retry
- [ ] Fallback mechanism
- [ ] NDJSON warning emission
- [ ] Integration tests

### Chunking Methods
- [ ] Sentence splitter implementation
- [ ] Regex splitter implementation
- [ ] Topic chunking (TextTiling)
- [ ] HTML boundary detection
- [ ] Token counting (words + tiktoken)
- [ ] Configuration integration
- [ ] Golden test fixtures

### Structured Extraction
- [ ] CSS selector engine
- [ ] Custom :has-text() pseudo
- [ ] XPath safe subset
- [ ] 12 post-processors
- [ ] Merge policy logic
- [ ] Audit logging
- [ ] Schema validation

### Table Extraction
- [ ] HTML table parser
- [ ] Nested table support
- [ ] RFC 4180 CSV formatter
- [ ] Markdown formatter
- [ ] LLM reconstruction
- [ ] Artifact generation
- [ ] Integration tests

### Query-Aware Crawling
- [ ] BM25-lite implementation
- [ ] Configurable weights
- [ ] Frontier reordering
- [ ] Content similarity
- [ ] Embeddings integration
- [ ] Early stopping logic
- [ ] Spider integration

### Infrastructure
- [ ] Feature flags system
- [ ] Profile management
- [ ] Configuration schema
- [ ] API documentation
- [ ] Migration guide
- [ ] Performance benchmarks

---

*Last updated: September 26, 2025*
*Status: Implementation roadmap with detailed specifications ready*