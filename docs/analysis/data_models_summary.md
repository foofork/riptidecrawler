# Data Models Analysis Summary

**Analyst**: Data Models Specialist
**Date**: 2025-11-03
**Codebase**: EventMesh (RipTide)

## Executive Summary

Comprehensive analysis of EventMesh data structures reveals:
- **200+ struct definitions** across 30+ crates
- **60+ enum types** for type-safe state machines
- **3 primary storage mechanisms**: Redis, filesystem, in-memory
- **Event-driven architecture** with 5 major event types
- **High schema coupling** in event/extraction domains

## Storage Architecture

### 1. Redis/DragonflyDB (Primary Distributed Storage)
**Implementation**: `riptide-cache`, `riptide-persistence`

**Key Structures**:
- `CacheManager` - HTTP caching with Redis backend
- `PersistentCacheManager` - Long-term cache with spillover
- `DistributedCache` - Multi-node coordination
- `StateManager` - Session state persistence

**Configuration**:
```rust
RedisConfig {
    url: "redis://localhost:6379",
    pool_size: 10,
    ttl_default: 24 * 60 * 60,  // 24 hours
    compression: Lz4,
    eviction_policy: LRU,
}
```

**Key Patterns**:
```
riptide:v1:cache:{url_hash}     -> Cached HTTP responses
riptide:v1:session:{session_id} -> Session state
riptide:v1:wasm:{module_hash}   -> WASM module cache
riptide:v1:tenant:{tenant_id}   -> Tenant data
```

### 2. Filesystem Storage
**Use Cases**:
- Artifacts storage (extracted content)
- Cache spillover (when Redis full)
- State checkpoints (crawl resumption)
- WASM module cache (AOT compilation)

**Structures**:
- `FileStorage` - File-based persistence
- `DatabaseStorage` - SQLite integration
- `StorageBackend` - Trait abstraction

### 3. In-Memory Storage
**Use Cases**:
- WASM instance pooling (`StratifiedInstancePool`)
- URL frontier queues (`FrontierManager`)
- Event routing (`EventBus`)
- Real-time metrics (`MetricsCollector`)

## Core Domain Models

### Extraction Domain (`riptide-extraction`)
| Type | Purpose | Storage | Coupling |
|------|---------|---------|----------|
| `ExtractedDoc` | Extracted content | Redis | **High** |
| `ExtractionRequest` | Extraction task | In-memory | Medium |
| `Chunk` | Content chunk | In-memory | Low |
| `ChunkingConfig` | Chunking settings | Config | Low |

**Schema Coupling**: HIGH - tightly coupled to extraction pipeline

### Spider Domain (`riptide-spider`)
| Type | Purpose | Storage | Coupling |
|------|---------|---------|----------|
| `Spider` | Main coordinator | In-memory | Medium |
| `CrawlRequest` | Crawl task | In-memory | **High** |
| `CrawlResult` | Crawl output | Redis | **High** |
| `FrontierManager` | URL queue | In-memory | Medium |

**Schema Coupling**: HIGH - core to crawling logic

### Events Domain (`riptide-events`)
| Type | Purpose | Storage | Coupling |
|------|---------|---------|----------|
| `EventBus` | Event distribution | In-memory | Low |
| `PoolEvent` | Pool operations | In-memory | **High** |
| `ExtractionEvent` | Extraction ops | In-memory | **High** |
| `CrawlEvent` | Crawl ops | In-memory | **High** |
| `HealthEvent` | Health checks | In-memory | Medium |
| `MetricsEvent` | Performance | In-memory | Medium |

**Schema Coupling**: HIGH - domain-specific event schemas

### Pool Domain (`riptide-pool`)
| Type | Purpose | Storage | Coupling |
|------|---------|---------|----------|
| `PooledInstance` | WASM wrapper | In-memory | Medium |
| `CircuitBreakerState` | Error handling | In-memory | Low |
| `MemoryStats` | Memory tracking | In-memory | Low |

**Schema Coupling**: MEDIUM - some WASM-specific logic

### API Domain (`riptide-api`)
| Type | Purpose | Storage | Coupling |
|------|---------|---------|----------|
| `CrawlBody` | API request | In-memory | **High** |
| `CrawlResponse` | API response | In-memory | **High** |
| `HealthResponse` | Health check | In-memory | Medium |
| `DeepSearchBody` | Search request | In-memory | **High** |

**Schema Coupling**: HIGH - public API contract

## Event System Architecture

### Event Types
```rust
// Pool operations
PoolEvent {
    InstanceCreated, InstanceAcquired, InstanceReleased,
    InstanceFailed, InstanceUnhealthy, PoolExhausted
}

// Extraction operations
ExtractionEvent {
    Started, Completed, Failed, Timeout, FallbackUsed
}

// Crawl operations
CrawlEvent {
    Started, Completed, Failed, Timeout, AiEnhancementFailed
}

// Health states
HealthEvent {
    Healthy, Degraded, Unhealthy, Critical
}

// Metrics categories
MetricsEvent {
    Performance, Memory, Cache, Pool
}
```

### Event Routing
- **Broadcast**: All handlers receive all events
- **PatternBased**: Route by event type patterns (e.g., `pool.*`)
- **SeverityBased**: Route by severity level
- **Custom**: Handler-defined routing logic

### Event Handlers
- `LoggingEventHandler` - Structured logging
- `MetricsEventHandler` - Metrics aggregation
- `TelemetryEventHandler` - OpenTelemetry integration
- `HealthEventHandler` - Health tracking

## Serialization Patterns

**Serde Coverage**: 95%+ of data models

**Common Pattern**:
```rust
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct MyModel {
    // fields...
}
```

**Supported Formats**:
- JSON (primary API format)
- YAML (configuration files)
- TOML (alternative config)
- NDJSON (streaming)
- Binary (Redis storage)

## Schema Coupling Analysis

### 🔴 High Coupling (Requires Careful Migration)
**Types**: `PoolEvent`, `ExtractionEvent`, `CrawlEvent`, `CrawlRequest`, `CrawlResult`, `ExtractedDoc`, `ApiKey`, `BrowserFingerprint`

**Characteristics**:
- Tightly coupled to domain logic
- Schema changes ripple across crates
- Public API contracts

**Impact**: Breaking changes require coordinated updates

**Recommendation**: Version schemas with adapters

### 🟡 Medium Coupling (Moderate Refactoring)
**Types**: `HealthEvent`, `MetricsEvent`, `SecurityContext`, `SessionState`, `DetectionScore`

**Characteristics**:
- Some domain-specific logic
- Used across multiple crates
- Internal abstractions

**Impact**: Moderate refactoring effort

**Recommendation**: Abstract domain logic

### 🟢 Low Coupling (Easy to Refactor)
**Types**: `PerformanceMetrics`, `Alert`, `CircuitBreakerState`, Config structs, `Chunk`, `MemoryStats`

**Characteristics**:
- Generic/reusable
- Minimal dependencies
- Simple data containers

**Impact**: Easy to refactor

**Recommendation**: Extract to shared crates

## Configuration Architecture

**Total Config Structs**: 80+

**Naming Pattern**: `*Config` suffix

**Key Configurations**:
- `SpiderConfig` - Spider behavior
- `StealthConfig` - Anti-detection
- `SecurityConfig` - Security features
- `PersistenceConfig` - Storage settings
- `MonitoringConfig` - Observability
- `SearchConfig` - Search integration
- `BudgetConfig` - Cost tracking
- `SessionConfig` - Session management

**Builder Pattern**: Used for complex configs

**Environment Overrides**: Supported via serde

## Key Findings

1. **✅ Comprehensive Serialization**: 95%+ coverage with Serde
2. **✅ Strong Separation**: Crate boundaries enforce modularity
3. **✅ Event-Driven**: Clean event bus architecture
4. **⚠️ High Event Coupling**: Event schemas tightly coupled to domains
5. **⚠️ API Leakage**: Internal models exposed in API
6. **⚠️ Redis Details**: Storage implementation leaks into models
7. **✅ Multi-Tenancy**: Well-designed tenant isolation
8. **✅ WASM Pooling**: Sophisticated memory management
9. **✅ Circuit Breakers**: Resilience patterns throughout
10. **✅ Configuration**: Well-structured config system

## Decoupling Recommendations

### Priority 1: Event Schema Versioning
**Issue**: Event types tightly coupled to domain logic
**Impact**: Schema changes break all subscribers
**Solution**: Introduce versioned event schemas with adapters
**Affected Crates**: `riptide-events`, `riptide-pool`, `riptide-extraction`

```rust
// Current (tightly coupled)
pub struct PoolEvent {
    base: BaseEvent,
    operation: PoolOperation,
    // ... domain-specific fields
}

// Proposed (versioned)
pub mod v1 {
    pub struct PoolEvent { /* ... */ }
}

pub mod v2 {
    pub struct PoolEvent { /* ... */ }
}

// Adapter
impl From<v1::PoolEvent> for v2::PoolEvent { /* ... */ }
```

### Priority 2: API DTO Layer
**Issue**: Request/response types expose internal structures
**Impact**: Internal changes break API contracts
**Solution**: Add DTOs layer between API and domain
**Affected Crates**: `riptide-api`, `riptide-types`

```rust
// DTOs for API
pub mod dto {
    pub struct CrawlRequest { /* public contract */ }
}

// Domain models (can change freely)
pub mod domain {
    pub struct CrawlRequest { /* internal structure */ }
}

// Mapping
impl From<dto::CrawlRequest> for domain::CrawlRequest { /* ... */ }
```

### Priority 3: Config Centralization
**Issue**: Config structs scattered across 30+ crates
**Impact**: Hard to find and manage configurations
**Solution**: Centralize in `riptide-config` crate
**Affected Crates**: All

### Priority 4: Storage Abstraction
**Issue**: Redis details leak into domain models
**Impact**: Hard to swap storage implementations
**Solution**: Repository pattern with trait abstraction
**Affected Crates**: `riptide-persistence`, `riptide-cache`

```rust
// Storage trait
pub trait Repository<T> {
    async fn get(&self, key: &str) -> Result<Option<T>>;
    async fn set(&self, key: &str, value: T) -> Result<()>;
}

// Redis implementation
pub struct RedisRepository<T> { /* ... */ }

// File implementation
pub struct FileRepository<T> { /* ... */ }
```

### Priority 5: Metrics Deduplication
**Issue**: Metrics structures duplicated across crates
**Impact**: Inconsistent metrics, wasted code
**Solution**: Extract common metrics to `riptide-metrics`
**Affected Crates**: `riptide-monitoring`, `riptide-performance`

## Migration Complexity Assessment

| Component | Complexity | Reason |
|-----------|-----------|--------|
| Event System | 🔴 **High** | Central to architecture, many subscribers |
| Persistence Layer | 🟡 **Medium** | Well abstracted but widely used |
| API Contracts | 🟡 **Medium** | Public facing, needs versioning |
| Configuration | 🟢 **Low** | Already serializable, easy to move |
| Monitoring | 🟢 **Low** | Loosely coupled, minimal dependencies |

## Next Steps

1. **Immediate**: Review event schema coupling with architect
2. **Short-term**: Design versioned event schema system
3. **Medium-term**: Implement DTO layer for API
4. **Long-term**: Centralize configuration, abstract storage

## Files Generated

- `/workspaces/eventmesh/docs/analysis/data_models_catalog.json` - Complete catalog
- `/workspaces/eventmesh/docs/analysis/data_models_summary.md` - This summary

## Coordination

- Task started: `npx claude-flow@alpha hooks pre-task`
- Task completed: `npx claude-flow@alpha hooks post-task`
- Memory stored: `hive/analysis/data-models`
