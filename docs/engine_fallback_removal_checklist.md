# Engine Fallback Removal - Quick Execution Checklist

## 🎯 Overview
Remove deprecated `engine_fallback.rs` (484 lines) from CLI - **SAFE & LOW RISK**

## 📋 Quick Facts
- ✅ No active usage in CLI codebase
- ✅ Complete feature parity in `riptide-reliability::engine_selection`
- ⚠️ 3 test files need import updates
- ⏱️ Estimated time: 30-45 minutes

## 🚀 Execution Steps

### 1. Update Test Files (3 files)

#### File 1: `/workspaces/eventmesh/tests/unit/singleton_thread_safety_tests.rs`
```rust
// FIND & REPLACE (2 occurrences):
use riptide_cli::commands::engine_fallback::EngineType;
// WITH:
use riptide_reliability::engine_selection::Engine;

// FIND & REPLACE in code:
EngineType::Raw    → Engine::Raw
EngineType::Wasm   → Engine::Wasm
EngineType::Headless → Engine::Headless
```

#### File 2: `/workspaces/eventmesh/tests/unit/singleton_integration_tests.rs`
```rust
// FIND & REPLACE (5 occurrences):
use riptide_cli::commands::engine_fallback::EngineType;
// WITH:
use riptide_reliability::engine_selection::Engine;

// FIND & REPLACE in code:
EngineType → Engine (all variants)
```

#### File 3: `/workspaces/eventmesh/tests/integration/singleton_integration_tests.rs`
```rust
// FIND & REPLACE (1 occurrence):
use riptide_cli::commands::engine_fallback::EngineType;
// WITH:
use riptide_reliability::engine_selection::Engine;

// FIND & REPLACE in code:
EngineType → Engine (all variants)
```

### 2. Handle Phase 3 Test

#### File: `/workspaces/eventmesh/tests/phase3/direct_execution_tests.rs`

**Option A (Recommended)**: Delete the test function
```rust
// DELETE THIS FUNCTION (around line 101):
async fn test_engine_fallback_chain() { ... }
```

**Option B**: Update to test the new module
```rust
// UPDATE imports and rewrite test to use:
use riptide_reliability::engine_selection::decide_engine;
```

### 3. Remove Module Declaration

#### File: `/workspaces/eventmesh/crates/riptide-cli/src/commands/mod.rs`
```rust
// DELETE LINE 5:
pub mod engine_fallback;
```

### 4. Delete Source File

```bash
rm /workspaces/eventmesh/crates/riptide-cli/src/commands/engine_fallback.rs
```

### 5. Validation Commands

```bash
# Check compilation
cargo check --package riptide-cli

# Run tests
cargo test --package riptide-cli --lib
cargo test --test singleton_thread_safety_tests
cargo test --test singleton_integration_tests
cargo test --test cli_tests

# Full build
cargo build --release --package riptide-cli

# Check for warnings
cargo clippy --package riptide-cli
```

### 6. Update CHANGELOG

#### File: `/workspaces/eventmesh/CHANGELOG.md`
```markdown
### Removed
- **BREAKING**: Removed deprecated `engine_fallback` module from CLI
  - Use `riptide-reliability::engine_selection` instead
  - Deprecated since v1.1.0
  - All functionality available in shared library
```

## 🔍 Verification Checklist

- [ ] All test imports updated (3 files)
- [ ] All `EngineType` → `Engine` replacements done
- [ ] Module declaration removed from mod.rs
- [ ] Source file deleted
- [ ] `cargo check` passes
- [ ] All tests pass
- [ ] Release build succeeds
- [ ] No clippy warnings
- [ ] CHANGELOG updated
- [ ] Changes committed

## 🆘 Rollback (if needed)

```bash
git checkout HEAD -- crates/riptide-cli/src/commands/engine_fallback.rs
git checkout HEAD -- crates/riptide-cli/src/commands/mod.rs
git checkout HEAD -- tests/unit/singleton_thread_safety_tests.rs
git checkout HEAD -- tests/unit/singleton_integration_tests.rs
git checkout HEAD -- tests/integration/singleton_integration_tests.rs
cargo clean --package riptide-cli
cargo build --package riptide-cli
```

## 📊 Impact Analysis

### Files Modified: 6
- 3 test files (import changes)
- 1 mod.rs (remove declaration)
- 1 source file (delete)
- 1 CHANGELOG (documentation)

### Lines Removed: 485
- 484 lines from engine_fallback.rs
- 1 line from mod.rs

### Risk Level: **LOW** ✅
- No active runtime dependencies
- Complete feature parity
- Easy rollback available
- Comprehensive test coverage

---

**See**: `/workspaces/eventmesh/docs/engine_fallback_removal_plan.md` for detailed analysis
