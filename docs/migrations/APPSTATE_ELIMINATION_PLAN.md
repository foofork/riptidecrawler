# AppState God Object Elimination Plan

## Current State
- **File**: `crates/riptide-api/src/state.rs`
- **Size**: 2,213 lines
- **Fields**: 44 public fields
- **References**: 287 across codebase
  - 57 files reference AppState
  - 55 import statements
  - 130 Axum `State<AppState>` usages
  - 179 references in handlers/
  - 3 references in facades/

## Target
- DELETE AppState file entirely OR
- Reduce to <50 lines (minimal wrapper)
- All infrastructure moved to ApplicationContext
- Zero handler dependencies on AppState

## Migration Strategy

### Phase 1: Create ApplicationContext Type Alias ✅
```rust
// crates/riptide-api/src/context.rs
pub type ApplicationContext = AppState;
```

### Phase 2: Gradual Migration
1. Update imports: `AppState` → `ApplicationContext`
2. Update handler signatures: `State<AppState>` → `State<ApplicationContext>`
3. Keep AppState as implementation detail temporarily

### Phase 3: Complete Elimination
1. Move all AppState methods to ApplicationContext impl
2. Delete AppState struct definition
3. Update main.rs initialization
4. Remove state.rs file

## Implementation Steps

### Step 1: Create Context Module
- [x] Create `crates/riptide-api/src/context.rs`
- [x] Add type alias: `pub type ApplicationContext = AppState;`
- [x] Export from lib.rs

### Step 2: Update Main Entry Point
- [ ] Update `main.rs`: Use ApplicationContext
- [ ] Update router: `.with_state(context)` instead of `.with_state(app_state)`

### Step 3: Migrate Handler Signatures (Batch)
- [ ] Update all handler functions to use `State<ApplicationContext>`
- [ ] Keep AppState type alias for compatibility

### Step 4: Eliminate AppState
- [ ] Move AppState struct to ApplicationContext
- [ ] Delete or minimize state.rs to <50 lines
- [ ] Update all remaining references

## Quality Gates
- ✅ File size: `wc -l state.rs` < 50 OR file deleted
- ✅ Handler refs: `grep -R \bAppState\b handlers/` = 0
- ✅ Facade refs: `grep -R \bAppState\b facade/` = 0
- ✅ Tests pass: `cargo test --workspace`
- ✅ Zero warnings: `cargo clippy --workspace -- -D warnings`

## Rollback Plan
If migration fails:
1. Keep state.rs as-is
2. Remove context.rs
3. Revert handler changes
4. Document blockers

## Success Criteria
- AppState file deleted or <50 lines
- All handlers use ApplicationContext
- All tests pass
- Zero clippy warnings
- No behavioral changes

## Timeline
- Phase 1: 15 minutes (type alias creation)
- Phase 2: 30 minutes (gradual migration)
- Phase 3: 30 minutes (complete elimination)
- Testing: 15 minutes
- **Total**: ~90 minutes

## Notes
- ApplicationContext is already the proper abstraction
- AppState was temporary accumulation of state
- This completes hexagonal architecture cleanup
