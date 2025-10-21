# CLI Integration Quick Reference

**Date:** 2025-10-21
**For:** Hive Mind Collective
**See:** `/docs/hive/CLI-INTEGRATION-ARCHITECTURE.md` for full details

---

## TL;DR

**Question:** How do extracted modules integrate with OptimizedExecutor?
**Answer:** Direct imports with singletons. Zero refactoring needed.

---

## Copy-Paste Examples

### Before Extraction
```rust
// File: crates/riptide-cli/src/commands/optimized_executor.rs
use super::adaptive_timeout::AdaptiveTimeoutManager;
```

### After Extraction
```rust
// File: crates/riptide-cli/src/commands/optimized_executor.rs
use riptide_intelligence::adaptive_timeout::AdaptiveTimeoutManager;
```

**That's it!** Everything else stays the same.

---

## All 6 Singletons

| Module | Current Location | Target Crate | Import After |
|--------|-----------------|--------------|--------------|
| AdaptiveTimeoutManager | `cli/commands/adaptive_timeout.rs` | `riptide-intelligence` | `use riptide_intelligence::adaptive_timeout::AdaptiveTimeoutManager;` |
| BrowserPoolManager | `cli/commands/browser_pool_manager.rs` | `riptide-headless` | `use riptide_headless::pool_manager::BrowserPoolManager;` |
| WasmAotCache | `cli/commands/wasm_aot_cache.rs` | `riptide-wasm-runtime` | `use riptide_wasm_runtime::aot_cache::WasmAotCache;` |
| WasmCache | `cli/commands/wasm_cache.rs` | `riptide-wasm-runtime` | `use riptide_wasm_runtime::module_cache::WasmCache;` |
| EngineSelectionCache | `cli/commands/engine_cache.rs` | `riptide-extraction` | `use riptide_extraction::engine_cache::EngineSelectionCache;` |
| PerformanceMonitor | `cli/commands/performance_monitor.rs` | `riptide-monitoring` | `use riptide_monitoring::performance_monitor::PerformanceMonitor;` |

---

## Initialization Order

```rust
// File: crates/riptide-cli/src/main.rs
#[tokio::main]
async fn main() -> Result<()> {
    // Async singletons (order matters)
    AdaptiveTimeoutManager::initialize_global().await?;
    WasmAotCache::initialize_global().await?;
    BrowserPoolManager::initialize_global().await?;

    // Lazy singletons (auto-initialize on first use)
    // - EngineSelectionCache::get_global()
    // - WasmCache::get_global()
    // - PerformanceMonitor::get_global()

    cli::run().await
}
```

---

## Migration Commands

```bash
# Create new crates
cargo new --lib crates/riptide-intelligence
cargo new --lib crates/riptide-wasm-runtime
cargo new --lib crates/riptide-monitoring

# Move files
mv crates/riptide-cli/src/commands/adaptive_timeout.rs \
   crates/riptide-intelligence/src/adaptive_timeout.rs

mv crates/riptide-cli/src/commands/browser_pool_manager.rs \
   crates/riptide-headless/src/pool_manager.rs

mv crates/riptide-cli/src/commands/wasm_aot_cache.rs \
   crates/riptide-wasm-runtime/src/aot_cache.rs

mv crates/riptide-cli/src/commands/wasm_cache.rs \
   crates/riptide-wasm-runtime/src/module_cache.rs

mv crates/riptide-cli/src/commands/engine_cache.rs \
   crates/riptide-extraction/src/engine_cache.rs

mv crates/riptide-cli/src/commands/performance_monitor.rs \
   crates/riptide-monitoring/src/performance_monitor.rs

# Update imports in optimized_executor.rs
# (See "All 6 Singletons" table above)

# Test
cargo test --workspace
```

---

## P0 Methods to Restore First

```rust
// Before extraction, restore these from dead code:
BrowserPoolManager::get_stats() -> PoolStats
BrowserPoolManager::shutdown() -> Result<()>
BrowserPoolManager::browser_id() -> &str
BrowserPoolManager::new_page() -> Result<Page>
BrowserPoolManager::checkin() -> Result<()>
BrowserPoolManager::update_stats()
BrowserPoolManager::start_instance_health_monitoring() -> Result<()>
BrowserPoolManager::validate_instance_health() -> bool
```

See `/docs/hive/code-restoration-implementation-plan.md` for full list.

---

## Test Checklist

```bash
# Per-crate unit tests
cargo test -p riptide-intelligence
cargo test -p riptide-wasm-runtime
cargo test -p riptide-monitoring
cargo test -p riptide-headless

# CLI integration test
cargo test -p riptide-cli --test optimized_executor_integration

# Full workspace
cargo test --workspace

# Build verification
cargo build --workspace --release
```

---

## Decision Matrix

| Question | Answer |
|----------|--------|
| Use facade pattern? | ❌ NO - Direct imports simpler |
| Create orchestration crate? | ❌ NO - Singletons already coordinate |
| Refactor OptimizedExecutor? | ❌ NO - Just change imports |
| Change initialization code? | ✅ YES - Move to main.rs |
| Change API usage? | ❌ NO - Same singleton methods |

---

## Common Pitfalls

1. **Don't create facade** - Use direct imports
2. **Don't break singletons** - Keep `get_global()` pattern
3. **Don't duplicate code** - One source of truth per module
4. **Don't skip P0 restoration** - OptimizedExecutor needs those methods
5. **Don't forget main.rs init** - Async singletons need early init

---

## Validation

After migration, verify:

```bash
# 1. All imports resolve
cargo check --workspace

# 2. All tests pass
cargo test --workspace

# 3. CLI works
cargo run -p riptide-cli -- extract --url https://example.com

# 4. API can use modules
cargo run -p riptide-api
```

---

**Quick Reference Complete**
**Full Details:** `/docs/hive/CLI-INTEGRATION-ARCHITECTURE.md`
