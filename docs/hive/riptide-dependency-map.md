# RipTide Crate Dependency Map

**Last Updated**: 2025-10-19
**Total Crates**: 27

---

## Visual Dependency Hierarchy

```
Layer 0: Foundation
════════════════════════════════════════════════════════════════
┌────────────────────────────────────────────────────────────┐
│                      riptide-types                         │
│  (Shared types, no dependencies)                           │
└────────────────────────────────────────────────────────────┘


Layer 1: Infrastructure Base
════════════════════════════════════════════════════════════════
┌─────────────────┐  ┌─────────────────┐  ┌─────────────────┐
│ riptide-config  │  │riptide-security │  │riptide-monitoring│
└────────┬────────┘  └────────┬────────┘  └────────┬────────┘
         │                    │                     │
         └────────────────────┴─────────────────────┘
                              │
                   ┌──────────▼──────────┐
                   │   riptide-events    │
                   └──────────┬──────────┘


Layer 2: Resource Management
════════════════════════════════════════════════════════════════
         ┌──────────▼──────────┐    ┌──────────────────┐
         │   riptide-pool      │    │  riptide-cache   │
         │  (instance pooling) │    │  (multi-level)   │
         └──────────┬──────────┘    └────────┬─────────┘
                    │                        │
                    └───────────┬────────────┘


Layer 3: Content Processing
════════════════════════════════════════════════════════════════
┌─────────────┐  ┌─────────────┐  ┌─────────────┐  ┌─────────────┐
│riptide-fetch│  │riptide-spider│ │riptide-stealth│ │riptide-search│
└──────┬──────┘  └──────┬──────┘  └──────┬──────┘  └──────┬──────┘
       │                │                │                │
       └────────────────┴────────────────┴────────────────┘
                              │
              ┌───────────────▼───────────────┐
              │    riptide-extraction         │
              │  (HTML, CSS, Regex, Tables)   │
              └───────────────┬───────────────┘
                              │
              ┌───────────────▼───────────────┐
              │       riptide-pdf             │
              │   (PDF text extraction)       │
              └───────────────────────────────┘


Layer 4: Browser Abstraction
════════════════════════════════════════════════════════════════
              ┌────────────────────────────────┐
              │ riptide-browser-abstraction    │
              │  (Browser trait definitions)   │
              └───────────────┬────────────────┘
                              │
              ┌───────────────▼────────────────┐
              │      riptide-engine            │
              │  (CDP connection multiplexing) │
              └───────────────┬────────────────┘
                              │
              ┌───────────────▼────────────────┐
              │  riptide-headless-hybrid       │
              │  (Hybrid launcher strategy)    │
              └────────────────────────────────┘


Layer 5: Orchestration Hub ⭐
════════════════════════════════════════════════════════════════
              ┌────────────────────────────────┐
              │       riptide-core             │
              │  RELIABILITY & ORCHESTRATION   │
              │                                │
              │  • Circuit Breakers            │
              │  • ReliableExtractor           │
              │  • Graceful Degradation        │
              │  • Component Composition       │
              │  • Backward Compatibility      │
              └───────────────┬────────────────┘
                              │
              ┌───────────────┴────────────────┐
              │                                │
   ┌──────────▼──────────┐        ┌───────────▼───────────┐
   │  riptide-headless   │        │ riptide-intelligence  │
   │ (Browser orchestra) │        │  (LLM abstraction)    │
   └──────────┬──────────┘        └───────────┬───────────┘


Layer 6: Unified Interface
════════════════════════════════════════════════════════════════
              ┌────────────────────────────────┐
              │      riptide-facade            │
              │  (Unified API for all crates)  │
              └───────────────┬────────────────┘


Layer 7: Application
════════════════════════════════════════════════════════════════
    ┌─────────────────┴──────────────────┐
    │                                    │
┌───▼───────┐  ┌─────────────┐  ┌───────▼────────┐
│riptide-api│  │ riptide-cli │  │ riptide-workers│
│ (HTTP API)│  │ (CLI tool)  │  │ (Background)   │
└───────────┘  └─────────────┘  └────────────────┘


Supporting Crates (Cross-Layer)
════════════════════════════════════════════════════════════════
┌─────────────────┐  ┌──────────────────┐  ┌────────────────┐
│riptide-persistence│ │riptide-streaming │ │riptide-performance│
└─────────────────┘  └──────────────────┘  └────────────────┘

┌─────────────────┐
│riptide-test-utils│
└─────────────────┘
```

---

## Dependency Counts by Crate

### Zero Dependencies (Foundation)
- **riptide-types** (0 internal deps)

### Single Dependency Layer
- **riptide-config** → types
- **riptide-security** → types
- **riptide-monitoring** → types
- **riptide-browser-abstraction** → types

### Light Dependency (2-3 crates)
- **riptide-events** → types, monitoring
- **riptide-pool** → types, events
- **riptide-cache** → types, pool, events
- **riptide-fetch** → types, config
- **riptide-spider** → types, fetch
- **riptide-extraction** → types, spider
- **riptide-stealth** → types
- **riptide-search** → types

### Medium Dependency (4-6 crates)
- **riptide-engine** → types, config, browser-abstraction, stealth
- **riptide-pdf** → types, extraction
- **riptide-headless-hybrid** → types, config, engine

### Heavy Dependency (Orchestration Layer)
- **riptide-core** → 13 internal crates ⭐
- **riptide-headless** → core, engine, stealth
- **riptide-intelligence** → core, events, types
- **riptide-facade** → ALL specialized crates
- **riptide-api** → core, facade, engine, + many

---

## Dependency Flow Patterns

### 1. Foundation Pattern
```
riptide-types (shared by all)
```
**Purpose**: Common types, zero dependencies

### 2. Infrastructure Pattern
```
types → config/security/monitoring → events → pool/cache
```
**Purpose**: Build up infrastructure capabilities layer by layer

### 3. Content Processing Pattern
```
types → fetch → spider → extraction → pdf
```
**Purpose**: Progressive content acquisition and processing

### 4. Browser Pattern
```
types → browser-abstraction → engine → headless-hybrid
```
**Purpose**: Progressive browser automation capabilities

### 5. Orchestration Pattern
```
ALL specialized crates → riptide-core → higher-level crates
```
**Purpose**: Aggregate capabilities with reliability patterns

---

## Key Observations

### 1. riptide-core is the Integration Hub
- **13 dependencies** (most of any crate)
- **3 dependents** (api, headless, intelligence)
- **Position**: Between infrastructure and application layers
- **Role**: Orchestration with reliability patterns

### 2. riptide-types is the Foundation
- **0 dependencies**
- **27 dependents** (all crates)
- **Position**: Bottom of dependency tree
- **Role**: Shared type definitions

### 3. riptide-facade is the Unification Layer
- **Dependencies**: Nearly all specialized crates
- **Dependents**: api, cli
- **Position**: Just below application layer
- **Role**: Unified developer interface

### 4. Layered Architecture Emerges Naturally
```
Application (api, cli, workers)
     ↑
Unified Interface (facade)
     ↑
Orchestration (core ⭐, headless, intelligence)
     ↑
Browser Layer (browser-abstraction, engine, hybrid)
     ↑
Content Processing (fetch, spider, extraction, pdf, stealth, search)
     ↑
Resource Management (pool, cache)
     ↑
Infrastructure (events, monitoring, security)
     ↑
Configuration (config)
     ↑
Foundation (types)
```

---

## Circular Dependencies (None!)

### Previously Resolved
- riptide-core ↔ riptide-headless (resolved by feature gating)
- riptide-core ↔ riptide-intelligence (resolved by trait abstraction)
- riptide-extraction ↔ riptide-core (resolved by moving to riptide-types)

### Current State
✅ **No circular dependencies detected**

All dependencies flow in one direction (bottom-up in layer diagram).

---

## Recommendations

### 1. For New Crates
**Ask**: Which layer does this belong to?
- Foundation → extend riptide-types
- Infrastructure → depend on types, config
- Content → depend on fetch/extraction
- Browser → depend on engine
- Orchestration → depend on core
- Application → depend on facade

### 2. For New Features
**Ask**: Does this need orchestration or is it specialized?
- Specialized → add to appropriate specialized crate
- Orchestration → add to riptide-core
- Cross-cutting → consider new infrastructure crate

### 3. For Refactoring
**Principle**: Dependencies should flow upward through layers
- Types → Infrastructure → Content → Browser → Orchestration → Application
- Never skip layers (except for types, which is universal)

---

## Dependency Metrics

```
Total Crates:                27
Average Dependencies:        ~4.5
Most Dependencies:           riptide-core (13) ⭐
Fewest Dependencies:         riptide-types (0)
Most Depended Upon:          riptide-types (27)
Least Depended Upon:         Application crates (0)

Dependency Density:          Moderate (well-layered)
Circular Dependencies:       0 ✅
Orphaned Crates:            0 ✅
```

---

**Conclusion**: The riptide architecture exhibits excellent layering with riptide-core serving as the natural orchestration hub that provides reliability patterns for higher-level crates.
