# Refactoring Checklist Template

**Use this checklist for each file refactoring task**

---

## 📋 File Information

- **File**: `[path/to/file.rs]`
- **Current LOC**: `[number]`
- **Target LOC**: `<600 (ideally <400)`
- **Assigned to**: `[name]`
- **Start date**: `[date]`
- **Target date**: `[date]`

---

## 🎯 Pre-Refactoring

### Analysis
- [ ] Read and understand the entire file
- [ ] Identify logical boundaries and responsibilities
- [ ] List all public APIs that must be preserved
- [ ] Document current test coverage
- [ ] Create backup branch: `git checkout -b backup/[filename]-original`

### Planning
- [ ] Design new module structure
- [ ] Plan for backward compatibility
- [ ] Identify potential breaking changes
- [ ] Create todo list for extraction order
- [ ] Review with team (if needed)

### Baseline Metrics
- [ ] Run tests: `cargo test --package [crate]`
- [ ] Record clippy warnings: `cargo clippy --package [crate]`
- [ ] Measure build time: `time cargo build --package [crate]`
- [ ] Document current API surface

---

## 🔨 During Refactoring

### Setup
- [ ] Create feature branch: `git checkout -b refactor/[filename]`
- [ ] Create new module directory structure
- [ ] Add module exports to parent `mod.rs`

### Extraction Process
For each component to extract:

#### Component 1: `[component_name]`
- [ ] Create file: `[path/to/new_file.rs]`
- [ ] Move code (aim for <300 LOC)
- [ ] Add documentation
- [ ] Add unit tests
- [ ] Update imports in original file
- [ ] Run: `cargo check --package [crate]`
- [ ] Run: `cargo test --package [crate] --lib [module]`
- [ ] Run: `cargo clippy --package [crate]`
- [ ] Commit: `refactor([scope]): extract [component] to separate module`

#### Component 2: `[component_name]`
- [ ] Create file: `[path/to/new_file.rs]`
- [ ] Move code (aim for <300 LOC)
- [ ] Add documentation
- [ ] Add unit tests
- [ ] Update imports in original file
- [ ] Run: `cargo check --package [crate]`
- [ ] Run: `cargo test --package [crate] --lib [module]`
- [ ] Run: `cargo clippy --package [crate]`
- [ ] Commit: `refactor([scope]): extract [component] to separate module`

#### Component 3: `[component_name]`
- [ ] Create file: `[path/to/new_file.rs]`
- [ ] Move code (aim for <300 LOC)
- [ ] Add documentation
- [ ] Add unit tests
- [ ] Update imports in original file
- [ ] Run: `cargo check --package [crate]`
- [ ] Run: `cargo test --package [crate] --lib [module]`
- [ ] Run: `cargo clippy --package [crate]`
- [ ] Commit: `refactor([scope]): extract [component] to separate module`

_Add more components as needed_

### Finalization
- [ ] Refactor main file to orchestrate components
- [ ] Ensure main file is <400 LOC
- [ ] Update all internal imports
- [ ] Update module re-exports
- [ ] Add integration tests if needed

---

## ✅ Post-Refactoring

### Quality Checks
```bash
# Format
- [ ] cargo fmt --all
- [ ] cargo fmt --all --check

# Clippy (zero warnings)
- [ ] cargo clippy --package [crate] -- -D warnings
- [ ] cargo clippy --all-targets --all-features -- -D warnings

# Tests
- [ ] cargo test --package [crate]
- [ ] cargo test --package [crate] --all-features
- [ ] cargo test --workspace  # Ensure no breaking changes

# Documentation
- [ ] cargo doc --package [crate] --no-deps
- [ ] cargo doc --package [crate] --no-deps --document-private-items

# Build
- [ ] cargo build --package [crate]
- [ ] cargo build --release --package [crate]

# Benchmarks (if applicable)
- [ ] cargo bench --package [crate] --no-run
```

### Validation
- [ ] All files <600 LOC (check with: `wc -l [files]`)
- [ ] All components <300 LOC ideally
- [ ] Zero clippy warnings
- [ ] All tests passing
- [ ] No new compiler warnings
- [ ] Documentation complete and accurate

### Performance
- [ ] Measure new build time: `time cargo build --package [crate]`
- [ ] Compare build times (before vs after)
- [ ] Run benchmarks if applicable
- [ ] Verify no performance regression

### API Compatibility
- [ ] All public APIs preserved
- [ ] Backward compatibility maintained
- [ ] Deprecation warnings added if needed
- [ ] Migration guide written if breaking changes

---

## 📝 Documentation Updates

### Code Documentation
- [ ] All public items have doc comments
- [ ] Examples added to doc comments
- [ ] Module-level documentation complete
- [ ] Internal architecture documented

### External Documentation
- [ ] Update CHANGELOG.md
- [ ] Update relevant README files
- [ ] Update architecture diagrams if needed
- [ ] Update API documentation

---

## 🔍 Code Review

### Self-Review
- [ ] Read through all changes
- [ ] Check for leftover TODOs
- [ ] Verify consistent naming
- [ ] Check for dead code
- [ ] Verify error handling
- [ ] Check for unwrap()/expect() usage

### Prepare for Review
- [ ] Write clear PR description
- [ ] List breaking changes (if any)
- [ ] Include before/after metrics
- [ ] Add screenshots/examples if helpful
- [ ] Tag relevant reviewers

---

## 🚀 Submission

### Create Pull Request
- [ ] Push branch: `git push origin refactor/[filename]`
- [ ] Create PR against `main` or `develop`
- [ ] Fill out PR template
- [ ] Link to refactoring plan
- [ ] Add labels: `refactoring`, `code-quality`

### PR Description Template
```markdown
## Refactoring: [filename]

### Overview
Refactored `[path/to/file]` to improve maintainability and reduce complexity.

### Changes
- Split into [N] modules:
  - `[module1]` - [responsibility]
  - `[module2]` - [responsibility]
  - `[module3]` - [responsibility]

### Metrics
| Metric | Before | After | Change |
|--------|--------|-------|--------|
| LOC (main file) | [N] | [N] | -[N]% |
| Total LOC | [N] | [N] | +[N]% (due to documentation) |
| Clippy warnings | [N] | 0 | ✅ |
| Build time | [T]s | [T]s | [+/-]% |
| Test coverage | [N]% | [N]% | [+/-]% |

### Breaking Changes
- [ ] None
- [ ] Listed below:
  - [Change description]

### Related Issues
- Addresses #[issue-number]
- Part of refactoring plan: docs/REFACTORING_PLAN.md

### Checklist
- [ ] All quality checks pass
- [ ] Documentation updated
- [ ] Tests added/updated
- [ ] No performance regression
- [ ] Backward compatible (or migration guide provided)
```

---

## ✨ Post-Merge

### Cleanup
- [ ] Delete feature branch
- [ ] Delete backup branch (if no issues)
- [ ] Update progress in REFACTORING_PLAN.md
- [ ] Update metrics dashboard

### Monitoring
- [ ] Monitor CI/CD for any issues
- [ ] Check for user reports
- [ ] Monitor performance metrics
- [ ] Watch for related issues

### Celebration
- [ ] Update team on progress
- [ ] Share learnings
- [ ] Document any patterns for reuse
- [ ] Take a well-deserved break! ☕

---

## 📊 Metrics Summary

### Before
- **LOC**: `[number]`
- **Clippy warnings**: `[number]`
- **Test coverage**: `[percentage]`
- **Build time**: `[seconds]`
- **Number of responsibilities**: `[number]`

### After
- **LOC (main file)**: `[number]` (target: <400)
- **LOC (total with new modules)**: `[number]`
- **Clippy warnings**: `0` ✅
- **Test coverage**: `[percentage]`
- **Build time**: `[seconds]` ([+/-]%)
- **Number of modules**: `[number]`
- **Avg LOC per module**: `[number]` (target: <300)

### Improvements
- ✅ Reduced complexity
- ✅ Improved testability
- ✅ Better separation of concerns
- ✅ Enhanced documentation
- ✅ Zero clippy warnings
- ✅ All tests passing

---

**Notes**: Keep this checklist updated as you progress. Mark items as complete with `[x]`.
