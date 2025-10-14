# Stratified WASM Pool Benchmarks

## Overview

Comprehensive performance benchmarking suite for the P2-1 stratified instance pool implementation. This document describes the benchmark methodology, scenarios, and expected performance targets for the 3-tier WASM instance pool architecture.

## Architecture

The Stratified Instance Pool implements a 3-tier caching strategy:

- **Hot Tier**: Instantly available instances (0-5ms acquisition)
- **Warm Tier**: Fast activation instances (10-50ms acquisition)
- **Cold Tier**: On-demand creation instances (100-200ms baseline)

### Design Goals

- **40-60% latency reduction** compared to simple queue baseline
- **70% hot tier hit rate** under typical workloads
- **Automatic promotion** of frequently-used instances
- **Efficient memory utilization** with tiered eviction

## Benchmark Suite

### 1. Hot Tier Acquisition (`bench_hot_tier_acquisition`)

**Target**: 0-5ms acquisition latency

**Test Scenarios**:
- Hot tier sizes: 1, 2, 4, 8 instances
- Measures pure acquisition overhead from hot tier
- Validates instant availability promise

**Expected Results**:
```
hot_tier_size/1    500 ns - 2 μs
hot_tier_size/2    500 ns - 2 μs
hot_tier_size/4    600 ns - 2.5 μs
hot_tier_size/8    700 ns - 3 μs
```

**Key Metrics**:
- Mean acquisition time
- Standard deviation
- P50, P95, P99 latencies

### 2. Warm Tier Acquisition (`bench_warm_tier_acquisition`)

**Target**: 10-50ms acquisition latency

**Test Scenarios**:
- Warm tier sizes: 2, 4, 8, 16 instances
- Simulates activation overhead for warm instances
- Tests fallback path when hot tier is empty

**Expected Results**:
```
warm_tier_size/2     15-40 ms
warm_tier_size/4     12-35 ms
warm_tier_size/8     10-30 ms
warm_tier_size/16    10-25 ms
```

**Key Metrics**:
- Activation time overhead
- Consistency of warm tier performance
- Fallback efficiency

### 3. Cold Tier Acquisition (`bench_cold_tier_acquisition`)

**Target**: 100-200ms baseline (for comparison)

**Test Scenarios**:
- Cold tier sizes: 4, 8, 16 instances
- Establishes baseline for new instance creation
- Validates performance improvement metrics

**Expected Results**:
```
cold_tier_size/4     100-180 ms
cold_tier_size/8     110-190 ms
cold_tier_size/16    120-200 ms
```

**Key Metrics**:
- Cold start overhead
- Baseline comparison for improvement calculations

### 4. Promotion Effectiveness (`bench_promotion_effectiveness`)

**Target**: < 100 μs for promotion decision

**Test Scenarios**:
- 12 instances with varying access frequencies (0.1 - 0.65)
- Tests automatic promotion algorithm
- Validates frequency-based tier assignment

**Expected Results**:
```
promote_warm_to_hot    50-100 μs
```

**Key Metrics**:
- Promotion decision time
- Accuracy of frequency detection
- Number of promotions per cycle

### 5. Hit Rate Tracking (`bench_hit_rate_tracking`)

**Target**: Accurate tier hit tracking with < 1% overhead

**Test Scenarios**:
- **Hot dominant**: 70% hot tier capacity (8/4/2 distribution)
- **Balanced**: Even distribution (4/8/4)
- **Cold dominant**: More cold tier instances (2/4/10)

**Expected Hit Rates**:
```
hot_dominant      75-85% hot hits
balanced          45-55% hot hits
cold_dominant     20-30% hot hits
```

**Key Metrics**:
- Hot tier hit rate
- Warm tier hit rate
- Cold tier miss rate
- Promotion count
- Metric collection overhead

### 6. Stratified vs Baseline Comparison (`bench_stratified_vs_baseline`)

**Target**: 40-60% improvement over simple queue

**Access Patterns**:

#### Hot-Biased (70/20/10)
- 70% requests hit hot tier
- 20% requests hit warm tier
- 10% requests hit cold tier

**Expected Results**:
```
stratified/hot_biased     1-3 μs   (fast path dominant)
simple_queue/hot_biased   2-5 μs   (no optimization)

Improvement: 40-50%
```

#### Uniform (33/33/34)
- Even distribution across all tiers

**Expected Results**:
```
stratified/uniform        10-25 ms  (mixed performance)
simple_queue/uniform      15-35 ms  (average case)

Improvement: 30-40%
```

#### Cold-Start (10/30/60)
- Majority of requests miss hot cache
- Simulates system startup or cache invalidation

**Expected Results**:
```
stratified/cold_start     50-100 ms  (gradual warming)
simple_queue/cold_start   80-140 ms  (no optimization)

Improvement: 25-35%
```

### 7. Load Patterns (`bench_load_patterns`)

**Test Scenarios**:

#### Sequential Access
- 100 consecutive acquire/release operations
- Tests steady-state performance

**Expected Results**:
```
sequential_access    50-150 μs per operation
```

#### Burst Access
- 5 rapid acquires followed by 5 rapid releases
- Tests pool under spike load

**Expected Results**:
```
burst_access    250-500 μs per burst cycle
```

**Key Metrics**:
- Throughput under different load patterns
- Pool stability during bursts
- Queue depth variations

### 8. Metrics Overhead (`bench_metrics_overhead`)

**Target**: < 1 μs overhead for metrics collection

**Test Scenarios**:
- Pure metrics retrieval
- Metrics during acquisition

**Expected Results**:
```
get_metrics              200-500 ns
acquire_with_metrics     1-4 μs
```

**Key Metrics**:
- Metrics collection time
- Overhead percentage
- Impact on acquisition latency

### 9. Concurrent Simulation (`bench_concurrent_simulation`)

**Test Scenarios**:
- Simulated thread counts: 2, 4, 8
- Rapid acquire/release cycles

**Expected Results**:
```
threads/2     10-20 μs per operation
threads/4     15-30 μs per operation
threads/8     20-40 μs per operation
```

**Key Metrics**:
- Scalability under concurrent access
- Contention effects
- Lock overhead

### 10. Memory Efficiency (`bench_memory_efficiency`)

**Test Scenarios**:
- Total instance counts: 8, 16, 32, 64
- Tests pool scaling behavior

**Expected Results**:
```
total_instances/8     < 100 KB overhead
total_instances/16    < 200 KB overhead
total_instances/32    < 400 KB overhead
total_instances/64    < 800 KB overhead
```

**Key Metrics**:
- Memory overhead per instance
- Metadata efficiency
- Scaling characteristics

## Running Benchmarks

### Basic Execution

```bash
# Run all stratified pool benchmarks
cargo bench --bench stratified_pool_bench --features benchmarks

# Run specific benchmark
cargo bench --bench stratified_pool_bench --features benchmarks -- hot_tier

# Generate HTML reports
cargo bench --bench stratified_pool_bench --features benchmarks -- --save-baseline v1
```

### Baseline Comparison

```bash
# Save baseline
cargo bench --bench stratified_pool_bench --features benchmarks -- --save-baseline baseline-simple-queue

# Implement stratified pool
# ... make changes ...

# Compare against baseline
cargo bench --bench stratified_pool_bench --features benchmarks -- --baseline baseline-simple-queue
```

### Continuous Benchmarking

```bash
# Monitor performance over time
cargo bench --bench stratified_pool_bench --features benchmarks -- --save-baseline $(git rev-parse --short HEAD)

# View historical data
ls target/criterion/
```

## Benchmark Validation Criteria

### Performance Targets

✅ **Hot Tier**: < 5ms acquisition (target: 0-2ms)
✅ **Warm Tier**: < 50ms acquisition (target: 10-30ms)
✅ **Cold Tier**: 100-200ms baseline
✅ **Overall Improvement**: 40-60% vs simple queue
✅ **Hot Hit Rate**: > 70% under typical load
✅ **Metrics Overhead**: < 1 μs per operation

### Quality Metrics

- **Consistency**: P95/P50 ratio < 2.0 (low variance)
- **Scalability**: Linear scaling up to 64 instances
- **Memory Efficiency**: < 15 KB overhead per instance
- **Promotion Accuracy**: > 80% of frequently-used instances promoted

## Interpreting Results

### Reading Criterion Output

```
hot_tier_acquisition/hot_tier_size/4
                        time:   [1.2345 μs 1.2567 μs 1.2789 μs]
                        change: [-15.234% -12.567% -9.890%] (p = 0.00 < 0.05)
                        Performance has improved.
```

- **First line**: Benchmark name and parameters
- **Second line**: [lower bound, estimate, upper bound] with 95% confidence
- **Third line**: Change vs baseline
- **Fourth line**: Statistical significance

### Performance Regression Detection

A performance regression is indicated when:

1. Mean latency increases > 10%
2. P99 latency increases > 20%
3. Standard deviation increases > 50%
4. Hit rate decreases > 5%

### Optimization Opportunities

Look for these patterns in results:

- **High variance**: Indicates inconsistent performance (optimize hot path)
- **Linear degradation**: Pool size too small (increase capacity)
- **Poor hit rates**: Promotion threshold too high (tune frequency thresholds)
- **High overhead**: Metrics collection too expensive (optimize tracking)

## Implementation Validation

### Test Coverage

The benchmark suite validates:

✅ All three tier acquisitions (hot/warm/cold)
✅ Promotion algorithm correctness
✅ Hit rate tracking accuracy
✅ Performance vs baseline comparison
✅ Various load patterns (sequential, burst, random)
✅ Scalability under concurrent access
✅ Memory efficiency at scale

### Expected vs Actual

After implementation, compare actual results against targets:

| Benchmark | Target | Actual | Status |
|-----------|--------|--------|--------|
| Hot tier | 0-5ms | _TBD_ | ⏳ |
| Warm tier | 10-50ms | _TBD_ | ⏳ |
| Cold tier | 100-200ms | _TBD_ | ⏳ |
| Improvement | 40-60% | _TBD_ | ⏳ |
| Hit rate | 70%+ | _TBD_ | ⏳ |

## Troubleshooting

### Common Issues

**Problem**: Hot tier acquisition > 5ms
- **Cause**: Excessive locking or contention
- **Solution**: Use lock-free data structures or reduce critical sections

**Problem**: Hit rate < 70%
- **Cause**: Promotion threshold too conservative
- **Solution**: Lower access_frequency threshold for hot tier

**Problem**: High variance in results
- **Cause**: Background system activity or GC pauses
- **Solution**: Run benchmarks in isolated environment, increase sample size

**Problem**: Memory overhead > 15 KB/instance
- **Cause**: Excessive metadata or history tracking
- **Solution**: Reduce history buffer size, optimize metadata structures

## Statistical Methodology

### Criterion Configuration

- **Sample size**: 100-1000 iterations per benchmark
- **Measurement time**: 5-15 seconds per benchmark
- **Warm-up time**: 3 seconds
- **Confidence level**: 95%
- **Outlier detection**: Enabled (automatic)

### Noise Reduction

- Run benchmarks in CI with dedicated resources
- Disable CPU frequency scaling
- Close background applications
- Use consistent WASM module for testing
- Multiple iterations for statistical significance

## Future Enhancements

### Planned Benchmarks

1. **Multi-tenant scenarios**: Different workloads per tenant
2. **Memory pressure**: Behavior under memory constraints
3. **Adaptive tuning**: Dynamic threshold adjustments
4. **Real WASM modules**: Using actual extraction workloads
5. **Long-running stability**: 24-hour stress testing

### Integration with CI

```yaml
# .github/workflows/benchmarks.yml
- name: Run Stratified Pool Benchmarks
  run: |
    cargo bench --bench stratified_pool_bench --features benchmarks -- --save-baseline ci-${{ github.sha }}

- name: Compare vs Baseline
  run: |
    cargo bench --bench stratified_pool_bench --features benchmarks -- --baseline ci-main
```

## References

- **Implementation**: `/workspaces/eventmesh/crates/riptide-core/src/memory_manager.rs`
- **Benchmark Code**: `/workspaces/eventmesh/crates/riptide-core/benches/stratified_pool_bench.rs`
- **Criterion Docs**: https://bheisler.github.io/criterion.rs/book/
- **WASM Performance**: https://wasmtime.dev/

## Conclusion

This benchmark suite provides comprehensive validation of the stratified pool implementation. The combination of micro-benchmarks (individual tier performance) and macro-benchmarks (real-world access patterns) ensures that the 40-60% latency improvement target is measurable and achievable.

Regular execution of these benchmarks during development will catch performance regressions early and guide optimization efforts.

---

**Document Version**: 1.0
**Last Updated**: 2025-10-14
**Benchmark Suite Version**: 0.1.0
