# RipTide Browser - Architecture Design

**Status**: Phase 3 - Consolidation Architecture
**Created**: 2025-10-21
**Last Updated**: 2025-10-21

## Executive Summary

The `riptide-browser` crate consolidates browser automation infrastructure from three legacy crates (`riptide-engine`, `riptide-headless`, `riptide-headless-hybrid`) into a unified, well-organized module structure.

**Key Benefits**:
- **Single Source of Truth**: All browser automation in one crate
- **Clear Module Boundaries**: Separation of concerns (pool, CDP, launcher, API)
- **Reduced Duplication**: Shared models and utilities
- **Easier Testing**: Unified test infrastructure
- **Better Documentation**: Comprehensive architecture overview

---

## Architecture Overview

```text
┌─────────────────────────────────────────────────────────────────────────┐
│                          riptide-browser                                │
├─────────────────────────────────────────────────────────────────────────┤
│                                                                         │
│  ┌──────────────┐  ┌──────────────┐  ┌──────────────┐  ┌───────────┐ │
│  │     pool     │  │   cdp_pool   │  │   launcher   │  │ http_api  │ │
│  │              │  │              │  │              │  │           │ │
│  │  Browser     │  │     CDP      │  │  Headless    │  │   Axum    │ │
│  │  Instance    │  │  Connection  │  │  Launcher    │  │  Routes   │ │
│  │  Pool Mgmt   │  │  Pool Mgmt   │  │  (Facade)    │  │  (CDP)    │ │
│  │              │  │              │  │              │  │           │ │
│  │ • PooledBrowser│ • PooledConn │  │ • LaunchSession│ • /render  │ │
│  │ • BrowserCheckout│ • Batching │  │ • LauncherStats│ • /healthz│ │
│  │ • Tiered Health│ • Affinity  │  │ • Stealth Int  │ • AppState│ │
│  │ • Memory Limits│ • Wait Queue │  │              │  │           │ │
│  └───────┬──────┘  └───────┬──────┘  └───────┬──────┘  └─────┬─────┘ │
│          │                 │                 │                │       │
│          └─────────────────┴─────────────────┴────────────────┘       │
│                                  │                                     │
│                           ┌──────▼──────┐                             │
│                           │   models    │                             │
│                           │             │                             │
│                           │  Shared     │                             │
│                           │  Types &    │                             │
│                           │  Models     │                             │
│                           └─────────────┘                             │
│                                                                         │
└─────────────────────────────────┬───────────────────────────────────────┘
                                  │
                         ┌────────▼────────┐
                         │  spider_chrome  │
                         │ riptide-stealth │
                         └─────────────────┘
```

---

## Module Design

### 1. `pool` - Browser Pool Management

**Purpose**: Lifecycle management of browser instances with resource tracking.

**Key Components**:
- `BrowserPool`: Main pool manager
  - Browser lifecycle (create, reuse, cleanup)
  - Semaphore-based concurrency control
  - Tiered health monitoring (fast + full checks)
  - Memory limit enforcement
  - Event-driven monitoring

- `PooledBrowser`: Individual browser instance
  - Unique profile directories (required for Chrome SingletonLock)
  - Health status tracking
  - Usage statistics
  - Automatic cleanup on drop

- `BrowserCheckout`: RAII-style checkout
  - Automatic return to pool on drop
  - Timeout-based cleanup
  - CDP connection integration

**Features**:
- **QW-1**: 4x capacity improvement (max_pool_size: 20)
- **QW-2**: 5x faster failure detection via tiered health checks
- **QW-3**: -30% memory footprint via memory limits

**Configuration**:
```rust
pub struct BrowserPoolConfig {
    pub min_pool_size: usize,
    pub max_pool_size: usize,           // Default: 20 (QW-1)
    pub initial_pool_size: usize,
    pub idle_timeout: Duration,
    pub max_lifetime: Duration,

    // Tiered health monitoring (QW-2)
    pub enable_tiered_health_checks: bool,
    pub fast_check_interval: Duration,  // 2s liveness check
    pub full_check_interval: Duration,  // 15s comprehensive check

    // Memory limits (QW-3)
    pub enable_memory_limits: bool,
    pub memory_soft_limit_mb: u64,      // 400MB - trigger cleanup
    pub memory_hard_limit_mb: u64,      // 500MB - force eviction
}
```

---

### 2. `cdp_pool` - CDP Connection Pooling

**Purpose**: Chrome DevTools Protocol connection multiplexing for reduced latency.

**Key Components**:
- `CdpConnectionPool`: Connection lifecycle manager
  - Connection reuse across requests
  - Command batching (~50% latency reduction)
  - Session affinity routing
  - Priority-based wait queues

- `PooledConnection`: Individual CDP connection
  - Session ID tracking
  - Health status
  - Latency tracking (avg, p50, p95, p99)
  - Reuse rate metrics

- `CdpCommand`: Batchable CDP command
  - Command name and parameters
  - Timestamp for batching windows
  - Execution result tracking

**Features**:
- **P1-B4**: 30% latency reduction through multiplexing
- **Command Batching**: Reduces round-trips by ~50%
- **Session Affinity**: Routes related requests to same connection
- **Priority Queuing**: Handles pool saturation gracefully

**Configuration**:
```rust
pub struct CdpPoolConfig {
    pub max_connections_per_browser: usize,  // Default: 10
    pub connection_idle_timeout: Duration,
    pub max_connection_lifetime: Duration,

    // Batching
    pub enable_batching: bool,               // Default: true
    pub batch_timeout: Duration,             // 50ms batching window
    pub max_batch_size: usize,               // 10 commands

    // Health checks
    pub enable_health_checks: bool,
    pub health_check_interval: Duration,
}
```

---

### 3. `launcher` - High-Level Launcher

**Purpose**: Facade for browser session management with stealth integration.

**Key Components**:
- `HeadlessLauncher`: Main facade
  - Browser pool integration
  - Stealth feature application
  - Statistics tracking
  - Session lifecycle management

- `LaunchSession`: Managed browser session
  - Automatic cleanup on drop
  - Page access via trait object
  - Navigation helpers
  - Stealth preset support

- `LauncherConfig`: Configuration
  - Pool settings
  - Stealth presets
  - Timeout configuration

**Features**:
- **Stealth Integration**: Via `riptide-stealth` crate
- **Automatic Cleanup**: RAII-style session management
- **Statistics Tracking**: Success rate, latency metrics
- **Pool Transparency**: Seamless browser reuse

**API Example**:
```rust
use riptide_browser::launcher::HeadlessLauncher;

let launcher = HeadlessLauncher::new().await?;

// Launch with default stealth
let session = launcher.launch_page_default("https://example.com").await?;

// Access page via trait object
let page = session.page();
let content = page.content().await?;

// Session automatically cleaned up on drop
```

---

### 4. `http_api` - HTTP Endpoints

**Purpose**: REST API for remote browser control via CDP.

**Key Components**:
- `AppState`: Shared launcher instance
- `/render`: Dynamic content rendering endpoint
- `/healthz`: Health check endpoint
- Request/response models

**Features**:
- **Axum-based**: Modern async HTTP framework
- **CORS Support**: Cross-origin requests
- **Tracing**: Request/response logging
- **Error Handling**: Structured error responses

**Routes**:
```rust
Router::new()
    .route("/healthz", get(health_check))
    .route("/render", post(render))
    .with_state(AppState { launcher })
    .layer(CorsLayer::permissive())
    .layer(TraceLayer::new_for_http())
```

---

### 5. `models` - Shared Types

**Purpose**: Common types and models used across modules.

**Key Types**:
- `BrowserCapabilities`: Browser feature detection
- `SessionStats`: Session-level metrics
- `PoolConfig`: Shared configuration
- Error types and results

---

## Dependency Architecture

```text
┌─────────────────────────────────────────┐
│         riptide-browser                 │
│  (unified browser infrastructure)       │
└──────────────┬──────────────────────────┘
               │
        ┌──────┴──────┐
        │             │
┌───────▼────┐  ┌────▼──────────────┐
│ spider_    │  │ riptide-          │
│ chrome     │  │ stealth           │
│            │  │                   │
│ Browser/   │  │ Anti-detection    │
│ Page/CDP   │  │ features          │
└────────────┘  └───────────────────┘

Internal Dependencies:
- riptide-types: Core types
- riptide-config: Configuration
- riptide-browser-abstraction: Browser traits
```

---

## Migration Strategy

### Phase 1: Create New Crate (Current)
- ✅ Define Cargo.toml with dependencies
- ✅ Design lib.rs with module exports
- ✅ Document architecture in ARCHITECTURE.md

### Phase 2: Copy Module Files
- Copy `pool.rs` from `riptide-engine`
- Copy `cdp_pool.rs` from `riptide-engine`
- Copy `launcher.rs` from `riptide-engine`
- Copy HTTP API from `riptide-headless`
- Copy models from all three crates
- Merge hybrid launcher into `launcher.rs`

### Phase 3: Update Imports
- Update module paths (e.g., `crate::pool` → `riptide_browser::pool`)
- Update re-exports in `lib.rs`
- Fix circular dependency issues
- Update tests

### Phase 4: Deprecate Legacy Crates
- Update `riptide-headless` to re-export from `riptide-browser`
- Update `riptide-engine` to re-export from `riptide-browser`
- Mark `riptide-headless-hybrid` as deprecated
- Update workspace dependencies

### Phase 5: Cleanup
- Remove duplicate code from legacy crates
- Update documentation
- Update integration tests
- Final validation

---

## Design Decisions

### ADR-001: Module Organization
**Context**: Need clear separation of concerns for browser automation.

**Decision**: Organize by functional area:
- `pool`: Browser instance management
- `cdp_pool`: CDP connection management
- `launcher`: High-level facade
- `http_api`: REST API endpoints
- `models`: Shared types

**Rationale**:
- Clear boundaries prevent circular dependencies
- Each module has single responsibility
- Easy to test modules in isolation
- Natural extension points for new features

---

### ADR-002: Consolidation vs. Separate Crates
**Context**: Three crates (`riptide-engine`, `riptide-headless`, `riptide-headless-hybrid`) had overlapping functionality.

**Decision**: Consolidate into single `riptide-browser` crate.

**Rationale**:
- **Reduces duplication**: Shared models, utilities
- **Single source of truth**: Browser automation logic
- **Easier refactoring**: No cross-crate coordination
- **Better documentation**: Unified architecture overview
- **Simpler testing**: All browser tests in one place

**Trade-offs**:
- ❌ Larger crate size (acceptable for workspace projects)
- ❌ Potentially longer compile times (mitigated by module structure)
- ✅ Much simpler dependency management
- ✅ Easier to maintain and extend

---

### ADR-003: HTTP API Location
**Context**: Should HTTP API be in `riptide-browser` or separate crate?

**Decision**: Include HTTP API as `http_api` module in `riptide-browser`.

**Rationale**:
- **Tight coupling**: API directly uses launcher/pool
- **Simplicity**: No need for separate crate overhead
- **Optional feature**: Can be feature-gated if needed
- **Common use case**: Most deployments need HTTP API

**Alternative Considered**: Separate `riptide-browser-http` crate
- ❌ Adds complexity without clear benefit
- ❌ Would require duplicate models
- ✅ Could reduce compile times (minimal gain)

---

### ADR-004: Dependency on spider_chrome
**Context**: Need CDP implementation for browser automation.

**Decision**: Use `spider_chrome` which re-exports `chromiumoxide`.

**Rationale**:
- **High concurrency**: Better async/await handling
- **Active maintenance**: Regular updates
- **Compatibility**: Re-exports chromiumoxide for drop-in replacement
- **Proven**: Used successfully in Phase 2

**Notes**:
- spider_chrome handles CDP protocol improvements
- riptide-browser manages browser instances and pooling
- Profile isolation still required (Chrome-level requirement)

---

## Performance Targets

### Browser Pool (QW-1, QW-2, QW-3)
- **Capacity**: 4x improvement (max_pool_size: 20)
- **Failure Detection**: 5x faster (2s liveness checks)
- **Memory**: -30% footprint (soft/hard limits)

### CDP Pool (P1-B4)
- **Latency**: 30% reduction via multiplexing
- **Reuse Rate**: >70% connection reuse
- **Batching**: ~50% reduction in round-trips

### Overall
- **Startup Time**: <500ms for launcher initialization
- **Session Creation**: <100ms average (pool hit)
- **Memory per Browser**: <500MB (hard limit)

---

## Testing Strategy

### Unit Tests
- Browser pool lifecycle
- CDP connection pooling
- Launcher session management
- HTTP API endpoints

### Integration Tests
- Pool scaling under load
- CDP multiplexing benefits
- Memory pressure handling
- Health check recovery

### Performance Tests
- Latency benchmarks
- Connection reuse rates
- Memory usage profiling
- Concurrent session handling

---

## Future Extensions

### Phase 4: Advanced Features
- **Distributed Pool**: Multi-node browser pool
- **GPU Rendering**: Optional GPU acceleration
- **Browser Profiles**: Persistent user profiles
- **Session Replay**: Record/replay browser sessions

### Phase 5: Observability
- **Metrics Export**: Prometheus metrics
- **Distributed Tracing**: OpenTelemetry integration
- **Performance Dashboards**: Grafana dashboards
- **Alerting**: SLA monitoring and alerts

---

## Conclusion

The `riptide-browser` crate provides a unified, well-architected foundation for browser automation:

1. **Clear Structure**: Modules organized by functional area
2. **High Performance**: Pool management, CDP multiplexing
3. **Production Ready**: Health monitoring, memory limits
4. **Easy to Use**: Simple facade API with RAII cleanup
5. **Well Documented**: Comprehensive architecture overview

This architecture supports the Phase 3 consolidation goals while maintaining flexibility for future extensions.
