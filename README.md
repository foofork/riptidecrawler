# RipTide — Work in Progress

> **⚠️ ARCHIVE NOTICE**
>
> **This repository is now archived and serves as a prototype/reference implementation.**
>
> This open-source version of RipTide was instrumental in validating our core concepts for high-performance web crawling and extraction in Rust. We're grateful to everyone who explored, tested, or contributed feedback during this phase.
>
> **Active development has moved to a private repository** where we're building the production version with enhanced features, performance optimizations, and enterprise-grade reliability.
>
> **What this means:**
> - ✅ This code remains available for reference and learning
> - ✅ The prototype demonstrates our architectural approach and design patterns
> - ❌ No new features or updates will be added to this repository
> - ❌ Issues and pull requests will not be actively monitored
>
> Thank you for your interest in RipTide. This prototype helped us prove the viability of our approach and informed the direction of our production system.

---

High-performance web crawling & extraction in Rust. Built for speed, reliability, and clean layering.

## Status

**Not production-ready.** This is a prototype implementation. API v1.0 and a major refactor were in progress at the time of archival (thin handlers, ports/adapters, infra consolidation).   

## What it does

* Fast HTML/PDF extraction (native Rust, optional WASM sandbox)
* Smart crawling (frontier management, headless fallback)
* Real-time streaming (NDJSON/SSE/WebSocket)

## ⚡ Quick Test (No Auth, No Setup)

**Want to try RipTide instantly?** Use Docker test mode with zero configuration:

```bash
# Start test environment (no authentication required)
docker compose -f docker-compose.test.yml up -d

# Test extraction:
curl -X POST http://localhost:8080/api/extract \
  -H "Content-Type: application/json" \
  -d '{"url": "https://example.com"}'

# Or run automated tests:
./scripts/quick-test.sh
```

**🔥 That's it!** See [docs/TESTING_GUIDE.md](docs/TESTING_GUIDE.md) for more testing options.

---

## Quick start (local dev)

```bash
git clone <repo>
cp .env.test .env      # Pre-configured for testing (no auth)
# Minimal mode (no Redis)
docker compose -f docker-compose.test.yml up -d
# or with Cargo:
cargo run --release
curl http://localhost:8080/health
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
