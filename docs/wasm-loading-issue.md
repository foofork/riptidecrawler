# WASM Loading Blocks API Startup

## Problem

The API fails health checks in CI because WASM compilation blocks `AppState::new()`, preventing the server from starting within the 60-second health check timeout.

## Root Cause

1. **Blocking Initialization**: `AppState::new()` at `state.rs:521`:
   ```rust
   let extractor = Arc::new(WasmExtractor::new(&config.wasm_path).await?);
   ```

2. **WASM Compilation**: `wasm_extraction.rs:315`:
   ```rust
   let component_bytes = std::fs::read(wasm_path)?;
   let component = Component::new(&engine, component_bytes)?; // <-- Blocks here
   ```

3. **Cranelift Translation**: The `Component::new()` call triggers Cranelift compilation:
   ```
   DEBUG cranelift_codegen::timing::enabled: timing: Starting Translate WASM function
   ```

   This can take 30-60+ seconds for a 3.3MB WASM file.

## Impact

- **CI Health Checks Fail**: API never responds to `/healthz` because server hasn't finished starting
- **Long Startup Times**: API takes 60+ seconds to become ready
- **Poor User Experience**: Deployments appear to hang or fail

## Evidence from CI Logs

```
🚀 Starting API server...
Started API with PID: 5980
✅ Process is running
⚠️  No log output yet (file is empty)

🏥 Waiting for API health endpoint...
⏳ Attempt 30/30: Still waiting...
❌ API failed to become healthy after 60 seconds
```

The process is running and bound to port 8080, but hung at WASM loading.

## Solutions

### Option 1: AOT (Ahead-of-Time) Compilation Cache ✅ **Recommended**

Enable Wasmtime's AOT cache to pre-compile WASM once and reuse:

```rust
// In wasm_extraction.rs
let mut wasmtime_config = Config::new();
wasmtime_config.wasm_component_model(true);
wasmtime_config.cache_config_load_default()?; // Enable AOT cache
```

**Benefits**:
- First run: ~60s compilation, cached
- Subsequent runs: <1s to load from cache
- No API changes needed

### Option 2: Lazy/Async WASM Loading

Load WASM in the background after server starts:

```rust
// Start server immediately
let app_state = AppState::new_without_wasm(config).await?;
tokio::spawn(async move {
    app_state.init_wasm().await;
});
```

**Benefits**:
- Health checks pass immediately
- Server responds while WASM loads
- Graceful degradation

**Drawbacks**:
- Requires API changes
- First requests may fail or wait for WASM
- More complex error handling

### Option 3: Increase Health Check Timeout

Increase CI timeout from 60s to 120s:

```yaml
# .github/workflows/api-validation.yml
timeout: 120  # Was 60
```

**Benefits**:
- No code changes
- Simple fix

**Drawbacks**:
- Doesn't solve the problem, just masks it
- Still slow startup in production
- Not recommended

## Recommended Implementation

**Phase 1** (Immediate): Enable AOT caching
```rust
// crates/riptide-extraction/src/wasm_extraction.rs:299
wasmtime_config.cache_config_load_default()?;
```

**Phase 2** (Future): Async loading for even faster startup
- Move WASM loading to background task
- Add `/ready` endpoint separate from `/healthz`
- `/healthz` = server alive, `/ready` = fully initialized

## Related Files

- `crates/riptide-api/src/state.rs:521` - WASM loading call
- `crates/riptide-extraction/src/wasm_extraction.rs:292-316` - WASM initialization
- `.github/workflows/api-validation.yml` - Health check timeout

## Testing

To reproduce locally:
```bash
cargo build --release --bin riptide-api
time ./target/release/riptide-api  # Measure startup time
# Should see Cranelift compilation logs
```

To test with AOT cache:
```bash
# First run: slow (compiles and caches)
time ./target/release/riptide-api

# Second run: fast (loads from cache)
time ./target/release/riptide-api  # Should be <5s
```
