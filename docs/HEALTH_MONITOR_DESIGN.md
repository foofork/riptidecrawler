# HealthMonitorBuilder API Design

## Executive Summary

The `HealthMonitorBuilder` already exists and is fully implemented in `/workspaces/eventmesh/crates/riptide-intelligence/src/health.rs` (lines 451-501). The two disabled integration tests can be re-enabled immediately as the builder provides all required functionality.

## Current Implementation Status

### ✅ Already Implemented

1. **HealthMonitorBuilder** - Fully functional builder pattern (lines 451-501)
2. **MockLlmProvider.set_healthy()** - Health control method (lines 78-80)
3. **HealthMonitor** - Complete health monitoring system (lines 130-448)
4. **FailoverManager** - Automatic provider failover (failover.rs)

### Builder Pattern API

The existing `HealthMonitorBuilder` provides the following builder methods:

```rust
pub struct HealthMonitorBuilder {
    config: HealthCheckConfig,
}

impl HealthMonitorBuilder {
    // Constructor
    pub fn new() -> Self

    // Configuration Methods (Chainable)
    pub fn with_interval(mut self, interval: Duration) -> Self
    pub fn with_timeout(mut self, timeout: Duration) -> Self
    pub fn with_failure_threshold(mut self, threshold: u32) -> Self
    pub fn with_success_threshold(mut self, threshold: u32) -> Self
    pub fn with_degraded_threshold(mut self, threshold: f64) -> Self
    pub fn with_critical_threshold(mut self, threshold: f64) -> Self

    // Build Method
    pub fn build(self) -> HealthMonitor
}
```

### Configuration Defaults

```rust
HealthCheckConfig::default() {
    interval: Duration::from_secs(30),           // Check every 30s
    timeout: Duration::from_secs(10),            // 10s timeout per check
    failure_threshold: 3,                        // 3 failures = unhealthy
    success_threshold: 2,                        // 2 successes = recovered
    degraded_threshold: 10.0,                    // 10% error rate = degraded
    critical_threshold: 50.0,                    // 50% error rate = critical
}
```

## Integration Test Analysis

### Test 1: `test_automatic_provider_failover` (line 456)

**Location:** `/workspaces/eventmesh/crates/riptide-intelligence/tests/integration_tests.rs:456-545`

**Required API Usage:**
```rust
// Create health monitor
let health_monitor = Arc::new(
    HealthMonitorBuilder::new()
        .with_interval(Duration::from_millis(100))
        .with_timeout(Duration::from_millis(50))
        .with_failure_threshold(2)
        .build(),
);

// Add providers
health_monitor
    .add_provider("primary".to_string(), primary_provider.clone())
    .await;
health_monitor
    .add_provider("secondary".to_string(), secondary_provider.clone())
    .await;

// Start monitoring
health_monitor.start().await.unwrap();

// Trigger failover
primary_provider.set_healthy(false);
sleep(Duration::from_millis(200)).await; // Allow health check to detect failure

// Cleanup
health_monitor.stop().await;
```

**Status:** ✅ All required APIs are implemented

### Test 2: `test_comprehensive_error_handling_and_recovery` (line 802)

**Location:** `/workspaces/eventmesh/crates/riptide-intelligence/tests/integration_tests.rs:802-847`

**Required API Usage:**
```rust
// Create health monitor with defaults
let health_monitor = Arc::new(HealthMonitorBuilder::new().build());
let recovering_provider = Arc::new(MockLlmProvider::new());

// Initially unhealthy
recovering_provider.set_healthy(false);
health_monitor
    .add_provider("recovering".to_string(), recovering_provider.clone())
    .await;

// Check initial unhealthy status
let health_result = health_monitor.check_provider("recovering").await;
if let Some(result) = health_result {
    assert!(!result.success, "Provider should be initially unhealthy");
}

// Simulate recovery
recovering_provider.set_healthy(true);

// Check recovered status
let recovered_result = health_monitor.check_provider("recovering").await;
if let Some(result) = recovered_result {
    assert!(result.success, "Provider should recover to healthy");
}
```

**Status:** ✅ All required APIs are implemented

## HealthMonitor Core API

### Provider Management
```rust
pub async fn add_provider(&self, name: String, provider: Arc<dyn LlmProvider>)
pub async fn remove_provider(&self, name: &str)
```

### Lifecycle Control
```rust
pub async fn start(&self) -> Result<()>
pub async fn stop(&self)
```

### Health Status Queries
```rust
pub async fn get_health_status(&self) -> HashMap<String, HealthCheckResult>
pub async fn get_provider_health(&self, name: &str) -> Option<HealthCheckResult>
pub async fn check_provider(&self, name: &str) -> Option<HealthCheckResult>
pub async fn get_healthy_providers(&self) -> Vec<String>
pub async fn get_providers_by_health(&self, level: HealthLevel) -> Vec<String>
```

### Metrics Access
```rust
pub async fn get_metrics(&self) -> HashMap<String, ProviderMetrics>
pub async fn get_provider_metrics(&self, name: &str) -> Option<ProviderMetrics>
```

### Event Monitoring
```rust
pub async fn take_event_receiver(&self) -> Option<mpsc::UnboundedReceiver<HealthEvent>>
```

## Data Structures

### HealthCheckResult
```rust
pub struct HealthCheckResult {
    pub provider_name: String,
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub level: HealthLevel,              // Healthy, Degraded, Critical, Unavailable
    pub response_time: Duration,
    pub success: bool,
    pub error_message: Option<String>,
    pub metrics: ProviderMetrics,
}
```

### ProviderMetrics
```rust
pub struct ProviderMetrics {
    pub total_requests: u64,
    pub successful_requests: u64,
    pub failed_requests: u64,
    pub avg_response_time: Duration,
    pub min_response_time: Duration,
    pub max_response_time: Duration,
    pub error_rate: f64,
    pub requests_per_minute: f64,
    pub tokens_per_second: f64,
    pub cost_per_request: f64,
    pub uptime_percentage: f64,
    pub last_request_time: Option<chrono::DateTime<chrono::Utc>>,
}
```

### HealthEvent
```rust
pub enum HealthEvent {
    StatusChanged { provider_name: String, old_status: HealthLevel, new_status: HealthLevel, timestamp },
    ProviderAdded { provider_name: String, timestamp },
    ProviderRemoved { provider_name: String, timestamp },
    HealthCheckFailed { provider_name: String, error: String, timestamp },
    MetricsUpdated { provider_name: String, metrics: ProviderMetrics, timestamp },
}
```

## MockLlmProvider Integration

### Health Control API
```rust
pub fn set_healthy(&self, healthy: bool)
pub fn is_healthy(&self) -> bool
```

### Implementation Details
- Uses `AtomicBool` for thread-safe health status
- Integrated with `health_check()` method
- Returns `IntelligenceError::Provider` when unhealthy
- Supports testing health monitoring and recovery scenarios

### Health Check Behavior
```rust
async fn health_check(&self) -> Result<()> {
    // Respects delay_ms configuration
    if let Some(delay) = self.delay_ms {
        sleep(Duration::from_millis(delay)).await;
    }

    // Check configured health status
    if !self.is_healthy.load(Ordering::SeqCst) {
        return Err(IntelligenceError::Provider("Mock provider is unhealthy".to_string()));
    }

    // Check should_fail configuration
    if self.should_fail {
        Err(IntelligenceError::Provider("Mock provider is configured to fail".to_string()))
    } else {
        Ok(())
    }
}
```

## FailoverManager Integration

The `HealthMonitor` is designed to work seamlessly with `FailoverManager` for automatic provider failover:

```rust
// Create health monitor
let health_monitor = Arc::new(
    HealthMonitorBuilder::new()
        .with_interval(Duration::from_millis(100))
        .build(),
);

// Create failover manager with health monitor
let (failover_manager, mut event_rx) =
    FailoverManager::new(FailoverConfig::default(), health_monitor.clone());

// Add providers to both systems
health_monitor.add_provider("primary".to_string(), primary_provider.clone()).await;
failover_manager.add_provider(primary_provider.clone(), primary_priority).await.unwrap();

// Start health monitoring
health_monitor.start().await.unwrap();

// Failover manager automatically uses health status for provider selection
```

## Test Enabling Strategy

### Priority 1: Enable Test 1 (Provider Failover)

**File:** `/workspaces/eventmesh/crates/riptide-intelligence/tests/integration_tests.rs`
**Line:** 456

**Changes Required:**
1. Remove `#[ignore]` attribute (line 456)
2. Uncomment test code (lines 459-545)

**Expected Behavior:**
- Creates health monitor with fast polling (100ms interval)
- Adds primary and secondary providers
- Simulates primary failure via `set_healthy(false)`
- Verifies failover to secondary provider
- Tests failback when primary recovers

### Priority 2: Enable Test 2 (Error Handling)

**File:** `/workspaces/eventmesh/crates/riptide-intelligence/tests/integration_tests.rs`
**Line:** 802

**Changes Required:**
1. Remove `#[ignore]` attribute (line 802)
2. Uncomment test code (lines 805-846)

**Expected Behavior:**
- Tests invalid configuration handling
- Verifies provider health failure detection
- Tests recovery from unhealthy to healthy state
- Uses manual health checks via `check_provider()`

## Architecture Diagram

```
┌─────────────────────────────────────────────────────────────┐
│                   HealthMonitorBuilder                      │
│  (Builder Pattern for Configuration)                        │
└────────────────────┬────────────────────────────────────────┘
                     │ .build()
                     ▼
┌─────────────────────────────────────────────────────────────┐
│                      HealthMonitor                          │
│  ┌───────────────────────────────────────────────────────┐  │
│  │  Providers: HashMap<String, Arc<dyn LlmProvider>>    │  │
│  │  Health Status: HashMap<String, HealthCheckResult>   │  │
│  │  Metrics: HashMap<String, ProviderMetrics>           │  │
│  └───────────────────────────────────────────────────────┘  │
│                                                              │
│  Methods:                                                    │
│  • add_provider() / remove_provider()                       │
│  • start() / stop()                                         │
│  • check_provider()                                         │
│  • get_health_status() / get_metrics()                      │
└────────────────────┬────────────────────────────────────────┘
                     │
                     │ Monitors
                     ▼
┌─────────────────────────────────────────────────────────────┐
│                   MockLlmProvider                           │
│  ┌───────────────────────────────────────────────────────┐  │
│  │  is_healthy: AtomicBool                               │  │
│  │  delay_ms: Option<u64>                                │  │
│  │  should_fail: bool                                    │  │
│  └───────────────────────────────────────────────────────┘  │
│                                                              │
│  Methods:                                                    │
│  • set_healthy(bool)    ← Test control                     │
│  • health_check()       ← Called by HealthMonitor          │
│  • is_healthy()         ← Query current status             │
└────────────────────┬────────────────────────────────────────┘
                     │
                     │ Health Events
                     ▼
┌─────────────────────────────────────────────────────────────┐
│                    FailoverManager                          │
│  (Uses health status for automatic failover decisions)     │
└─────────────────────────────────────────────────────────────┘
```

## Testing Workflow

### Phase 1: Manual Health Check
```rust
let monitor = Arc::new(HealthMonitorBuilder::new().build());
let provider = Arc::new(MockLlmProvider::new());

// Set unhealthy
provider.set_healthy(false);
monitor.add_provider("test".to_string(), provider.clone()).await;

// Manual check
let result = monitor.check_provider("test").await;
assert!(!result.unwrap().success);
```

### Phase 2: Automated Monitoring
```rust
let monitor = Arc::new(
    HealthMonitorBuilder::new()
        .with_interval(Duration::from_millis(100))
        .build()
);

monitor.add_provider("test".to_string(), provider.clone()).await;
monitor.start().await.unwrap();

// Wait for automated health check
sleep(Duration::from_millis(200)).await;

// Check status
let health = monitor.get_provider_health("test").await;
```

### Phase 3: Failover Integration
```rust
// Combine with FailoverManager
let (failover_manager, mut events) =
    FailoverManager::new(config, health_monitor.clone());

// Monitor failover events
while let Some(event) = events.recv().await {
    match event {
        FailoverEvent::ProviderFailover { from, to, .. } => {
            println!("Failover: {} -> {}", from, to);
        }
        _ => {}
    }
}
```

## Performance Characteristics

### Default Configuration
- **Health Check Interval:** 30 seconds
- **Timeout:** 10 seconds per check
- **Memory:** O(N) where N = number of providers
- **CPU:** Minimal (background polling)

### Fast Configuration (Testing)
- **Health Check Interval:** 100 milliseconds
- **Timeout:** 50 milliseconds per check
- **Use Case:** Integration tests requiring rapid detection

### Thread Safety
- All methods are async and thread-safe
- Uses `Arc<RwLock<>>` for shared state
- `AtomicBool` for health status in MockLlmProvider

## Recommendations

### Immediate Actions

1. **Re-enable Test 1** (`test_automatic_provider_failover`)
   - Remove `#[ignore]` on line 456
   - Uncomment lines 459-545
   - Run: `cargo test test_automatic_provider_failover`

2. **Re-enable Test 2** (`test_comprehensive_error_handling_and_recovery`)
   - Remove `#[ignore]` on line 802
   - Uncomment lines 805-846
   - Run: `cargo test test_comprehensive_error_handling_and_recovery`

3. **Update Test Documentation**
   - Remove TODO comments referencing non-existent builder
   - Update comments to reflect actual implementation

### Future Enhancements

1. **Circuit Breaker Integration**
   - Already supported via FailoverManager
   - Can add circuit breaker state to HealthCheckResult

2. **Metrics Persistence**
   - Consider persisting metrics across restarts
   - Add metrics export to Prometheus/OpenTelemetry

3. **Advanced Health Checks**
   - Support custom health check logic per provider
   - Add synthetic transaction testing

4. **Dynamic Configuration**
   - Support runtime configuration updates
   - Add per-provider health check intervals

## Code Examples

### Basic Usage
```rust
use std::sync::Arc;
use std::time::Duration;
use riptide_intelligence::{HealthMonitorBuilder, MockLlmProvider};

#[tokio::main]
async fn main() {
    // Create health monitor
    let monitor = Arc::new(
        HealthMonitorBuilder::new()
            .with_interval(Duration::from_secs(30))
            .with_timeout(Duration::from_secs(10))
            .with_failure_threshold(3)
            .build()
    );

    // Create and add provider
    let provider = Arc::new(MockLlmProvider::with_name("test-provider"));
    monitor.add_provider("test-provider".to_string(), provider).await;

    // Start monitoring
    monitor.start().await.unwrap();

    // Query health status
    let health = monitor.get_provider_health("test-provider").await;
    println!("Provider health: {:?}", health);

    // Stop monitoring
    monitor.stop().await;
}
```

### Testing Health Recovery
```rust
#[tokio::test]
async fn test_health_recovery() {
    let monitor = Arc::new(HealthMonitorBuilder::new().build());
    let provider = Arc::new(MockLlmProvider::new());

    // Start unhealthy
    provider.set_healthy(false);
    monitor.add_provider("test".to_string(), provider.clone()).await;

    // Check unhealthy
    let result = monitor.check_provider("test").await.unwrap();
    assert!(!result.success);

    // Recover
    provider.set_healthy(true);

    // Check healthy
    let result = monitor.check_provider("test").await.unwrap();
    assert!(result.success);
}
```

## Conclusion

**The HealthMonitorBuilder API is fully implemented and production-ready.** Both disabled integration tests can be immediately re-enabled by removing the `#[ignore]` attributes and uncommenting the test code. No new implementation is required.

The existing API provides:
- ✅ Flexible builder pattern for configuration
- ✅ Comprehensive health monitoring capabilities
- ✅ Integration with MockLlmProvider for testing
- ✅ Seamless failover manager integration
- ✅ Rich metrics and event system
- ✅ Thread-safe async operations

**Next Steps:**
1. Remove `#[ignore]` attributes from both tests
2. Uncomment test code
3. Run tests to verify functionality
4. Update any outdated comments in test files
