# Riptide Python Bindings

Python bindings for the Riptide web scraping framework using PyO3.

## Status: PyO3 Spike (Phase 2, Week 9, Step 1)

**Current Phase:** Testing async runtime integration with PyO3.

**Objective:** Verify that tokio async runtime works correctly with PyO3 before proceeding with full Python SDK development.

### Acceptance Criteria

- [ ] Async runtime works in PyO3
- [ ] No deadlocks or panics
- [ ] Go/no-go decision on Python SDK approach

## PyO3 Spike Tests

The spike includes comprehensive tests for:

1. **Basic async operations** - Verify tokio runtime creation
2. **Concurrent operations** - Test multiple parallel tasks
3. **Timeout handling** - Verify timeout mechanisms work
4. **Error handling** - Test Rust→Python exception conversion
5. **Class wrapping** - Test RipTideSpike class with async methods
6. **Simulated crawl** - Test crawl-like async operations
7. **Simulated spider** - Test spider-like concurrent operations

## Building the Spike

### Prerequisites

- Python 3.8+
- Rust toolchain
- maturin (`pip install maturin`)

### Build and Test

```bash
# Navigate to riptide-py directory
cd crates/riptide-py

# Install maturin if not already installed
pip install maturin

# Build and install in development mode
maturin develop

# Run spike tests
python examples/spike_test.py
```

### Expected Output

```
==================================================
PyO3 Spike: Async Runtime Integration Tests
==================================================

Test 1: Basic async operation...
✅ PASS: Async runtime works!

Test 2: Concurrent async operations...
✅ PASS: Completed 5 concurrent tasks
   - Task 0 completed
   - Task 1 completed
   ...

Summary
==================================================
Passed: 10/10 tests

✅ GO: Async runtime integration works!
✅ PyO3 + tokio is viable for Python SDK
```

## Architecture

```
Python Layer
    ↓
PyO3 Bindings (riptide-py)
    ↓
Tokio Runtime (block_on)
    ↓
CrawlFacade (riptide-facade)
    ↓
Production Pipeline Code (1,640 lines)
```

## Spike Implementation

### Rust Side (`src/lib.rs`)

```rust
#[pyfunction]
fn test_async_basic() -> PyResult<String> {
    let rt = Runtime::new()?;
    rt.block_on(async {
        Ok("Async runtime works!".to_string())
    })
}

#[pyclass]
struct RipTideSpike {
    runtime: Runtime,
    initialized: bool,
}

#[pymethods]
impl RipTideSpike {
    fn test_async_method(&self) -> PyResult<String> {
        self.runtime.block_on(async {
            // Async operation
            Ok("Success!".to_string())
        })
    }
}
```

### Python Side (`examples/spike_test.py`)

```python
import riptide

# Test basic async
result = riptide.test_async_basic()

# Test class with async methods
spike = riptide.RipTideSpike()
result = spike.test_async_method()
```

## Go/No-Go Decision

### Go Criteria (Must Pass)

- ✅ Tokio runtime creation succeeds
- ✅ Basic async operations work
- ✅ Concurrent tasks don't deadlock
- ✅ Timeout mechanisms function
- ✅ Error handling works correctly
- ✅ Class instantiation with runtime succeeds
- ✅ Async methods callable from Python

### No-Go Scenarios

- ❌ Runtime creation fails consistently
- ❌ Deadlocks occur in concurrent operations
- ❌ Panics or segfaults
- ❌ Memory leaks detected
- ❌ Performance unacceptable

## Next Steps (if GO)

### Step 2: Core Bindings (Week 9-11, 2 weeks)

1. Wrap CrawlFacade in Python class
2. Implement extract(), spider(), crawl() methods
3. Add Python type hints
4. Implement async/await support
5. Error handling and logging
6. Basic documentation

### Step 3: Python Packaging (Week 11-12, 1 week)

1. Configure maturin for PyPI
2. Add wheels for multiple platforms
3. CI/CD for automated builds
4. Python examples and tutorials
5. Complete API documentation

## Development

### Running Rust Tests

```bash
cargo test -p riptide-py
```

### Running Python Tests

```bash
maturin develop && python -m pytest tests/
```

### Formatting

```bash
cargo fmt --package riptide-py
black examples/ tests/
```

## License

Licensed under either of:

- Apache License, Version 2.0 ([LICENSE-APACHE](../../LICENSE-APACHE))
- MIT license ([LICENSE-MIT](../../LICENSE-MIT))

at your option.
