# Instructions for Branch Agent

You don't have access to cargo/clippy/build tools in your environment. We've captured **complete diagnostics** for you.

## 📋 Your Task

**Fix 124 issues:** 27 errors + 97 warnings across the entire workspace, then commit your changes. We'll verify the build after you commit.

### Issue Breakdown
- **27 Errors** (11 compilation + 16 clippy) - Must fix these first
- **97 Warnings** (32 compiler + 65 clippy) - All must be fixed (`RUSTFLAGS="-D warnings"`)

## 📦 Diagnostic Files to Use

**Location:** `/workspaces/eventmesh/reports/`

1. **check-complete.json** - All compiler errors and warnings
2. **clippy-complete.json** - All clippy lint errors and warnings

These JSON files contain:
- Exact file paths, line numbers, and column numbers
- Full error/warning messages explaining the problem
- Suggested fixes (replacement code)
- Code snippets showing the context

## 🔍 How to Parse the Diagnostics

Each line in the JSON files is a separate diagnostic message. Here's how to extract the information:

### Get All Errors
```bash
jq -r 'select(.reason=="compiler-message" and .message.level=="error") |
  "\(.message.spans[0].file_name):\(.message.spans[0].line_start) - \(.message.message)"' \
  reports/check-complete.json reports/clippy-complete.json
```

### Get All Warnings
```bash
jq -r 'select(.reason=="compiler-message" and .message.level=="warning") |
  "\(.message.spans[0].file_name):\(.message.spans[0].line_start) - \(.message.message)"' \
  reports/check-complete.json reports/clippy-complete.json
```

### Get Suggested Fixes
```bash
jq -r 'select(.reason=="compiler-message") |
  .message.children[]? |
  select(.spans[0].suggested_replacement != null) |
  "\(.message): Replace with: \(.spans[0].suggested_replacement)"' \
  reports/check-complete.json
```

## 🎯 What to Fix

Work through the diagnostics systematically:

### 1. Fix All Compilation Errors First
These prevent the code from building. They MUST be fixed.

Common error types:
- **E0433** - Unresolved imports (wrong crate name, missing dependency)
- **E0308** - Type mismatches (return type doesn't match)
- **E0061** - Wrong number of function arguments
- **E0425** - Unresolved name or path

### 2. Fix All Warnings
We ran `RUSTFLAGS="-D warnings"`, warnings are treated as errors. All must be fixed.

Common warning types (amoung others you know):
- **unused_imports** - Remove unused `use` statements
- **unused_variables** - Prefix with `_` or remove if truly unused
- **dead_code** - Remove unused functions or add `#[allow(dead_code)]`
- **unexpected_cfgs** - Fix feature flag names in `#[cfg(feature = "...")]`
- **clippy::too_many_arguments** - Refactor to use a config struct

## 📝 Process

1. **Parse the diagnostics** to see all issues
2. **Fix errors first** (sorted by file, then line number)
3. **Fix warnings** (sorted by file, then line number)
4. **Commit your changes** with a descriptive message

## ✅ Acceptance Criteria

Your work is done when you've:
- Fixed all compilation errors in check-complete.json
- Fixed all clippy errors in clippy-complete.json
- Fixed all warnings in both files
- Committed all changes

**DO NOT** try to build, run clippy, or run tests. You don't have those tools. Just fix the issues and commit. We'll verify the build after your commit.

## 💡 Tips

### For Unused Imports
```bash
# Find the exact import line
grep -n "use.*NoOpExtractor" crates/riptide-spider/src/builder.rs

# Remove it
sed -i '/use.*NoOpExtractor/d' crates/riptide-spider/src/builder.rs
```

### For Unused Variables
```bash
# Change config -> _config
sed -i 's/let config =/let _config =/g' crates/riptide-pool/tests/native_pool_tests.rs
```

### For Type Mismatches
Read the diagnostic carefully - it tells you:
- Expected type
- Found type
- Suggested fix (sometimes with exact code)

### For Wrong Argument Count
The diagnostic shows:
- Function signature
- What you called it with
- What arguments are missing

## 🚀 Example Workflow

```bash
# 1. See what needs fixing
jq -r 'select(.reason=="compiler-message" and .message.level=="error")' \
  reports/check-complete.json | head -20

# 2. Fix issues one by one
vim crates/riptide-facade/src/facades/crawl_facade.rs

# 3. Commit when done
git add -A
git commit -m "Fix all compilation errors and warnings

- Fixed [N] compilation errors (E0433, E0061, E0308)
- Fixed [M] clippy errors
- Cleaned up unused imports and variables
- All diagnostics from check-complete.json and clippy-complete.json addressed

🤖 Generated with Claude Code
Co-Authored-By: Claude <noreply@anthropic.com>"
```

## 📚 Reference

**Diagnostic file structure:**
```json
{
  "reason": "compiler-message",
  "message": {
    "level": "error" | "warning",
    "message": "human-readable description",
    "code": { "code": "E0308" },
    "spans": [{
      "file_name": "path/to/file.rs",
      "line_start": 123,
      "line_end": 123,
      "column_start": 5,
      "column_end": 15,
      "text": [{ "text": "actual code snippet" }]
    }],
    "children": [{
      "level": "help",
      "message": "suggestion",
      "spans": [{
        "suggested_replacement": "fixed code"
      }]
    }]
  }
}
```

---

**Ready?** Parse the diagnostics, fix all issues, commit in large batches until done. We'll handle verification!
