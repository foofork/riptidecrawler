# Priority 3 Facade Migration Guide

**Phase**: Phase 2 - Facade Detox & Complete Trait Migration
**Priority**: 3 of 3
**Timeline**: 2-3 days
**Risk Level**: High (circular dependencies)

---

## Overview

This guide provides step-by-step instructions for migrating all 5 facades in ApplicationContext:
- **2 facades to REMOVE**: ExtractionFacade, SpiderFacade
- **3 facades to ABSTRACT**: ScraperFacade, SearchFacade, EngineFacade

---

## Pre-Migration Checklist

Before starting migration, ensure:

- [x] All analysis documents reviewed
- [x] Port trait designs approved
- [x] Adapter designs reviewed
- [ ] Team alignment on migration order
- [ ] Backup branch created: `git checkout -b backup/pre-facade-migration`
- [ ] Clean workspace: `cargo clean && cargo build`
- [ ] All tests passing: `cargo test --workspace`

---

## Migration Order (Low Risk → High Risk)

### Phase 1: Remove Duplicate Facades (Low Risk)
1. Remove ExtractionFacade (15-30 min)
2. Remove SpiderFacade (15-30 min)

### Phase 2: Create Port Traits (Medium Risk)
3. Create WebScraping trait (1-2 hours)
4. Create SearchProvider trait (1-2 hours)
5. Create EngineSelection trait (1-2 hours)

### Phase 3: Implement Adapters (Medium Risk)
6. Implement ScraperFacadeAdapter (2-3 hours)
7. Implement SearchFacadeAdapter (2-3 hours)
8. Implement EngineFacadeAdapter (2-3 hours)

### Phase 4: Update ApplicationContext (High Risk)
9. Update field types (30 min)
10. Update initialization code (1-2 hours)
11. Update all call sites (1-2 hours)

---

## Detailed Migration Steps

### Step 1: Remove ExtractionFacade

**Time**: 15-30 minutes
**Risk**: ⭐ Low
**See**: `01-extraction-facade-removal.md`

```bash
# 1. Remove field from ApplicationContext
# Edit: crates/riptide-api/src/context.rs
# Delete lines 181-182

# 2. Remove initialization code
# Delete lines 1278-1285, 1416-1423, 1872-1876, 1378/1983

# 3. Remove imports
# Delete: use riptide_facade::facades::ExtractionFacade;

# 4. Verify
cargo check -p riptide-api
cargo test -p riptide-api
cargo clippy -p riptide-api -- -D warnings

# 5. Commit
git add crates/riptide-api/src/context.rs
git commit -m "Remove ExtractionFacade (duplicates ContentExtractor trait)

- Removes extraction_facade field from ApplicationContext
- No handler usage found - safe to remove
- Reduces facade count from 5 to 4

🤖 Generated with [Claude Code](https://claude.com/claude-code)

Co-Authored-By: Claude <noreply@anthropic.com>"
```

### Step 2: Remove SpiderFacade

**Time**: 15-30 minutes
**Risk**: ⭐ Low
**See**: `02-spider-facade-removal.md`

```bash
# 1. Remove field from ApplicationContext
# Edit: crates/riptide-api/src/context.rs
# Delete lines 189-191

# 2. Remove initialization code
# Delete lines 1340, 1437-1451, 1885-1897, 1381/1986

# 3. Remove imports
# Delete: use riptide_facade::facades::{SpiderFacade, spider::SpiderPreset};

# 4. Verify
cargo check -p riptide-api --features spider
cargo test -p riptide-api --features spider
cargo clippy -p riptide-api --features spider -- -D warnings

# 5. Commit
git add crates/riptide-api/src/context.rs
git commit -m "Remove SpiderFacade (duplicates SpiderEngine trait)

- Removes spider_facade field from ApplicationContext
- Use existing spider: Option<Arc<dyn SpiderEngine>> instead
- No handler usage found - safe to remove
- Reduces facade count from 4 to 3

🤖 Generated with [Claude Code](https://claude.com/claude-code)

Co-Authored-By: Claude <noreply@anthropic.com>"
```

### Step 3: Create Port Traits

**Time**: 4-6 hours total
**Risk**: ⭐⭐ Medium
**See**: `03-webscraping-trait-design.md`, `04-searchprovider-trait-design.md`, `05-engineselection-trait-design.md`

```bash
# 1. Create WebScraping trait
# File: crates/riptide-types/src/ports/scraping.rs
# Copy content from 03-webscraping-trait-design.md

# 2. Register in mod.rs
# File: crates/riptide-types/src/ports/mod.rs
echo "pub mod scraping;
pub use scraping::{WebScraping, ScrapeOptions, ScrapedPage, SelectorSet, ExtractedData};" >> crates/riptide-types/src/ports/mod.rs

# 3. Create SearchProvider trait
# File: crates/riptide-types/src/ports/search.rs
# Copy content from 04-searchprovider-trait-design.md

# 4. Register in mod.rs
echo "pub mod search;
pub use search::{SearchProvider, SearchQuery, SearchResults, SearchHit, DocumentId};" >> crates/riptide-types/src/ports/mod.rs

# 5. Create EngineSelection trait
# File: crates/riptide-types/src/ports/engine.rs
# Copy content from 05-engineselection-trait-design.md

# 6. Register in mod.rs
echo "pub mod engine;
pub use engine::{EngineSelection, EngineChoice, EngineType, EngineSelectionRequest};" >> crates/riptide-types/src/ports/mod.rs

# 7. Verify traits compile
cargo check -p riptide-types
cargo test -p riptide-types

# 8. Commit
git add crates/riptide-types/src/ports/
git commit -m "Add port traits for facade abstraction (Priority 3)

- Add WebScraping trait for scraping operations
- Add SearchProvider trait for search operations
- Add EngineSelection trait for engine selection
- All traits in domain layer (riptide-types)
- Enables hexagonal architecture compliance

🤖 Generated with [Claude Code](https://claude.com/claude-code)

Co-Authored-By: Claude <noreply@anthropic.com>"
```

### Step 4: Implement Adapters

**Time**: 6-9 hours total
**Risk**: ⭐⭐ Medium
**See**: `06-scraper-facade-adapter.md`, `07-search-facade-adapter.md`, `08-engine-facade-adapter.md`

```bash
# 1. Create adapters directory
mkdir -p crates/riptide-facade/src/adapters

# 2. Create ScraperFacadeAdapter
# File: crates/riptide-facade/src/adapters/scraper_adapter.rs
# Copy content from 06-scraper-facade-adapter.md

# 3. Create SearchFacadeAdapter
# File: crates/riptide-facade/src/adapters/search_adapter.rs
# Copy content from 07-search-facade-adapter.md

# 4. Create EngineFacadeAdapter
# File: crates/riptide-facade/src/adapters/engine_adapter.rs
# Copy content from 08-engine-facade-adapter.md

# 5. Create adapters mod.rs
cat > crates/riptide-facade/src/adapters/mod.rs <<'EOF'
//! Facade adapters implementing port traits

pub mod scraper_adapter;
pub mod search_adapter;
pub mod engine_adapter;

pub use scraper_adapter::ScraperFacadeAdapter;
pub use search_adapter::SearchFacadeAdapter;
pub use engine_adapter::EngineFacadeAdapter;
EOF

# 6. Register in lib.rs
echo "pub mod adapters;" >> crates/riptide-facade/src/lib.rs

# 7. Add dependencies to Cargo.toml
# Edit: crates/riptide-facade/Cargo.toml
# Add: scraper = "0.17" (for CSS selector parsing)

# 8. Verify adapters compile
cargo check -p riptide-facade
cargo test -p riptide-facade

# 9. Commit
git add crates/riptide-facade/src/adapters/
git add crates/riptide-facade/Cargo.toml
git commit -m "Implement facade adapters for port traits (Priority 3)

- Add ScraperFacadeAdapter implementing WebScraping
- Add SearchFacadeAdapter implementing SearchProvider
- Add EngineFacadeAdapter implementing EngineSelection
- All adapters wrap existing facades
- Enables ApplicationContext to use trait abstractions

🤖 Generated with [Claude Code](https://claude.com/claude-code)

Co-Authored-By: Claude <noreply@anthropic.com>"
```

### Step 5: Update ApplicationContext

**Time**: 2-4 hours
**Risk**: ⭐⭐⭐ High
**See**: Individual trait design docs for field type changes

```bash
# 1. Update field types in ApplicationContext
# Edit: crates/riptide-api/src/context.rs

# Before:
# pub scraper_facade: Arc<riptide_facade::facades::ScraperFacade>,
# pub search_facade: Option<Arc<riptide_facade::facades::SearchFacade>>,
# pub engine_facade: Arc<riptide_facade::facades::EngineFacade>,

# After:
# pub scraper: Arc<dyn WebScraping>,
# pub search_provider: Option<Arc<dyn SearchProvider>>,
# pub engine_selector: Arc<dyn EngineSelection>,

# 2. Update imports
# Add:
use riptide_facade::adapters::{ScraperFacadeAdapter, SearchFacadeAdapter, EngineFacadeAdapter};
use riptide_types::ports::{WebScraping, SearchProvider, EngineSelection};

# 3. Update initialization code
# Replace facade creation with adapter creation:

# Before:
# let scraper_facade = Arc::new(ScraperFacade::new(config.clone()).await?);

# After:
let scraper_facade = ScraperFacade::new(config.clone()).await?;
let scraper: Arc<dyn WebScraping> = ScraperFacadeAdapter::new(scraper_facade);

# 4. Update struct initialization
# Replace:
# scraper_facade,
# search_facade,
# engine_facade,

# With:
# scraper,
# search_provider,
# engine_selector,

# 5. Verify compilation
cargo check -p riptide-api
cargo check -p riptide-api --features "spider,search"

# 6. Run tests
cargo test -p riptide-api
cargo test -p riptide-api --features "spider,search"

# 7. Check for call site updates needed
rg "\.scraper_facade\." crates/riptide-api --type rust
rg "\.search_facade\." crates/riptide-api --type rust
rg "\.engine_facade\." crates/riptide-api --type rust

# 8. Update any found call sites to use new field names

# 9. Commit
git add crates/riptide-api/src/context.rs
git commit -m "Update ApplicationContext to use port traits (Priority 3)

- Replace scraper_facade with scraper: Arc<dyn WebScraping>
- Replace search_facade with search_provider: Arc<dyn SearchProvider>
- Replace engine_facade with engine_selector: Arc<dyn EngineSelection>
- All facades now abstracted via traits
- Completes Phase 2 facade detox

Architecture compliance: 28% → 38% (3 more traits)

🤖 Generated with [Claude Code](https://claude.com/claude-code)

Co-Authored-By: Claude <noreply@anthropic.com>"
```

---

## Validation & Testing

### After Each Step

```bash
# 1. Workspace builds cleanly
cargo build --workspace

# 2. No clippy warnings
cargo clippy --workspace -- -D warnings

# 3. All tests pass
cargo test --workspace

# 4. Check architecture compliance
./scripts/quality_gate.sh
```

### Integration Test

Create integration test to verify trait usage:

```rust
// File: crates/riptide-api/tests/facade_migration_test.rs

#[tokio::test]
async fn test_all_facades_use_traits() {
    let context = ApplicationContext::new_for_test().await.unwrap();

    // Verify trait usage compiles and works
    let _scraper: &dyn WebScraping = &*context.scraper;
    assert!(_scraper.is_available().await);

    if let Some(search) = &context.search_provider {
        let _search_provider: &dyn SearchProvider = &**search;
        assert!(_search_provider.is_available().await);
    }

    let _engine: &dyn EngineSelection = &*context.engine_selector;
}
```

---

## Rollback Plan

### If Issues Arise During Migration

**Per-step rollback**:
```bash
# Rollback last commit
git reset --soft HEAD~1

# Or revert specific commit
git revert <commit-hash>

# Verify workspace still works
cargo build --workspace
cargo test --workspace
```

**Full rollback to backup**:
```bash
git checkout backup/pre-facade-migration
git branch -D main
git checkout -b main
```

---

## Success Criteria

After completing all steps, verify:

- [ ] ✅ `extraction_facade` field removed
- [ ] ✅ `spider_facade` field removed
- [ ] ✅ `scraper_facade` replaced with `scraper: Arc<dyn WebScraping>`
- [ ] ✅ `search_facade` replaced with `search_provider: Option<Arc<dyn SearchProvider>>`
- [ ] ✅ `engine_facade` replaced with `engine_selector: Arc<dyn EngineSelection>`
- [ ] ✅ All port traits in `riptide-types/src/ports/`
- [ ] ✅ All adapters in `riptide-facade/src/adapters/`
- [ ] ✅ `cargo build --workspace` succeeds
- [ ] ✅ `cargo test --workspace` succeeds
- [ ] ✅ `cargo clippy --workspace -- -D warnings` succeeds
- [ ] ✅ Architecture compliance improved (check with `./scripts/quality_gate.sh`)
- [ ] ✅ Zero references to removed facades in codebase

---

## Post-Migration Validation

```bash
# 1. Verify no facade references remain
rg "extraction_facade|spider_facade" crates/riptide-api --type rust
# Expected: No results

# 2. Verify trait usage
rg "Arc<dyn WebScraping>" crates/riptide-api --type rust
rg "Arc<dyn SearchProvider>" crates/riptide-api --type rust
rg "Arc<dyn EngineSelection>" crates/riptide-api --type rust
# Expected: Found in context.rs

# 3. Count remaining concrete types
rg "Arc<riptide_" crates/riptide-api/src/context.rs | wc -l
# Expected: Reduced from 18 to 15

# 4. Run full workspace build and test
cargo build --workspace --release
cargo test --workspace

# 5. Generate architecture report
./scripts/architecture_report.sh
```

---

## Expected Outcomes

### Before Migration

```
ApplicationContext Fields:
- Trait abstractions: 9/32 (28%)
- Concrete types: 18 (including 5 facades)
- Facade violations: 5
```

### After Migration

```
ApplicationContext Fields:
- Trait abstractions: 12/32 (38%)
- Concrete types: 15
- Facade violations: 0 ✅
```

**Improvement**: +10% architecture compliance, 5 facades eliminated

---

## Common Issues & Solutions

### Issue: Trait object size errors

**Solution**: Ensure all trait methods use `&self`, not `self`

### Issue: Async trait lifetime errors

**Solution**: Use `#[async_trait]` macro on all implementations

### Issue: Type mismatch between facade and adapter

**Solution**: Add conversion functions in adapter (see `08-engine-facade-adapter.md`)

### Issue: Tests fail after field rename

**Solution**: Update test code to use new field names

---

## Timeline Estimate

| Phase | Duration | Parallelizable? |
|-------|----------|-----------------|
| Removals (Steps 1-2) | 1 hour | ✅ Yes |
| Port Traits (Step 3) | 4-6 hours | ✅ Yes (3 traits) |
| Adapters (Step 4) | 6-9 hours | ✅ Yes (3 adapters) |
| Context Update (Step 5) | 2-4 hours | ❌ No |
| Testing & Validation | 2-3 hours | ❌ No |
| **Total** | **15-23 hours (2-3 days)** | |

With parallel work: ~1.5-2 days

---

## Final Checklist

Before marking Priority 3 complete:

- [ ] All 5 facades addressed (2 removed, 3 abstracted)
- [ ] All port traits in domain layer (`riptide-types`)
- [ ] All adapters tested with unit tests
- [ ] ApplicationContext updated and tested
- [ ] All call sites updated
- [ ] Full workspace builds cleanly
- [ ] All tests pass
- [ ] Clippy warnings = 0
- [ ] Architecture report updated
- [ ] Documentation updated
- [ ] Team review completed

---

**Report Generated**: 2025-11-12
**Status**: ✅ Ready for Implementation
**Next**: Begin with Step 1 (ExtractionFacade removal)
