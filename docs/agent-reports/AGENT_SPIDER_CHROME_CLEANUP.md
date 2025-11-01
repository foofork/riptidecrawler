# Agent: Spider-Chrome Integration Cleanup

## Mission
Complete spider-chrome integration cleanup (remove migration TODOs)

## Context
**CLARIFICATION:** This is NOT a migration - spider_chrome v2.37.128 is already integrated and re-exports chromiumoxide types. Just need to clean up TODOs and access types properly.

## Current State
```bash
cargo tree -p riptide-extraction | grep spider
# Shows: spider_chrome is already a dependency
```

## Tasks
1. ✅ Verify spider_chrome dependency in Cargo.toml
2. ⬜ Remove TODO comments in:
   - `crates/riptide-cli/src/commands/render.rs:688`
   - `crates/riptide-cli/src/commands/render.rs:776`
   - `crates/riptide-cli/src/main.rs:18,69,171`
3. ⬜ Access chromiumoxide types via `spider_chrome::chromiumoxide::*`
4. ⬜ Test render command compilation
5. ⬜ Verify no regressions in browser functionality

## Code Pattern
```rust
// OLD TODO comment:
// TODO(chromiumoxide-migration): Re-implement with proper type access

// NEW clean code:
use spider_chrome::chromiumoxide::{Browser, Page, Element};
// Direct usage - types already available
```

## Affected Files
- `crates/riptide-cli/src/commands/render.rs`
- `crates/riptide-cli/src/main.rs`

## Hooks Protocol
```bash
npx claude-flow@alpha hooks pre-task --description "spider-chrome-cleanup"
npx claude-flow@alpha hooks post-edit --file "crates/riptide-cli/src/commands/render.rs" --memory-key "swarm/spider-cleanup/render"
npx claude-flow@alpha hooks post-edit --file "crates/riptide-cli/src/main.rs" --memory-key "swarm/spider-cleanup/main"
npx claude-flow@alpha hooks post-task --task-id "spider-chrome-cleanup"
```

## Success Criteria
- All chromiumoxide TODOs removed
- Code compiles cleanly
- Types accessed via spider_chrome re-exports
- No functional regressions

## Estimated Time: 1-2 hours
