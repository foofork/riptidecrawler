# Claude Web Development Workflow

**Last Updated:** 2025-11-05
**Status:** Active workflow for Claude Web collaboration

---

## 🎯 When to Use Claude Web vs Claude Code

### Claude Web: Development & Implementation
- ✅ **Feature implementation** - Writing new code
- ✅ **Refactoring** - Restructuring existing code
- ✅ **Documentation** - Writing guides and reports
- ✅ **Architecture planning** - Designing systems
- ✅ **Multi-agent coordination** - Using swarm patterns
- ✅ **Code analysis** - Reading and understanding code
- ✅ **PR creation** - Creating branches and PRs

### Claude Code: Verification & Integration
- ✅ **Compilation verification** - Running cargo check/build
- ✅ **Test execution** - Running full test suites
- ✅ **Error fixing** - Iterative compile-fix cycles
- ✅ **PR review & merge** - Final verification and merging
- ✅ **Environment troubleshooting** - Fixing rustup/cargo issues
- ✅ **CI/CD debugging** - Fixing pipeline failures

---

## 📋 Phase-by-Phase Assignment

### Week 2-2.5: TDD Guide + Test Fixtures
**Assign to:** Claude Web (with verification by Claude Code)

**Claude Web Tasks:**
1. Create TDD guide documentation (docs/phase0/TDD-GUIDE.md)
2. Create test fixture utilities (crates/riptide-test-fixtures/)
3. Write example TDD workflows
4. Document test patterns

**Claude Code Tasks:**
1. Verify guide examples compile and run
2. Run test fixture test suites
3. Ensure integration with existing tests
4. Merge PR after verification

### Week 0-1: Utils Consolidation (Future)
**Assign to:** Claude Web (implementation) + Claude Code (migration verification)

**Claude Web:** Create consolidated utils crate
**Claude Code:** Verify all migrations complete (`rg` old patterns returns 0)

### Week 2.5-5.5: Spider Decoupling (Mostly Complete)
**Status:** Already done, but follow-up fixes can use this pattern

---

## 🔧 Environment Setup Protocol

### For Claude Web

**At Session Start:**
```bash
# 1. Check environment
whoami  # Should be: codespace
source ~/.bashrc
./check-env.sh

# 2. Verify correct branch
git branch --show-current

# 3. If cargo fails, proceed anyway
# Create code WITHOUT verification
# Document what SHOULD work in PR description
```

**If Cargo Commands Fail:**
```markdown
⚠️ **Environment Note**: Cargo commands failed in my environment.

**Code Changes:**
- [List all files created/modified]

**Verification Needed:**
```bash
# Commands that should be run by Claude Code:
cargo check -p [crate]
cargo test -p [crate]
cargo clippy -p [crate] -- -D warnings
```

**Expected Results:**
- [ ] All builds pass
- [ ] Tests pass: [specific test count]
- [ ] Zero clippy warnings
```

### For Claude Code

**When Reviewing Claude Web PRs:**
```bash
# 1. Checkout PR branch
gh pr checkout [PR-NUMBER]

# 2. Verify environment
source ~/.bashrc
df -h / | head -2  # >5GB free

# 3. Run verification suite
cargo check -p [crate]
cargo test -p [crate]
cargo clippy -p [crate] -- -D warnings

# 4. Fix any issues
# ... make fixes ...

# 5. Merge
gh pr merge [PR-NUMBER] --squash
```

---

## 📝 Task Templates

### Template: Documentation Task (Claude Web)

```markdown
Task: Create [Document Name]

**Location:** docs/[path]/[file].md
**Scope:** [What to document]

**Sections Required:**
1. Overview
2. [Section 2]
3. [Section 3]
4. Examples
5. Best Practices

**Success Criteria:**
- [ ] All sections complete
- [ ] Code examples included
- [ ] Markdown formatting correct
- [ ] Cross-referenced with roadmap

**No compilation needed** - Documentation only
```

### Template: Code Implementation Task (Claude Web)

```markdown
Task: Implement [Feature Name]

**Branch:** feature/[phase]-[name]
**Crate:** crates/[crate-name]

**Implementation:**
1. Create [file1] with [functionality]
2. Create [file2] with [functionality]
3. Add tests in tests/[file]
4. Update Cargo.toml dependencies

**Expected Tests:**
- `test_[scenario_1]` should pass
- `test_[scenario_2]` should pass
- Total: [X] tests

**Verification Commands for Claude Code:**
```bash
cargo check -p [crate]
cargo test -p [crate]
cargo clippy -p [crate] -- -D warnings
```

**Known Limitations:**
- May not be able to run cargo in my environment
- Will document expected results in PR
- Claude Code to verify compilation
```

### Template: Refactoring Task (Claude Web)

```markdown
Task: Refactor [Component]

**Branch:** refactor/[name]
**Files Modified:** [List expected files]

**Changes:**
1. Extract [code] to [location]
2. Consolidate [duplicated code]
3. Update imports in [X] files

**Migration Verification (for Claude Code):**
```bash
# Should return 0 files after refactoring:
rg "old_pattern" --type rust | grep -v target
```

**Testing:**
- All existing tests should still pass
- No new tests required (refactoring only)
```

---

## 🚀 Practical Example: Week 2-2.5

### Message to Claude Web:

```markdown
# Task: Week 2-2.5 - TDD Guide + Test Fixtures

## Context
- **Roadmap:** docs/roadmap/RIPTIDE-V1-DEFINITIVE-ROADMAP.md (Week 2-2.5)
- **Branch:** feature/phase0-week2-tdd-guide
- **Environment Docs:** docs/ENVIRONMENT-SETUP-INSTRUCTIONS.md

## Your Tasks

### 1. TDD Guide Documentation
**Create:** docs/phase0/TDD-GUIDE.md

**Content:**
- Overview of TDD approach for RipTide
- Red-Green-Refactor cycle
- Test fixture usage patterns
- Example workflows for common scenarios
- Integration with existing test infrastructure

### 2. Test Fixtures Crate
**Create:** crates/riptide-test-fixtures/

**Structure:**
```
crates/riptide-test-fixtures/
├── Cargo.toml
├── src/
│   ├── lib.rs
│   ├── fixtures/
│   │   ├── mod.rs
│   │   ├── redis.rs      # Redis test fixtures
│   │   ├── http.rs       # HTTP mock servers
│   │   └── spider.rs     # Spider test data
│   └── builders/
│       ├── mod.rs
│       └── api.rs         # Builder patterns for tests
└── tests/
    └── integration_test.rs
```

**Implementation:**
- Redis test fixtures with cleanup
- HTTP mock servers (using wiremock)
- Spider test data generators
- Builder patterns for test setup

### 3. Example Workflows
**Create:** docs/phase0/TDD-EXAMPLES.md

**Examples:**
- Adding a new API endpoint (full TDD cycle)
- Refactoring existing code with tests
- Testing async code patterns
- Integration test patterns

## Environment Notes

⚠️ **If cargo commands fail:**
1. Continue with code implementation
2. Document expected behavior in PR
3. Note: "Verification needed by Claude Code"

## Deliverables

Create PR with:
- [ ] docs/phase0/TDD-GUIDE.md
- [ ] docs/phase0/TDD-EXAMPLES.md
- [ ] crates/riptide-test-fixtures/ (complete crate)
- [ ] PR description with verification checklist

## Verification (for Claude Code)
```bash
cargo check -p riptide-test-fixtures
cargo test -p riptide-test-fixtures
cargo doc -p riptide-test-fixtures --no-deps
```

Expected: All commands succeed, [X] tests pass

## Success Criteria
- [ ] Documentation complete and comprehensive
- [ ] Test fixtures crate structure in place
- [ ] Example code compiles (or documented if not verifiable)
- [ ] PR ready for Claude Code review
```

---

## 📊 Coordination Pattern

```
┌─────────────────┐
│  Claude Web     │
│  (Development)  │
└────────┬────────┘
         │
         │ 1. Create branch
         │ 2. Implement features
         │ 3. Write documentation
         │ 4. Create PR
         │
         ▼
┌─────────────────┐
│  GitHub PR      │
│  (Review Queue) │
└────────┬────────┘
         │
         │ 5. Notify completion
         │
         ▼
┌─────────────────┐
│  Claude Code    │
│  (Verification) │
└────────┬────────┘
         │
         │ 6. Checkout PR
         │ 7. Run cargo commands
         │ 8. Fix compilation issues
         │ 9. Run tests
         │ 10. Merge to main
         │
         ▼
┌─────────────────┐
│  Main Branch    │
│  (Production)   │
└─────────────────┘
```

---

## ⚡ Quick Reference Commands

### For Claude Web (Create PR)
```bash
git checkout -b feature/[phase]-[name]
# ... make changes ...
git add .
git commit -m "feat([scope]): [description]"
git push origin feature/[phase]-[name]
gh pr create --base main --title "[Title]" --body "[Description]"
```

### For Claude Code (Review & Merge)
```bash
gh pr checkout [PR-NUM]
cargo check --workspace
cargo test --workspace
cargo clippy --all -- -D warnings
gh pr merge [PR-NUM] --squash
```

---

## 📋 Current Status

**Active PRs from Claude Web:** 0
**Completed PRs:**
- PR #6: Week 1.5-2 Configuration & Feature Gates (✅ Merged)

**Next Assignment:** Week 2-2.5 - TDD Guide + Test Fixtures

---

**Remember:**
- Claude Web excels at **creation and implementation**
- Claude Code excels at **verification and integration**
- Use environment docs to minimize setup issues
- Document verification needs clearly in PRs
- Iterate quickly with this workflow
