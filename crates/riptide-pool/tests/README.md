# Native Extractor Pool Test Suite

Comprehensive test suite for the NativeExtractorPool implementation following TDD principles.

## Test Organization

### 1. Unit Tests (`native_pool_tests.rs`)

#### Pool Lifecycle Tests
- **test_native_pool_creation**: Validates pool initialization with correct parameters
- **test_pool_warmup_creates_initial_instances**: Ensures pre-warming creates initial pool size
- **test_pool_acquire_and_release**: Tests basic acquire/release cycle
- **test_pool_shutdown_gracefully**: Verifies clean shutdown and resource cleanup
- **test_pool_rejects_operations_after_shutdown**: Ensures shutdown state is enforced

#### Health Monitoring Tests
- **test_health_check_detects_unhealthy_instances**: Validates health check logic
- **test_periodic_health_checks**: Tests automated health check intervals
- **test_idle_timeout_removes_instances**: Verifies idle instance cleanup
- **test_instance_reuse_limit**: Tests max reuse count enforcement

#### Resource Limit Tests
- **test_pool_respects_max_size**: Ensures pool size limits are honored
- **test_memory_limit_enforcement**: Validates memory limit checks
- **test_pool_exhaustion_timeout**: Tests timeout behavior when pool is exhausted

#### Metrics Collection Tests
- **test_metrics_track_extractions**: Validates extraction metrics
- **test_metrics_track_pool_utilization**: Tests utilization metrics
- **test_metrics_track_instance_lifecycle**: Verifies lifecycle event tracking

#### Concurrent Access Tests
- **test_concurrent_acquisition**: Tests parallel instance acquisition
- **test_thread_safety**: Validates thread-safe operations across 100 concurrent tasks

#### Error Handling & Recovery Tests
- **test_extraction_error_recovery**: Tests recovery from extraction failures
- **test_multiple_failures_mark_unhealthy**: Validates failure threshold logic
- **test_pool_recovers_from_all_unhealthy_instances**: Tests pool recovery

#### Configuration Validation Tests
- **test_config_validation_rejects_invalid_values**: Validates config validation logic
- **test_config_from_env_variables**: Tests environment-based configuration

#### Performance Tests
- **test_instance_reuse_performance**: Measures reuse performance benefits
- **test_parallel_extraction_performance**: Tests parallel extraction throughput

#### Integration Scenarios
- **test_realistic_extraction_workflow**: Validates end-to-end extraction
- **test_stress_test_sustained_load**: 5-second sustained load test

**Total Unit Tests**: 27

### 2. Performance Benchmarks (`benches/native_pool_bench.rs`)

#### Benchmark Groups

1. **Pooled vs Non-Pooled Extraction**
   - `with_pool`: Measures pooled extraction performance
   - `without_pool`: Baseline without pooling

2. **Instance Reuse Benefits**
   - Tests with 1, 10, 100, 1000 reuse counts
   - Measures efficiency gains from reuse

3. **Native vs WASM Comparison**
   - `native_pool`: Native extraction performance
   - `wasm_pool`: WASM extraction performance (when enabled)

4. **Concurrent Throughput**
   - Tests with 1, 2, 4, 8, 16 concurrent tasks
   - Measures scalability

5. **Memory Efficiency**
   - `small_document`: Small HTML documents
   - `large_document`: 1000+ paragraph documents

6. **Pool Overhead**
   - `acquire_release`: Pure pool operation overhead
   - `instance_creation`: Instance creation cost

7. **Realistic Workload**
   - Mixed document sizes (70% small, 20% medium, 10% large)
   - Varying concurrency patterns

**Total Benchmark Groups**: 7

### 3. Integration Tests (`riptide-api/tests/native_pool_integration.rs`)

#### End-to-End Tests
- **test_e2e_native_pool_extraction**: Full API extraction flow
- **test_e2e_batch_extraction**: Batch processing

#### Failover Scenarios
- **test_native_primary_wasm_fallback**: Normal operation with native
- **test_fallback_on_native_failure**: Failover on extraction failure
- **test_fallback_on_native_timeout**: Failover on timeout
- **test_fallback_on_pool_exhaustion**: Failover when pool exhausted
- **test_no_fallback_when_disabled**: Error when fallback disabled

#### State Integration
- **test_state_tracks_pool_metrics**: Metrics tracking in state
- **test_state_health_check_includes_pool**: Health check integration
- **test_state_graceful_shutdown**: Shutdown coordination

#### Error Propagation
- **test_extraction_errors_propagate_correctly**: Error type validation
- **test_pool_errors_include_context**: Contextual error messages

#### Concurrent Integration
- **test_concurrent_api_requests**: 100 concurrent requests
- **test_mixed_native_and_wasm_requests**: Mixed extraction methods

#### Pipeline Integration
- **test_pipeline_with_native_extraction**: Full pipeline flow

#### Metrics & Monitoring
- **test_prometheus_metrics_exposed**: Prometheus integration
- **test_distributed_tracing_integration**: Tracing integration

#### Configuration
- **test_invalid_config_rejected**: Config validation
- **test_config_from_environment**: Environment configuration

**Total Integration Tests**: 18

## Test Coverage Requirements

### Coverage Targets
- **Pool Lifecycle**: 100%
- **Health Monitoring**: 100%
- **Error Scenarios**: 100%
- **Performance Benchmarks**: Basic set
- **Overall Target**: >90%

### Key Test Scenarios

#### Critical Path Tests
1. Pool creation and warmup
2. Instance acquisition and release
3. Health checks and cleanup
4. Failover to WASM
5. Concurrent access
6. Graceful shutdown

#### Edge Cases
1. Pool exhaustion
2. All instances unhealthy
3. Configuration validation failures
4. Timeout scenarios
5. Memory limit violations
6. Concurrent stress (100+ tasks)

#### Performance Validation
1. Instance reuse benefits (expected: 30-50% faster than recreation)
2. Native vs WASM (expected: native 2-3x faster)
3. Concurrent throughput (expected: scales linearly to 8 cores)
4. Pool overhead (expected: <10μs per acquire/release)

## Running Tests

### Run All Tests
```bash
cargo test -p riptide-pool --lib --tests
```

### Run Specific Test Suite
```bash
# Unit tests only
cargo test -p riptide-pool native_pool_tests

# Integration tests
cargo test -p riptide-api native_pool_integration

# With logging
RUST_LOG=debug cargo test -p riptide-pool -- --nocapture
```

### Run Benchmarks
```bash
# All benchmarks
cargo bench -p riptide-pool

# Specific benchmark
cargo bench -p riptide-pool pooled_extraction

# With detailed output
cargo bench -p riptide-pool -- --verbose
```

### Coverage Report
```bash
# Install tarpaulin
cargo install cargo-tarpaulin

# Generate coverage
cargo tarpaulin -p riptide-pool --out Html --output-dir coverage

# Open report
open coverage/index.html
```

## Test Data

### Sample HTML Documents

#### Small Document (~500 bytes)
```html
<html>
<head><title>Small Test</title></head>
<body><article><h1>Title</h1><p>Content</p></article></body>
</html>
```

#### Medium Document (~5KB)
- 10-20 paragraphs
- Multiple sections
- Links and media

#### Large Document (~50KB)
- 100+ paragraphs
- Complex structure
- Many links and images

## Expected Benchmark Results

### Pooled vs Non-Pooled
- **Pooled**: ~80-100μs per extraction
- **Non-Pooled**: ~150-200μs per extraction
- **Speedup**: 1.5-2x

### Native vs WASM
- **Native**: ~100μs per extraction
- **WASM**: ~200-300μs per extraction
- **Speedup**: 2-3x

### Concurrent Throughput
- **1 task**: ~10 extractions/sec
- **4 tasks**: ~40 extractions/sec
- **8 tasks**: ~80 extractions/sec
- **16 tasks**: ~120 extractions/sec (contention)

### Memory Usage
- **Per Instance**: ~10-20MB
- **Small Doc**: +1-2MB
- **Large Doc**: +5-10MB

## Test Coordination

### Hooks Integration

All tests follow the coordination protocol:

```bash
# Before tests
npx claude-flow@alpha hooks pre-task --description "Run Native Pool Tests"

# After tests
npx claude-flow@alpha hooks post-task --task-id "native-pool-tests"

# Store results
npx claude-flow@alpha hooks post-edit \
  --file "crates/riptide-pool/tests/native_pool_tests.rs" \
  --memory-key "swarm/tester/test-results"
```

### Memory Keys

Test results stored in coordination memory:

- `swarm/tester/test-results` - Test execution results
- `swarm/tester/coverage` - Coverage metrics
- `swarm/tester/benchmarks` - Benchmark results
- `swarm/tester/status` - Current testing status

## Success Criteria

### Must Pass
- ✅ All 27 unit tests pass
- ✅ All 18 integration tests pass
- ✅ No test warnings or errors
- ✅ Coverage >90%

### Performance Goals
- ✅ Pooled extraction faster than non-pooled
- ✅ Native extraction faster than WASM
- ✅ Concurrent throughput scales linearly (up to 8 tasks)
- ✅ Pool overhead <10μs

### Code Quality
- ✅ No clippy warnings
- ✅ All documentation complete
- ✅ Tests follow naming conventions
- ✅ Error messages are descriptive

## Troubleshooting

### Common Issues

#### Tests Timeout
- Increase timeout: `RUST_TEST_TIME_UNIT=10000 cargo test`
- Check for deadlocks in concurrent tests

#### Flaky Tests
- Add retry logic for timing-sensitive tests
- Use `tokio::time::pause()` for deterministic timing

#### Memory Issues
- Reduce concurrent task count
- Check for resource leaks
- Use `valgrind` or `heaptrack` for debugging

#### Benchmark Variance
- Ensure system is idle during benchmarks
- Run multiple iterations
- Check for background processes

## Next Steps

1. **Implementation**: Create `NativeExtractorPool` to satisfy tests
2. **Validation**: Ensure all tests pass
3. **Benchmarking**: Run performance benchmarks
4. **Integration**: Wire into API layer
5. **Documentation**: Update API docs with pool usage

## Related Files

- Implementation: `crates/riptide-pool/src/native_pool.rs` (to be created)
- Config: `crates/riptide-pool/src/config.rs`
- Integration: `crates/riptide-api/src/state.rs`
- Native Parser: `crates/riptide-extraction/src/native_parser/`
