# RipTide Development Roadmap - AI-First Evolution

## Current Status (Updated: 2025-09-26)

* **v1.0.0 Production Release**: Complete - See [`COMPLETED.md`](./COMPLETED.md) for shipped features
* **Focus**: Achieving feature parity while maintaining production strengths
* **Architecture**: Production-first (queues, WASM isolation, streaming) + AI-first features

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

```rust
pub trait SearchProvider {
    async fn search(&self, q: &str, limit: u32, country: &str, locale: &str)
        -> anyhow::Result<Vec<SearchHit>>;
}
pub enum SearchBackend { Serper, None } // add more later
```

**Tasks:**
- [ ] Create SearchProvider trait in `riptide-core`
- [ ] Implement SerperProvider (existing code)
- [ ] Add NoneProvider with helpful 501 response
- [ ] Config: `search.backend = "serper" | "none"`
- [ ] Update handlers to use abstraction

**Acceptance:** `/deepsearch` degrades gracefully without API keys

---

### 2) Minimal Working LLM Integration
**Goal:** Make `strategy = "llm"` functional without external hooks

```rust
pub trait LlmProvider {
    async fn extract(&self, prompt: &str, schema: &serde_json::Value, text: &str)
        -> anyhow::Result<serde_json::Value>;
}
pub enum LlmBackend { OpenAI, Anthropic, LocalNone }
```

**Tasks:**
- [ ] Create LlmProvider trait
- [ ] Implement OpenAI provider (reqwest + API)
- [ ] Implement Anthropic provider
- [ ] Schema validation with `schemars`
- [ ] Graceful fallback when unavailable
- [ ] Environment: `LLM_BACKEND`, `OPENAI_API_KEY`, `ANTHROPIC_API_KEY`

**Acceptance:** `/crawl` with `{"strategy":"llm","schema":{...}}` returns validated JSON

---

### 3) Complete Chunking Feature Set
**Goal:** Crawl4AI parity with 5 chunking methods

```rust
pub enum ChunkMethod {
    Sliding {token_max, overlap},
    Fixed {token_max},
    Sentence,
    Regex {pattern},
    Topic {window, smoothing}
}
```

**Tasks:**
- [ ] Implement sentence splitter (rule-based)
- [ ] Implement regex splitter
- [ ] Implement topic chunking (TextTiling algorithm)
- [ ] Add chunking config to CrawlOptions
- [ ] Golden tests for each method

**Acceptance:** Each method produces deterministic chunks within token budgets

---

### 4) Structured Extraction (CSS/XPath + LLM)
**Goal:** Schema-guided extraction with selector fallback

```json
{
  "strategy": "css_json",
  "schema": {
    "title": {"css": "h1", "attr":"text"},
    "price": {"css": ".price", "attr":"text", "post":"currency"},
    "images": {"css": "img", "attr":"src", "multi": true}
  },
  "llm_fallback": true
}
```

**Tasks:**
- [ ] CSS/XPath extractor implementation
- [ ] Schema parser and validator
- [ ] LLM fallback for missing fields
- [ ] JSON schema output validation
- [ ] Integration with extraction pipeline

**Acceptance:** CSS extraction works reliably; LLM fills gaps when enabled

---

### 5) Table Extraction
**Goal:** Extract HTML tables with optional LLM enhancement

**Tasks:**
- [ ] HTML table parser (colspan/rowspan handling)
- [ ] Markdown/CSV conversion
- [ ] LLM table reconstruction for broken markup
- [ ] Integration with extraction strategies
- [ ] Test fixtures with real tables

**Acceptance:** HTML tables → CSV/MD deterministically; LLM repairs broken tables

---

### 6) Query-Aware Deep Crawling
**Goal:** Information foraging with query-driven prioritization

```rust
// Frontier scoring formula
S = α * BM25(title+anchor, query) +
    β * URLSignals(depth/path/dup) +
    γ * IntraDomainDiversity +
    δ * TextSim(prev_chunks, new_chunks)
```

**Tasks:**
- [ ] BM25 scoring implementation
- [ ] Query-aware frontier management
- [ ] Content similarity scoring
- [ ] Early stopping based on query relevance
- [ ] Integration with existing Spider module

**Acceptance:** Query-driven crawls stop earlier on irrelevant branches

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

## 📚 References

- **Completed Work:** [`COMPLETED.md`](./COMPLETED.md)
- **Architecture:** [`docs/architecture/`](./architecture/)
- **API Documentation:** [`docs/api/`](./api/)
- **Performance Reports:** [`docs/benchmarks/`](./benchmarks/)

---

*Last updated: September 26, 2025*
*Status: Planning AI-parity implementation*