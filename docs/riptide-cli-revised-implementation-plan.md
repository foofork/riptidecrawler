# RipTide CLI Revised Implementation Plan - Production Grade

Based on the comprehensive CLI specification in `/workspaces/eventmesh/cli-and-plans.md`

## Executive Summary

Implement a production-grade CLI following the **Raw → WASM → Headless** progressive enhancement pattern with CSS/Regex/LLM extraction strategies. Focus on deterministic contracts, observability, and offline-friendly operation.

---

# Core Architecture Changes

## Engine Pipeline (NOT Wasm)
* **Raw:** Clean HTML, no JavaScript needed
* **WASM:** Unclean/static content with scripts
* **Headless:** SPA/JS-heavy sites requiring browser execution

## Strategy Types
* **CSS:** DOM selectors for structured content
* **Regex:** Pattern matching for data extraction
* **LLM:** Future AI-based extraction (placeholder)
* **Chaining:** `chain:css,regex` - sequential fallback
* **Parallel:** `parallel:all` - run all, pick best by confidence
* **Fallback:** `fallback:css` - explicit fallback strategy

---

# Implementation Phases - Aligned with Spec

## Phase 1: Core Infrastructure & WASM Fix (Day 1)
**Goal:** Fix critical blockers and establish engine pipeline

### 1.1 Fix WASM Module Path
```rust
// In config.rs or main.rs
pub struct WasmConfig {
    module_path: PathBuf,  // configurable via --wasm-path
    preload: bool,         // wasm.preload={on,off}
    memory_limit_mb: usize,
    init_timeout_ms: u64,
}

// Search order:
// 1. CLI flag: --wasm-path
// 2. Config: wasm.module_path
// 3. Env: RIPTIDE_WASM_PATH
// 4. Default: /opt/riptide/wasm/riptide_extractor_wasm.wasm
```

### 1.2 Implement Engine Gating
```rust
pub enum Engine {
    Auto,     // Smart selection based on content
    Raw,      // Direct HTML parsing
    Wasm,     // WASM-based extraction
    Headless, // Full browser rendering
}

impl Engine {
    pub async fn gate_decision(html: &str, url: &Url) -> Engine {
        // Check JS intensity, DOM depth, resource fetches
        if has_spa_markers(html) { return Engine::Headless; }
        if needs_cleaning(html) { return Engine::Wasm; }
        Engine::Raw
    }
}
```

### 1.3 Add Graceful Fallbacks
```bash
# Commands to implement:
riptide extract --url <url> --no-wasm         # Force skip WASM
riptide extract --url <url> --init-timeout-ms 8000  # Longer timeout
riptide validate --wasm                       # Check WASM setup
```

### Validation Checkpoint:
```bash
# These should work without hanging:
riptide extract --url https://example.com --engine raw
riptide extract --url https://example.com --engine wasm --timeout 5000
riptide system check --production
```

---

## Phase 2: Extract Command Implementation (Day 1-2)
**Goal:** Implement full extract command as per spec

### 2.1 Command Structure
```rust
// extract.rs - aligned with spec
pub struct ExtractCommand {
    // Input sources
    url: Option<String>,
    input_file: Option<PathBuf>,
    stdin: bool,

    // Engine selection
    engine: Engine,  // auto|raw|wasm|headless

    // Strategy configuration
    strategy: Strategy,  // auto|css|regex|llm|chain:css,regex|parallel:all

    // Schema support
    schema: Option<PathBuf>,
    schema_id: Option<String>,

    // Extraction options
    selector: Vec<String>,
    regex: Vec<String>,
    chunk: ChunkMode,

    // Output control
    show_confidence: bool,
    metadata: bool,
    confidence_threshold: f32,  // default 0.75

    // Headless options
    headless_wait: WaitCondition,
    headless_timeout: u64,
    proxy: Option<String>,
    stealth: StealthLevel,

    // Artifacts
    save_artifacts: Option<PathBuf>,

    // Output format
    output: OutputFormat,  // json|ndjson|table|yaml|md
}
```

### 2.2 Output Contract (JSON)
```json
{
  "url": "https://example.com",
  "engine": "wasm",
  "strategy": "chain:css,regex",
  "timestamp": "2025-10-15T09:12:14Z",
  "content": {
    "markdown": "...",
    "fields": {
      "title": "...",
      "author": "...",
      "date": "..."
    }
  },
  "confidence": 0.92,
  "metadata": {
    "http": {"status": 200, "content_type": "text/html", "bytes": 12456},
    "timings_ms": {"fetch": 120, "gate": 8, "extract": 240, "total": 410},
    "gate_decision": "wasm",
    "schema_id": "news/article@v1",
    "pii": {"emails": 0, "phones": 0}
  },
  "artifacts": {
    "html": "artifacts/abc.html",
    "dom": "artifacts/abc.dom.json"
  },
  "errors": []
}
```

### 2.3 Strategy Router Implementation
```rust
pub struct StrategyRouter {
    confidence_threshold: f32,
}

impl StrategyRouter {
    pub async fn execute(&self, config: StrategyConfig) -> ExtractionResult {
        match config {
            StrategyConfig::Auto => self.auto_strategy(),
            StrategyConfig::Chain(strategies) => self.chain_strategy(strategies),
            StrategyConfig::Parallel(strategies) => self.parallel_strategy(strategies),
            StrategyConfig::Fallback(primary, fallback) => self.fallback_strategy(primary, fallback),
        }
    }

    async fn auto_strategy(&self) -> ExtractionResult {
        // Check domain profile & schema cache
        // Default: css → regex → llm with confidence gating
    }
}
```

---

## Phase 3: Additional Core Commands (Day 2-3)
**Goal:** Implement render, crawl, pdf, table, search commands

### 3.1 Render Command
```bash
riptide render --url <url> \
  --wait {load,network-idle,selector:#ready} \
  --screenshot {none,viewport,full} \
  --pdf --html --dom --har \
  --cookie-jar <file> --storage-state <file> \
  --proxy <url> --stealth {off,low,med,high,auto}
```

### 3.2 Crawl Command
```bash
riptide crawl --url <url> \
  --depth <n> --max-pages <n> \
  --rate <req/s> --delay-ms <ms> \
  --robots {respect,ignore} \
  --allow <glob> --deny <glob> \
  --state {save,load} <file> --resume \
  --with-content --stream -o ndjson
```

### 3.3 PDF Command Suite
```bash
riptide pdf extract --file <doc.pdf> --tables --images --ocr --out out.json
riptide pdf to-md --file <doc.pdf> --out out.md
riptide pdf info --file <doc.pdf>
riptide pdf stream --file <doc.pdf> -o ndjson
```

### 3.4 Table Command
```bash
riptide table extract --url <url> --merge-cells --classify --out tables.json
riptide table to-csv --input tables.json --out out.csv
riptide table to-md --input tables.json --out out.md
```

### 3.5 Search Command
```bash
riptide search --query "rust web scraping" \
  --limit 10 --domain github.com \
  --include-content --follow-depth 1 \
  --max-pages 50 --strategy auto -o table
```

---

## Phase 4: Schema & Domain Intelligence (Day 3-4)
**Goal:** Implement schema learning and domain profiles

### 4.1 Schema Commands
```bash
riptide schema learn --url https://site/page --goal article --out schema.json
riptide schema test --schema schema.json --url https://site/page --report
riptide schema diff --old old.json --new new.json
riptide schema push --schema schema.json --schema-id site/article@v1
riptide schema list
riptide schema show <id>
riptide schema rm <id>
```

### 4.2 Domain Profiles
```bash
riptide domain init example.com
riptide domain profile set example.com \
  --stealth auto --wait network-idle \
  --engine auto --strategy auto \
  --rate 1.5 --robots respect \
  --schema-id news/article@v1 \
  --confidence-threshold 0.75
riptide domain drift --url https://example.com \
  --baseline baseline.dom.json --report
```

---

## Phase 5: System & Operations Commands (Day 4)
**Goal:** Implement monitoring, validation, and admin commands

### 5.1 Core Operations
```bash
# Job management
riptide job submit extract --url ... --strategy auto --stream
riptide job list
riptide job status <id>
riptide job logs <id> --follow

# Cache management
riptide cache status
riptide cache warm --url-file urls.txt
riptide cache clear --domain example.com

# Session management
riptide session new --name acme --cookie-jar jar.json
riptide session export --name acme --out state.json

# Metrics & monitoring
riptide metrics show
riptide metrics tail --interval 2s
riptide metrics export --prom --file metrics.prom
```

### 5.2 Validation & System
```bash
riptide validate --comprehensive  # Full preflight checks
riptide system check --production # CPU, WASM, Redis, headless
riptide system profile            # Performance baseline
```

---

## Phase 6: Testing & Benchmarking (Day 4-5)
**Goal:** Comprehensive testing against real-world URLs

### 6.1 Test Suite Implementation
```bash
#!/bin/bash
# tests/run-corpus.sh
set -euo pipefail
URLS=${1:-tests/corpus.txt}
OUT="artifacts/$(date +%Y%m%d_%H%M%S)"
mkdir -p "$OUT"

while read -r url; do
  riptide extract --url "$url" \
    --engine auto --strategy auto \
    --timeout 15000 \
    --save-artifacts "$OUT" \
    -o ndjson >> "$OUT/results.ndjson" || true
done < "$URLS"

riptide test report --dir "$OUT" --format markdown > "$OUT/report.md"
```

### 6.2 Performance Benchmarks
```bash
riptide bench urls --file tests/bench.txt \
  --iterations 5 \
  --targets "static<500,news<1000,complex<3000" \
  --export bench.json
```

### 6.3 Test Coverage Commands
```bash
riptide test suite --urls tests/corpus.txt --out report/ --stream
riptide test report --dir report/ --format markdown
riptide test coverage --strategies --urls tests/corpus.txt
```

---

## Phase 7: Production Validation (Day 5-6)
**Goal:** Ensure production readiness

### 7.1 Production Checklist
```bash
# Run comprehensive validation
riptide validate --comprehensive

# Check system requirements
riptide system check --production

# Run performance benchmarks
riptide bench urls --file tests/bench.txt --iterations 5 --export bench.json

# Test extraction success rates
riptide test suite --urls tests/corpus.txt --out report/
```

### 7.2 Success Criteria (from spec)
* Static content: >90% success rate
* News sites: >85% success rate
* E-commerce: >70% success rate
* SPA (with headless): >80% post-integration
* Overall errors: <1% (non-policy)

### 7.3 Performance Gates
* P95 latency: static <500ms, news <1s, complex <3s
* Memory ceiling: <100MB/extraction (avg), watchdog 2GB/job

---

# Critical Implementation Details

## Exit Codes (Must Match Spec)
```rust
pub enum ExitCode {
    Ok = 0,
    PartialSuccess = 2,
    ValidationError = 3,
    NetworkError = 4,
    ExtractionFailure = 5,
    HeadlessFailure = 6,
    PolicyViolation = 7,
    CacheError = 8,
    PluginError = 9,
    InternalError = 10,
}
```

## Global Options Implementation
```rust
pub struct GlobalOptions {
    output: OutputFormat,       // json|ndjson|table|yaml|md
    verbose: u8,                // -v, -vv, -vvv
    trace: bool,
    quiet: bool,
    no_color: bool,
    api_url: Option<String>,
    api_key: Option<String>,
    profile: Option<String>,
    config: Option<PathBuf>,
    timeout: Option<u64>,
    retries: Option<u32>,
    concurrency: Option<u32>,
    save_artifacts: Option<PathBuf>,
    pii_scan: bool,
    telemetry: bool,
}
```

## Artifacts Structure
```
artifacts/
├── <timestamp>/
│   ├── <url_hash>.html
│   ├── <url_hash>.dom.json
│   ├── <url_hash>.pdf
│   ├── <url_hash>.har
│   ├── <url_hash>.png
│   ├── run.log
│   └── trace.json
```

## Config Precedence
1. CLI flags (highest priority)
2. Environment variables
3. Profile settings
4. Config file (lowest priority)

---

# Daily Execution Plan (Revised)

## Day 1: Infrastructure & Core Extract
- Morning: Fix WASM path, implement engine gating
- Afternoon: Implement extract command with all flags

## Day 2: Additional Commands
- Morning: Implement render, crawl commands
- Afternoon: Implement pdf, table, search commands

## Day 3: Intelligence Layer
- Morning: Schema learning and testing
- Afternoon: Domain profiles and drift detection

## Day 4: Operations & Testing
- Morning: Job, cache, session, metrics commands
- Afternoon: Test suite implementation

## Day 5: Benchmarking & Validation
- Morning: Performance benchmarks
- Afternoon: Production validation

## Day 6: Polish & Documentation
- Morning: UX improvements (help, examples)
- Afternoon: Final testing and documentation

---

# Quick Reference - Key Commands

## Working Today (After Phase 1)
```bash
riptide extract --url <url> --engine raw --strategy css
riptide extract --input-file page.html --strategy regex -o json
riptide validate --wasm
riptide system check --production
```

## Target Commands (After Full Implementation)
```bash
# Adaptive extraction
riptide extract --url https://example.com --engine auto --strategy auto

# Headless rendering
riptide render --url https://spa.site --wait network-idle --screenshot full

# Crawling with extraction
riptide crawl --url https://docs.site --depth 3 --with-content --stream

# PDF processing
riptide pdf extract --file doc.pdf --tables --out tables.json
riptide pdf to-md --file doc.pdf --out doc.md

# Schema learning
riptide schema learn --url https://news.site/article --goal article --out schema.json

# Performance testing
riptide bench urls --file tests/bench.txt --iterations 5
```

---

This revised plan aligns with the comprehensive CLI specification, focusing on the Raw → WASM → Headless pipeline and removing all references to Wasm strategy.