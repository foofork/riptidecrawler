# Riptide Architecture Documentation

**Last Updated**: 2025-11-11
**Phase**: 1-2 Complete
**Status**: Ready for Implementation

---

## Quick Navigation

### 📋 Start Here

**[ARCHITECTURE_DELIVERABLES.md](./ARCHITECTURE_DELIVERABLES.md)** - Executive summary and index of all deliverables

### 📖 Architectural Specifications

1. **[port-trait-specifications.md](./port-trait-specifications.md)**
   - 12 port trait definitions (11 complete, 1 new)
   - Interface specifications for all abstractions
   - Implementation requirements
   - Usage examples

2. **[application-context-design.md](./application-context-design.md)**
   - Complete ApplicationContext structure (20 ports)
   - Facade factory pattern
   - Dependency injection strategies
   - Zero circular dependency architecture

3. **[migration-strategy.md](./migration-strategy.md)**
   - 8-phase migration plan (24-32 hours)
   - Rollback procedures per phase
   - Parallelization strategy
   - Success criteria and validation

---

## Architecture Overview

### Hexagonal Architecture (Ports & Adapters)

```
┌─────────────────────────────────────────────────────────┐
│                    riptide-api                           │
│          (Composition Root + HTTP Handlers)              │
│                                                          │
│  ApplicationContext: Wires all dependencies              │
│  FacadeFactory: Creates facades with injected ports      │
└──────────────────┬──────────────────────────────────────┘
                   │ injects
                   ↓
┌─────────────────────────────────────────────────────────┐
│                 riptide-facade                           │
│            (Application Use-Cases)                       │
│                                                          │
│  CrawlFacade, BrowserFacade, ScraperFacade, etc.        │
│  Only depends on port traits (no infrastructure)        │
└──────────────────┬──────────────────────────────────────┘
                   │ uses
                   ↓
┌─────────────────────────────────────────────────────────┐
│                 riptide-types                            │
│            (Domain + Port Traits)                        │
│                                                          │
│  Port Traits: CacheStorage, CircuitBreaker, Pool, etc.  │
│  Domain Types: ExtractedDoc, CrawlResult, etc.          │
└──────────────────┬──────────────────────────────────────┘
                   ↑ implements
                   │
┌───────────────────────────────────────────────────────────┐
│         Infrastructure Crates (Adapters)                  │
│                                                           │
│  riptide-cache       → RedisCache, InMemoryCache         │
│  riptide-persistence → PostgresRepository                │
│  riptide-browser     → BrowserPool                       │
│  riptide-reliability → AtomicCircuitBreaker              │
│  riptide-pool        → WasmInstancePool                  │
└───────────────────────────────────────────────────────────┘
```

### Key Principles

1. **Dependency Inversion**: Application depends on abstractions (ports), not concrete implementations
2. **Interface Segregation**: Each port has single, well-defined responsibility
3. **Testability**: All ports have in-memory implementations for fast testing
4. **Zero Circular Dependencies**: Strict layering enforced at compile-time
5. **Async-First**: All I/O operations are async for maximum concurrency

---

## Port Catalog

| Port Name | Status | File | Description |
|-----------|--------|------|-------------|
| CacheStorage | ✅ | cache.rs | Backend-agnostic cache abstraction |
| CircuitBreaker | 🔨 | reliability.rs | Fault tolerance state machine |
| HealthCheck | ✅ | health.rs | Component health monitoring |
| ResourcePool | ✅ | pool.rs | Generic resource pooling (WASM, Browser, LLM) |
| MetricsRegistry | ✅ | metrics.rs | Low-level + business metrics |
| Repository | ✅ | repository.rs | Domain entity persistence (CRUD) |
| EventBus | ✅ | events.rs | Domain event pub/sub |
| IdempotencyStore | ✅ | idempotency.rs | Duplicate request prevention |
| SessionStorage | ✅ | session.rs | Session management |
| StreamingTransport | ✅ | streaming.rs | Real-time SSE/WebSocket |
| RateLimiter | ✅ | rate_limit.rs | Per-host + global rate limiting |
| HttpClient | ✅ | http.rs | HTTP client abstraction |

**Legend**: ✅ Complete | 🔨 To Implement

---

## ApplicationContext Ports

The ApplicationContext wires 20 port implementations:

### Core Infrastructure (2)
- `clock: Arc<dyn Clock>`
- `entropy: Arc<dyn Entropy>`

### Persistence Layer (2)
- `transaction_manager: Arc<dyn TransactionManager>`
- `repository_factory: Arc<dyn RepositoryFactory>`

### Event System (1)
- `event_bus: Arc<dyn EventBus>`

### Caching & Storage (3)
- `cache: Arc<dyn CacheStorage>`
- `idempotency_store: Arc<dyn IdempotencyStore>`
- `session_storage: Arc<dyn SessionStorage>`

### Resource Management (3)
- `wasm_pool: Arc<dyn Pool<WasmInstance>>`
- `browser_pool: Arc<dyn Pool<BrowserSession>>`
- `llm_pool: Arc<dyn Pool<LlmClient>>`

### Reliability & Resilience (3)
- `headless_circuit_breaker: Arc<dyn CircuitBreaker>`
- `llm_circuit_breaker: Arc<dyn CircuitBreaker>`
- `rate_limiter: Arc<dyn RateLimiter>`

### HTTP Clients (2)
- `http_client: Arc<dyn HttpClient>`
- `headless_http_client: Arc<dyn HttpClient>`

### Observability (3)
- `health_registry: Arc<dyn HealthRegistry>`
- `metrics_collector: Arc<dyn MetricsCollector>`
- `business_metrics: Arc<dyn BusinessMetrics>`

### Streaming (2)
- `sse_transport: Arc<dyn StreamingTransport>`
- `websocket_transport: Arc<dyn StreamingTransport>`

---

## Migration Timeline

### 8-Phase Plan (18-24 hours calendar time)

```
Phase 2.1: CircuitBreaker Port (2-3h) [CRITICAL PATH]
    ↓
Phase 2.2: Adapter Implementations (4-6h) [3 parallel tasks]
    ↓
Phase 2.3: ApplicationContext Update (2-3h) [CRITICAL PATH]
    ↓
Phase 2.4: Facade Factory Pattern (3-4h) [CRITICAL PATH]
    ↓
Phase 2.5: Facade Refactoring (6-8h) [7 parallel tasks]
    ↓
Phase 2.6: Handler Updates (4-5h) [Parallelizable]
    ↓
Phase 2.7: Cleanup Dependencies (2-3h)
    ↓
Phase 2.8: Validation (2-3h)
```

**Critical Path**: 2.1 → 2.3 → 2.4

**Parallelization**:
- Phase 2.2: 3 adapter implementations
- Phase 2.5: 7 facade refactorings
- Phase 2.6: Per-handler updates

---

## Implementation Checklist

### Phase 2.1: CircuitBreaker Port ⏳
- [ ] Create `riptide-types/src/ports/reliability.rs`
- [ ] Define `CircuitBreaker` trait
- [ ] Define state types (State, Permit, Metrics, Config)
- [ ] Export from `ports/mod.rs`
- [ ] Tests pass, clippy clean

### Phase 2.2: Adapter Implementations
- [ ] AtomicCircuitBreaker (riptide-utils)
- [ ] StateBasedCircuitBreaker (riptide-reliability)
- [ ] InMemoryCircuitBreaker (riptide-api/stubs)

### Phase 2.3: ApplicationContext Update
- [ ] Add new port fields
- [ ] Update `new()` for production
- [ ] Update `for_testing()` with stubs
- [ ] Update builder

### Phase 2.4: Facade Factory
- [ ] Create `facade_factory.rs`
- [ ] Define `FacadeFactory` trait
- [ ] Implement `DefaultFacadeFactory`
- [ ] Add 7 factory methods

### Phase 2.5: Facade Refactoring
- [ ] CrawlFacade
- [ ] BrowserFacade
- [ ] ScraperFacade
- [ ] PipelineFacade
- [ ] SpiderFacade
- [ ] LlmFacade (feature-gated)
- [ ] SearchFacade (feature-gated)

### Phase 2.6: Handler Updates
- [ ] Update AppState to use ApplicationContext
- [ ] Update handlers to use factory
- [ ] Update main.rs server init

### Phase 2.7: Cleanup
- [ ] Remove infrastructure imports from facades
- [ ] Update Cargo.toml dependencies
- [ ] Verify zero circular dependencies

### Phase 2.8: Validation
- [ ] All tests pass
- [ ] Clippy clean
- [ ] Documentation updated
- [ ] Quality gates pass
- [ ] Performance benchmarks pass

---

## Testing Strategy

### Test Pyramid

```
          /\
         /  \  E2E Tests (Integration)
        /────\
       /      \  Integration Tests (Facades + Real Adapters)
      /────────\
     /          \  Unit Tests (Ports + Mock Adapters)
    /────────────\
```

### Testing Per Phase

- **Phase 2.1-2.2**: Unit tests for ports and adapters
- **Phase 2.3**: Unit tests for ApplicationContext wiring
- **Phase 2.4**: Unit tests for facade factory
- **Phase 2.5**: Unit + Integration tests for facades
- **Phase 2.6**: Integration + E2E tests for handlers
- **Phase 2.7-2.8**: Full system validation

---

## Rollback Strategy

Each phase has independent rollback procedure:

1. **Identify failing phase** from git log
2. **Create rollback branch** (`rollback-phase-2.X`)
3. **Revert commits** for that phase
4. **Run rollback tests** (checklist in migration-strategy.md)
5. **Deploy rollback** to production

**Key**: Each phase maintains backward compatibility until completion.

---

## Success Criteria

- ✅ Zero circular dependencies (verified with `cargo tree`)
- ✅ All facades use port traits only (no infrastructure imports)
- ✅ ApplicationContext wires all dependencies at composition root
- ✅ Facade factory pattern implemented
- ✅ All tests pass (unit, integration, E2E)
- ✅ No performance regression (benchmarks pass)
- ✅ Documentation complete (ADR + updated docs)
- ✅ Production deployment successful

---

## Memory Storage

All architectural designs stored in memory:

**Namespace**: `riptide-migration`

**Keys**:
- `architecture/port-traits` - Port catalog and specifications
- `architecture/application-context` - ApplicationContext structure
- `architecture/migration-strategy` - Migration plan and rollback

**Retrieve**:
```bash
npx claude-flow@alpha hooks memory-retrieve --key "architecture/*"
```

---

## Related Documents

- [CLAUDE.md](../../CLAUDE.md) - Development guidelines
- [docs/roadmap/](../roadmap/) - Project roadmap
- [tests/validation-reports/](../../tests/validation-reports/) - Test reports

---

## Next Steps

**Immediate**: Begin Phase 2.1 - Implement CircuitBreaker Port

**Assignee**: Backend Developer or Implementation Agent

**Reference**: See [migration-strategy.md](./migration-strategy.md), Section "Phase 2.1"

**Estimated Time**: 2-3 hours

**Priority**: CRITICAL PATH (blocks facade refactoring)

---

**Questions?** Refer to architectural specifications or memory keys.
