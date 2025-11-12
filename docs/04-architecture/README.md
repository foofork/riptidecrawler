# Architecture Documentation

> **🎯 NEW: Comprehensive Hexagonal Architecture Guide**
>
> See [HEXAGONAL_ARCHITECTURE.md](./HEXAGONAL_ARCHITECTURE.md) for a deep-dive into RipTide's hexagonal architecture implementation, including all 30+ port traits, dependency flow, testing strategies, and architectural patterns. This is the definitive guide to understanding the system's architecture.

System design, architecture decisions, and technical specifications for RipTide.

## 📚 Core Architecture Documents

### Overview
- **[HEXAGONAL_ARCHITECTURE.md](./HEXAGONAL_ARCHITECTURE.md)** - Deep-dive into hexagonal architecture and ports & adapters pattern (⏱️ 2 hours) **← NEW! Comprehensive guide**
- **[ARCHITECTURE.md](./ARCHITECTURE.md)** - Complete system architecture (⏱️ 30 min)
- **[DESIGN.md](./DESIGN.md)** - Design philosophy and patterns (⏱️ 20 min)

### Components
- **[System Overview](./components/system-overview.md)** - High-level component overview
- **[System Design](./components/SYSTEM_DESIGN.md)** - Detailed system design
- **[System Diagram](./components/system-diagram.md)** - Visual architecture diagrams
- **[Integration Crosswalk](./components/integration-crosswalk.md)** - Flow-to-implementation mapping

## 🏗️ Component Architecture

### Core Components
- **[WASM Integration](./components/WASM_INTEGRATION_GUIDE.md)** - WebAssembly component design
- **[Telemetry Implementation](./components/TELEMETRY_IMPLEMENTATION.md)** - Observability architecture
- **[Resource Manager](./components/RESOURCE_MANAGER_REFACTORING.md)** - Resource management design
- **[Reliability Guide](./components/RELIABILITY_USAGE_GUIDE.md)** - System reliability patterns

### Specialized Components
- **[CDP Multiplexing](./components/P1-B4-cdp-multiplexing-design.md)** - Chrome DevTools Protocol design
- **[Streaming Pipeline](./components/streaming-pipeline-integration-design.md)** - Real-time streaming architecture
- **[Streaming Dataflow](./components/streaming-integration-dataflow.md)** - Data flow diagrams
- **[Facade Patterns](./components/facade-composition-patterns.md)** - API composition strategies

## 🎯 Critical Path Architecture

- **[Hive Critical Path](./components/hive-critical-path-architecture.md)** - Performance-critical paths
- **[Metrics Implementation](./components/metrics-implementation-summary.md)** - Metrics collection design

## 📋 Architecture Decision Records (ADRs)

### Decision Records
- **[ADR-006: Spider-Chrome Compatibility](./components/ADR-006-spider-chrome-compatibility.md)** - Browser integration decisions

### Implementation Plans
- **[P1-A4 Implementation Strategy](./components/P1-A4-implementation-strategy.md)** - Phase 1 implementation
- **[P2-F1 Core Elimination](./components/P2-F1-RIPTIDE-CORE-ELIMINATION-GUIDE.md)** - Core refactoring plan
- **[P2-F1 Completion Report](./components/P2-F1-COMPLETION-REPORT.md)** - Implementation results

## 🔄 Migration & Refactoring

### Configuration
- **[Day 2 Config Migration](./components/DAY2-CONFIG-MIGRATION.md)** - Configuration system evolution
- **[Refactoring Handoff](./components/REFACTORING_HANDOFF.md)** - Refactoring documentation

## 🗄️ Persistence & Integration

- **[Persistence Integration](./persistence-integration/)** - Data persistence design

## 🎨 Design Patterns

- **[Design Documents](./design/)** - Detailed design specifications

## 📊 Architecture Diagrams

```
┌─────────────────────────────────────────────────────────┐
│                    API Layer                            │
│  (REST endpoints, streaming, WebSocket)                 │
└─────────────────────┬───────────────────────────────────┘
                      │
┌─────────────────────┴───────────────────────────────────┐
│              Service Orchestration                      │
│  (Session mgmt, job queue, resource coordination)       │
└─────────────────────┬───────────────────────────────────┘
                      │
        ┌─────────────┼─────────────┐
        │             │             │
┌───────▼──────┐ ┌───▼────────┐ ┌─▼──────────┐
│ WASM Extract │ │ Spider     │ │ Headless   │
│ (Fast path)  │ │ (Crawling) │ │ (JS heavy) │
└──────────────┘ └────────────┘ └────────────┘
        │             │             │
        └─────────────┼─────────────┘
                      │
        ┌─────────────┴─────────────┐
        │                           │
┌───────▼──────┐         ┌──────────▼────────┐
│ Redis Cache  │         │ Telemetry/Metrics │
└──────────────┘         └───────────────────┘
```

## 🎓 Learning Path

**Beginner** (2 hours):
1. [System Overview](./components/system-overview.md)
2. [HEXAGONAL_ARCHITECTURE.md](./HEXAGONAL_ARCHITECTURE.md) - **Start here for architecture principles**
3. [ARCHITECTURE.md](./ARCHITECTURE.md)
4. [Integration Crosswalk](./components/integration-crosswalk.md)

**Intermediate** (4 hours):
1. [HEXAGONAL_ARCHITECTURE.md](./HEXAGONAL_ARCHITECTURE.md) - Deep dive
2. [DESIGN.md](./DESIGN.md)
3. [System Design](./components/SYSTEM_DESIGN.md)
4. [WASM Integration](./components/WASM_INTEGRATION_GUIDE.md)
5. [Streaming Pipeline](./components/streaming-pipeline-integration-design.md)

**Advanced** (Full day):
1. Component-specific documentation
2. ADRs and decision records
3. Implementation strategies
4. Performance optimization paths

## 🔗 Related Documentation

- **[API Reference](../02-api-reference/README.md)** - API implementation details
- **[Development Guide](../05-development/README.md)** - Developer documentation
- **[Advanced Topics](../07-advanced/README.md)** - Performance and optimization
- **[Deployment](../06-deployment/README.md)** - Production architecture

## 📖 Quick Reference

| Topic | Document | Time |
|-------|----------|------|
| **Hexagonal Architecture** | [HEXAGONAL_ARCHITECTURE.md](./HEXAGONAL_ARCHITECTURE.md) | 2 hours |
| **System Overview** | [system-overview.md](./components/system-overview.md) | 15 min |
| **Complete Architecture** | [ARCHITECTURE.md](./ARCHITECTURE.md) | 30 min |
| **Design Patterns** | [DESIGN.md](./DESIGN.md) | 20 min |
| **WASM Integration** | [WASM_INTEGRATION_GUIDE.md](./components/WASM_INTEGRATION_GUIDE.md) | 25 min |
| **Streaming Architecture** | [streaming-pipeline-integration-design.md](./components/streaming-pipeline-integration-design.md) | 20 min |

---

**Ready to build?** → [Development Guide](../05-development/README.md)
