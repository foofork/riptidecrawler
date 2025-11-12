# 🚀 Phase 2: Facade Detox & Complete Trait Migration

**Objective**: Migrate remaining 18 concrete types to trait abstractions
**Target**: 100% hexagonal architecture compliance
**Current**: 28% compliant (9/32 fields)
**Remaining**: 18 concrete infrastructure types

---

## 📋 Table of Contents

1. [Overview](#overview)
2. [Priority 1: Critical Infrastructure](#priority-1-critical-infrastructure)
3. [Priority 2: Metrics Consolidation](#priority-2-metrics-consolidation)
4. [Priority 3: Facade Layer Cleanup](#priority-3-facade-layer-cleanup)
5. [Implementation Timeline](#implementation-timeline)
6. [Testing Strategy](#testing-strategy)
7. [Rollback Plan](#rollback-plan)

---

## Overview

### Current State

ApplicationContext contains 18 concrete infrastructure types that violate hexagonal architecture:

- **8 Critical Infrastructure** types (ResourceManager, HealthChecker, etc.)
- **5 Metrics** types (consolidation opportunity)
- **5 Facades** (circular dependency risk)

### Target State

All fields in ApplicationContext use `Arc<dyn Trait>` following dependency inversion principle.

### Success Criteria

- ✅ Zero concrete infrastructure types in ApplicationContext
- ✅ All dependencies injected via trait abstractions
- ✅ Workspace builds with zero errors/warnings
- ✅ All tests pass
- ✅ No performance regression

---

## Priority 1: Critical Infrastructure

**Timeline**: 2-3 days
**Risk**: Medium
**Impact**: High

### 1.1 ResourceManager → ResourceManagement Trait

**Current Type**: `Arc<ResourceManager>`
**Location**: `crates/riptide-api/src/context.rs:1956`

#### Port Trait Design

```rust
// File: crates/riptide-types/src/ports/resource.rs

#[async_trait]
pub trait ResourceManagement: Send + Sync {
    /// Get current resource status
    async fn get_resource_status(&self) -> ResourceStatus;

    /// Get available capacity
    async fn get_capacity(&self) -> ResourceCapacity;

    /// Request resource allocation
    async fn request_resources(&self, amount: usize) -> RiptideResult<AllocationId>;

    /// Release allocated resources
    async fn release_resources(&self, id: AllocationId) -> RiptideResult<()>;

    /// Get resource health metrics
    async fn health_metrics(&self) -> ResourceHealthMetrics;
}

#[derive(Debug, Clone)]
pub struct ResourceStatus {
    pub available: usize,
    pub in_use: usize,
    pub reserved: usize,
}

#[derive(Debug, Clone)]
pub struct ResourceCapacity {
    pub total: usize,
    pub available: usize,
    pub percentage_used: f32,
}
```

#### Adapter Implementation

```rust
// File: crates/riptide-resources/src/adapters/resource_manager_adapter.rs

pub struct ResourceManagerAdapter {
    inner: Arc<ResourceManager>,
}

impl ResourceManagerAdapter {
    pub fn new(manager: ResourceManager) -> Arc<Self> {
        Arc::new(Self {
            inner: Arc::new(manager),
        })
    }
}

#[async_trait]
impl ResourceManagement for ResourceManagerAdapter {
    async fn get_resource_status(&self) -> ResourceStatus {
        // Delegate to inner ResourceManager
        self.inner.get_resource_status().await
    }

    // ... implement other trait methods
}
```

#### Migration Steps

1. Create port trait in `riptide-types/src/ports/resource.rs`
2. Implement adapter in `riptide-resources/src/adapters/`
3. Update ApplicationContext field type
4. Update all call sites to use trait methods
5. Run tests

**Files to Modify**:
- `crates/riptide-types/src/ports/resource.rs` (create)
- `crates/riptide-resources/src/adapters/resource_manager_adapter.rs` (create)
- `crates/riptide-api/src/context.rs` (update type)
- `crates/riptide-api/src/handlers/telemetry.rs` (update usage)

---

### 1.2 SessionManager → SessionManagement Trait

**Current Type**: `Arc<SessionManager>`
**Location**: `crates/riptide-api/src/context.rs:1961`

#### Port Trait Design

```rust
// File: crates/riptide-types/src/ports/session.rs

#[async_trait]
pub trait SessionManagement: Send + Sync {
    /// Create new session
    async fn create_session(&self, config: SessionConfig) -> RiptideResult<SessionId>;

    /// Get active session
    async fn get_session(&self, id: &SessionId) -> RiptideResult<Option<Session>>;

    /// Update session data
    async fn update_session(&self, id: &SessionId, data: SessionData) -> RiptideResult<()>;

    /// Terminate session
    async fn terminate_session(&self, id: &SessionId) -> RiptideResult<()>;

    /// List active sessions
    async fn list_active_sessions(&self) -> RiptideResult<Vec<SessionId>>;
}
```

---

### 1.3 HealthChecker → HealthCheck Trait

**Current Type**: `Arc<HealthChecker>`
**Location**: `crates/riptide-api/src/context.rs:1960`

#### Port Trait Design

```rust
// File: crates/riptide-types/src/ports/health.rs

#[async_trait]
pub trait HealthCheck: Send + Sync {
    /// Check system health
    async fn check_health(&self) -> HealthStatus;

    /// Check specific component health
    async fn check_component(&self, component: &str) -> ComponentHealth;

    /// Get health history
    async fn health_history(&self, duration: Duration) -> Vec<HealthSnapshot>;

    /// Register health probe
    async fn register_probe(&self, name: String, probe: Box<dyn HealthProbe>) -> RiptideResult<()>;
}

pub struct HealthStatus {
    pub overall: HealthState,
    pub components: HashMap<String, ComponentHealth>,
    pub timestamp: DateTime<Utc>,
}

pub enum HealthState {
    Healthy,
    Degraded,
    Unhealthy,
}
```

---

### 1.4 EventBus → EventPublisher Trait

**Current Type**: `Arc<EventBus>`
**Location**: `crates/riptide-api/src/context.rs:1969`

#### Port Trait Design

```rust
// File: crates/riptide-types/src/ports/events.rs

#[async_trait]
pub trait EventPublisher: Send + Sync {
    /// Publish event to subscribers
    async fn publish<E: Event>(&self, event: E) -> RiptideResult<()>;

    /// Subscribe to event type
    async fn subscribe<E: Event>(&self, handler: EventHandler<E>) -> SubscriptionId;

    /// Unsubscribe from events
    async fn unsubscribe(&self, id: SubscriptionId) -> RiptideResult<()>;

    /// Get event history
    async fn event_history(&self, limit: usize) -> Vec<EventRecord>;
}

pub trait Event: Send + Sync + 'static {
    fn event_type(&self) -> &'static str;
    fn timestamp(&self) -> DateTime<Utc>;
}
```

---

### 1.5 StreamingModule → StreamingProvider Trait

**Current Type**: `Arc<StreamingModule>`
**Location**: `crates/riptide-api/src/context.rs:1962`

#### Port Trait Design

```rust
// File: crates/riptide-types/src/ports/streaming.rs

#[async_trait]
pub trait StreamingProvider: Send + Sync {
    /// Start streaming operation
    async fn start_stream(&self, config: StreamConfig) -> RiptideResult<StreamHandle>;

    /// Get stream metrics
    async fn metrics(&self) -> StreamMetrics;

    /// Stop streaming
    async fn stop_stream(&self, handle: StreamHandle) -> RiptideResult<()>;
}
```

---

### 1.6 TelemetrySystem → TelemetryBackend Trait

**Current Type**: `Option<Arc<TelemetrySystem>>`
**Location**: `crates/riptide-api/src/context.rs:1963`

#### Port Trait Design

```rust
// File: crates/riptide-types/src/ports/telemetry.rs

#[async_trait]
pub trait TelemetryBackend: Send + Sync {
    /// Record span
    async fn record_span(&self, span: Span) -> RiptideResult<()>;

    /// Record metric
    async fn record_metric(&self, metric: Metric) -> RiptideResult<()>;

    /// Flush telemetry data
    async fn flush(&self) -> RiptideResult<()>;
}
```

---

### 1.7 MonitoringSystem → MonitoringBackend Trait

**Current Type**: `Arc<MonitoringSystem>`
**Location**: `crates/riptide-api/src/context.rs:1972`

#### Port Trait Design

```rust
// File: crates/riptide-types/src/ports/monitoring.rs

#[async_trait]
pub trait MonitoringBackend: Send + Sync {
    /// Report metric
    async fn report_metric(&self, name: &str, value: f64, tags: MetricTags) -> RiptideResult<()>;

    /// Query metrics
    async fn query_metrics(&self, query: MetricQuery) -> RiptideResult<Vec<MetricPoint>>;

    /// Get health score
    async fn health_score(&self) -> RiptideResult<f32>;

    /// Get system status
    async fn status(&self) -> String;
}
```

---

### 1.8 FetchEngine → FetchProvider Trait (or Remove)

**Current Type**: `Arc<FetchEngine>`
**Location**: `crates/riptide-api/src/context.rs:1974`

**Analysis**: FetchEngine likely duplicates HttpClient functionality.

**Options**:

**Option A: Remove FetchEngine**
- Validate that HttpClient provides all needed functionality
- Replace all FetchEngine usage with HttpClient
- Remove FetchEngine field entirely

**Option B: Create FetchProvider Trait** (if additional features exist)
```rust
#[async_trait]
pub trait FetchProvider: Send + Sync {
    async fn fetch_with_retry(&self, url: &str, retry_config: RetryConfig) -> RiptideResult<HttpResponse>;
    async fn fetch_batch(&self, urls: Vec<String>) -> RiptideResult<Vec<HttpResponse>>;
}
```

**Recommendation**: Option A (remove) unless FetchEngine provides unique features not in HttpClient.

---

## Priority 2: Metrics Consolidation

**Timeline**: 1-2 days
**Risk**: Low
**Impact**: Medium

### Problem Statement

Five separate metrics types create duplication and confusion:
1. BusinessMetrics
2. TransportMetrics
3. CombinedMetrics
4. PdfMetricsCollector
5. PerformanceMetrics

### Solution: Unified MetricsCollector Trait

#### Port Trait Design

```rust
// File: crates/riptide-types/src/ports/metrics.rs

#[async_trait]
pub trait MetricsCollector: Send + Sync {
    /// Record counter metric
    async fn record_counter(&self, name: &str, value: u64, labels: MetricLabels);

    /// Record gauge metric
    async fn record_gauge(&self, name: &str, value: f64, labels: MetricLabels);

    /// Record histogram metric
    async fn record_histogram(&self, name: &str, value: f64, labels: MetricLabels);

    /// Get metric snapshot
    async fn snapshot(&self) -> MetricsSnapshot;
}

pub type MetricLabels = HashMap<String, String>;

pub struct MetricsSnapshot {
    pub counters: HashMap<String, u64>,
    pub gauges: HashMap<String, f64>,
    pub histograms: HashMap<String, Vec<f64>>,
    pub timestamp: DateTime<Utc>,
}
```

#### Composite Adapter

```rust
// File: crates/riptide-metrics/src/adapters/composite_metrics.rs

pub struct CompositeMetricsAdapter {
    business: Arc<BusinessMetrics>,
    transport: Arc<TransportMetrics>,
    pdf: Arc<PdfMetricsCollector>,
    performance: Arc<Mutex<PerformanceMetrics>>,
}

#[async_trait]
impl MetricsCollector for CompositeMetricsAdapter {
    async fn record_counter(&self, name: &str, value: u64, labels: MetricLabels) {
        // Route to appropriate internal collector based on name/label
        match name {
            n if n.starts_with("business.") => self.business.record(name, value),
            n if n.starts_with("transport.") => self.transport.record(name, value),
            n if n.starts_with("pdf.") => self.pdf.record(name, value),
            n if n.starts_with("perf.") => {
                let mut perf = self.performance.lock().await;
                perf.record(name, value);
            }
            _ => warn!("Unknown metrics category: {}", name),
        }
    }

    async fn snapshot(&self) -> MetricsSnapshot {
        // Aggregate all metrics into single snapshot
        let mut snapshot = MetricsSnapshot::default();

        snapshot.merge(self.business.snapshot().await);
        snapshot.merge(self.transport.snapshot().await);
        snapshot.merge(self.pdf.snapshot().await);
        snapshot.merge(self.performance.lock().await.snapshot());

        snapshot
    }
}
```

#### Migration Steps

1. Create `MetricsCollector` trait
2. Implement `CompositeMetricsAdapter`
3. Replace 5 separate metrics fields with single `metrics: Arc<dyn MetricsCollector>`
4. Update all call sites
5. Verify metrics still work correctly

**Benefits**:
- Reduces ApplicationContext fields from 5 to 1
- Unified metrics interface
- Easier to add new metrics categories
- Simplified testing

---

## Priority 3: Facade Layer Cleanup

**Timeline**: 2-3 days
**Risk**: High (circular dependencies)
**Impact**: High

### Problem Statement

Five facades in ApplicationContext create tight coupling and circular dependency risk:
1. ExtractionFacade
2. ScraperFacade
3. SpiderFacade
4. SearchFacade
5. EngineFacade

### Analysis: Facade Duplication

| Facade | Duplicates Trait? | Action |
|--------|-------------------|--------|
| ExtractionFacade | Yes (ContentExtractor) | Remove - use ContentExtractor directly |
| SpiderFacade | Yes (SpiderEngine) | Remove - use SpiderEngine directly |
| ScraperFacade | No | Create WebScraping trait |
| SearchFacade | No | Create SearchProvider trait |
| EngineFacade | No | Create EngineSelection trait |

### 3.1 Remove ExtractionFacade

**Current**: `Arc<ExtractionFacade>`
**Action**: Remove field, use existing `Arc<dyn ContentExtractor>` instead

**Steps**:
1. Grep for all `extraction_facade` usage
2. Replace with `extractor` field calls
3. Remove `extraction_facade` field from ApplicationContext
4. Remove facade imports

### 3.2 Remove SpiderFacade

**Current**: `Option<Arc<SpiderFacade>>`
**Action**: Remove field, use existing `Option<Arc<dyn SpiderEngine>>` instead

**Steps**:
1. Grep for all `spider_facade` usage
2. Replace with `spider` field calls
3. Remove `spider_facade` field from ApplicationContext
4. Remove facade imports

### 3.3 ScraperFacade → WebScraping Trait

**Current**: `Arc<ScraperFacade>`

#### Port Trait Design

```rust
// File: crates/riptide-types/src/ports/scraping.rs

#[async_trait]
pub trait WebScraping: Send + Sync {
    /// Scrape single URL
    async fn scrape_url(&self, url: &str, options: ScrapeOptions) -> RiptideResult<ScrapedPage>;

    /// Scrape multiple URLs
    async fn scrape_batch(&self, urls: Vec<String>, options: ScrapeOptions) -> RiptideResult<Vec<ScrapedPage>>;

    /// Extract data with CSS selectors
    async fn extract_with_selectors(&self, html: &str, selectors: SelectorSet) -> RiptideResult<ExtractedData>;
}
```

### 3.4 SearchFacade → SearchProvider Trait

**Current**: `Option<Arc<SearchFacade>>`

#### Port Trait Design

```rust
// File: crates/riptide-types/src/ports/search.rs

#[async_trait]
pub trait SearchProvider: Send + Sync {
    /// Search content
    async fn search(&self, query: SearchQuery) -> RiptideResult<SearchResults>;

    /// Index document
    async fn index_document(&self, doc: SearchDocument) -> RiptideResult<DocumentId>;

    /// Delete document
    async fn delete_document(&self, id: DocumentId) -> RiptideResult<()>;
}
```

### 3.5 EngineFacade → EngineSelection Trait

**Current**: `Arc<EngineFacade>`

#### Port Trait Design

```rust
// File: crates/riptide-types/src/ports/engine.rs

#[async_trait]
pub trait EngineSelection: Send + Sync {
    /// Select best engine for URL
    async fn select_engine(&self, url: &str, options: EngineOptions) -> EngineChoice;

    /// Get available engines
    async fn available_engines(&self) -> Vec<EngineInfo>;

    /// Validate engine compatibility
    async fn validate_compatibility(&self, url: &str, engine: EngineType) -> bool;
}
```

---

## Implementation Timeline

### Week 1: Critical Infrastructure (Days 1-3)

**Day 1**:
- ResourceManagement trait + adapter
- SessionManagement trait + adapter
- Update ApplicationContext
- Run tests

**Day 2**:
- HealthCheck trait + adapter
- EventPublisher trait + adapter
- Update ApplicationContext
- Run tests

**Day 3**:
- StreamingProvider trait + adapter
- TelemetryBackend trait + adapter
- MonitoringBackend trait + adapter
- Remove/abstract FetchEngine
- Run full workspace build + tests

### Week 2: Metrics & Facades (Days 4-6)

**Day 4**:
- MetricsCollector trait design
- CompositeMetricsAdapter implementation
- Replace 5 metrics fields with single field
- Update all call sites
- Run tests

**Day 5**:
- Remove ExtractionFacade (use ContentExtractor)
- Remove SpiderFacade (use SpiderEngine)
- WebScraping trait + ScraperFacade adapter
- Run tests

**Day 6**:
- SearchProvider trait + SearchFacade adapter
- EngineSelection trait + EngineFacade adapter
- Final ApplicationContext cleanup
- Full workspace build + clippy + tests
- Architecture validation

---

## Testing Strategy

### Unit Tests

Each adapter must have unit tests:
```rust
#[cfg(test)]
mod tests {
    #[tokio::test]
    async fn test_resource_manager_adapter() {
        let manager = ResourceManager::new();
        let adapter = ResourceManagerAdapter::new(manager);

        let status = adapter.get_resource_status().await.unwrap();
        assert!(status.total > 0);
    }
}
```

### Integration Tests

Test full ApplicationContext with all traits:
```rust
#[tokio::test]
async fn test_application_context_with_trait_abstractions() {
    let config = test_config();
    let context = ApplicationContext::new_for_test(config).await.unwrap();

    // Verify all trait objects work
    assert!(context.http_client.get("https://example.com").await.is_ok());
    assert!(context.cache.set("key", b"value", None).await.is_ok());
    // ... test all traits
}
```

### Contract Tests

Each port trait needs contract tests:
```rust
pub async fn test_resource_management_contract<T: ResourceManagement>(impl_: Arc<T>) {
    // Test that implementation follows contract
    let status = impl_.get_resource_status().await.unwrap();
    assert!(status.total >= status.in_use);

    let capacity = impl_.get_capacity().await.unwrap();
    assert!(capacity.percentage_used >= 0.0 && capacity.percentage_used <= 100.0);
}
```

---

## Rollback Plan

### Phase Rollback

Each phase (Priority 1/2/3) is independent. If issues arise:

1. **Priority 1 Rollback**: Revert to concrete types for affected infrastructure
2. **Priority 2 Rollback**: Restore 5 separate metrics types
3. **Priority 3 Rollback**: Restore facades

### Git Strategy

```bash
# Create feature branch per priority
git checkout -b phase2/priority1-infrastructure
git checkout -b phase2/priority2-metrics
git checkout -b phase2/priority3-facades

# Each phase is independently revertible
git revert <phase-commit>
```

### Validation Points

After each priority phase:
- ✅ Workspace builds with zero errors/warnings
- ✅ All tests pass
- ✅ No performance regression (run benchmarks)
- ✅ Architecture compliance improves

If any validation fails, rollback that phase and investigate.

---

## Success Metrics

### Quantitative

- **Architecture Compliance**: 28% → 100%
- **Concrete Types**: 18 → 0
- **Build Status**: ✅ PASS (maintain)
- **Clippy Warnings**: 0 (maintain)
- **Test Coverage**: No regression
- **Performance**: < 5% regression acceptable

### Qualitative

- Code is more testable (can inject mocks)
- Infrastructure is swappable
- Clean hexagonal architecture
- Reduced coupling
- Clearer separation of concerns

---

## Risk Mitigation

### High Risk: Circular Dependencies from Facades

**Mitigation**:
- Analyze dependency graph before making changes
- Remove facades incrementally
- Validate no circular dependencies after each change
- Use `cargo tree` to verify dependency graph

### Medium Risk: Breaking Changes to Public API

**Mitigation**:
- Keep changes internal to ApplicationContext
- Maintain public API surface
- Version bump if breaking changes needed
- Provide migration guide

### Low Risk: Performance Regression

**Mitigation**:
- Run benchmarks before and after
- Use `Arc<dyn Trait>` efficiently (already in use)
- Profile if regression detected

---

## Appendix A: Dependency Graph

```
Current (with concrete types):
riptide-api
├── ResourceManager (concrete)
├── SessionManager (concrete)
├── HealthChecker (concrete)
├── EventBus (concrete)
├── ExtractionFacade (concrete)
└── ... 13 more concrete types

Target (with trait abstractions):
riptide-api
├── Arc<dyn ResourceManagement> ✅
├── Arc<dyn SessionManagement> ✅
├── Arc<dyn HealthCheck> ✅
├── Arc<dyn EventPublisher> ✅
├── Arc<dyn WebScraping> ✅
└── ... all traits!
```

---

## Appendix B: File Checklist

### New Files to Create

**Port Traits** (riptide-types/src/ports/):
- [ ] resource.rs - ResourceManagement
- [ ] session.rs - SessionManagement
- [ ] health.rs - HealthCheck
- [ ] events.rs - EventPublisher
- [ ] streaming.rs - StreamingProvider
- [ ] telemetry.rs - TelemetryBackend
- [ ] monitoring.rs - MonitoringBackend
- [ ] metrics.rs - MetricsCollector
- [ ] scraping.rs - WebScraping
- [ ] search.rs - SearchProvider
- [ ] engine.rs - EngineSelection

**Adapters**:
- [ ] riptide-resources/src/adapters/resource_manager_adapter.rs
- [ ] riptide-session/src/adapters/session_manager_adapter.rs
- [ ] riptide-health/src/adapters/health_checker_adapter.rs
- [ ] riptide-events/src/adapters/event_bus_adapter.rs
- [ ] riptide-streaming/src/adapters/streaming_module_adapter.rs
- [ ] riptide-telemetry/src/adapters/telemetry_system_adapter.rs
- [ ] riptide-monitoring/src/adapters/monitoring_system_adapter.rs
- [ ] riptide-metrics/src/adapters/composite_metrics.rs
- [ ] riptide-facade/src/adapters/scraper_facade_adapter.rs
- [ ] riptide-facade/src/adapters/search_facade_adapter.rs
- [ ] riptide-facade/src/adapters/engine_facade_adapter.rs

### Files to Modify

- [ ] crates/riptide-api/src/context.rs (ApplicationContext field types)
- [ ] crates/riptide-api/src/handlers/*.rs (update trait usage)
- [ ] crates/riptide-api/src/health.rs (update trait usage)
- [ ] All files using the 18 concrete types

---

**Document Version**: 1.0
**Last Updated**: 2025-11-12
**Status**: Ready for Implementation
