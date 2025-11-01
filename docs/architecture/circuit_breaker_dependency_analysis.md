# Circuit Breaker Dependency Analysis

## Current State: Circular Dependency

```
┌─────────────────────────────────────────────────────────────────┐
│                      CIRCULAR DEPENDENCY                         │
└─────────────────────────────────────────────────────────────────┘

riptide-extraction
    │
    └──> riptide-spider
            │
            └──> riptide-fetch
                    │
                    └──> riptide-reliability [default features]
                            │
                            └──> [events feature enabled by default]
                                    │
                                    └──> riptide-pool
                                            │
                                            └──> riptide-extraction [native-pool]
                                                    │
                                                    └──> ⚠️ CYCLE COMPLETES
```

### Dependency Chain Analysis

```
Step 1: riptide-extraction → riptide-spider
  Reason: Spider extracts links from HTML content

Step 2: riptide-spider → riptide-fetch
  Reason: Spider fetches pages via HTTP

Step 3: riptide-fetch → riptide-reliability
  Reason: Fetch uses CircuitBreaker for fault tolerance

Step 4: riptide-reliability → riptide-pool [via events feature]
  Reason: Events feature integrates with pool system

Step 5: riptide-pool → riptide-extraction [via native-pool feature]
  Reason: Pool manages native extraction workers

Step 6: riptide-extraction → CYCLE COMPLETES ❌
```

## Proposed Solution: Move to riptide-types

```
┌─────────────────────────────────────────────────────────────────┐
│                      NO CIRCULAR DEPENDENCY                      │
└─────────────────────────────────────────────────────────────────┘

                    riptide-types
                    (Foundation)
                         │
                         │ (CircuitBreaker lives here)
                         │
         ┌───────────────┼───────────────┐
         │               │               │
         ▼               ▼               ▼
  riptide-fetch   riptide-spider   riptide-reliability
         │               │               │
         │               │               └──> [events] ──> riptide-pool
         │               │                                      │
         │               └──────────────────────────────────────┤
         │                                                      │
         └──────────────────────────────────────────────────────┼──> riptide-extraction
                                                                │
                                                                ▼
                                                           ✅ NO CYCLE!
```

See CIRCUIT_BREAKER_REFACTORING_PLAN.md for detailed migration steps.
