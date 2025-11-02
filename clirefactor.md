# RipTide CLI Refactoring - Quick Reference

**⚠️ NOTE**: This is a quick reference. For the **complete, comprehensive refactoring plan**, see:

👉 **[/docs/CLI-REFACTORING-PLAN.md](/docs/CLI-REFACTORING-PLAN.md)** 👈

---

## Quick Summary

**Goal**: Refactor RipTide Rust CLI from a "fat" client (with embedded business logic) to a "thin" HTTP client.

### Current State
- **Location**: `/crates/riptide-cli/`
- **Problem**: 27 dependencies including business logic (extraction, browser, PDF, workers)
- **Architecture**: Can execute locally OR via API (dual-mode)

### Target State
- **Architecture**: Pure HTTP client (API-only)
- **Dependencies**: ~15 minimal deps (clap, reqwest, tokio, serde, colored, indicatif)
- **Size**: Binary < 15MB
- **Behavior**: Identical to Node.js CLI (`/cli/`)

---

## Key Design Principles

1. **No Business Logic**: CLI only makes HTTP requests
2. **Spec-Driven**: `/cli-spec/cli.yaml` is single source of truth
3. **Streaming-First**: NDJSON streaming for real-time results
4. **Exit Codes**: 0=success, 1=user/network, 2=server, 3=invalid args
5. **Output Separation**: stdout=data, stderr=progress/errors

---

## Commands → API Endpoints

| Command | API Endpoint | Method | Streaming |
|---------|-------------|--------|-----------|
| `riptide crawl <urls>` | `/crawl` | POST | `/crawl/stream` |
| `riptide spider <url>` | `/spider/crawl` | POST | No |
| `riptide search <query>` | `/deepsearch` | POST | `/deepsearch/stream` |
| `riptide doctor` | `/healthz` | GET | No |

---

## Project Structure (Target)

```
crates/riptide-cli/
├── Cargo.toml              # ~15 minimal dependencies
├── src/
│   ├── main.rs             # <150 lines - parse & dispatch
│   ├── client.rs           # HTTP client wrapper
│   ├── commands/
│   │   ├── crawl.rs        # POST /crawl
│   │   ├── spider.rs       # POST /spider/crawl
│   │   ├── search.rs       # POST /deepsearch
│   │   └── doctor.rs       # GET /healthz + diagnostics
│   ├── output/
│   │   ├── json.rs
│   │   ├── table.rs
│   │   └── text.rs
│   └── config.rs
└── tests/
    ├── integration/
    └── snapshots/
```

---

## Minimal Dependencies (15 total)

```toml
[dependencies]
# Core (7)
anyhow, clap, tokio, serde, serde_json, serde_yaml, reqwest

# CLI utilities (6)
colored, indicatif, comfy-table, dirs, ctrlc, env_logger

# Utilities (2)
url, chrono
```

**Removed** (21 dependencies):
- All `riptide-*` crates (extraction, browser, pdf, workers, cache, etc.)
- Business logic dependencies (spider_chrome, scraper, etc.)

---

## Configuration Precedence

```
CLI Flags > Environment Variables > Config File > Defaults
```

**Config File**: `~/.config/riptide/config.yaml`
```yaml
api:
  url: "http://localhost:8080"
  key: "your-api-key"

output:
  format: "text"  # json, table, text

crawl:
  concurrency: 5
  cache_mode: "auto"
```

---

## Implementation Timeline

| Week | Phase | Deliverables |
|------|-------|-------------|
| 1 | Foundation | CLI spec YAML, clean dependencies |
| 2 | Core Commands | crawl, spider, search, doctor |
| 3 | Output Formatting | json, table, text formatters |
| 4 | Config & Tests | Config file, 90%+ coverage |
| 5 | CI/CD | Automated builds, releases |

---

## Success Criteria

- ✅ All Node CLI commands have Rust equivalents
- ✅ Streaming works (NDJSON)
- ✅ Binary < 15MB
- ✅ Dependencies ≤ 15
- ✅ Tests pass on Linux/macOS/Windows
- ✅ 90%+ code coverage

---

## Migration Impact

**Before** (v0.9.x):
```bash
# CLI could run without API
riptide extract --url "https://example.com" --direct
```

**After** (v1.0):
```bash
# CLI requires API server
riptide crawl https://example.com

# If API not running → helpful error:
Error: Cannot connect to API server at http://localhost:8080

💡 Remediation:
  1. Start API: ./target/release/riptide-api
  2. Or set URL: export RIPTIDE_BASE_URL=http://production:8080
```

---

## Why This Refactoring?

### Problems Solved
1. **Maintainability**: Business logic in one place (API server)
2. **Complexity**: CLI becomes simple HTTP client
3. **Deployment**: Smaller binary, fewer dependencies
4. **Testing**: Mock API server for tests
5. **Consistency**: Same architecture as Node CLI

### Trade-offs
- **Requires API**: No offline mode (by design)
- **Network Dependency**: CLI fails if API unreachable
- **Migration Effort**: Users must run API server

---

## Final v1.0 Scope

**7 Commands** (after thorough analysis):
1. `extract` - Advanced extraction (PRIMARY - 914 lines in current CLI)
2. `spider` - Deep crawling
3. `search` - Web search
4. `render` - JS-heavy sites
5. `doctor` - Diagnostics
6. `config` - Configuration (critical gap)
7. `session` - Auth crawling

**Timeline**: 6 weeks | **Coverage**: 100% workflows | **Reduction**: 80% code

## Next Steps

1. **Read Full Plan**: [/docs/CLI-REFACTORING-PLAN.md](/docs/CLI-REFACTORING-PLAN.md)
2. **Extraction Analysis**: [/docs/CLI-EXTRACTION-STRATEGY-ANALYSIS.md](/docs/CLI-EXTRACTION-STRATEGY-ANALYSIS.md)
3. **Review CLI Spec**: Check `/cli-spec/cli.yaml` structure in full plan
4. **Start Phase 1**: Clean dependencies, create spec parser
5. **Weekly Reviews**: Track progress against timeline

---

## Questions?

- **Full Documentation**: `/docs/CLI-REFACTORING-PLAN.md`
- **API Endpoints**: `/docs/02-api-reference/ENDPOINT_CATALOG.md`
- **Node CLI Reference**: `/cli/` directory
- **Current Rust CLI**: `/crates/riptide-cli/`

---

**Last Updated**: 2025-01-15
**Document Owner**: RipTide Team
