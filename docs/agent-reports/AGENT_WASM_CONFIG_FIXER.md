# Agent: WASM Config Test Fixer

## Mission
Fix WASM configuration test failures in `config_env_tests.rs` (CRITICAL BLOCKER)

## Problem Analysis
The `wasm` field in `ApiConfig` is gated behind `#[cfg(feature = "wasm-extractor")]` but tests don't check this feature flag.

```rust
// In config.rs line 26-27:
#[cfg(feature = "wasm-extractor")]
pub wasm: WasmConfig,
```

## Solution Strategy
**Option B (Recommended):** Refactor tests to handle conditional WASM field

## Tasks
1. ✅ Read current test failures
2. ⬜ Add feature flag checks to tests
3. ⬜ Update test assertions for conditional WASM
4. ⬜ Verify all 8 compilation errors fixed
5. ⬜ Run `cargo test --package riptide-api --test config_env_tests`
6. ⬜ Document changes in commit message

## Affected Files
- `crates/riptide-api/tests/config_env_tests.rs`

## Hooks Protocol
```bash
npx claude-flow@alpha hooks pre-task --description "fix-wasm-config-tests"
# ... work ...
npx claude-flow@alpha hooks post-edit --file "crates/riptide-api/tests/config_env_tests.rs" --memory-key "swarm/wasm-fixer/completed"
npx claude-flow@alpha hooks post-task --task-id "wasm-config-fix"
```

## Success Criteria
- All tests compile
- Tests pass with and without `wasm-extractor` feature
- CI/CD green

## Estimated Time: 2-3 hours
