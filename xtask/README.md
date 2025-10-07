# RipTide EventMesh Code Quality Scanner

An automated code quality detection and refactoring tool for the RipTide EventMesh project.

## Overview

The `xtask` scanner helps identify and fix common code quality issues in the RipTide codebase:

- **Underscore-prefixed variables** (`let _var = expr;`) that may need handling
- **TODO/FIXME/HACK/XXX comments** that need tracking
- **Suspicious patterns** (guards, Results, spawns, builders)
- **RipTide-specific patterns** (SpiderConfigBuilder, ConnectionPool, Axum handlers)

## Installation

The tool is already part of the workspace. No additional installation needed.

## Usage

### Scan the Entire Codebase

```bash
cargo run -p xtask -- scan
```

This generates a detailed triage report at `.reports/triage.md`.

### Scan a Specific Crate

```bash
cargo run -p xtask -- scan --crate-name riptide-api
cargo run -p xtask -- scan --crate-name riptide-core
cargo run -p xtask -- scan --crate-name playground
```

### Apply Safe Automated Fixes

```bash
# Apply to entire codebase
cargo run -p xtask -- apply --mode simple

# Apply to specific crate
cargo run -p xtask -- apply --mode simple --crate-name riptide-api
```

**Safe transformations:**
- `let _foo = expr;` → `let _ = expr;` (single-line only)

### Verify After Fixes

```bash
cargo clippy --all-targets --all-features
cargo test
```

## Output Format

The scanner generates `.reports/triage.md` with:

### Structure
- **Per-crate breakdown** - Organized by workspace member
- **Category grouping** - Underscore bindings and TODOs separated
- **Sorted listings** - By file path and line number
- **Markdown tables** - Easy to read and edit

### Columns
| Column | Description |
|--------|-------------|
| ✔ | Manual checkbox for tracking progress |
| File | Relative file path |
| Line | Line number |
| Variable | Variable name (for underscore bindings) |
| Snippet | Code snippet or context |
| Initial | Automated suggestion |
| Decision | Your triage decision (fill this in) |
| Notes | Additional notes and rationale |

### Decision Types

Fill the **Decision** column with one of:

- `side_effect_only` - Convert to `let _ = expr;` or delete
- `promote_and_use` - Rename `_var` to `var` and use it
- `handle_result` - Add `?` or proper error handling
- `guard_lifetime` - Keep guard alive across critical section
- `detached_ok` - Intentional fire-and-forget task
- `wire_config` - Complete builder (`.build()?`) and use
- `todo_tasked` - Convert TODO to tracked issue
- `axum_handler` - Axum-specific handler pattern

## Detection Patterns

### Underscore Variables

Detects patterns like:
```rust
let _spider_config = SpiderConfigBuilder::new(...);
let _guard = lock.write();
let _handle = tokio::spawn(...);
let _result = fallible_operation();
```

### RipTide-Specific Patterns

Enhanced detection for:
- **SpiderConfigBuilder** - Suggests `wire_config?`
- **ConnectionPool** - Suggests `guard_lifetime?`
- **RwLockGuards** - Suggests `guard_lifetime?`
- **Axum handlers** - Suggests `axum_handler?`
- **Session guards** - Suggests `guard_lifetime?`

### TODO Comments

Finds all instances of:
- `TODO`
- `FIXME`
- `HACK`
- `XXX`

## Workflow

### 1. Initial Scan

```bash
cargo run -p xtask -- scan
git add .reports/triage.md
git commit -m "chore: generate code quality triage report"
```

### 2. Triage Phase

Open `.reports/triage.md` and for each finding:
1. Review the code context
2. Choose appropriate decision type
3. Add rationale in Notes column
4. Check the ✔ box when triaged

```bash
git commit -am "chore: complete triage decisions"
```

### 3. Auto-Fix Phase (Optional)

```bash
cargo run -p xtask -- apply --mode simple
cargo clippy -q && cargo test -q
git commit -am "refactor: apply simple underscore transformations"
```

### 4. Manual Fixes

Work through decisions batch by batch:

```bash
# Fix one module at a time
cargo clippy -q && cargo test -q
git add -p
git commit -m "refactor(module): apply triage decisions"
```

### 5. TODO Resolution

For each TODO:
- **Fix now** (best option), or
- Create GitHub issue and annotate:
  ```rust
  // TODO(@your-name, #123): reason/next step
  ```

## Examples

### Before
```rust
let _spider_config = SpiderConfigBuilder::new(state, url)
    .with_depth(2);
// Config never used!
```

### After (Decision: wire_config)
```rust
let spider_config = SpiderConfigBuilder::new(state, url)
    .with_depth(2)
    .build()?;
engine.apply(spider_config);
```

---

### Before
```rust
let _guard = store.write();
// Guard might drop too early!
```

### After (Decision: guard_lifetime)
```rust
let guard = store.write();
// Mutations here
drop(guard); // Explicit lifetime control
```

---

### Before
```rust
let _result = write_config(path, cfg);
// Error ignored!
```

### After (Decision: handle_result)
```rust
write_config(path, cfg)?;
// or: let _ = write_config(path, cfg).ok(); // best-effort
```

## Progress Tracking

The scanner includes progress indicators for large scans:

```
Scanning 538 files...
  Progress: 50/538 files scanned
  Progress: 100/538 files scanned
  Progress: 150/538 files scanned
  ...
Scan complete. Found 206 issues.
```

## Performance

- **Parallel scanning** with Rayon for fast analysis
- **Regex-based detection** for accuracy
- **Incremental compilation** for quick iterations
- Scans ~500+ files in seconds

## File Organization

```
xtask/
├── Cargo.toml           # Dependencies
├── README.md            # This file
└── src/
    └── main.rs          # Scanner implementation

.reports/
└── triage.md           # Generated report
```

## Error Handling

The tool handles:
- **File read failures** - Warns and continues
- **Binary files** - Automatically skipped
- **Large files** - Snippet truncation (80 chars)
- **Special characters** - Proper escaping in markdown

## Integration

### With CI/CD

```yaml
# Example GitHub Actions check
- name: Code quality scan
  run: |
    cargo run -p xtask -- scan
    # Check for undecided rows
    if grep -E "^\|.*\|.*\|.*\|.*\|.*\|.*\|\s*\|" .reports/triage.md; then
      echo "Found undecided triage items"
      exit 1
    fi
```

### With Pre-commit Hooks

```bash
# .git/hooks/pre-commit
#!/bin/bash
cargo run -p xtask -- scan --crate-name $(git diff --name-only | grep "^crates/" | cut -d/ -f2 | head -1)
```

## Best Practices

1. **Scan frequently** - Catch issues early
2. **Triage in batches** - Don't try to fix everything at once
3. **Test after each batch** - Maintain green CI
4. **Document decisions** - Use the Notes column
5. **Create issues for TODOs** - Track technical debt

## Extending the Scanner

To add new detection patterns, edit `xtask/src/main.rs`:

```rust
// Add new regex pattern
static RE_YOUR_PATTERN: Lazy<Regex> =
    Lazy::new(|| Regex::new(r#"your_pattern"#).unwrap());

// Add detection in scan_dir function
if RE_YOUR_PATTERN.is_match(&rhs) {
    "your_suggestion?"
}
```

## Support

For issues or feature requests, please file an issue in the repository.

## License

Apache-2.0
