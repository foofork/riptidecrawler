continue /workspaces/eventmesh/cli-and-plans.md via /workspaces/eventmesh/docs/riptide-cli-revised-implementation-plan.md

double check /workspaces/eventmesh/docs/riptide-features-audit.md

validation is complete when we can execute riptide commands and options on live websites that fit our diverse array of functionality.

ensure api is fully up to date, error free, and cicd passes in github.  

Deliver Foundations & Telemetry

Goal: a stable skeleton that measures itself.

Deliverables

Cargo workspace with crates wired:

riptide-core (fetch, HTTP, cache, queues)

riptide-intelligence (model/router interfaces; no models yet)

riptide-extraction (readability, JSON-LD, CSS/XPath, regex ladder)

riptide-dedup (SimHash/MinHash)

riptide-normalize (dates, currency, units)

riptide-eval (golden-set runner + reports)

Structured logging + metrics (per-stage spans; counters, timers)

Small golden set (100–300 URLs across page types) in repo for repeatable tests

DoD

cargo test runs unit/integration tests; riptide-eval produces a Markdown/CSV report

KPIs

Baseline eval runs in <10 min locally

Metrics emitted for: fetch, classify, extract, dedup, normalize


Side-items:
**Memory Safety (Miri):**
- Miri checks run on memory_manager tests
- Timeout: 5 minutes for CI efficiency
- Catches undefined behavior and memory issues


move out those non docs from docs/

run benches on urls as well as csv etc ...  consider minimal sqlite but keep agnostic- json storage locations and other storage locations (include placeholder for s3)
patch up the playground and run tests from there.
update the python sdk
