# Agent: Extractor Type Conflict Resolver

## Mission
Fix extractor module type conflicts and re-enable disabled modules (CRITICAL)

## Problem Analysis
Multiple extraction modules are commented out due to type mismatches:

```rust
// Line 37-38: Composition disabled
// TODO: Re-enable after resolving type mismatches between strategies and composition
// pub mod composition;

// Line 40-41: Confidence integration disabled
// TODO: Re-enable after fixing ExtractedContent type conflicts
// pub mod confidence_integration;

// Line 119-121: Re-exports disabled
// TODO: Re-enable these after resolving type conflicts
// pub use composition::{CompositionMode, StrategyComposer};
// pub use confidence_integration::{CssConfidenceScorer, ExtractedContent, WasmConfidenceScorer};
```

## Investigation Steps
1. ✅ Identify disabled modules
2. ⬜ Compile with verbose errors to see type mismatches
3. ⬜ Audit `strategies` module types
4. ⬜ Audit `composition` module types
5. ⬜ Audit `confidence_integration` module types
6. ⬜ Document type conflicts
7. ⬜ Create alignment plan

## Resolution Strategy
1. **Align ExtractedContent type** across all modules
2. **Fix composition layer** to match strategy types
3. **Update confidence integration** for type compatibility
4. **Re-enable modules incrementally**
5. **Add type safety tests**

## Affected Files
- `crates/riptide-extraction/src/lib.rs`
- `crates/riptide-extraction/src/composition/`
- `crates/riptide-extraction/src/confidence_integration.rs`
- `crates/riptide-extraction/src/strategies/`

## Hooks Protocol
```bash
npx claude-flow@alpha hooks pre-task --description "fix-extractor-types"
# For each file fixed:
npx claude-flow@alpha hooks post-edit --file "[path]" --memory-key "swarm/extractor-fixer/[module]"
npx claude-flow@alpha hooks post-task --task-id "extractor-type-fix"
```

## Success Criteria
- All extraction modules enabled
- Compilation succeeds
- Type conflicts resolved
- Tests pass
- No performance regression

## Estimated Time: 4-6 hours
