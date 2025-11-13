# RipTide — Work in Progress

High-performance web crawling & extraction in Rust. Built for speed, reliability, and clean layering.

## Status

**Not production-ready.** API v1.0 and a major refactor are in progress (thin handlers, ports/adapters, infra consolidation).   

## What it does

* Fast HTML/PDF extraction (native Rust, optional WASM sandbox)
* Smart crawling (frontier management, headless fallback)
* Real-time streaming (NDJSON/SSE/WebSocket)
* Multi-LLM provider routing and failover

## Quick start (local)

```bash
git clone <repo>
cp .env.example .env   # add SERPER_API_KEY if you use search
# Minimal mode (no Redis)
docker compose -f docker-compose.minimal.yml up -d
# or
cargo run --release -- --config config/deployment/minimal.toml
curl http://localhost:8080/healthz
```

## Deployment modes

* **Minimal**: single process, in-memory cache — best for dev/CI.
* **Enhanced**: add Redis for persistence & sessions.
* **Distributed**: API + workers + queue + optional Chrome pool for scale.

## Roadmap (why things may move)

* **Phase 0: Cleanup** — dedupe robots/Redis/rate limiting; define `CacheStorage`. 
* **Phase 1: Ports & Adapters** — traits for infra, adapters, DI `ApplicationContext`. 
* **Phase 3: Handlers → Facades** — handlers <50 LOC; business logic in facades. 
* **Phase 4: Infra consolidation** — one HTTP client with circuit breakers; streaming via ports. 

## Rough edges to expect

* Breaking API changes during v1.0 work
* Some crates still contain legacy wiring pending migration (see phases above)

## Where to look next

* `docs/00-getting-started/README.md` for a 5-minute setup
* `config/deployment/` for Minimal / Enhanced / Distributed configs
* `crates/riptide-facade/` to see the new application layer take shape (facade = use-cases). 

---

Questions or blockers? Open an issue with your mode (Minimal/Enhanced/Distributed) and your config snippet.
