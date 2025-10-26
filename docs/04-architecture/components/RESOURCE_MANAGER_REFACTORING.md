# Resource Manager Refactoring Plan

## Overview
This document outlines the comprehensive refactoring of the `resource_manager.rs` module to improve code organization, maintainability, and performance.

## Current State Analysis

### Strengths
- Comprehensive resource management implementation
- Good use of async/await patterns
- Proper RAII with Drop implementations
- Well-documented with inline comments
- Integration with external crates (chromiumoxide, riptide-headless)

### Areas for Improvement
1. **Separation of Concerns**: Large monolithic file (~889 lines) with multiple managers in one file
2. **Error Handling**: Using `anyhow::Result` everywhere lacks type safety
3. **Logging**: Inconsistent logging patterns across managers
4. **Performance**: Some opportunities for better async patterns
5. **Testing**: Test coverage could be improved
6. **Module Organization**: All sub-managers in one file reduces maintainability

## SPARC Methodology Application

### 1. Specification
**Requirements:**
- Maintain all existing functionality and backward compatibility
- Improve code organization and maintainability
- Enhance error handling with custom error types
- Add comprehensive logging and metrics
- Optimize async patterns where applicable
- Improve test coverage

### 2. Pseudocode

```
MODULE resource_manager:
  - Main ResourceManager coordinator
  - Delegates to specialized sub-managers

MODULE rate_limiter:
  - PerHostRateLimiter with token bucket
  - Cleanup background task
  - Host-specific metrics

MODULE wasm_manager:
  - WasmInstanceManager per-worker tracking
  - Health monitoring
  - Lifecycle management

MODULE memory_manager:
  - MemoryManager pressure detection
  - Cleanup and GC triggers
  - Usage tracking

MODULE performance_monitor:
  - Performance metrics collection
  - Degradation detection
  - Timeout tracking

MODULE errors:
  - Custom error types for each manager
  - Conversion from common error types
  - Better error context
```

### 3. Architecture

```
crates/riptide-api/src/
├── resource_manager/
│   ├── mod.rs               # Main coordinator & re-exports
│   ├── errors.rs            # Custom error types
│   ├── metrics.rs           # ResourceMetrics
│   ├── rate_limiter.rs      # PerHostRateLimiter
│   ├── wasm_manager.rs      # WasmInstanceManager
│   ├── memory_manager.rs    # MemoryManager
│   ├── performance.rs       # PerformanceMonitor
│   └── guards.rs            # Resource guards (RAII)
└── resource_manager.rs      # REMOVED (replaced by directory)
```

### 4. Refinement
- Incremental module extraction
- Preserve all public APIs
- Maintain test compatibility
- Add new tests for edge cases
- Performance benchmarks

### 5. Completion
- Integration testing
- Documentation updates
- Performance validation
- Code review readiness

## Implementation Plan

### Phase 1: Module Structure (Priority: High)
1. Create `resource_manager/` directory
2. Extract error types to `errors.rs`
3. Extract metrics to `metrics.rs`
4. Create module stub files

### Phase 2: Manager Extraction (Priority: High)
1. Extract `PerHostRateLimiter` → `rate_limiter.rs`
2. Extract `WasmInstanceManager` → `wasm_manager.rs`
3. Extract `MemoryManager` → `memory_manager.rs`
4. Extract `PerformanceMonitor` → `performance.rs`
5. Extract guards → `guards.rs`

### Phase 3: Main Coordinator (Priority: High)
1. Refactor `ResourceManager` in `mod.rs`
2. Update imports and re-exports
3. Preserve public API surface
4. Update documentation

### Phase 4: Enhancement (Priority: Medium)
1. Improve error handling with custom types
2. Add structured logging
3. Optimize async patterns
4. Add performance benchmarks

### Phase 5: Testing & Documentation (Priority: High)
1. Update existing tests
2. Add new test coverage
3. Update API documentation
4. Create usage examples

## Key Design Decisions

### 1. Error Handling
```rust
// Custom error types for better error handling
pub enum ResourceManagerError {
    BrowserPoolError(String),
    RateLimitError { retry_after: Duration },
    MemoryPressureError,
    WasmError(String),
    TimeoutError { operation: String },
    ConfigurationError(String),
}
```

### 2. Logging Strategy
- Use structured logging with `tracing`
- Consistent log levels:
  - `debug!`: Resource acquisition/release
  - `info!`: Manager initialization/shutdown
  - `warn!`: Resource pressure, cleanup triggers
  - `error!`: Failures that need attention

### 3. Performance Optimizations
- Use `tokio::spawn` for background tasks appropriately
- Minimize lock contention with fine-grained locking
- Use `Arc` efficiently to reduce cloning overhead
- Consider `parking_lot` for RwLock if profiling shows contention

### 4. Backward Compatibility
- Keep all public APIs unchanged
- Maintain same behavior for all operations
- Preserve test compatibility
- Document any subtle behavior changes

## Metrics & Success Criteria

### Code Quality
- [ ] All clippy warnings resolved
- [ ] Code organized into logical modules (<500 lines each)
- [ ] Comprehensive inline documentation
- [ ] Clear separation of concerns

### Performance
- [ ] No performance regression (benchmark comparison)
- [ ] Reduced lock contention (profiling)
- [ ] Efficient async patterns (tokio-console)

### Testing
- [ ] All existing tests pass
- [ ] New tests for edge cases added
- [ ] Integration tests updated
- [ ] Coverage >80% for new modules

### Documentation
- [ ] Architecture documentation updated
- [ ] API docs complete with examples
- [ ] Migration guide (if needed)
- [ ] Performance analysis documented

## Risk Mitigation

1. **Breaking Changes**: Extensive testing, maintain public API
2. **Performance Regression**: Benchmark before/after, profiling
3. **Concurrency Issues**: Thorough async testing, stress tests
4. **Integration Issues**: Test all dependent handlers

## Timeline Estimate

- Phase 1 (Module Structure): 1-2 hours
- Phase 2 (Manager Extraction): 3-4 hours
- Phase 3 (Main Coordinator): 2-3 hours
- Phase 4 (Enhancement): 2-3 hours
- Phase 5 (Testing & Docs): 2-3 hours

**Total**: 10-15 hours of focused development

## References

- [Rust API Guidelines](https://rust-lang.github.io/api-guidelines/)
- [Tokio Best Practices](https://tokio.rs/tokio/topics/bridging)
- [Error Handling Patterns](https://doc.rust-lang.org/book/ch09-00-error-handling.html)
- [Current Implementation](/workspaces/eventmesh/crates/riptide-api/src/resource_manager.rs)
