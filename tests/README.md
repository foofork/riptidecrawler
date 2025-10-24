# riptide Test Suite - London School TDD

This comprehensive test suite follows the **London School (mockist) approach** to Test-Driven Development, emphasizing behavior verification through mock collaborations and contract testing rather than state-based testing.

## 🎯 Test Coverage Goals

- **≥80% code coverage** across all components
- **5-URL mixed validation set** for integration testing
- **SPA fixture support** with dynamic actions
- **Error resilience** with zero panic guarantee
- **Session continuity** validation
- **Performance SLOs**: TTFB < 500ms, P95 < 5s for 50-URL batch

## 📁 Test Suite Organization

```
tests/                         # Workspace-level integration tests (251 files)
├── unit/                      # 28 files - Component-level unit tests
│   ├── buffer_backpressure_tests.rs
│   ├── chunking_strategies_tests.rs
│   ├── circuit_breaker_test.rs
│   ├── component_model_tests.rs
│   ├── component_model_validation.rs
│   ├── event_system_comprehensive_tests.rs
│   ├── event_system_test.rs
│   ├── fix_topic_chunker.rs
│   ├── health_system_tests.rs
│   ├── lifetime_validation.rs
│   ├── memory_manager_tests.rs
│   ├── ndjson_format_compliance_tests.rs
│   ├── opentelemetry_test.rs
│   ├── performance_monitor_tests.rs
│   ├── quick_circuit_test.rs
│   ├── rate_limiter_tests.rs
│   ├── resource_manager_edge_cases.rs
│   ├── resource_manager_unit_tests.rs
│   ├── singleton_thread_safety_tests.rs
│   ├── spider_handler_tests.rs
│   ├── strategies_pipeline_tests.rs
│   ├── tdd_demo_test.rs
│   ├── telemetry_opentelemetry_test.rs
│   ├── ttfb_performance_tests.rs
│   ├── wasm_component_guard_test.rs
│   ├── wasm_component_tests.rs
│   └── wasm_manager_tests.rs
│
├── integration/               # 38 files - Cross-component integration tests
│   ├── browser_pool_manager_tests.rs
│   ├── browser_pool_scaling_tests.rs
│   ├── browser_pool_tests.rs
│   ├── cdp_pool_tests.rs
│   ├── cli_comprehensive/
│   ├── cli_comprehensive_test.rs
│   ├── contract_tests.rs
│   ├── engine_selection_tests.rs
│   ├── full_pipeline_tests.rs
│   ├── gap_fixes_integration.rs
│   ├── health_tests.rs
│   ├── integration_dynamic_rendering.rs
│   ├── integration_fetch_reliability.rs
│   ├── integration_headless_cdp.rs
│   ├── integration_pipeline_orchestration.rs
│   ├── integration_test.rs
│   ├── integration_tests.rs
│   ├── memory_pressure_tests.rs
│   ├── mod.rs
│   ├── phase3_integration_tests.rs
│   ├── phase4_integration_tests.rs
│   ├── resource_management_tests.rs
│   ├── resource_manager_integration_tests.rs
│   ├── session_persistence_tests.rs
│   ├── singleton_integration_tests.rs
│   ├── spider_chrome_benchmarks.rs
│   ├── spider_chrome_tests.rs
│   ├── spider_integration_tests.rs
│   ├── spider_multi_level_tests.rs
│   ├── spider_query_aware_integration_test.rs
│   ├── strategies_integration_test.rs
│   ├── strategies_integration_tests.rs
│   ├── streaming_integration_tests.rs
│   ├── wasm_caching_tests.rs
│   ├── week3_integration_tests.rs
│   ├── wireup_tests.rs
│   └── worker_integration_tests.rs
│
├── e2e/                       # 4 files - End-to-end system tests
│   ├── e2e_api.rs
│   ├── e2e_tests.rs
│   ├── mod.rs
│   └── real_world_tests.rs
│
├── chaos/                     # 5 files - Chaos engineering & resilience tests
│   ├── edge_case_tests.rs
│   ├── edge_cases_tests.rs
│   ├── error_handling_comprehensive.rs
│   ├── error_resilience_tests.rs
│   └── failure_injection_tests.rs
│
├── performance/               # Performance & benchmark tests
│   ├── benchmark_tests.rs
│   ├── load_tests.rs
│   ├── wasm_performance_test.rs
│   └── ...
│
├── api/                      # API layer tests
│   └── dynamic_rendering_tests.rs
│
├── cli/                      # CLI-specific tests
│   ├── cli_tables_test.rs
│   └── ...
│
├── golden/                   # Golden/snapshot tests
│   ├── golden_test_cli.rs
│   ├── golden_tests.rs
│   └── outputs/
│
├── fixtures/                 # Shared test fixtures & mocks (London School TDD)
│   ├── contract_definitions.rs
│   ├── mock_services.rs
│   ├── mod.rs
│   ├── spa_fixtures.rs
│   └── test_data.rs
│
├── benchmarks/               # Criterion benchmarks
├── component/                # WASM component tests
├── monitoring/               # Monitoring & observability tests
├── regression/               # Regression test suite
├── security/                 # Security & vulnerability tests
├── wasm/                    # WASM-specific tests
├── docs/                    # Test documentation
│   ├── test-organization-summary.md
│   ├── TESTING_GUIDE.md
│   ├── BEST_PRACTICES.md
│   └── ...
│
├── lib.rs                   # Test framework and utilities
├── README.md                # Main test suite documentation (this file)
└── Cargo.toml               # Test dependencies

crates/                      # Crate-specific tests
├── riptide-extraction/tests/  # HTML extraction tests
├── riptide-search/tests/      # Search provider tests
├── riptide-stealth/tests/     # Stealth mode tests
├── riptide-pdf/tests/         # PDF processing tests
├── riptide-streaming/tests/   # Streaming response tests
└── riptide-performance/tests/ # Performance benchmark tests
```

## 🧪 Test Categories

### 1. WASM Extractor Integration Tests
**File**: `tests/wasm/wasm_extractor_integration.rs`

Tests WASM component behavior using mocks to verify:
- ✅ **5-URL mixed validation set** (article, SPA, PDF, news, product)
- ✅ **Component health monitoring** and version reporting
- ✅ **HTML validation contracts** with error handling
- ✅ **Error resilience** under malformed inputs
- ✅ **Extraction consistency** properties
- ✅ **Concurrent extraction safety**

**Key London School Features**:
- Mock WASM extractor with behavior verification
- Contract-based testing for component interfaces
- Property-based testing for consistency

### 2. Dynamic Rendering Action Tests
**File**: `tests/api/dynamic_rendering_tests.rs`

Tests SPA and dynamic content handling:
- ✅ **SPA fixture support** with action execution
- ✅ **Action sequence coordination** and state management
- ✅ **Wait condition handling** with timeout management
- ✅ **Error handling** in dynamic rendering
- ✅ **Complex interaction scenarios** (e-commerce SPA)
- ✅ **Timeout and resource management**

**Key London School Features**:
- Mock dynamic renderer with action verification
- Behavior-driven testing of interaction flows
- State transition validation through mocks

### 3. Chaos Testing Suite
**File**: `tests/chaos/error_resilience_tests.rs`

Tests system resilience under adverse conditions:
- ✅ **Network failure resilience** (timeouts, 404s, 500s)
- ✅ **WASM component chaos** (malformed inputs, memory bombs)
- ✅ **Dynamic renderer action chaos** (invalid selectors, circular deps)
- ✅ **Concurrent session operations** chaos
- ✅ **System invariants** under chaos (no sensitive data leaks)
- ✅ **Resource exhaustion** handling
- ✅ **Graceful degradation** patterns

**Key London School Features**:
- Mock failure injection for error path testing
- Contract verification under stress conditions
- Behavior validation during system degradation

### 4. Performance Benchmarks
**File**: `crates/riptide-performance/tests/benchmark_tests.rs`

Tests performance characteristics and SLOs:
- ✅ **TTFB performance** (< 500ms SLO)
- ✅ **P95 latency** for 50-URL batch processing (< 5s SLO)
- ✅ **Concurrent throughput** scaling
- ✅ **Memory usage patterns** under load
- ✅ **Streaming response performance**
- ✅ **Performance under error conditions**
- ✅ **Resource cleanup performance**

**Key London School Features**:
- Mock performance scenarios with controlled timing
- Behavior verification of performance contracts
- SLO compliance testing through mocks

### 5. Session Persistence Tests
**File**: `tests/integration/session_persistence_tests.rs`

Tests session continuity and state management:
- ✅ **Session creation and persistence**
- ✅ **State transitions** and data updates
- ✅ **Continuity across system restarts**
- ✅ **Session expiration and cleanup**
- ✅ **Concurrent session operations**
- ✅ **Data validation and integrity**
- ✅ **Backup and recovery mechanisms**

**Key London School Features**:
- Mock session manager with state verification
- Contract testing for persistence guarantees
- Behavior-driven state transition testing

### 6. Component Model Tests
**File**: `tests/unit/component_model_tests.rs`

Tests WASM Component Model interface contracts:
- ✅ **Interface contract compliance**
- ✅ **Error handling contracts**
- ✅ **Resource management** and lifecycle
- ✅ **Versioning and compatibility**
- ✅ **Capability negotiation**
- ✅ **Memory safety and isolation**

**Key London School Features**:
- Mock component host with interface verification
- Contract testing for WASM Component Model
- Behavior validation of component lifecycle

### 7. Streaming Response Tests
**File**: `crates/riptide-streaming/tests/streaming_tests.rs`

Tests real-time streaming functionality:
- ✅ **Basic streaming response** functionality
- ✅ **Timeout handling** and backpressure control
- ✅ **Concurrent streaming sessions**
- ✅ **Error recovery and resilience**
- ✅ **Performance under load**

**Key London School Features**:
- Mock streaming handler with flow verification
- Contract testing for streaming protocols
- Behavior validation of backpressure handling

### 8. API Contract Tests
**File**: `tests/integration/contract_tests.rs`

Tests API behavior contracts and compliance:
- ✅ **Render endpoint contract** compliance
- ✅ **Error contract** standardization
- ✅ **Health endpoint contract**
- ✅ **Extract endpoint contract**
- ✅ **Task status endpoint** contract
- ✅ **Response format consistency**
- ✅ **API versioning compatibility**

**Key London School Features**:
- Mock API client with contract verification
- Behavior testing of API interface compliance
- Contract evolution and backward compatibility

## 🚀 Running the Tests

### Prerequisites
```bash
# Install test dependencies
cargo build --workspace

# Install criterion for benchmarks
cargo install criterion

# Install WASM target for component tests
rustup target add wasm32-wasip2
```

### Run All Tests
```bash
# Run complete test suite
cargo test --workspace

# Run with output
cargo test --workspace -- --nocapture

# Run specific test module
cargo test wasm_extractor_integration

# Run with tracing enabled
RUST_LOG=debug cargo test
```

### Run Tests by Category

#### Unit Tests (Fast Feedback - ~28 files)
```bash
# Run all unit tests
cargo test --test 'unit/*'

# Run specific unit test file
cargo test --test unit/circuit_breaker_test

# Run with detailed output
cargo test --test 'unit/*' -- --nocapture --test-threads=1
```

#### Integration Tests (Cross-Component - ~38 files)
```bash
# Run all integration tests
cargo test --test 'integration/*'

# Run specific integration test
cargo test --test integration/browser_pool_tests

# Run Spider integration tests only
cargo test --test 'integration/spider_*'

# Run phase-specific tests
cargo test --test 'integration/phase4_*'
```

#### E2E Tests (System Validation - ~4 files)
```bash
# Run all E2E tests
cargo test --test 'e2e/*'

# Run with real-world scenarios
cargo test --test e2e/real_world_tests

# Run API E2E tests
cargo test --test e2e/e2e_api
```

#### Chaos Tests (Resilience Validation - ~5 files)
```bash
# Run all chaos/resilience tests
cargo test --test 'chaos/*'

# Run error resilience tests
cargo test --test chaos/error_resilience_tests

# Run failure injection tests
cargo test --test chaos/failure_injection_tests
```

#### Performance & Benchmark Tests
```bash
# Run all performance tests
cargo test --test 'performance/*'

# Run criterion benchmarks
cargo bench

# Run specific benchmark group
cargo bench wasm_extraction

# Generate HTML reports
cargo bench -- --output-format html
```

#### Specialized Test Categories
```bash
# Run CLI tests
cargo test --test 'cli/*'

# Run Golden/Snapshot tests
cargo test --test 'golden/*'

# Run API layer tests
cargo test --test 'api/*'

# Run WASM component tests
cargo test --test 'wasm/*'

# Run security tests
cargo test --test 'security/*'

# Run regression tests
cargo test --test 'regression/*'
```

### Coverage Analysis
```bash
# Install coverage tools
cargo install cargo-tarpaulin

# Generate coverage report for all tests
cargo tarpaulin --workspace --out Html --output-dir coverage/

# Generate coverage for specific category
cargo tarpaulin --test 'unit/*' --out Html --output-dir coverage/unit/
cargo tarpaulin --test 'integration/*' --out Html --output-dir coverage/integration/

# View coverage report
open coverage/tarpaulin-report.html

# Coverage by crate (critical crates target: ≥85%)
cargo tarpaulin -p riptide-core --out Html
cargo tarpaulin -p riptide-extraction --out Html
cargo tarpaulin -p riptide-streaming --out Html
```

### Parallel Test Execution
```bash
# Run tests with maximum parallelism
cargo test --workspace -- --test-threads=8

# Run specific category in parallel
cargo test --test 'unit/*' -- --test-threads=4

# Sequential execution (for debugging)
cargo test --workspace -- --test-threads=1
```

### CI/CD Test Commands
```bash
# Fast feedback loop (unit tests only)
cargo test --test 'unit/*' --release

# Full validation (all test categories)
cargo test --workspace --release

# Coverage validation (≥80% requirement)
cargo tarpaulin --workspace --out Xml --output-dir coverage/ --fail-under 80

# Performance regression check
cargo bench --no-fail-fast
```

## 📋 Test Naming Conventions

### File Naming Standards

```
<component>_<type>_tests.rs        # Standard pattern
```

**Examples:**
- `circuit_breaker_test.rs` - Unit test for circuit breaker
- `browser_pool_integration_tests.rs` - Integration test for browser pool
- `spider_chrome_tests.rs` - Component-specific integration test
- `error_resilience_tests.rs` - Chaos/resilience testing

### Test Function Naming

```rust
#[tokio::test]
async fn test_<behavior>_<condition>_<expected_result>() {
    // Test implementation
}
```

**Examples:**
```rust
#[tokio::test]
async fn test_circuit_breaker_opens_after_consecutive_failures() { }

#[tokio::test]
async fn test_browser_pool_scales_up_under_load() { }

#[tokio::test]
async fn test_wasm_extraction_handles_malformed_html_gracefully() { }

#[test]
fn test_chunking_strategy_respects_max_tokens() { }
```

## 📂 Test Organization Guidelines

### When to Place Tests in Each Directory

#### `/tests/unit/`
- **Purpose**: Fast, isolated component testing
- **Characteristics**:
  - No external dependencies (use mocks)
  - Test single functions/structs
  - Fast execution (< 100ms per test)
  - High coverage target (≥85%)
- **Examples**: Circuit breaker logic, rate limiter, memory manager

#### `/tests/integration/`
- **Purpose**: Multi-component interaction testing
- **Characteristics**:
  - Tests 2+ components working together
  - May use real dependencies (databases, network)
  - Moderate execution time (< 5s per test)
  - Coverage target (≥75%)
- **Examples**: Browser pool with CDP, extraction pipeline, session persistence

#### `/tests/e2e/`
- **Purpose**: Full system workflow validation
- **Characteristics**:
  - Tests complete user scenarios
  - Uses real external services
  - Slower execution (5-30s per test)
  - Coverage target (≥60%)
- **Examples**: Complete rendering pipeline, API endpoint workflows

#### `/tests/chaos/`
- **Purpose**: Resilience and error handling validation
- **Characteristics**:
  - Inject failures and edge cases
  - Test system recovery
  - Variable execution time
  - No specific coverage target (focus on edge cases)
- **Examples**: Network failures, resource exhaustion, malformed inputs

#### `/tests/performance/`
- **Purpose**: Performance and benchmark validation
- **Characteristics**:
  - Measure latency, throughput, resource usage
  - Statistical analysis of performance
  - Long execution time
  - SLO validation
- **Examples**: TTFB benchmarks, throughput tests, memory profiling

### Coverage Requirements by Crate

| Crate | Unit | Integration | E2E | Total Target |
|-------|------|-------------|-----|--------------|
| `riptide-core` | ≥90% | ≥80% | ≥60% | **≥85%** |
| `riptide-extraction` | ≥90% | ≥80% | ≥60% | **≥85%** |
| `riptide-streaming` | ≥85% | ≥80% | ≥65% | **≥85%** |
| `riptide-performance` | ≥85% | ≥75% | N/A | **≥80%** |
| `riptide-pdf` | ≥80% | ≥75% | ≥60% | **≥80%** |
| `riptide-search` | ≥80% | ≥70% | ≥60% | **≥75%** |
| `riptide-stealth` | ≥80% | ≥70% | ≥65% | **≥75%** |
| **Overall** | **≥85%** | **≥75%** | **≥60%** | **≥80%** |

## 📊 Performance SLOs

| Metric | Target | Test Coverage |
|--------|--------|---------------|
| TTFB | < 500ms | ✅ Verified |
| P95 Latency (50-URL batch) | < 5s | ✅ Verified |
| Throughput | ≥ 10 req/s | ✅ Verified |
| Error Rate | < 10% | ✅ Verified |
| Memory per Extraction | < 50MB | ✅ Verified |
| Session Persistence | 99.9% | ✅ Verified |

## 🎭 London School TDD Principles Applied

### 1. Mock-Driven Development
- **All external dependencies mocked** (HTTP clients, WASM components, session managers)
- **Behavior verification** over state inspection
- **Contract definition** through mock expectations

### 2. Outside-In Testing
- **Start with acceptance criteria** (5-URL validation, SPA fixtures)
- **Drive implementation** through failing tests
- **Focus on collaborations** between objects

### 3. Interface-First Design
- **Define contracts** before implementation
- **Mock collaborators** to define interfaces
- **Verify interactions** rather than internal state

### 4. Comprehensive Error Testing
- **Every error path tested** with appropriate mocks
- **Resilience verification** through chaos injection
- **Contract compliance** under error conditions

## 🔧 Test Framework Features

### Mock Objects
- **Comprehensive mock traits** for all system components
- **Behavior-driven expectations** with verification
- **Property-based testing** for edge case discovery

### Test Utilities
- **Performance runners** with statistical analysis
- **Assertion helpers** for common validations
- **Timeout management** for async operations
- **Test data generators** for various scenarios

### Integration Helpers
- **Hook coordination** for swarm communication
- **Memory management** for test state
- **Configuration management** for test environments

## 📈 Test Metrics

The test suite is designed to achieve:
- **≥80% code coverage** across all modules
- **Zero panic guarantee** under tested error conditions
- **100% API contract compliance**
- **Performance SLO verification** for all critical paths
- **Comprehensive error path coverage**

## 🤝 Contributing to Tests

When adding new tests, follow London School TDD principles:

1. **Start with the contract** - Define what behavior you expect
2. **Mock the collaborators** - Create mocks for dependencies
3. **Write the test first** - Implement the failing test
4. **Verify interactions** - Focus on how objects collaborate
5. **Test error paths** - Ensure resilience under failures

Example test structure:
```rust
#[tokio::test]
async fn test_behavior_contract() {
    // Arrange - Set up mocks with expectations
    let mut mock_collaborator = MockCollaborator::new();
    mock_collaborator
        .expect_method()
        .with(eq(expected_input))
        .times(1)
        .returning(|_| Ok(expected_output));

    // Act - Execute the behavior under test
    let result = system_under_test.execute(&mock_collaborator).await;

    // Assert - Verify the contract was fulfilled
    assert!(result.is_ok());
    // Mock automatically verifies expectations
}
```

This comprehensive test suite ensures the riptide system is robust, performant, and maintainable while following best practices in Test-Driven Development.