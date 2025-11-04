# Memory Leak Detection

Comprehensive memory leak detection system for the riptide-api memory manager.

## Overview

The leak detection system monitors allocation/deallocation patterns to identify potential memory leaks before they become critical. It provides:

- **Baseline tracking**: Compare current memory usage against historical baselines
- **Growth rate analysis**: Calculate bytes-per-minute growth rates
- **Component attribution**: Track which components are leaking memory
- **Severity classification**: Categorize leaks by impact (Low/Medium/High/Critical)
- **Actionable recommendations**: Get specific guidance on addressing leaks

## Architecture

### LeakDetector

The core `LeakDetector` struct provides thread-safe leak detection with:

```rust
pub struct LeakDetector {
    baseline: RwLock<Option<MemorySnapshot>>,
    history: RwLock<VecDeque<MemorySnapshot>>,
    component_tracking: RwLock<HashMap<String, ComponentMemory>>,
    config: LeakDetectionConfig,
}
```

**Key Features:**
- Baseline memory snapshots for comparison
- Historical memory samples (up to 100 by default)
- Per-component allocation tracking
- Configurable detection thresholds

### LeakReport

Detection results are returned as a comprehensive `LeakReport`:

```rust
pub struct LeakReport {
    pub has_leaks: bool,
    pub overall_growth_rate: f64,
    pub growth_mb_per_minute: f64,
    pub time_window_secs: u64,
    pub leak_candidates: Vec<LeakCandidate>,
    pub recommendations: Vec<String>,
    pub baseline_mb: Option<usize>,
    pub current_mb: usize,
    pub timestamp: u64,
}
```

## Usage

### Setting Baseline

Establish a baseline for leak comparison:

```rust
memory_manager.set_leak_baseline();
```

This captures the current memory state as the reference point for future leak detection.

### Tracking Allocations

Track allocations with component attribution:

```rust
// Track allocation for specific component
memory_manager.track_allocation_by_component("cache", 100);

// Track deallocation
memory_manager.track_deallocation_by_component("cache", 50);
```

### Detecting Leaks

Generate a leak report:

```rust
let report = memory_manager.detect_leaks();

if report.has_leaks {
    println!("Growth rate: {:.2}%", report.overall_growth_rate);
    println!("Growth: {:.2} MB/min", report.growth_mb_per_minute);

    for candidate in &report.leak_candidates {
        println!(
            "Component '{}': {} MB ({:?})",
            candidate.component,
            candidate.allocated_mb,
            candidate.severity
        );
    }

    for recommendation in &report.recommendations {
        println!("💡 {}", recommendation);
    }
}
```

## API Endpoint

### GET `/api/v1/memory/leaks`

Retrieve current memory leak report.

**Response:**

```json
{
  "has_leaks": true,
  "overall_growth_rate": 15.3,
  "growth_mb_per_minute": 2.5,
  "time_window_secs": 600,
  "leak_candidates": [
    {
      "component": "cache",
      "allocated_mb": 1250,
      "net_allocations": 100,
      "age_secs": 3600,
      "severity": "Critical",
      "growth_rate": 85.2
    }
  ],
  "recommendations": [
    "CRITICAL: Component 'cache' has 1250 MB allocated with 85.2% growth rate",
    "Consider triggering garbage collection or cleanup"
  ],
  "baseline_mb": 100,
  "current_mb": 1350,
  "timestamp": 1699012345
}
```

**Example Request:**

```bash
curl http://localhost:8080/api/v1/memory/leaks
```

## Configuration

Customize leak detection behavior:

```rust
use riptide_api::resource_manager::memory_manager::LeakDetectionConfig;

let config = LeakDetectionConfig {
    growth_threshold: 5.0,              // 5% growth threshold
    time_window_secs: 600,              // 10-minute window
    max_history_samples: 100,           // Keep 100 samples
    min_allocations_for_leak: 10,      // Minimum 10 allocations
};

let detector = LeakDetector::with_config(config);
```

### Configuration Parameters

| Parameter | Default | Description |
|-----------|---------|-------------|
| `growth_threshold` | 5.0 | Percentage growth to trigger leak alert |
| `time_window_secs` | 600 | Time window for growth calculation (seconds) |
| `max_history_samples` | 100 | Maximum historical snapshots to retain |
| `min_allocations_for_leak` | 10 | Minimum allocations to consider as leak |

## Leak Severity

Leaks are classified into four severity levels:

### Critical
- Allocated memory > 1000 MB
- Growth rate > 80%
- **Action**: Immediate investigation required

### High
- Allocated memory > 500 MB
- Growth rate > 50%
- **Action**: Priority investigation

### Medium
- Allocated memory > 100 MB
- Growth rate > 20%
- **Action**: Monitor closely

### Low
- Below Medium thresholds
- **Action**: Track trends

## Performance

The leak detection system is optimized for production use:

- **Detection time**: < 50ms for 100+ components
- **Memory overhead**: ~1-2 MB for tracking data
- **Thread-safe**: Uses `Arc<RwLock>` for concurrent access
- **Non-blocking**: All operations are lock-free or use read locks

## Integration with Memory Manager

The `LeakDetector` is fully integrated with `MemoryManager`:

```rust
pub struct MemoryManager {
    // ... other fields
    leak_detector: Arc<LeakDetector>,
}

impl MemoryManager {
    pub fn detect_leaks(&self) -> LeakReport {
        self.leak_detector.detect_leaks(self.current_usage_mb())
    }

    pub fn set_leak_baseline(&self) {
        self.leak_detector.set_baseline(self.current_usage_mb());
    }
}
```

## Best Practices

### 1. Set Baseline Early
```rust
// During initialization
memory_manager.set_leak_baseline();
```

### 2. Use Component Attribution
```rust
// Track allocations by component
memory_manager.track_allocation_by_component("cache", size);
memory_manager.track_allocation_by_component("buffer", size);
```

### 3. Regular Detection
```rust
// Check for leaks periodically
let report = memory_manager.detect_leaks();
if report.has_leaks {
    trigger_cleanup().await;
}
```

### 4. Monitor Recommendations
```rust
// Act on recommendations
for rec in &report.recommendations {
    if rec.contains("CRITICAL") {
        // Immediate action
        memory_manager.trigger_gc().await;
    }
}
```

## Testing

The system includes 13+ comprehensive tests:

- ✅ No leaks baseline
- ✅ Gradual leak detection
- ✅ Spike detection
- ✅ Component attribution
- ✅ Threshold configuration
- ✅ Concurrent tracking
- ✅ Leak history
- ✅ False positive avoidance
- ✅ Report format validation
- ✅ Integration with memory manager
- ✅ Performance validation (<50ms)
- ✅ Severity classification
- ✅ Recommendations generation

Run tests:

```bash
cargo test --test memory_leak_detection_tests
```

## Monitoring Dashboard Integration

The leak detection API can be integrated with monitoring dashboards:

```javascript
// Fetch leak report
const response = await fetch('/api/v1/memory/leaks');
const report = await response.json();

// Display alerts
if (report.has_leaks) {
    const criticalLeaks = report.leak_candidates.filter(
        c => c.severity === 'Critical'
    );

    if (criticalLeaks.length > 0) {
        showAlert('Critical memory leaks detected!', criticalLeaks);
    }
}
```

## Troubleshooting

### High False Positive Rate

If you're seeing too many false positives:

1. Increase `min_allocations_for_leak`:
   ```rust
   config.min_allocations_for_leak = 20;
   ```

2. Adjust `growth_threshold`:
   ```rust
   config.growth_threshold = 10.0; // 10% instead of 5%
   ```

### Missing Leaks

If leaks aren't being detected:

1. Ensure baseline is set:
   ```rust
   memory_manager.set_leak_baseline();
   ```

2. Check time window:
   ```rust
   config.time_window_secs = 300; // Shorter 5-minute window
   ```

3. Lower threshold:
   ```rust
   config.growth_threshold = 2.0; // More sensitive
   ```

## Future Enhancements

Planned improvements:

- [ ] Stack trace capture for leak sources
- [ ] Automatic leak remediation
- [ ] Machine learning-based leak prediction
- [ ] Integration with distributed tracing
- [ ] Memory profiler integration
- [ ] Leak pattern recognition

## References

- [Memory Manager Documentation](../crates/riptide-api/src/resource_manager/memory_manager.rs)
- [Resource Manager API](../crates/riptide-api/src/handlers/resources.rs)
- [Test Suite](../crates/riptide-api/tests/memory_leak_detection_tests.rs)
